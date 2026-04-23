use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub auto_install_hooks: bool,
    pub launch_at_login: bool,
    pub display: DisplayConfig,
    pub layout: LayoutConfig,
    pub shortcuts: ShortcutConfig,
    pub sound: SoundConfig,
    pub monitored_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub monitor: String, // "auto", "primary", "builtin"
    pub position: String, // "top-center", "top-left", "top-right"
    pub opacity: f32,
    pub scale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub style: String, // "clean", "detailed", "compact"
    pub show_tool_names: bool,
    pub show_session_time: bool,
    pub show_cwd: bool,
    pub max_visible_sessions: usize,
    pub dwell_time_secs: f32,
    pub expand_on_hover: bool,
    pub hide_when_empty: bool,
    pub expand_on_subagent_done: bool,
    pub click_outside_dismisses: bool,
    pub notch_follows_active_window: bool,
    pub auto_configure_terminal_titles: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub toggle_panel: String,
    pub approve_all: String,
    pub deny_all: String,
    pub bypass_permissions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundConfig {
    pub enabled: bool,
    pub volume: f32,
    pub pack: String,
    pub events: SoundEventConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundEventConfig {
    pub session_start: bool,
    pub session_end: bool,
    pub permission_request: bool,
    pub approval: bool,
    pub error: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_install_hooks: true,
            launch_at_login: false,
            display: DisplayConfig {
                monitor: "auto".into(),
                position: "top-center".into(),
                opacity: 0.95,
                scale: 1.0,
            },
            layout: LayoutConfig {
                style: "clean".into(),
                show_tool_names: true,
                show_session_time: true,
                show_cwd: false,
                max_visible_sessions: 8,
                dwell_time_secs: 4.0,
                expand_on_hover: true,
                hide_when_empty: false,
                expand_on_subagent_done: false,
                click_outside_dismisses: false,
                notch_follows_active_window: false,
                auto_configure_terminal_titles: false,
            },
            shortcuts: ShortcutConfig {
                toggle_panel: "CmdOrCtrl+Shift+V".into(),
                approve_all: "CmdOrCtrl+Shift+A".into(),
                deny_all: "CmdOrCtrl+Shift+D".into(),
                bypass_permissions: "CmdOrCtrl+Shift+B".into(),
            },
            sound: SoundConfig {
                enabled: true,
                volume: 0.5,
                pack: "default".into(),
                events: SoundEventConfig {
                    session_start: true,
                    session_end: false,
                    permission_request: true,
                    approval: true,
                    error: true,
                },
            },
            monitored_tools: vec![
                "claude".into(),
                "codex".into(),
                "gemini".into(),
                "cursor".into(),
                "windsurf".into(),
                "copilot".into(),
                "opencode".into(),
                "codebuddy".into(),
                "qoder".into(),
            ],
        }
    }
}

impl AppConfig {
    fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap().join(".config"))
            .join("vibe-island");
        fs::create_dir_all(&config_dir).ok();
        config_dir.join("config.json")
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        fs::write(&path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}
