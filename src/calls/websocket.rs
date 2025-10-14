use actix_identity::Identity;
use actix_web::{
    HttpRequest, HttpResponse, Result as ActixResult, get,
    http::header::{self, ContentType},
    rt, web,
};
use actix_ws::{AggregatedMessage, CloseReason, handle};
use futures::StreamExt;
use std::sync::Arc;
use tera::Context;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::{
    calls::{
        entities::{ServerMessage, SignalingMessage},
        signalling_server::SignalingServer,
    },
    infrastructure::templates::TEMPLATES,
    shared::response::AppError,
};

#[derive(Debug)]
pub enum OutgoingMessage {
    Text(String),
    Binary(Vec<u8>),
    Close(Option<CloseReason>),
}

#[get("/ws/rooms/{room_id}")]
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    identity: Option<Identity>,
    room_id: web::Path<String>,
    server: web::Data<Arc<SignalingServer>>,
) -> ActixResult<HttpResponse> {
    println!("Hit by client");
    let room_id = room_id.into_inner();
    let user_id: i32 = match identity
        .and_then(|id| id.id().ok())
        .and_then(|id_str| id_str.parse::<i32>().ok())
    {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Found()
                .append_header((header::LOCATION, "/vibecall/auth/login"))
                .finish());
        }
    };

    println!("room_id: {room_id} and user_id: {user_id}");

    let (response, mut session, msg_stream) = handle(&req, stream)?;

    let mut msg_stream = msg_stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    let (tx, mut rx): (
        UnboundedSender<OutgoingMessage>,
        UnboundedReceiver<OutgoingMessage>,
    ) = unbounded_channel();

    if let Err(e) = server
        .add_connection(user_id, room_id.clone(), tx.clone())
        .await
    {
        eprintln!("[{}] Failed to add connection: {:?}", user_id, e);
        return Err(AppError::InternalServerError(e.to_string()).into());
    }

    let user_id_clone = user_id;
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                OutgoingMessage::Text(text) => {
                    if let Err(e) = session.text(text).await {
                        eprintln!("[{}] Send text error: {:?}", user_id_clone, e);
                        break;
                    }
                }
                OutgoingMessage::Binary(bin) => {
                    if let Err(e) = session.binary(bin).await {
                        eprintln!("[{}] Send binary error: {:?}", user_id_clone, e);
                        break;
                    }
                }
                OutgoingMessage::Close(reason) => {
                    let _ = session.close(reason).await;
                    break;
                }
            }
        }
    });

    rt::spawn(async move {
        while let Some(msg) = msg_stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    println!("[{}] Received text: {}", user_id, text);
                    if let Err(e) =
                        handle_text_message(user_id, &room_id, &text, server.get_ref(), &tx).await
                    {
                        eprintln!("[{}] Error handling text message: {}", user_id, e);
                        let error_msg = ServerMessage::Error {
                            message: e.to_string(),
                        };
                        if let Ok(json) = serde_json::to_string(&error_msg) {
                            let _ = tx.send(OutgoingMessage::Text(json));
                        }
                    }
                }
                Ok(AggregatedMessage::Binary(_bin)) => {
                    println!("[{}] Received binary message", user_id);
                }
                Ok(AggregatedMessage::Ping(_ping)) => {}
                Ok(AggregatedMessage::Close(_reason)) => {
                    println!("[{}] WebSocket close received", user_id);
                    break;
                }
                Ok(AggregatedMessage::Pong(_)) => {}
                Err(e) => {
                    eprintln!("[{}] WebSocket error: {:?}", user_id, e);
                    break;
                }
            }
        }

        println!("[{}] Connection closed, cleaning up", user_id);
        server.remove_connection(user_id).await;

        let user = match server.get_caller_info(user_id).await {
            Ok(user) => user,
            Err(e) => {
                eprintln!("[{}] Failed to get caller info: {:?}", user_id, e);
                return;
            }
        };

        let message = ServerMessage::UserLeft {
            user_id,
            user_name: user.1,
        };
        if let Ok(json) = serde_json::to_string(&message) {
            server.broadcast_to_room(&room_id, user_id, &json);
        }
    });

    Ok(response)
}

async fn handle_text_message(
    user_id: i32,
    _room_id: &str,
    text: &str,
    server: &Arc<SignalingServer>,
    tx: &UnboundedSender<OutgoingMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let message: SignalingMessage = serde_json::from_str(text)?;
    println!("Signaling message is {:?}", message);

    match message {
        SignalingMessage::Join {
            room_id: msg_room_id,
            user_id: msg_user_id,
        } => {
            let call_id = server.join_call(msg_user_id, msg_room_id.clone()).await?;

            let users = server.get_room_users(&msg_room_id).await;

            let response = ServerMessage::UserJoined {
                user_id: msg_user_id,
                users: users.clone(),
            };
            let json = serde_json::to_string(&response)?;
            tx.send(OutgoingMessage::Text(json))
                .map_err(|e| format!("Failed to send message: {}", e))?;

            let broadcast_msg = ServerMessage::UserJoined {
                user_id: msg_user_id,
                users: users.clone(),
            };
            let broadcast_json = serde_json::to_string(&broadcast_msg)?;
            server.broadcast_to_room(&msg_room_id, msg_user_id, &broadcast_json);

            println!(
                "[{}] Joined call {} in room {}",
                msg_user_id, call_id, msg_room_id
            );
        }

        SignalingMessage::Leave { room_id } => {
            server.remove_connection(user_id).await;
            let user = server.get_caller_info(user_id).await?;

            let message = ServerMessage::UserLeft {
                user_id,
                user_name: user.1,
            };
            if let Ok(json) = serde_json::to_string(&message) {
                server.broadcast_to_room(&room_id, user_id, &json);
            }
        }

        SignalingMessage::Offer {
            target_user_id,
            sdp,
        } => {
            let message = ServerMessage::Offer { from: user_id, sdp };
            let json = serde_json::to_string(&message)?;
            server.send_to_user(target_user_id, &json)?;
            println!("[{}] Sent offer to [{}]", user_id, target_user_id);
        }

        SignalingMessage::Answer {
            target_user_id,
            sdp,
        } => {
            let message = ServerMessage::Answer { from: user_id, sdp };
            let json = serde_json::to_string(&message)?;
            server.send_to_user(target_user_id, &json)?;
            println!("[{}] Sent answer to [{}]", user_id, target_user_id);
        }

        SignalingMessage::IceCandidate {
            target_user_id,
            candidate,
            sdp_mid,
            sdp_m_line_index,
        } => {
            let message = ServerMessage::IceCandidate {
                from: user_id,
                candidate,
                sdp_mid,
                sdp_m_line_index,
            };
            let json = serde_json::to_string(&message)?;
            server.send_to_user(target_user_id, &json)?;
        }
    }

    Ok(())
}

#[get("/ws/video-call")]
pub async fn test_videocall() -> actix_web::Result<HttpResponse> {
    let mut context = Context::new();
    context.insert("title", "VideoCall Test");

    let rendered = TEMPLATES
        .render("videos.html", &context)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered))
}
