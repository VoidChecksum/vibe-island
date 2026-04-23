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

/// Find the PID of the terminal emulator that owns the given TTY (Linux /proc scan).
#[cfg(target_os = "linux")]
fn tty_owner_pid(tty_path: &str) -> Option<u32> {
    let tty_name = tty_path.trim_start_matches("/dev/");
    let proc = std::fs::read_dir("/proc").ok()?;
    let mut best_pid: Option<u32> = None;
    for entry in proc.flatten() {
        let pid_str = entry.file_name();
        let pid: u32 = pid_str.to_string_lossy().parse().ok()?;
        let stat_path = format!("/proc/{}/stat", pid);
        if let Ok(stat) = std::fs::read_to_string(&stat_path) {
            // field 7 (0-indexed) is tty_nr — we use fd/0 → readlink instead
        }
        let _ = stat_path; // suppress unused warning
        let fd0 = format!("/proc/{}/fd/0", pid);
        if let Ok(link) = std::fs::read_link(&fd0) {
            if link.to_string_lossy().contains(tty_name) {
                best_pid = Some(pid);
            }
        }
    }
    best_pid
}

/// Focus a window by PID on X11 using xdotool.
#[cfg(target_os = "linux")]
fn focus_by_pid_x11(pid: u32) -> bool {
    let out = std::process::Command::new("xdotool")
        .args(["search", "--pid", &pid.to_string(), "--limit", "1", "windowactivate", "--sync"])
        .output();
    out.map(|o| o.status.success()).unwrap_or(false)
}

/// Focus a window by PID on Sway/wlroots Wayland.
#[cfg(target_os = "linux")]
fn focus_by_pid_sway(pid: u32) -> bool {
    let out = std::process::Command::new("swaymsg")
        .arg(format!("[pid={}] focus", pid))
        .output();
    out.map(|o| o.status.success()).unwrap_or(false)
}

/// Focus a window by PID on Hyprland.
#[cfg(target_os = "linux")]
fn focus_by_pid_hyprland(pid: u32) -> bool {
    let out = std::process::Command::new("hyprctl")
        .args(["dispatch", "focuswindow", &format!("pid:{}", pid)])
        .output();
    out.map(|o| o.status.success()).unwrap_or(false)
}

/// Focus a window by PID on KDE Plasma (Wayland or X11).
#[cfg(target_os = "linux")]
fn focus_by_pid_kde(pid: u32) -> bool {
    // KDE uses KWin scripting; simplest cross-version approach: wmctrl -ip on XWayland
    let out = std::process::Command::new("wmctrl")
        .args(["-l", "-p"])
        .output();
    if let Ok(o) = out {
        for line in String::from_utf8_lossy(&o.stdout).lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let Ok(p) = parts[2].parse::<u32>() {
                    if p == pid {
                        let wid = parts[0];
                        let _ = std::process::Command::new("wmctrl")
                            .args(["-ia", wid])
                            .output();
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// tmux: focus pane by TTY path.
#[cfg(target_os = "linux")]
fn focus_tmux_by_tty(tty_path: &str) -> bool {
    let tty_name = tty_path.trim_start_matches("/dev/");
    let output = std::process::Command::new("tmux")
        .args(["list-panes", "-a", "-F", "#{pane_tty} #{session_name}:#{window_index}.#{pane_index} #{pane_active}"])
        .output();
    if let Ok(out) = output {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[0].contains(tty_name) {
                let _ = std::process::Command::new("tmux")
                    .args(["select-pane", "-t", parts[1]])
                    .output();
                let _ = std::process::Command::new("tmux")
                    .args(["select-window", "-t", parts[1]])
                    .output();
                return true;
            }
        }
    }
    false
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

            // iTerm2
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
            let result = std::process::Command::new("osascript").args(["-e", &script]).output();
            if let Ok(out) = result {
                if String::from_utf8_lossy(&out.stdout).trim() == "ok" { return; }
            }

            // Terminal.app
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
            let result = std::process::Command::new("osascript").args(["-e", &script]).output();
            if let Ok(out) = result {
                if String::from_utf8_lossy(&out.stdout).trim() == "ok" { return; }
            }

            // Ghostty: write sentinel file (Ghostty reads it to focus the matching tab)
            let sentinel = format!("/tmp/vibe-island-ghostty-title-{}", &session_id[..session_id.len().min(8)]);
            if let Some(ref title) = title_hint {
                let _ = std::fs::write(&sentinel, title);
            }

            // Kitty: focus via kitty remote control (if KITTY_LISTEN_ON is set)
            if let Ok(listen) = std::env::var("KITTY_LISTEN_ON") {
                let _ = std::process::Command::new("kitty")
                    .args(["@", "--to", &listen, "focus-window"])
                    .output();
                return;
            }

            // Warp: generic activate by bundle id
            let script = r#"tell application id "dev.warp.Warp-Stable" to activate"#;
            let _ = std::process::Command::new("osascript").args(["-e", script]).output();
        }
    }

    #[cfg(target_os = "linux")]
    {
        // 1. tmux: works inside any terminal, highest priority
        if std::env::var("TMUX").is_ok() {
            if let Some(ref tty_path) = tty {
                if focus_tmux_by_tty(tty_path) { return; }
            }
        }

        // 2. Resolve terminal emulator PID from TTY (works for all terminals)
        let term_pid = tty.as_deref().and_then(tty_owner_pid);

        let compositor = if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            "hyprland"
        } else if std::env::var("SWAYSOCK").is_ok() {
            "sway"
        } else if std::env::var("KDE_FULL_SESSION").is_ok() || std::env::var("KDE_SESSION_VERSION").is_ok() {
            "kde"
        } else if std::env::var("WAYLAND_DISPLAY").is_ok() {
            "wayland-generic"
        } else {
            "x11"
        };

        if let Some(pid) = term_pid {
            let focused = match compositor {
                "hyprland" => focus_by_pid_hyprland(pid),
                "sway" => focus_by_pid_sway(pid),
                "kde" => focus_by_pid_kde(pid) || focus_by_pid_x11(pid),
                _ => focus_by_pid_x11(pid), // x11 or wayland-generic via XWayland
            };
            if focused { return; }
        }

        // 3. Fallback: title-based xdotool (X11 / XWayland) or swaymsg
        if let Some(ref title) = title_hint {
            if compositor == "sway" || compositor == "wayland-generic" {
                let _ = std::process::Command::new("swaymsg")
                    .arg(format!("[title=\"{}\"] focus", title))
                    .output();
            } else {
                let _ = std::process::Command::new("xdotool")
                    .args(["search", "--name", title, "windowactivate"])
                    .output();
            }
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
