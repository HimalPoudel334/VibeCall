use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};
use actix_ws::{CloseReason, Message, MessageStream, handle};
use futures::{StreamExt, TryStreamExt};
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::calls::{
    contract::UserIdParam,
    entities::{ServerMessage, SignalingMessage},
    signalling_server::SignalingServer,
};

#[derive(Debug)]
pub enum OutgoingMessage {
    Text(String),
    Binary(Vec<u8>),
    Close(Option<CloseReason>),
}
/*
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    user: web::Json<UserIdParam>,
    room_id: web::Path<String>,
    server: web::Data<Arc<SignalingServer>>,
) -> ActixResult<HttpResponse> {
    let room_id = room_id.into_inner();
    let user_id = user.user_id.clone();

    // Establish WebSocket connection
    let (response, mut session, msg_stream) = handle(&req, stream)?;

    // Convert low-level stream into aggregated message stream
    let mut msg_stream = msg_stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    // Create a channel for outgoing messages
    let (tx, mut rx): (
        UnboundedSender<OutgoingMessage>,
        UnboundedReceiver<OutgoingMessage>,
    ) = unbounded_channel();

    // Register the connection with the signaling server
    server.add_connection(user_id.clone(), room_id.clone(), tx.clone());

    // Task to handle outgoing messages
    let mut session_clone = session.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                OutgoingMessage::Text(text) => {
                    if let Err(e) = session_clone.text(text).await {
                        eprintln!("[{}] Send text error: {:?}", user_id, e);
                    }
                }
                OutgoingMessage::Binary(bin) => {
                    if let Err(e) = session_clone.binary(bin).await {
                        eprintln!("[{}] Send binary error: {:?}", user_id, e);
                    }
                }
                OutgoingMessage::Close(reason) => {
                    let _ = session_clone.close(reason).await;
                }
            }
        }
    });

    // Task to handle incoming messages
    let server_clone = server.get_ref().clone();
    tokio::spawn(async move {
        while let Some(msg) = msg_stream.try_next().await.transpose() {
            match msg {
                Ok(actix_ws::AggregatedMessage::Text(text)) => {
                    println!("[{}] Received text: {}", user_id, text);
                }
                Ok(actix_ws::AggregatedMessage::Binary(bin)) => {
                    println!("[{}] Received binary message", user_id);
                }
                Ok(actix_ws::AggregatedMessage::Ping(ping)) => {
                    let _ = session.pong(&ping).await;
                }
                Ok(actix_ws::AggregatedMessage::Close(reason)) => {
                    let _ = session.close(reason).await;
                }
                Ok(actix_ws::AggregatedMessage::Pong(pong)) => {
                    let _ = session.pong(&pong).await;
                }
                Err(e) => {
                    eprintln!("[{}] WebSocket error: {:?}", user_id, e);
                }
            }
        }

        // Client disconnected
        server_clone.remove_connection(user_id);
    });

    Ok(response)
}

async fn handle_websocket_messages(
    user_id: i32,
    room_id: String,
    mut session: actix_ws::Session,
    mut msg_stream: MessageStream,
    server: Arc<SignalingServer>,
) {
    while let Some(msg) = msg_stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) =
                    handle_text_message(user_id, &room_id, &text, &server, &mut session).await
                {
                    eprintln!("eprintln handling text message: {}", e);
                    let error_msg = ServerMessage::Error {
                        message: e.to_string(),
                    };
                    if let Ok(json) = serde_json::to_string(&error_msg) {
                        let _ = session.text(json).await;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("WebSocket closed for user {}", user_id);
                break;
            }
            Ok(Message::Ping(bytes)) => {
                let _ = session.pong(&bytes).await;
            }
            Ok(_) => {
                // Ignore other message types
            }
            Err(e) => {
                eprintln!("WebSocket eprintln for user {}: {}", user_id, e);
                break;
            }
        }
    }

    // Clean up connection
    server.remove_connection(user_id);
    println!("Cleaned up connection for user {}", user_id);
}

async fn handle_text_message(
    user_id: i32,
    room_id: &str,
    text: &str,
    server: &Arc<SignalingServer>,
    session: &mut actix_ws::Session,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let message: SignalingMessage = serde_json::from_str(text)?;

    match &message {
        SignalingMessage::Join { room_id, user_id } => {
            let user = server.join_call(*user_id, room_id.to_string()).await?;

            // // Send current users list to the new user
            // let response = ServerMessage::UserJoined {
            //     user: users.iter().find(|u| u.id == *user_id).unwrap().clone(),
            //     users,
            // };
            //
            let users = server.get_room_users(&room_id);

            let json = serde_json::to_string(&response)?;
            session.text(json).await?;
        }
        SignalingMessage::Leave { room_id } => {
            server.remove_connection(user_id);
        }
        _ => {
            server.handle_signaling_message(user_id, message)?;
        }
    }

    Ok(())
}
*/
