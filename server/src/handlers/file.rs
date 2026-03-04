use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::header,
    response::Response,
    Json,
};
use chrono::Utc;
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::AppState;

const ALLOWED_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "srt", "ass", "ssa", "vtt",
];

/// Magic bytes validation for uploaded media files
fn is_valid_media_magic(data: &[u8], ext: &str) -> bool {
    if data.len() < 12 {
        return matches!(ext, "srt" | "ass" | "ssa" | "vtt");
    }
    if data.len() >= 8 && &data[4..8] == b"ftyp" { return true; }
    if data.starts_with(b"RIFF") { return true; }
    if data.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) { return true; }
    if data.starts_with(b"FLV") { return true; }
    if data.starts_with(&[0x30, 0x26, 0xB2, 0x75]) { return true; }
    if matches!(ext, "srt" | "ass" | "ssa" | "vtt") { return true; }
    false
}

/// Sanitize filename to prevent path traversal
fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|c| !c.is_control() && *c != '\0')
        .collect();
    cleaned.replace("../", "_").replace("..\\", "_").replace('/', "_").replace('\\', "_")
}

pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    let upload_dir = PathBuf::from(&state.config.upload_dir);
    fs::create_dir_all(&upload_dir)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create upload dir: {}", e)))?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        let raw_filename = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let filename = sanitize_filename(&raw_filename);

        // 校验文件扩展名
        let ext = filename
            .rsplit('.')
            .next()
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
            return Err(AppError::BadRequest(format!(
                "File type '{}' not allowed",
                ext
            )));
        }

        let file_id = Uuid::new_v4().to_string();
        let save_filename = format!("{}_{}", &file_id[..8], filename);
        let save_path = upload_dir.join(&save_filename);

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read file data: {}", e)))?;

        let file_size = data.len() as u64;
        if file_size > state.config.max_file_size {
            let max_mb = state.config.max_file_size / (1024 * 1024);
            return Err(AppError::BadRequest(format!(
                "File too large ({}MB). Maximum: {}MB",
                file_size / (1024 * 1024), max_mb
            )));
        }

        // Magic bytes validation
        if !is_valid_media_magic(&data, &ext) {
            return Err(AppError::BadRequest(
                "File content does not match expected media format".to_string(),
            ));
        }

        fs::write(&save_path, &data)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to save file: {}", e)))?;

        let now = Utc::now().timestamp();

        // 保存文件记录
        sqlx::query(
            "INSERT INTO files (id, filename, file_hash, file_size, uploader_id, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&file_id)
        .bind(&filename)
        .bind("") // hash 由客户端计算
        .bind(file_size as i64)
        .bind("anonymous") // 后续从 JWT 获取
        .bind(now)
        .execute(&state.db)
        .await?;

        return Ok(Json(json!({
            "id": file_id,
            "filename": filename,
            "size": file_size,
            "url": format!("/api/files/{}", file_id),
        })));
    }

    Err(AppError::BadRequest("No file uploaded".to_string()))
}

pub async fn download(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Response> {
    let file = sqlx::query_as::<_, (String, String)>(
        "SELECT id, filename FROM files WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await?;

    let (file_id, filename) = file
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    let upload_dir = PathBuf::from(&state.config.upload_dir);

    // 查找匹配的文件
    let mut entries = fs::read_dir(&upload_dir)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read upload dir: {}", e)))?;

    let mut file_path = None;
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(&file_id[..8]) {
            file_path = Some(entry.path());
            break;
        }
    }

    let path = file_path.ok_or_else(|| AppError::NotFound("File not found on disk".to_string()))?;

    let data = fs::read(&path)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read file: {}", e)))?;

    let response = Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(response)
}