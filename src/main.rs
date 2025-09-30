use std::sync::Arc;

use actix_files::Files;
use actix_web::{App, HttpServer, web::Data};
use vibecall::{
    calls, infrastructure, rooms,
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
    let _call_service: Arc<dyn calls::CallService> = Arc::new(calls::CallServiceImpl::new(
        call_repo,
        room_service.clone(),
        user_service.clone(),
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
            .configure(users::routes::user_routes)
            .configure(infrastructure::routes::infrastructure_routes)
            .configure(rooms::routes::room_routes)
            .service(
                Files::new("/media", "./media")
                    .use_last_modified(true)
                    .prefer_utf8(true)
                    .use_etag(true) // Better caching
                    .disable_content_disposition() // Prevent some attacks
                    .index_file("404.html"),
            )
    })
    .bind((server_address, server_port))?
    .run()
    .await?;

    Ok(())
}
