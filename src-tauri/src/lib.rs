mod config;
mod hooks;
mod sessions;
mod socket;
mod sound;
mod platform;

use sessions::SessionStore;
use socket::SocketServer;
use sound::SoundManager;
use hooks::HookInstaller;
use config::AppConfig;

use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

pub type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub sessions: SessionStore,
    pub config: AppConfig,
    pub sound: SoundManager,
}

#[tauri::command]
async fn get_sessions(state: tauri::State<'_, SharedState>) -> Result<Vec<sessions::Session>, String> {
    let s = state.read().await;
    Ok(s.sessions.list())
}

#[tauri::command]
async fn approve_permission(
    state: tauri::State<'_, SharedState>,
    session_id: String,
    decision: String,
    reason: Option<String>,
) -> Result<(), String> {
    let s = state.read().await;
    s.sessions
        .respond_permission(&session_id, &decision, reason.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn answer_question(
    state: tauri::State<'_, SharedState>,
    session_id: String,
    answers: serde_json::Value,
) -> Result<(), String> {
    let s = state.read().await;
    s.sessions
        .respond_question(&session_id, answers)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_config(state: tauri::State<'_, SharedState>) -> Result<AppConfig, String> {
    let s = state.read().await;
    Ok(s.config.clone())
}

#[tauri::command]
async fn update_config(
    state: tauri::State<'_, SharedState>,
    config: AppConfig,
) -> Result<(), String> {
    let mut s = state.write().await;
    s.config = config.clone();
    s.config.save().map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_hooks() -> Result<String, String> {
    let results = HookInstaller::install_all().map_err(|e| e.to_string())?;
    Ok(results.join("\n"))
}

#[tauri::command]
async fn play_sound(
    state: tauri::State<'_, SharedState>,
    sound_name: String,
) -> Result<(), String> {
    let s = state.read().await;
    s.sound.play(&sound_name).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_platform_info() -> Result<platform::PlatformInfo, String> {
    Ok(platform::get_info())
}

#[tauri::command]
async fn jump_to_terminal(
    state: tauri::State<'_, SharedState>,
    session_id: String,
) -> Result<(), String> {
    let s = state.read().await;
    let sessions = s.sessions.list_async().await;
    drop(s);
    if let Some(session) = sessions.into_iter().find(|s| s.id == session_id) {
        platform::jump_to_terminal(&session);
    }
    Ok(())
}

#[tauri::command]
async fn uninstall_hooks() -> Result<String, String> {
    let results = hooks::HookInstaller::uninstall_all().map_err(|e| e.to_string())?;
    Ok(results.join("\n"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Ensure run dir exists
    let run_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".vibe-island/run");
    std::fs::create_dir_all(&run_dir).ok();
    let osc2_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".vibe-island/cache/osc2-titles");
    std::fs::create_dir_all(&osc2_dir).ok();

    let config = AppConfig::load().unwrap_or_default();
    let state: SharedState = Arc::new(RwLock::new(AppState {
        sessions: SessionStore::new(),
        config,
        sound: SoundManager::new(),
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![
            get_sessions,
            approve_permission,
            answer_question,
            get_config,
            update_config,
            install_hooks,
            play_sound,
            get_platform_info,
            jump_to_terminal,
            uninstall_hooks,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let state = state.clone();

            // Start socket server
            tauri::async_runtime::spawn(async move {
                let server = SocketServer::new(state.clone(), handle.clone());
                if let Err(e) = server.start().await {
                    tracing::error!("Socket server error: {}", e);
                }
            });

            // Install hooks on first launch
            let config_state = app.state::<SharedState>().inner().clone();
            tauri::async_runtime::spawn(async move {
                let s = config_state.read().await;
                if s.config.auto_install_hooks {
                    drop(s);
                    if let Err(e) = HookInstaller::install_all() {
                        tracing::error!("Hook install error: {}", e);
                    }
                }
            });

            // Platform-specific window setup
            platform::setup_window(app);

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "notch" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Vibe Island");
}
