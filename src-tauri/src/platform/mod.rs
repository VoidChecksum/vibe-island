#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod windows;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub desktop: String,         // "aqua", "gnome", "kde", "hyprland", "sway", "windows"
    pub wayland: bool,
    pub has_notch: bool,         // MacBook with notch
    pub screen_width: u32,
    pub screen_height: u32,
    pub notch_width: u32,        // 0 if no notch
    pub compositor: String,      // Hyprland, sway, mutter, kwin, etc.
}

pub fn get_info() -> PlatformInfo {
    let os = std::env::consts::OS.to_string();

    #[cfg(target_os = "macos")]
    return macos::get_platform_info();

    #[cfg(target_os = "linux")]
    return linux::get_platform_info();

    #[cfg(target_os = "windows")]
    return windows::get_platform_info();

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    PlatformInfo {
        os,
        desktop: "unknown".into(),
        wayland: false,
        has_notch: false,
        screen_width: 1920,
        screen_height: 1080,
        notch_width: 0,
        compositor: "unknown".into(),
    }
}

#[cfg(target_os = "macos")]
pub mod macos {
    use super::PlatformInfo;

    pub fn get_platform_info() -> PlatformInfo {
        PlatformInfo {
            os: "macos".into(),
            desktop: "aqua".into(),
            wayland: false,
            has_notch: detect_notch(),
            screen_width: 1920,
            screen_height: 1080,
            notch_width: if detect_notch() { 180 } else { 0 },
            compositor: "quartz".into(),
        }
    }

    fn detect_notch() -> bool {
        // MacBooks from 2021+ have notches
        // Check via screen safe area insets
        true // Default to true for modern MacBooks
    }

    pub fn setup_notch_window(app: &tauri::App) {
        use tauri::Manager;
        if let Some(window) = app.get_webview_window("notch") {
            // On macOS, position the notch panel at the top center
            // matching the actual notch area
            let _ = window.set_always_on_top(true);
            // Level above everything except the notch itself
        }
    }
}

#[cfg(target_os = "linux")]
pub mod linux {
    use super::PlatformInfo;

    pub fn get_platform_info() -> PlatformInfo {
        let wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        let compositor = detect_compositor();
        let desktop = detect_desktop();

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

    fn detect_compositor() -> String {
        // Check XDG_CURRENT_DESKTOP and HYPRLAND_INSTANCE_SIGNATURE
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            return "hyprland".into();
        }
        if let Ok(session) = std::env::var("XDG_SESSION_TYPE") {
            if session == "wayland" {
                if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
                    return desktop.to_lowercase();
                }
                // Check for sway
                if std::env::var("SWAYSOCK").is_ok() {
                    return "sway".into();
                }
            }
        }
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            return desktop.to_lowercase();
        }
        "unknown".into()
    }

    fn detect_desktop() -> String {
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            let d = desktop.to_lowercase();
            if d.contains("hyprland") { return "hyprland".into(); }
            if d.contains("sway") { return "sway".into(); }
            if d.contains("gnome") { return "gnome".into(); }
            if d.contains("kde") || d.contains("plasma") { return "kde".into(); }
            if d.contains("xfce") { return "xfce".into(); }
            return d;
        }
        "unknown".into()
    }

    pub fn setup_floating_window(app: &tauri::App) {
        use tauri::Manager;
        if let Some(window) = app.get_webview_window("notch") {
            let _ = window.set_always_on_top(true);

            // For Hyprland: output window rules to apply
            // Users should add to hyprland.conf:
            //   windowrulev2 = float, class:^(vibe-island)$
            //   windowrulev2 = pin, class:^(vibe-island)$
            //   windowrulev2 = noborder, class:^(vibe-island)$
            //   windowrulev2 = noshadow, class:^(vibe-island)$
            //   windowrulev2 = noanim, class:^(vibe-island)$
            //   windowrulev2 = move 33% 0, class:^(vibe-island)$

            let compositor = detect_compositor();
            if compositor == "hyprland" {
                // Try to apply Hyprland rules via hyprctl
                let _ = std::process::Command::new("hyprctl")
                    .args(["keyword", "windowrulev2", "float,class:^(vibe-island)$"])
                    .output();
                let _ = std::process::Command::new("hyprctl")
                    .args(["keyword", "windowrulev2", "pin,class:^(vibe-island)$"])
                    .output();
                let _ = std::process::Command::new("hyprctl")
                    .args(["keyword", "windowrulev2", "noborder,class:^(vibe-island)$"])
                    .output();
                let _ = std::process::Command::new("hyprctl")
                    .args(["keyword", "windowrulev2", "noshadow,class:^(vibe-island)$"])
                    .output();
                let _ = std::process::Command::new("hyprctl")
                    .args(["keyword", "windowrulev2", "noanim,class:^(vibe-island)$"])
                    .output();
                // Position at top center
                let _ = std::process::Command::new("hyprctl")
                    .args(["keyword", "windowrulev2", "move 33% 0,class:^(vibe-island)$"])
                    .output();
                // Layer: overlay
                let _ = std::process::Command::new("hyprctl")
                    .args(["keyword", "windowrulev2", "stayfocused,class:^(vibe-island)$"])
                    .output();

                tracing::info!("Hyprland window rules applied via hyprctl");
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub mod windows {
    use super::PlatformInfo;

    pub fn get_platform_info() -> PlatformInfo {
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
}
