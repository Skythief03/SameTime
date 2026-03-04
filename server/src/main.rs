use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod config;
mod db;
mod error;
mod handlers;
mod models;
mod services;
mod ws;

use services::room_manager::RoomManager;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub room_manager: Arc<RoomManager>,
    pub config: config::Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = config::Config::load();
    let db = db::init(&config.database_url).await?;
    let room_manager = Arc::new(RoomManager::new());

    let state = AppState {
        db,
        room_manager,
        config: config.clone(),
    };

    let app = Router::new()
        // Health check
        .route("/api/health", get(handlers::health))
        // Auth routes
        .route("/api/auth/register", post(handlers::auth::register))
        .route("/api/auth/login", post(handlers::auth::login))
        // Room routes
        .route("/api/rooms", post(handlers::room::create_room))
        .route("/api/rooms/:id", get(handlers::room::get_room))
        .route("/api/rooms/:id/join", post(handlers::room::join_room))
        // File routes
        .route("/api/files/upload", post(handlers::file::upload))
        .route("/api/files/:id", get(handlers::file::download))
        // WebSocket
        .route("/ws/:room_id", get(handlers::ws::ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    // 后台任务：每60秒清理空房间（最大存活24小时）
    let cleanup_manager = state.room_manager.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            cleanup_manager.cleanup_stale_rooms(86400);
        }
    });

    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}