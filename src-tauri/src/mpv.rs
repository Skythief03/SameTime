use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

#[cfg(unix)]
use std::os::unix::net::UnixStream;

pub struct MpvController {
    process: Mutex<Option<Child>>,
    /// Mutex to serialize IPC commands (prevents Windows named pipe PIPE_BUSY errors)
    ipc_lock: Mutex<()>,
    ipc_path: String,
}

impl MpvController {
    pub fn new() -> Self {
        let ipc_path = if cfg!(windows) {
            r"\\.\pipe\mpv-sametime".to_string()
        } else {
            "/tmp/mpv-sametime.sock".to_string()
        };

        Self {
            process: Mutex::new(None),
            ipc_lock: Mutex::new(()),
            ipc_path,
        }
    }

    fn send_command(&self, command: &str) -> Result<String, String> {
        // Serialize all IPC access to prevent concurrent pipe open on Windows
        let _lock = self.ipc_lock.lock().map_err(|e| format!("IPC lock error: {}", e))?;

        #[cfg(unix)]
        {
            let mut stream = UnixStream::connect(&self.ipc_path)
                .map_err(|e| format!("IPC connect error: {}", e))?;

            let cmd = format!("{}\n", command);
            stream
                .write_all(cmd.as_bytes())
                .map_err(|e| format!("IPC write error: {}", e))?;

            let mut reader = BufReader::new(stream);
            let mut response = String::new();
            reader
                .read_line(&mut response)
                .map_err(|e| format!("IPC read error: {}", e))?;

            Ok(response)
        }

        #[cfg(windows)]
        {
            use std::fs::OpenOptions;

            // Retry opening the pipe in case of transient PIPE_BUSY
            let mut pipe = None;
            for attempt in 0..5 {
                match OpenOptions::new().read(true).write(true).open(&self.ipc_path) {
                    Ok(p) => { pipe = Some(p); break; }
                    Err(_) if attempt < 4 => {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        continue;
                    }
                    Err(e) => return Err(format!("IPC connect error: {}", e)),
                }
            }
            let mut pipe = pipe.unwrap();

            let cmd = format!("{}\n", command);
            pipe.write_all(cmd.as_bytes())
                .map_err(|e| format!("IPC write error: {}", e))?;
            pipe.flush()
                .map_err(|e| format!("IPC flush error: {}", e))?;

            let mut reader = BufReader::new(pipe);
            let mut response = String::new();
            reader
                .read_line(&mut response)
                .map_err(|e| format!("IPC read error: {}", e))?;

            Ok(response)
        }
    }
}

impl Default for MpvController {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command]
pub fn mpv_play(file_path: String, state: tauri::State<MpvController>) -> Result<(), String> {
    // 先停止现有进程
    if let Ok(mut process) = state.process.lock() {
        if let Some(mut p) = process.take() {
            let _ = p.kill();
            let _ = p.wait();
        }
    }

    // 清理旧的 socket 文件
    #[cfg(unix)]
    {
        let _ = std::fs::remove_file(&state.ipc_path);
    }

    // 启动 mpv
    let mut cmd = Command::new("mpv");
    cmd.args([
        &format!("--input-ipc-server={}", state.ipc_path),
        "--pause",
        "--keep-open=yes",
        "--idle=no",
        "--force-window=yes",
        "--ontop",
        "--no-border",
        "--autofit=70%",
        "--geometry=50%:50%",
        &file_path,
    ])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null());

    // Windows: 防止弹出控制台窗口
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let child = cmd.spawn()
        .map_err(|e| format!("Failed to start mpv: {}", e))?;

    if let Ok(mut process) = state.process.lock() {
        *process = Some(child);
    }

    // 等待 IPC socket 就绪（最多 5 秒）
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(5);
    while start.elapsed() < timeout {
        if state.send_command(r#"{ "command": ["get_property", "pause"] }"#).is_ok() {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Err("mpv IPC not ready after 5s".to_string())
}

#[tauri::command]
pub fn mpv_stop(state: tauri::State<MpvController>) -> Result<(), String> {
    // 先尝试通过 IPC 优雅退出
    let _ = state.send_command(r#"{ "command": ["quit"] }"#);

    // 确保进程被终止
    if let Ok(mut process) = state.process.lock() {
        if let Some(mut p) = process.take() {
            let _ = p.kill();
            let _ = p.wait();
        }
    }

    // 清理 socket 文件
    #[cfg(unix)]
    {
        let _ = std::fs::remove_file(&state.ipc_path);
    }

    Ok(())
}

#[tauri::command]
pub fn mpv_seek(position: f64, state: tauri::State<MpvController>) -> Result<(), String> {
    let cmd = format!(
        r#"{{ "command": ["seek", {}, "absolute"] }}"#,
        position
    );
    state.send_command(&cmd)?;
    Ok(())
}

#[tauri::command]
pub fn mpv_set_pause(paused: bool, state: tauri::State<MpvController>) -> Result<(), String> {
    let cmd = format!(
        r#"{{ "command": ["set_property", "pause", {}] }}"#,
        paused
    );
    state.send_command(&cmd)?;
    Ok(())
}

#[tauri::command]
pub fn mpv_set_volume(volume: f64, state: tauri::State<MpvController>) -> Result<(), String> {
    let cmd = format!(
        r#"{{ "command": ["set_property", "volume", {}] }}"#,
        volume
    );
    state.send_command(&cmd)?;
    Ok(())
}

#[tauri::command]
pub fn mpv_get_position(state: tauri::State<MpvController>) -> Result<f64, String> {
    let cmd = r#"{ "command": ["get_property", "time-pos"] }"#;
    let response = state.send_command(cmd)?;

    // 解析 mpv JSON 响应
    let parsed: serde_json::Value =
        serde_json::from_str(&response).map_err(|e| format!("Parse error: {}", e))?;

    parsed["data"]
        .as_f64()
        .ok_or_else(|| "Failed to get position".to_string())
}

#[tauri::command]
pub fn mpv_check() -> Result<String, String> {
    let mut cmd = Command::new("mpv");
    cmd.arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd.output().map_err(|_| "mpv not found".to_string())?;
    let version = String::from_utf8_lossy(&output.stdout);
    let first_line = version.lines().next().unwrap_or("unknown").to_string();
    Ok(first_line)
}