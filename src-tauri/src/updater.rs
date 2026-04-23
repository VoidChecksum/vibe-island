use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::UpdaterExt;

#[derive(Debug, serde::Serialize, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub body: Option<String>,
}

/// Called once on startup — silently checks for update and emits "update-available" if found.
pub async fn check_on_startup(app: AppHandle) {
    // Small delay so UI is ready
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    match app.updater() {
        Ok(updater) => {
            match updater.check().await {
                Ok(Some(update)) => {
                    let info = UpdateInfo {
                        version: update.version.clone(),
                        body: update.body.clone(),
                    };
                    let _ = app.emit("update-available", &info);
                    if let Ok(mut slot) = app
                        .state::<std::sync::Mutex<Option<PendingUpdate>>>()
                        .lock()
                    {
                        *slot = Some(PendingUpdate { version: update.version.clone() });
                    }
                    // Store the raw update bytes / metadata for later install
                    // We re-check on install_update instead of holding the Update object
                    // (Update is not Send across the tauri async boundary easily)
                }
                Ok(None) => {}
                Err(e) => tracing::debug!("Update check error: {}", e),
            }
        }
        Err(e) => tracing::debug!("Updater not configured: {}", e),
    }
}

#[derive(Clone)]
pub struct PendingUpdate {
    pub version: String,
}

/// Frontend calls this to check manually.
#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    match updater.check().await.map_err(|e| e.to_string())? {
        Some(update) => {
            let info = UpdateInfo {
                version: update.version.clone(),
                body: update.body.clone(),
            };
            if let Ok(mut slot) = app.state::<std::sync::Mutex<Option<PendingUpdate>>>().lock() {
                *slot = Some(PendingUpdate { version: update.version.clone() });
            }
            Ok(Some(info))
        }
        None => Ok(None),
    }
}

/// Downloads + installs then restarts.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    match updater.check().await.map_err(|e| e.to_string())? {
        Some(update) => {
            update
                .download_and_install(|_chunk, _total| {}, || {})
                .await
                .map_err(|e| e.to_string())?;
            app.restart();
        }
        None => {}
    }
    Ok(())
}
