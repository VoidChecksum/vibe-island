mod config;
mod hooks;
mod sessions;
mod socket;
mod sound;
mod platform;
mod updater;
mod ssh_remote;

use sessions::SessionStore;
use socket::SocketServer;
use sound::SoundManager;
use hooks::HookInstaller;
use config::AppConfig;

use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::RwLock;

pub type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub sessions: SessionStore,
    pub config: AppConfig,
    pub sound: SoundManager,
    pub ssh_remote: ssh_remote::SshRemoteManager,
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
    s.sound.configure(&config.sound);
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

/// Toggle Claude Code bypass/auto-approve mode for a session.
/// Writes CLAUDE_BYPASS_PERMISSIONS=1 env-flag into a well-known file
/// that the hook reads on next invocation.
#[tauri::command]
async fn set_bypass_mode(session_id: String, enabled: bool) -> Result<(), String> {
    let flag_dir = dirs::home_dir()
        .or_else(|| dirs::data_local_dir())
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".vibe-island/bypass-flags");
    std::fs::create_dir_all(&flag_dir).map_err(|e| e.to_string())?;
    let flag_file = flag_dir.join(&session_id);
    if enabled {
        std::fs::write(&flag_file, "1").map_err(|e| e.to_string())?;
    } else {
        let _ = std::fs::remove_file(&flag_file);
    }
    Ok(())
}

#[tauri::command]
async fn ssh_connect(
    state: tauri::State<'_, SharedState>,
    host: String,
    user: String,
    port: Option<u16>,
    key_path: Option<String>,
    remote_socket: Option<String>,
) -> Result<ssh_remote::RemoteInfo, String> {
    let s = state.read().await;
    s.ssh_remote.connect(host, user, port.unwrap_or(22), key_path, remote_socket).await
}

#[tauri::command]
async fn ssh_disconnect(state: tauri::State<'_, SharedState>, host: String) -> Result<(), String> {
    let s = state.read().await;
    s.ssh_remote.disconnect(&host).await
}

#[tauri::command]
async fn ssh_list_remotes(state: tauri::State<'_, SharedState>) -> Result<Vec<ssh_remote::RemoteInfo>, String> {
    let s = state.read().await;
    Ok(s.ssh_remote.list().await)
}

#[tauri::command]
async fn cleanup_sessions(state: tauri::State<'_, SharedState>) -> Result<usize, String> {
    let timeout = {
        let s = state.read().await;
        s.config.layout.session_idle_cleanup_secs
    };
    let s = state.read().await;
    Ok(s.sessions.cleanup_idle(timeout).await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Ensure run dir exists
    let vi_base = dirs::home_dir()
        .or_else(|| dirs::data_local_dir())
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".vibe-island");
    let run_dir = vi_base.join("run");
    std::fs::create_dir_all(&run_dir).ok();
    let osc2_dir = vi_base.join("cache/osc2-titles");
    std::fs::create_dir_all(&osc2_dir).ok();

    let config = AppConfig::load().unwrap_or_default();
    let sound = SoundManager::from_config(&config.sound);
    let state: SharedState = Arc::new(RwLock::new(AppState {
        sessions: SessionStore::new(),
        config,
        sound,
        ssh_remote: ssh_remote::SshRemoteManager::new(),
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .manage(state.clone())
        .manage(std::sync::Mutex::<Option<updater::PendingUpdate>>::new(None))
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
            set_bypass_mode,
            cleanup_sessions,
            ssh_connect,
            ssh_disconnect,
            ssh_list_remotes,
            updater::check_for_update,
            updater::install_update,
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

            // Background update check (emits "update-available" to frontend)
            tauri::async_runtime::spawn(updater::check_on_startup(app.handle().clone()));

            // Session idle cleanup loop
            let cleanup_state = app.state::<SharedState>().inner().clone();
            let cleanup_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    let timeout = {
                        let s = cleanup_state.read().await;
                        s.config.layout.session_idle_cleanup_secs
                    };
                    if timeout > 0 {
                        let s = cleanup_state.read().await;
                        let removed = s.sessions.cleanup_idle(timeout).await;
                        if removed > 0 {
                            drop(s);
                            let _ = cleanup_handle.emit("session-update", "cleanup");
                        }
                    }
                }
            });

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
