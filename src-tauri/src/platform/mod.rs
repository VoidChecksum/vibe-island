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

fn read_osc2_title(session_id: &str) -> Option<String> {
    let dir = dirs::home_dir()?.join(".vibe-island/cache/osc2-titles");
    let prefix = &session_id[..session_id.len().min(16)];
    std::fs::read_to_string(dir.join(prefix)).ok()
}

pub fn jump_to_terminal(session: &crate::sessions::Session) {
    let session_id = session.id.clone();
    let tty = session.tty.clone();
    let title_hint = read_osc2_title(&session_id)
        .or_else(|| session.title.clone())
        .or_else(|| session.last_user_text.clone().map(|t| t[..t.len().min(20)].to_string()));

    #[cfg(target_os = "macos")]
    {
        if let Some(ref tty_path) = tty {
            let tty_name = tty_path.trim_start_matches("/dev/");
            let script = format!(r#"tell application "iTerm2"
    if not (it is running) then return "not running"
    repeat with aWin in windows
        repeat with aTab in tabs of aWin
            repeat with aSess in sessions of aTab
                if tty of aSess contains "{tty}" then
                    tell aWin to select
                    set miniaturized of aWin to false
                    set selected tab of aWin to aTab
                    tell aSess to select
                    return "ok"
                end if
            end repeat
        end repeat
    end repeat
    return "not found"
end tell"#, tty = tty_name);
            let result = std::process::Command::new("osascript")
                .args(["-e", &script])
                .output();
            if let Ok(out) = result {
                let s = String::from_utf8_lossy(&out.stdout);
                if s.trim() == "ok" { return; }
            }
        }

        if let Some(ref tty_path) = tty {
            let tty_name = tty_path.trim_start_matches("/dev/");
            let script = format!(r#"tell application "Terminal"
    if not (it is running) then return "not running"
    repeat with aWin in windows
        repeat with aTab in tabs of aWin
            if tty of aTab contains "{tty}" then
                set selected tab of aWin to aTab
                tell aWin to activate
                return "ok"
            end if
        end repeat
    end repeat
    return "not found"
end tell"#, tty = tty_name);
            let result = std::process::Command::new("osascript")
                .args(["-e", &script])
                .output();
            if let Ok(out) = result {
                let s = String::from_utf8_lossy(&out.stdout);
                if s.trim() == "ok" { return; }
            }
        }

        let sentinel = format!("/tmp/vibe-island-ghostty-title-{}", &session_id[..session_id.len().min(8)]);
        if let Some(ref title) = title_hint {
            let _ = std::fs::write(&sentinel, title);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            if let Some(ref tty_path) = tty {
                let _ = std::process::Command::new("hyprctl")
                    .args(["dispatch", "focuswindow", &format!("title:{}", tty_path)])
                    .output();
                return;
            }
        }

        if let Ok(_) = std::env::var("TMUX") {
            if let Some(ref tty_path) = tty {
                let tty_name = tty_path.trim_start_matches("/dev/");
                let output = std::process::Command::new("tmux")
                    .args(["list-panes", "-a", "-F", "#{pane_tty} #{session_name}:#{window_index}.#{pane_index}"])
                    .output();
                if let Ok(out) = output {
                    for line in String::from_utf8_lossy(&out.stdout).lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 && parts[0].contains(tty_name) {
                            let target = parts[1];
                            let _ = std::process::Command::new("tmux")
                                .args(["select-pane", "-t", target])
                                .output();
                            return;
                        }
                    }
                }
            }
        }

        if let Some(ref title) = title_hint {
            let _ = std::process::Command::new("xdotool")
                .args(["search", "--name", title, "windowactivate"])
                .output();
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(ref title) = title_hint {
            let ps = format!(
                r#"Add-Type -Name Win -Namespace User32 -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);'; Get-Process | Where-Object {{ $_.MainWindowTitle -like '*{}*' }} | Select-Object -First 1 | ForEach-Object {{ [User32.Win]::SetForegroundWindow($_.MainWindowHandle) }}"#,
                title
            );
            let _ = std::process::Command::new("powershell")
                .args(["-Command", &ps])
                .output();
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
