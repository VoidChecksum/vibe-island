export interface Session {
  id: string;
  source: string;
  status: SessionStatus;
  cwd?: string;
  last_user_text?: string;
  last_assistant_message?: string;
  tool_name?: string;
  tool_input?: Record<string, unknown>;
  title?: string;
  env: Record<string, string>;
  tty?: string;
  terminal_bundle_id?: string;
  server_port?: number;
  started_at: string;
  last_activity: string;
  codex_model?: string;
  codex_permission_mode?: string;
  codex_thread_id?: string;
  codex_title?: string;
  subagent_parent_id?: string;
  subagent_nickname?: string;
  subagent_role?: string;
}

export type SessionStatus =
  | "active"
  | "idle"
  | "pending"
  | "in_progress"
  | "completed"
  | "waiting_for_approval"
  | "waiting_for_answer";

export interface PlatformInfo {
  os: string;
  desktop: string;
  wayland: boolean;
  has_notch: boolean;
  screen_width: number;
  screen_height: number;
  notch_width: number;
  compositor: string;
}

export interface AppConfig {
  auto_install_hooks: boolean;
  launch_at_login: boolean;
  display: {
    monitor: string;
    position: string;
    opacity: number;
    scale: number;
  };
  layout: {
    style: string;
    show_tool_names: boolean;
    show_session_time: boolean;
    show_cwd: boolean;
    max_visible_sessions: number;
    dwell_time_secs: number;
    expand_on_hover: boolean;
    hide_when_empty: boolean;
    expand_on_subagent_done: boolean;
    click_outside_dismisses: boolean;
    notch_follows_active_window: boolean;
    auto_configure_terminal_titles: boolean;
    session_idle_cleanup_secs: number;
    auto_collapse_on_leave: boolean;
    task_complete_dwell_ms: number;
    disable_click_to_jump: boolean;
    panel_height: number;
  };
  shortcuts: {
    toggle_panel: string;
    approve_all: string;
    deny_all: string;
    bypass_permissions: string;
  };
  sound: {
    enabled: boolean;
    volume: number;
    pack: string;
    events: {
      session_start: boolean;
      session_end: boolean;
      permission_request: boolean;
      approval: boolean;
      error: boolean;
      input_required: boolean;
      resource_limit: boolean;
      user_spam: boolean;
    };
    quiet_hours: {
      enabled: boolean;
      start: string;
      end: string;
    };
    filters: {
      only_when_backgrounded: boolean;
      suppress_repeated_events_secs: number;
    };
  };
  usage: {
    show_usage_limits: boolean;
    provider: string;
    value_mode: string;
  };
  labs: {
    beta_updates: boolean;
    auto_mode: boolean;
    cursor_approval: boolean;
    codex_desktop_alerts: boolean;
    kiro_hints: boolean;
  };
  terminal: {
    disable_click_to_jump: boolean;
    warp_tab_jump: boolean;
    disable_claude_native_title: boolean;
    custom_jump_rules: Array<{
      name: string;
      bundle_id: string;
      command: string;
    }>;
  };
  monitored_tools: string[];
}

export const TOOL_COLORS: Record<string, string> = {
  claude: "#D97706",
  codex: "#10B981",
  gemini: "#6366F1",
  cursor: "#EC4899",
  copilot: "#8B5CF6",
  opencode: "#06B6D4",
  windsurf: "#14B8A6",
  codebuddy: "#F59E0B",
  qoder: "#EF4444",
  droid: "#84CC16",
  amp: "#8B5CF6",
  kimi: "#EC4899",
  kiro: "#F59E0B",
  hermes: "#10B981",
  cline: "#22D3EE",
  "pi-cli": "#F97316",
};

export const TOOL_LABELS: Record<string, string> = {
  claude: "Claude",
  codex: "Codex",
  gemini: "Gemini",
  cursor: "Cursor",
  copilot: "Copilot",
  opencode: "OpenCode",
  windsurf: "Windsurf",
  codebuddy: "CodeBuddy",
  qoder: "Qoder",
  droid: "Droid",
  amp: "Amp",
  kimi: "Kimi",
  kiro: "Kiro",
  hermes: "Hermes",
  cline: "Cline",
  "pi-cli": "π CLI",
};
