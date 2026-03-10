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
use tokio::io::AsyncWriteExt;
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
    // Extract just the filename component, stripping any directory path
    let basename = std::path::Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    // Remove control chars and null bytes
    let cleaned: String = basename
        .chars()
        .filter(|c| !c.is_control() && *c != '\0')
        .collect();
    if cleaned.is_empty() || cleaned == "." || cleaned == ".." {
        "unknown".to_string()
    } else {
        cleaned
    }
}

pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    tracing::info!("File upload request received");

    let upload_dir = PathBuf::from(&state.config.upload_dir);
    fs::create_dir_all(&upload_dir)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create upload dir: {}", e)))?;

    while let Some(mut field) = multipart
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
            tracing::warn!("Upload rejected: file type '{}' not allowed, filename={}", ext, filename);
            return Err(AppError::BadRequest(format!(
                "File type '{}' not allowed",
                ext
            )));
        }

        let file_id = Uuid::new_v4().to_string();
        let save_filename = format!("{}_{}", &file_id[..8], filename);
        let save_path = upload_dir.join(&save_filename);

        tracing::info!("Uploading file: filename={}, ext={}", filename, ext);

        // 流式写入文件，同时统计大小并检查 magic bytes
        let mut file = fs::File::create(&save_path)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create file: {}", e)))?;

        let mut file_size: u64 = 0;
        let mut first_chunk = Vec::new();
        let mut magic_checked = false;
        let max_file_size = state.config.max_file_size;

        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read chunk: {}", e)))?
        {
            file_size += chunk.len() as u64;

            // 检查文件大小限制
            if file_size > max_file_size {
                // 清理已写入的文件
                drop(file);
                let _ = fs::remove_file(&save_path).await;
                let max_mb = max_file_size / (1024 * 1024);
                tracing::warn!("Upload rejected: file too large, size>{}MB, max={}MB, filename={}", max_mb, max_mb, filename);
                return Err(AppError::BadRequest(format!(
                    "File too large. Maximum: {}MB",
                    max_mb
                )));
            }

            // 收集前几个字节用于 magic bytes 检查
            if !magic_checked {
                first_chunk.extend_from_slice(&chunk);
                if first_chunk.len() >= 12 {
                    if !is_valid_media_magic(&first_chunk, &ext) {
                        drop(file);
                        let _ = fs::remove_file(&save_path).await;
                        tracing::warn!("Upload rejected: invalid magic bytes, filename={}, ext={}", filename, ext);
                        return Err(AppError::BadRequest(
                            "File content does not match expected media format".to_string(),
                        ));
                    }
                    magic_checked = true;
                }
            }

            file.write_all(&chunk)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to write file: {}", e)))?;
        }

        // 如果文件太小未触发 magic 检查，在此补检
        if !magic_checked && !is_valid_media_magic(&first_chunk, &ext) {
            drop(file);
            let _ = fs::remove_file(&save_path).await;
            tracing::warn!("Upload rejected: invalid magic bytes, filename={}, ext={}", filename, ext);
            return Err(AppError::BadRequest(
                "File content does not match expected media format".to_string(),
            ));
        }

        file.flush().await
            .map_err(|e| AppError::Internal(format!("Failed to flush file: {}", e)))?;

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

        tracing::info!("File uploaded: id={}, filename={}, size={}KB", file_id, filename, file_size / 1024);

        return Ok(Json(json!({
            "id": file_id,
            "filename": filename,
            "size": file_size,
            "url": format!("/api/files/{}", file_id),
        })));
    }

    tracing::warn!("Upload failed: no file in request");
    Err(AppError::BadRequest("No file uploaded".to_string()))
}

pub async fn download(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Response> {
    tracing::info!("File download request: id={}", id);

    let file = sqlx::query_as::<_, (String, String)>(
        "SELECT id, filename FROM files WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await?;

    let (file_id, filename) = file
        .ok_or_else(|| {
            tracing::warn!("Download failed: file id={} not found in DB", id);
            AppError::NotFound("File not found".to_string())
        })?;

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

    let path = file_path.ok_or_else(|| {
        tracing::warn!("Download failed: file id={} not found on disk", file_id);
        AppError::NotFound("File not found on disk".to_string())
    })?;

    let file_handle = fs::File::open(&path)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to open file: {}", e)))?;

    let metadata = file_handle.metadata().await
        .map_err(|e| AppError::Internal(format!("Failed to read file metadata: {}", e)))?;

    tracing::info!("File served: id={}, filename={}, size={}KB", file_id, filename, metadata.len() / 1024);

    let stream = tokio_util::io::ReaderStream::new(file_handle);
    let body = Body::from_stream(stream);

    let response = Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_LENGTH, metadata.len())
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(body)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(response)
}
