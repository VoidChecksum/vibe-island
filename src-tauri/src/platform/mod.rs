use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub desktop: String,
    pub wayland: bool,
    pub has_notch: bool,
    pub screen_width: u32,
    pub screen_height: u32,
    pub notch_width: u32,
    pub compositor: String,
}

pub fn get_info() -> PlatformInfo {
    #[cfg(target_os = "macos")]
    {
        PlatformInfo {
            os: "macos".into(),
            desktop: "aqua".into(),
            wayland: false,
            has_notch: true,
            screen_width: 1920,
            screen_height: 1080,
            notch_width: 180,
            compositor: "quartz".into(),
        }
    }

    #[cfg(target_os = "linux")]
    {
        let wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        let compositor = if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            "hyprland".to_string()
        } else if std::env::var("SWAYSOCK").is_ok() {
            "sway".to_string()
        } else {
            std::env::var("XDG_CURRENT_DESKTOP")
                .unwrap_or_else(|_| "unknown".into())
                .to_lowercase()
        };
        let desktop = compositor.clone();

        PlatformInfo {
            os: "linux".into(),
            desktop,
            wayland,
            has_notch: false,
            screen_width: 1920,
            screen_height: 1080,
            notch_width: 0,
            compositor,
        }
    }

    #[cfg(target_os = "windows")]
    {
        PlatformInfo {
            os: "windows".into(),
            desktop: "windows".into(),
            wayland: false,
            has_notch: false,
            screen_width: 1920,
            screen_height: 1080,
            notch_width: 0,
            compositor: "dwm".into(),
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        PlatformInfo {
            os: std::env::consts::OS.into(),
            desktop: "unknown".into(),
            wayland: false,
            has_notch: false,
            screen_width: 1920,
            screen_height: 1080,
            notch_width: 0,
            compositor: "unknown".into(),
        }
    }
}

/// Platform-specific window setup
pub fn setup_window(app: &tauri::App) {
    use tauri::Manager;

    #[cfg(target_os = "macos")]
    {
        if let Some(window) = app.get_webview_window("notch") {
            let _ = window.set_always_on_top(true);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(window) = app.get_webview_window("notch") {
            let _ = window.set_always_on_top(true);

            // Hyprland: apply window rules via hyprctl
            if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
                for rule in &[
                    "float,class:^(vibe-island)$",
                    "pin,class:^(vibe-island)$",
                    "noborder,class:^(vibe-island)$",
                    "noshadow,class:^(vibe-island)$",
                    "noanim,class:^(vibe-island)$",
                    "move 33% 0,class:^(vibe-island)$",
                ] {
                    let _ = std::process::Command::new("hyprctl")
                        .args(["keyword", "windowrulev2", rule])
                        .output();
                }
                tracing::info!("Hyprland window rules applied");
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(window) = app.get_webview_window("notch") {
            let _ = window.set_always_on_top(true);
        }
    }
}
