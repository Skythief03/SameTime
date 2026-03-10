use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => {
                tracing::warn!("404 Not Found: {}", msg);
                (StatusCode::NOT_FOUND, msg.clone())
            }
            AppError::BadRequest(msg) => {
                tracing::warn!("400 Bad Request: {}", msg);
                (StatusCode::BAD_REQUEST, msg.clone())
            }
            AppError::Unauthorized(msg) => {
                tracing::warn!("401 Unauthorized: {}", msg);
                (StatusCode::UNAUTHORIZED, msg.clone())
            }
            AppError::Conflict(msg) => {
                tracing::warn!("409 Conflict: {}", msg);
                (StatusCode::CONFLICT, msg.clone())
            }
            AppError::Internal(msg) => {
                tracing::error!("500 Internal Error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
            AppError::Database(e) => {
                tracing::error!("500 Database Error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;