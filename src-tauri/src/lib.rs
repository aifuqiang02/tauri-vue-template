use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};

use chrono::Local;
use serde::Serialize;
use tauri::{Manager, Url};

#[cfg(desktop)]
use tauri_plugin_updater::UpdaterExt;

#[derive(Default)]
struct PendingUpdate(Mutex<Option<tauri_plugin_updater::Update>>);

struct AppLogger {
    directory: PathBuf,
    file_path: PathBuf,
    file: Mutex<File>,
}

impl AppLogger {
    fn new(log_dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&log_dir).map_err(|error| error.to_string())?;
        let file_path = log_dir.join("app.log");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|error| error.to_string())?;

        Ok(Self {
            directory: log_dir,
            file_path,
            file: Mutex::new(file),
        })
    }

    fn write(&self, level: &str, message: impl AsRef<str>) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("[{timestamp}] [{level}] {}\n", message.as_ref());

        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdaterStatus {
    enabled: bool,
    reason: Option<String>,
    current_version: String,
    endpoints: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePayload {
    version: String,
    current_version: String,
    date: Option<String>,
    body: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LogStatus {
    directory: String,
    file_path: String,
}

fn open_in_file_manager(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|error| error.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|error| error.to_string())?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn updater_public_key() -> Option<&'static str> {
    option_env!("TAURI_UPDATER_PUBLIC_KEY")
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn updater_endpoints() -> Vec<String> {
    option_env!("TAURI_UPDATER_ENDPOINTS")
        .map(|raw| {
            raw.split(['\n', '\r', ';'])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn parsed_updater_endpoints() -> Result<Vec<Url>, String> {
    updater_endpoints()
        .into_iter()
        .map(|endpoint| Url::parse(&endpoint).map_err(|error| error.to_string()))
        .collect()
}

fn updater_disabled_reason() -> Option<String> {
    if updater_public_key().is_none() {
        return Some("未配置更新公钥，应用内更新已禁用。".into());
    }

    if updater_endpoints().is_empty() {
        return Some("未配置更新地址，应用内更新已禁用。".into());
    }

    None
}

#[tauri::command]
fn greet(name: &str) -> String {
    println!("Backend was called with an argument: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn updater_status(app: tauri::AppHandle) -> UpdaterStatus {
    UpdaterStatus {
        enabled: updater_disabled_reason().is_none(),
        reason: updater_disabled_reason(),
        current_version: app.package_info().version.to_string(),
        endpoints: updater_endpoints(),
    }
}

#[tauri::command]
fn logger_status(logger: tauri::State<'_, AppLogger>) -> LogStatus {
    LogStatus {
        directory: logger.directory.display().to_string(),
        file_path: logger.file_path.display().to_string(),
    }
}

#[tauri::command]
fn open_logs_directory(logger: tauri::State<'_, AppLogger>) -> Result<(), String> {
    open_in_file_manager(&logger.directory)
}

#[tauri::command]
fn open_devtools(app: tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "找不到主窗口，无法打开调试控制台。".to_string())?;
    window.open_devtools();
    Ok(())
}

#[tauri::command]
async fn check_for_update(
    app: tauri::AppHandle,
    pending_update: tauri::State<'_, PendingUpdate>,
) -> Result<Option<UpdatePayload>, String> {
    let logger = app.state::<AppLogger>();
    logger.write("INFO", "开始检查更新");

    let pubkey = updater_public_key()
        .ok_or_else(|| "未配置更新公钥，请先设置 TAURI_UPDATER_PUBLIC_KEY。".to_string())?;
    let endpoints = parsed_updater_endpoints()?;

    if endpoints.is_empty() {
        return Err("未配置更新地址，请先设置 TAURI_UPDATER_ENDPOINTS。".into());
    }

    let update = app
        .updater_builder()
        .pubkey(pubkey)
        .endpoints(endpoints)
        .map_err(|error| error.to_string())?
        .build()
        .map_err(|error| error.to_string())?
        .check()
        .await
        .map_err(|error| {
            let message = error.to_string();
            logger.write("ERROR", format!("检查更新失败: {message}"));
            message
        })?;

    let payload = update.as_ref().map(|update| UpdatePayload {
        version: update.version.to_string(),
        current_version: update.current_version.to_string(),
        date: update.date.map(|date| date.to_string()),
        body: update.body.clone(),
    });

    let mut pending = pending_update
        .0
        .lock()
        .map_err(|_| "无法锁定待安装更新状态。".to_string())?;
    *pending = update;

    if let Some(update) = pending.as_ref() {
        logger.write(
            "INFO",
            format!(
                "发现新版本: current={}, latest={}",
                update.current_version, update.version
            ),
        );
    } else {
        logger.write("INFO", "未发现可用更新");
    }

    Ok(payload)
}

#[tauri::command]
async fn install_update(
    app: tauri::AppHandle,
    pending_update: tauri::State<'_, PendingUpdate>,
) -> Result<(), String> {
    let logger = app.state::<AppLogger>();
    let update = {
        let mut pending = pending_update
            .0
            .lock()
            .map_err(|_| "无法锁定待安装更新状态。".to_string())?;

        pending
            .take()
            .ok_or_else(|| "当前没有可安装的更新，请先执行一次检查更新。".to_string())?
    };

    logger.write(
        "INFO",
        format!(
            "开始安装更新: current={}, latest={}",
            update.current_version, update.version
        ),
    );

    update
        .download_and_install(
            |_chunk_length, _content_length| {},
            || {},
        )
        .await
        .map_err(|error| {
            let message = error.to_string();
            logger.write("ERROR", format!("安装更新失败: {message}"));
            message
        })?;

    logger.write("INFO", "更新安装完成，准备重启应用");
    app.restart();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(PendingUpdate::default())
        .setup(|app| {
            let log_dir = app
                .path()
                .app_log_dir()
                .map_err(|error| error.to_string())?;
            let logger = AppLogger::new(log_dir)?;
            logger.write("INFO", "应用启动");
            app.manage(logger);

            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            if cfg!(debug_assertions) {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_prevent_default::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            logger_status,
            open_logs_directory,
            open_devtools,
            updater_status,
            check_for_update,
            install_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
