use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoomRecord {
    pub id: String,
    pub name: String,
    pub host_id: String,
    pub password_hash: Option<String>,
    pub created_at: i64,
    pub closed_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JoinRoomRequest {
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RoomResponse {
    pub id: String,
    pub name: String,
    #[serde(rename = "hostId")]
    pub host_id: String,
    #[serde(rename = "currentTime")]
    pub current_time: f64,
    #[serde(rename = "isPlaying")]
    pub is_playing: bool,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
}