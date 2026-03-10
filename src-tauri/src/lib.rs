use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

mod mpv;

/// 计算文件的部分 hash（快速校验）
/// 采样策略：文件头 + 中间 + 文件尾，各 1MB
#[tauri::command]
fn calculate_file_hash(file_path: String) -> Result<String, String> {
    let mut file = File::open(&file_path).map_err(|e| e.to_string())?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    let file_size = metadata.len();

    let mut hasher = Sha256::new();
    let sample_size: usize = 1024 * 1024; // 1MB
    let mut buffer = vec![0u8; sample_size];

    // 读取文件头
    let bytes_read = file.read(&mut buffer).map_err(|e| e.to_string())?;
    hasher.update(&buffer[..bytes_read]);

    // 读取文件中间
    if file_size > sample_size as u64 * 2 {
        let mid_pos = file_size / 2 - sample_size as u64 / 2;
        file.seek(SeekFrom::Start(mid_pos))
            .map_err(|e| e.to_string())?;
        let bytes_read = file.read(&mut buffer).map_err(|e| e.to_string())?;
        hasher.update(&buffer[..bytes_read]);
    }

    // 读取文件尾
    if file_size > sample_size as u64 {
        file.seek(SeekFrom::End(-(sample_size as i64)))
            .map_err(|e| e.to_string())?;
        let bytes_read = file.read(&mut buffer).map_err(|e| e.to_string())?;
        hasher.update(&buffer[..bytes_read]);
    }

    // 加入文件大小
    hasher.update(file_size.to_le_bytes());

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

#[tauri::command]
fn get_file_size(file_path: String) -> Result<u64, String> {
    let metadata = std::fs::metadata(&file_path).map_err(|e| e.to_string())?;
    Ok(metadata.len())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(mpv::MpvController::new())
        .invoke_handler(tauri::generate_handler![
            calculate_file_hash,
            get_file_size,
            mpv::mpv_play,
            mpv::mpv_stop,
            mpv::mpv_seek,
            mpv::mpv_set_pause,
            mpv::mpv_set_volume,
            mpv::mpv_get_position,
            mpv::mpv_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}