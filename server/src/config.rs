use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub upload_dir: String,
    pub max_file_size: u64,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
}

impl Config {
    pub fn load() -> Self {
        // 从环境变量读取，或使用默认值
        Self {
            host: std::env::var("SAMETIME_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("SAMETIME_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./data/sametime.db?mode=rwc".to_string()),
            upload_dir: std::env::var("UPLOAD_DIR")
                .unwrap_or_else(|_| "./data/uploads".to_string()),
            max_file_size: std::env::var("MAX_FILE_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10 * 1024 * 1024 * 1024), // 10GB
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "sametime-dev-secret-change-in-production".to_string()),
            jwt_expiry_hours: std::env::var("JWT_EXPIRY_HOURS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(168), // 7 days
        }
    }
}