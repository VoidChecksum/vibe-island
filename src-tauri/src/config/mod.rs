use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_true")]
    pub auto_install_hooks: bool,
    #[serde(default)]
    pub launch_at_login: bool,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub layout: LayoutConfig,
    #[serde(default)]
    pub shortcuts: ShortcutConfig,
    #[serde(default)]
    pub sound: SoundConfig,
    #[serde(default)]
    pub usage: UsageConfig,
    #[serde(default)]
    pub labs: LabsConfig,
    #[serde(default)]
    pub terminal: TerminalConfig,
    #[serde(default = "default_monitored_tools")]
    pub monitored_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub monitor: String,  // "auto", "primary", "builtin"
    pub position: String, // "top-center", "top-left", "top-right"
    pub opacity: f32,
    pub scale: f32,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            monitor: "auto".into(),
            position: "top-center".into(),
            opacity: 0.95,
            scale: 1.0,
        }
    }
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
    pub session_idle_cleanup_secs: u64,
    #[serde(default)]
    pub auto_collapse_on_leave: bool,
    #[serde(default)]
    pub task_complete_dwell_ms: u64,
    #[serde(default)]
    pub disable_click_to_jump: bool,
    #[serde(default)]
    pub panel_height: u32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
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
            session_idle_cleanup_secs: 300,
            auto_collapse_on_leave: true,
            task_complete_dwell_ms: 2500,
            disable_click_to_jump: false,
            panel_height: 36,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub toggle_panel: String,
    pub approve_all: String,
    pub deny_all: String,
    pub bypass_permissions: String,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            toggle_panel: "CmdOrCtrl+Shift+V".into(),
            approve_all: "CmdOrCtrl+Shift+A".into(),
            deny_all: "CmdOrCtrl+Shift+D".into(),
            bypass_permissions: "CmdOrCtrl+Shift+B".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundConfig {
    pub enabled: bool,
    pub volume: f32,
    pub pack: String,
    pub events: SoundEventConfig,
    #[serde(default)]
    pub quiet_hours: QuietHoursConfig,
    #[serde(default)]
    pub filters: SoundFilterConfig,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 0.5,
            pack: "builtin-8bit".into(),
            events: SoundEventConfig::default(),
            quiet_hours: QuietHoursConfig::default(),
            filters: SoundFilterConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundEventConfig {
    pub session_start: bool,
    pub session_end: bool,
    pub permission_request: bool,
    pub approval: bool,
    pub error: bool,
    #[serde(default = "default_true")]
    pub input_required: bool,
    #[serde(default = "default_true")]
    pub resource_limit: bool,
    #[serde(default)]
    pub user_spam: bool,
}

impl Default for SoundEventConfig {
    fn default() -> Self {
        Self {
            session_start: true,
            session_end: false,
            permission_request: true,
            approval: true,
            error: true,
            input_required: true,
            resource_limit: true,
            user_spam: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHoursConfig {
    pub enabled: bool,
    pub start: String,
    pub end: String,
}

impl Default for QuietHoursConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            start: "22:00".into(),
            end: "08:00".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundFilterConfig {
    pub only_when_backgrounded: bool,
    pub suppress_repeated_events_secs: u64,
}

impl Default for SoundFilterConfig {
    fn default() -> Self {
        Self {
            only_when_backgrounded: false,
            suppress_repeated_events_secs: 8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageConfig {
    pub show_usage_limits: bool,
    pub provider: String,
    pub value_mode: String,
}

impl Default for UsageConfig {
    fn default() -> Self {
        Self {
            show_usage_limits: true,
            provider: "auto".into(),
            value_mode: "remaining".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabsConfig {
    pub beta_updates: bool,
    pub auto_mode: bool,
    pub cursor_approval: bool,
    pub codex_desktop_alerts: bool,
    pub kiro_hints: bool,
}

impl Default for LabsConfig {
    fn default() -> Self {
        Self {
            beta_updates: false,
            auto_mode: false,
            cursor_approval: true,
            codex_desktop_alerts: true,
            kiro_hints: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub disable_click_to_jump: bool,
    pub warp_tab_jump: bool,
    pub disable_claude_native_title: bool,
    pub custom_jump_rules: Vec<CustomJumpRule>,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            disable_click_to_jump: false,
            warp_tab_jump: true,
            disable_claude_native_title: false,
            custom_jump_rules: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomJumpRule {
    pub name: String,
    pub bundle_id: String,
    pub command: String,
}

fn default_true() -> bool { true }

fn default_monitored_tools() -> Vec<String> {
    vec![
        "claude".into(),
        "codex".into(),
        "gemini".into(),
        "cursor".into(),
        "windsurf".into(),
        "copilot".into(),
        "opencode".into(),
        "codebuddy".into(),
        "qoder".into(),
        "droid".into(),
        "amp".into(),
        "kimi".into(),
        "kiro".into(),
        "hermes".into(),
        "cline".into(),
        "pi-cli".into(),
    ]
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_install_hooks: true,
            launch_at_login: false,
            display: DisplayConfig::default(),
            layout: LayoutConfig::default(),
            shortcuts: ShortcutConfig::default(),
            sound: SoundConfig::default(),
            usage: UsageConfig::default(),
            labs: LabsConfig::default(),
            terminal: TerminalConfig::default(),
            monitored_tools: default_monitored_tools(),
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
        let content = serde_json::to_string_pretty(self)?;
        // Symlink-safe: write to temp then rename (atomic, preserves symlinks on most FSes)
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, &content)?;
        fs::rename(&tmp, &path)?;
        Ok(())
    }
}
