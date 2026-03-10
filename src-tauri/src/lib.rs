use std::sync::Mutex;

use serde::Serialize;
use tauri::Url;

#[cfg(desktop)]
use tauri_plugin_updater::UpdaterExt;

#[derive(Default)]
struct PendingUpdate(Mutex<Option<tauri_plugin_updater::Update>>);

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
async fn check_for_update(
    app: tauri::AppHandle,
    pending_update: tauri::State<'_, PendingUpdate>,
) -> Result<Option<UpdatePayload>, String> {
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
        .map_err(|error| error.to_string())?;

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

    Ok(payload)
}

#[tauri::command]
async fn install_update(
    app: tauri::AppHandle,
    pending_update: tauri::State<'_, PendingUpdate>,
) -> Result<(), String> {
    let update = {
        let mut pending = pending_update
            .0
            .lock()
            .map_err(|_| "无法锁定待安装更新状态。".to_string())?;

        pending
            .take()
            .ok_or_else(|| "当前没有可安装的更新，请先执行一次检查更新。".to_string())?
    };

    update
        .download_and_install(
            |_chunk_length, _content_length| {},
            || {},
        )
        .await
        .map_err(|error| error.to_string())?;

    app.restart();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(PendingUpdate::default())
        .setup(|app| {
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            #[cfg(debug_assertions)]
            {
                let window = tauri::Manager::get_webview_window(app, "main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_prevent_default::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            updater_status,
            check_for_update,
            install_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
