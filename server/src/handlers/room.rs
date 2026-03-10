use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::room::{CreateRoomRequest, RoomResponse};
use crate::AppState;

pub async fn create_room(
    State(state): State<AppState>,
    Json(req): Json<CreateRoomRequest>,
) -> AppResult<Json<RoomResponse>> {
    tracing::info!("Create room request: name={}, has_password={}", req.name, req.password.is_some());

    if req.name.trim().is_empty() {
        tracing::warn!("Create room failed: empty name");
        return Err(AppError::BadRequest("Room name cannot be empty".to_string()));
    }

    // 生成短房间ID（8位）
    let room_id = Uuid::new_v4().to_string()[..8].to_string();
    let now = Utc::now().timestamp();

    // 暂用 "anonymous" 作为 host_id，后续接入 JWT 解析
    let host_id = "anonymous".to_string();

    let password_hash = match &req.password {
        Some(pwd) if !pwd.is_empty() => {
            Some(bcrypt::hash(pwd, 10).map_err(|e| AppError::Internal(e.to_string()))?)
        }
        _ => None,
    };

    // 持久化到数据库
    sqlx::query(
        "INSERT INTO rooms (id, name, host_id, password_hash, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&room_id)
    .bind(&req.name)
    .bind(&host_id)
    .bind(&password_hash)
    .bind(now)
    .execute(&state.db)
    .await?;

    // 在内存中创建房间
    let username = "Host".to_string(); // 后续从 JWT 获取
    state
        .room_manager
        .create_room(room_id.clone(), req.name.clone(), host_id.clone(), username);

    tracing::info!("Room created: id={}, name={}", room_id, req.name);

    Ok(Json(RoomResponse {
        id: room_id,
        name: req.name,
        host_id,
        current_time: 0.0,
        is_playing: false,
        created_at: now,
    }))
}

pub async fn get_room(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<RoomResponse>> {
    tracing::debug!("Get room: id={}", id);

    let room = state
        .room_manager
        .get_room(&id)
        .ok_or_else(|| {
            tracing::warn!("Get room failed: id={} not found", id);
            AppError::NotFound("Room not found".to_string())
        })?;

    let host_id = room.host_id.read().map(|h| h.clone()).unwrap_or_default();
    let current_time = room.current_time.read().map(|v| *v).unwrap_or(0.0);
    let is_playing = room.is_playing.read().map(|v| *v).unwrap_or(false);

    Ok(Json(RoomResponse {
        id: room.id.clone(),
        name: room.name.clone(),
        host_id,
        current_time,
        is_playing,
        created_at: room.created_at,
    }))
}

#[derive(serde::Deserialize)]
pub struct JoinRoomBody {
    pub password: Option<String>,
}

pub async fn join_room(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<JoinRoomBody>,
) -> AppResult<Json<RoomResponse>> {
    tracing::info!("Join room request: room_id={}", id);

    // 验证房间密码
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT password_hash FROM rooms WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await?;

    if let Some((Some(hash),)) = row {
        // 房间有密码
        let provided = body.password.unwrap_or_default();
        if provided.is_empty() {
            tracing::warn!("Join room failed: room_id={} requires password", id);
            return Err(AppError::Unauthorized("此房间需要密码".to_string()));
        }
        let valid = bcrypt::verify(&provided, &hash)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if !valid {
            tracing::warn!("Join room failed: room_id={} wrong password", id);
            return Err(AppError::Unauthorized("密码错误".to_string()));
        }
    }

    let user_id = Uuid::new_v4().to_string(); // 后续从 JWT 获取
    let username = "Guest".to_string();

    let room = state
        .room_manager
        .join_room(&id, user_id.clone(), username.clone())
        .ok_or_else(|| {
            tracing::warn!("Join room failed: room_id={} not found in memory", id);
            AppError::NotFound("Room not found".to_string())
        })?;

    let host_id = room.host_id.read().map(|h| h.clone()).unwrap_or_default();
    let current_time = room.current_time.read().map(|v| *v).unwrap_or(0.0);
    let is_playing = room.is_playing.read().map(|v| *v).unwrap_or(false);
    let member_count = room.members.len();

    tracing::info!("User joined room: room_id={}, user_id={}, members={}", id, user_id, member_count);

    Ok(Json(RoomResponse {
        id: room.id.clone(),
        name: room.name.clone(),
        host_id,
        current_time,
        is_playing,
        created_at: room.created_at,
    }))
}
