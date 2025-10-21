use std::sync::Arc;

use actix_files::Files;
use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::{App, HttpServer, cookie::Key, middleware, web::Data};
use base64::{Engine, engine::general_purpose};
use vibecall::{
    auth,
    calls::{self, SignalingServer},
    infrastructure, rooms,
    shared::file_service::{FileService, LocalFileService},
    users,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server_address = "0.0.0.0";
    let server_port: u16 = 8085;

    let sqlite_pool = infrastructure::database::create_sqlite_pool("sqlite://./data/vibecall.db")
        .await
        .expect("Failed to create SQLite pool");

    sqlx::migrate!("./migrations")
        .run(&sqlite_pool)
        .await
        .expect("Failed to run database migrations");

    let base_url =
        std::env::var("BASE_URL").unwrap_or_else(|_| format!("http://localhost:{}", server_port));

    let file_service: Arc<dyn FileService> = Arc::new(LocalFileService::new(
        "./media",
        format!("{}/media", base_url),
    ));

    let user_repo = Arc::new(users::SqliteUserRepository::new(sqlite_pool.clone()));
    let user_service: Arc<dyn users::UserService> =
        Arc::new(users::UserServiceImpl::new(user_repo));

    let room_repo = Arc::new(rooms::SqliteRoomRepository::new(sqlite_pool.clone()));
    let room_service: Arc<dyn vibecall::rooms::RoomService> =
        Arc::new(rooms::RoomServiceImpl::new(room_repo));

    let call_repo = Arc::new(calls::SqliteCallRepository::new(sqlite_pool.clone()));
    let call_service: Arc<dyn calls::CallService> = Arc::new(calls::CallServiceImpl::new(
        call_repo,
        room_service.clone(),
        user_service.clone(),
    ));

    let signaling_server = Arc::new(SignalingServer::new(
        call_service.clone(),
        room_service.clone(),
    ));

    println!("Server started on {}:{}", server_address, server_port);

    HttpServer::new(move || {
        App::new()
            .app_data(
                actix_web::web::JsonConfig::default()
                    .error_handler(infrastructure::error_handler::json_error_handler),
            )
            .app_data(Data::new(sqlite_pool.clone()))
            .app_data(Data::new(file_service.clone()))
            .app_data(Data::new(user_service.clone()))
            .app_data(Data::new(room_service.clone()))
            .app_data(Data::new(call_service.clone()))
            .app_data(Data::new(signaling_server.clone()))
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(
                        general_purpose::STANDARD
                            .decode(
                                "FGzGcz8rEHaq+s4kxViffDuCYXqz5jOXZWwn5wTALk1HQoHv8RgihuWswJzKwFx2buKhui1NfBEX6KmBG67Irg=="
                                    .trim(),
                            )
                            .unwrap()
                            .as_ref(),
                    ),
                )
                .cookie_name("vibecall".to_owned())
                .cookie_secure(false)
                .session_lifecycle(
                    PersistentSession::default()
                        .session_ttl(actix_web::cookie::time::Duration::minutes(300)),
                )
                .build(),
            )
            .wrap(middleware::NormalizePath::trim())
            .service(
                Files::new("/static", "./static")
                    .use_last_modified(true)
                    .prefer_utf8(true)
                    .use_etag(true) // Better caching
                    .disable_content_disposition() // Prevent some attacks
                    .index_file("404.html"),
            )
            .configure(calls::routes::call_routes)
            .configure(auth::routes::auth_routes)
            .configure(users::routes::user_routes)
            .configure(infrastructure::routes::infrastructure_routes)
            .configure(rooms::routes::room_routes)
    })
    .bind((server_address, server_port))?
    .run()
    .await?;

    Ok(())
}
