use axum::{extract::State, Json};
use chrono::Utc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::user::{AuthResponse, LoginRequest, RegisterRequest};
use crate::AppState;

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    tracing::info!("Register request: username={}", req.username);

    // 校验参数
    if req.username.trim().is_empty() || req.password.len() < 4 {
        tracing::warn!("Register failed: invalid params for username={}", req.username);
        return Err(AppError::BadRequest(
            "Username cannot be empty and password must be at least 4 characters".to_string(),
        ));
    }

    // 检查用户名是否已存在
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE username = ?",
    )
    .bind(&req.username)
    .fetch_one(&state.db)
    .await?;

    if exists > 0 {
        tracing::warn!("Register failed: username={} already taken", req.username);
        return Err(AppError::Conflict("Username already taken".to_string()));
    }

    let user_id = Uuid::new_v4().to_string();
    let password_hash = bcrypt::hash(&req.password, 10)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;
    let now = Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO users (id, username, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&user_id)
    .bind(&req.username)
    .bind(&password_hash)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await?;

    // 生成 JWT
    let token = generate_jwt(&user_id, &req.username, &state.config.jwt_secret, state.config.jwt_expiry_hours)?;

    tracing::info!("User registered: id={}, username={}", user_id, req.username);

    Ok(Json(AuthResponse {
        user_id,
        username: req.username,
        token,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    tracing::info!("Login request: username={}", req.username);

    let user = sqlx::query_as::<_, (String, String, String)>(
        "SELECT id, username, password_hash FROM users WHERE username = ?",
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await?;

    let (user_id, username, password_hash) = user
        .ok_or_else(|| {
            tracing::warn!("Login failed: username={} not found", req.username);
            AppError::Unauthorized("Invalid username or password".to_string())
        })?;

    let valid = bcrypt::verify(&req.password, &password_hash)
        .map_err(|e| AppError::Internal(format!("Failed to verify password: {}", e)))?;

    if !valid {
        tracing::warn!("Login failed: wrong password for username={}", req.username);
        return Err(AppError::Unauthorized(
            "Invalid username or password".to_string(),
        ));
    }

    let token = generate_jwt(&user_id, &username, &state.config.jwt_secret, state.config.jwt_expiry_hours)?;

    tracing::info!("User logged in: id={}, username={}", user_id, username);

    Ok(Json(AuthResponse {
        user_id,
        username,
        token,
    }))
}

fn generate_jwt(user_id: &str, username: &str, secret: &str, expiry_hours: i64) -> AppResult<String> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct Claims {
        sub: String,
        username: String,
        exp: i64,
        iat: i64,
    }

    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: now + expiry_hours * 3600,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate token: {}", e)))
}
