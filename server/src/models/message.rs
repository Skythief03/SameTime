use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: String,
    pub room_id: String,
    pub user_id: String,
    pub content: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DanmakuRecord {
    pub id: String,
    pub room_id: String,
    pub user_id: String,
    pub content: String,
    pub video_timestamp: f64,
    pub color: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FileRecord {
    pub id: String,
    pub filename: String,
    pub file_hash: String,
    pub file_size: i64,
    pub uploader_id: String,
    pub room_id: Option<String>,
    pub created_at: i64,
}