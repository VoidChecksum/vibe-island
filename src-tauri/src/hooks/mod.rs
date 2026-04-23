use std::fs;
use std::path::PathBuf;

fn remove_managed_block(content: &str, sentinel: &str) -> String {
    let start = format!("# --- {} START", sentinel);
    let end = format!("# --- {} END", sentinel);
    let mut result = String::new();
    let mut skip = false;
    for line in content.lines() {
        if line.trim_start().starts_with(&start) {
            skip = true;
        }
        if !skip {
            result.push_str(line);
            result.push('\n');
        }
        if skip && line.trim_start().starts_with(&end) {
            skip = false;
        }
    }
    result
}

/// Installs hook scripts for all supported AI coding tools
pub struct HookInstaller;

impl HookInstaller {
    pub fn install_all() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::new();
        let home = dirs::home_dir().ok_or("No home directory")?;

        // Claude Code
        match Self::install_claude_hook(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Claude Code: {}", e)),
        }

        // Codex CLI
        match Self::install_codex_hook(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Codex: {}", e)),
        }

        // Gemini CLI
        match Self::install_gemini_hook(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Gemini: {}", e)),
        }

        // Cursor
        match Self::install_cursor_hook(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Cursor: {}", e)),
        }

        // OpenCode
        match Self::install_opencode_plugin(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("OpenCode: {}", e)),
        }

        match Self::install_amp_plugin(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Amp: {}", e)),
        }
        match Self::install_kimi_hook(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Kimi: {}", e)),
        }
        match Self::install_kiro_hook(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Kiro: {}", e)),
        }
        match Self::install_droid_hook(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Droid: {}", e)),
        }
        match Self::install_hermes_plugin(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Hermes: {}", e)),
        }

        match Self::install_cline_hook(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("CLINE: {}", e)),
        }

        match Self::install_picli_hook(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Pi-CLI: {}", e)),
        }

        match Self::install_copilot_hook(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Copilot: {}", e)),
        }

        match Self::install_windsurf_hook(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Windsurf: {}", e)),
        }

        match Self::install_codebuddy_hook(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("CodeBuddy: {}", e)),
        }

        match Self::install_qoder_hook(&home) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Qoder: {}", e)),
        }

        Ok(results)
    }

    fn install_copilot_hook(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // GitHub Copilot CLI stores config at ~/.config/gh-copilot/ or ~/.copilot/
        // It supports a hooks.json for lifecycle events
        let dirs_to_try = [
            home.join(".config/gh-copilot"),
            home.join(".copilot"),
        ];
        let config_dir = dirs_to_try.iter().find(|d| d.exists()).cloned()
            .unwrap_or_else(|| dirs_to_try[0].clone());
        fs::create_dir_all(&config_dir)?;
        let hook_path = config_dir.join("vibe-island-hook.py");
        fs::write(&hook_path, CLAUDE_HOOK_PY)?;
        let hooks_path = config_dir.join("hooks.json");
        let mut hooks: serde_json::Value = if hooks_path.exists() {
            serde_json::from_str(&fs::read_to_string(&hooks_path)?).unwrap_or(serde_json::json!({}))
        } else { serde_json::json!({}) };
        let obj = hooks.as_object_mut().ok_or("not an object")?;
        for event in &["PreToolUse", "PostToolUse", "SessionStart", "Stop", "Notification"] {
            obj.entry(*event).or_insert(serde_json::json!([{
                "type": "command",
                "command": format!("python3 {}", hook_path.display()),
                "timeout": 300000
            }]));
        }
        fs::write(&hooks_path, serde_json::to_string_pretty(&hooks)?)?;
        Ok("Copilot: hooks installed".into())
    }

    fn install_windsurf_hook(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Windsurf stores settings at ~/.windsurf/ and follows a similar pattern to VS Code
        let config_dir = home.join(".windsurf");
        if !config_dir.exists() {
            return Ok("Windsurf: not installed, skipped".into());
        }
        let hook_path = config_dir.join("vibe-island-hook.py");
        fs::write(&hook_path, CLAUDE_HOOK_PY)?;
        let settings_path = config_dir.join("settings.json");
        let mut settings: serde_json::Value = if settings_path.exists() {
            serde_json::from_str(&fs::read_to_string(&settings_path)?).unwrap_or(serde_json::json!({}))
        } else { serde_json::json!({}) };
        let obj = settings.as_object_mut().ok_or("not an object")?;
        obj.entry("vibe_island_hook").or_insert(serde_json::json!({
            "command": format!("python3 {}", hook_path.display()),
            "events": ["SessionStart", "SessionEnd", "PreToolUse", "PostToolUse"]
        }));
        fs::write(&settings_path, serde_json::to_string_pretty(&settings)?)?;
        Ok("Windsurf: hooks installed".into())
    }

    fn install_codebuddy_hook(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // CodeBuddy stores config at ~/.codebuddy/
        let config_dir = home.join(".codebuddy");
        if !config_dir.exists() {
            return Ok("CodeBuddy: not installed, skipped".into());
        }
        let hook_path = config_dir.join("vibe-island-hook.py");
        fs::write(&hook_path, CLAUDE_HOOK_PY)?;
        let config_path = config_dir.join("config.json");
        let mut config: serde_json::Value = if config_path.exists() {
            serde_json::from_str(&fs::read_to_string(&config_path)?).unwrap_or(serde_json::json!({}))
        } else { serde_json::json!({}) };
        let obj = config.as_object_mut().ok_or("not an object")?;
        obj.entry("vibe_island_hook").or_insert(serde_json::json!(format!("python3 {}", hook_path.display())));
        fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
        Ok("CodeBuddy: hooks installed".into())
    }

    fn install_qoder_hook(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Qoder stores config at ~/.qoder/
        let config_dir = home.join(".qoder");
        if !config_dir.exists() {
            return Ok("Qoder: not installed, skipped".into());
        }
        let hook_path = config_dir.join("vibe-island-hook.py");
        fs::write(&hook_path, CLAUDE_HOOK_PY)?;
        let config_path = config_dir.join("config.json");
        let mut config: serde_json::Value = if config_path.exists() {
            serde_json::from_str(&fs::read_to_string(&config_path)?).unwrap_or(serde_json::json!({}))
        } else { serde_json::json!({}) };
        let obj = config.as_object_mut().ok_or("not an object")?;
        obj.entry("vibe_island_hook").or_insert(serde_json::json!(format!("python3 {}", hook_path.display())));
        fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
        Ok("Qoder: hooks installed".into())
    }

    fn install_claude_hook(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let settings_path = home.join(".claude/settings.json");
        let hook_path = home.join(".claude/vibe-island-hook.py");

        // Write the hook script
        fs::create_dir_all(home.join(".claude"))?;
        fs::write(&hook_path, CLAUDE_HOOK_PY)?;

        // Update settings.json to register the hook
        let mut settings: serde_json::Value = if settings_path.exists() {
            let content = fs::read_to_string(&settings_path)?;
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        // Add hooks if not present
        let hooks = settings
            .as_object_mut()
            .ok_or("settings not an object")?
            .entry("hooks")
            .or_insert(serde_json::json!({}));

        let hook_entry = serde_json::json!([{
            "type": "command",
            "command": format!("python3 {}", hook_path.display()),
            "timeout": 300000
        }]);

        let hooks_obj = hooks.as_object_mut().ok_or("hooks not an object")?;

        for event in &["SessionStart", "UserPromptSubmit", "PreToolUse", "PostToolUse", "Notification", "Stop"] {
            if !hooks_obj.contains_key(*event) {
                hooks_obj.insert(event.to_string(), hook_entry.clone());
            }
        }

        fs::write(&settings_path, serde_json::to_string_pretty(&settings)?)?;

        Ok("Claude Code: hooks installed".into())
    }

    fn install_codex_hook(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let hooks_path = home.join(".codex/hooks.json");
        let hook_path = home.join(".codex/vibe-island-hook.py");

        fs::create_dir_all(home.join(".codex"))?;
        fs::write(&hook_path, CODEX_HOOK_PY)?;

        // Codex uses a different hooks format
        let mut hooks: serde_json::Value = if hooks_path.exists() {
            let content = fs::read_to_string(&hooks_path)?;
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        let hooks_obj = hooks.as_object_mut().ok_or("not an object")?;
        if !hooks_obj.contains_key("vibe-island") {
            hooks_obj.insert(
                "vibe-island".to_string(),
                serde_json::json!({
                    "command": format!("python3 {}", hook_path.display()),
                    "events": ["session.start", "session.end", "tool.start", "tool.end"]
                }),
            );
        }

        fs::write(&hooks_path, serde_json::to_string_pretty(&hooks)?)?;
        Ok("Codex: hooks installed".into())
    }

    fn install_gemini_hook(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let settings_path = home.join(".gemini/settings.json");
        let hook_path = home.join(".gemini/vibe-island-hook.py");

        fs::create_dir_all(home.join(".gemini"))?;
        fs::write(&hook_path, GEMINI_HOOK_PY)?;

        let mut settings: serde_json::Value = if settings_path.exists() {
            let content = fs::read_to_string(&settings_path)?;
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        let obj = settings.as_object_mut().ok_or("not an object")?;
        if !obj.contains_key("hooks") {
            obj.insert(
                "hooks".to_string(),
                serde_json::json!({
                    "vibe-island": {
                        "command": format!("python3 {}", hook_path.display())
                    }
                }),
            );
        }

        fs::write(&settings_path, serde_json::to_string_pretty(&settings)?)?;
        Ok("Gemini: hooks installed".into())
    }

    fn install_cursor_hook(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let hooks_path = home.join(".cursor/hooks.json");

        if !home.join(".cursor").exists() {
            return Ok("Cursor: not installed, skipped".into());
        }

        let hook_path = home.join(".cursor/vibe-island-hook.py");
        fs::write(&hook_path, CLAUDE_HOOK_PY)?; // Same format as Claude

        let mut hooks: serde_json::Value = if hooks_path.exists() {
            let content = fs::read_to_string(&hooks_path)?;
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        let obj = hooks.as_object_mut().ok_or("not an object")?;
        if !obj.contains_key("vibe-island") {
            obj.insert(
                "vibe-island".to_string(),
                serde_json::json!({
                    "command": format!("python3 {}", hook_path.display()),
                    "events": ["*"]
                }),
            );
        }

        fs::write(&hooks_path, serde_json::to_string_pretty(&hooks)?)?;
        Ok("Cursor: hooks installed".into())
    }

    fn install_opencode_plugin(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let plugin_dir = home.join(".config/opencode/plugins");
        let plugin_path = plugin_dir.join("vibe-island.js");

        fs::create_dir_all(&plugin_dir)?;
        fs::write(&plugin_path, OPENCODE_PLUGIN_JS)?;

        Ok("OpenCode: plugin installed".into())
    }

    fn install_amp_plugin(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let plugin_dir = home.join(".amp/plugins");
        let plugin_path = plugin_dir.join("vibe-island.js");
        fs::create_dir_all(&plugin_dir)?;
        fs::write(&plugin_path, AMP_PLUGIN_JS)?;
        Ok("Amp: plugin installed".into())
    }

    fn install_kimi_hook(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let config_path = home.join(".kimi/config.toml");
        fs::create_dir_all(home.join(".kimi"))?;
        let hook_path = home.join(".kimi/vibe-island-hook.py");
        fs::write(&hook_path, GEMINI_HOOK_PY)?;
        let existing = if config_path.exists() {
            fs::read_to_string(&config_path)?
        } else {
            String::new()
        };
        if !existing.contains("vibe-island Kimi hooks START") {
            let block = format!(
                "\n# --- vibe-island Kimi hooks START (managed, do not edit) ---\n[hooks]\npre_tool_call = \"python3 {}\"\npost_tool_call = \"python3 {}\"\n# --- vibe-island Kimi hooks END ---\n",
                hook_path.display(), hook_path.display()
            );
            let mut content = existing;
            content.push_str(&block);
            fs::write(&config_path, content)?;
        }
        Ok("Kimi: hooks installed".into())
    }

    fn install_kiro_hook(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let agents_dir = home.join(".kiro/agents");
        if !home.join(".kiro").exists() {
            return Ok("Kiro: not installed, skipped".into());
        }
        fs::create_dir_all(&agents_dir)?;
        let hook_path = home.join(".kiro/vibe-island-hook.py");
        fs::write(&hook_path, CLAUDE_HOOK_PY)?;
        let agent_path = agents_dir.join("vibe-island.json");
        fs::write(&agent_path, serde_json::to_string_pretty(&serde_json::json!({
            "name": "vibe-island",
            "description": "Vibe Island session monitor",
            "hooks": {
                "on_tool_call": format!("python3 {}", hook_path.display()),
                "on_session_start": format!("python3 {}", hook_path.display()),
                "on_session_end": format!("python3 {}", hook_path.display())
            }
        }))?)?;
        Ok("Kiro: agent hooks installed".into())
    }

    fn install_droid_hook(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !home.join(".droid").exists() {
            return Ok("Droid: not installed, skipped".into());
        }
        let config_path = home.join(".droid/config.json");
        let hook_path = home.join(".droid/vibe-island-hook.py");
        fs::write(&hook_path, CLAUDE_HOOK_PY)?;
        let mut config: serde_json::Value = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else { serde_json::json!({}) };
        let obj = config.as_object_mut().ok_or("not an object")?;
        obj.entry("vibe_island_hook").or_insert(serde_json::json!(format!("python3 {}", hook_path.display())));
        fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
        Ok("Droid: hooks installed".into())
    }

    fn install_hermes_plugin(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let plugin_dir = home.join(".hermes/plugins");
        let plugin_path = plugin_dir.join("vibe-island");
        fs::create_dir_all(&plugin_dir)?;
        fs::write(&plugin_path, HERMES_PLUGIN_JS)?;
        Ok("Hermes: plugin installed".into())
    }

    fn install_cline_hook(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // CLINE stores its MCP/hook config in ~/.cline/settings.json
        // It also respects a hooks directory: ~/.cline/hooks/
        let hook_dir = home.join(".cline/hooks");
        fs::create_dir_all(&hook_dir)?;
        let hook_path = hook_dir.join("vibe-island.py");
        fs::write(&hook_path, CLINE_HOOK_PY)?;

        // Update ~/.cline/settings.json
        let settings_path = home.join(".cline/settings.json");
        let mut settings: serde_json::Value = if settings_path.exists() {
            serde_json::from_str(&fs::read_to_string(&settings_path)?).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };
        let obj = settings.as_object_mut().ok_or("not object")?;
        obj.entry("hookCommand").or_insert(serde_json::json!(
            format!("python3 {}", hook_path.display())
        ));
        obj.entry("hookEvents").or_insert(serde_json::json!(
            ["PreToolUse", "PostToolUse", "Notification", "Stop"]
        ));
        fs::write(&settings_path, serde_json::to_string_pretty(&settings)?)?;
        Ok("CLINE: hook installed".into())
    }

    fn install_picli_hook(home: &PathBuf) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Pi-CLI (pi.ai CLI / Inflection Pi) stores config at ~/.pi/config.json
        let config_dir = home.join(".pi");
        fs::create_dir_all(&config_dir)?;
        let hook_path = config_dir.join("vibe-island-hook.py");
        fs::write(&hook_path, PICLI_HOOK_PY)?;

        let config_path = config_dir.join("config.json");
        let mut config: serde_json::Value = if config_path.exists() {
            serde_json::from_str(&fs::read_to_string(&config_path)?).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };
        let obj = config.as_object_mut().ok_or("not object")?;
        obj.entry("hook").or_insert(serde_json::json!({
            "command": format!("python3 {}", hook_path.display()),
            "events": ["session_start", "session_end", "tool_call", "tool_result"]
        }));
        fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
        Ok("Pi-CLI: hook installed".into())
    }

    pub fn uninstall_all() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::new();
        let home = dirs::home_dir().ok_or("No home directory")?;

        let _ = fs::remove_file(home.join(".claude/vibe-island-hook.py"));
        let settings_path = home.join(".claude/settings.json");
        if settings_path.exists() {
            if let Ok(content) = fs::read_to_string(&settings_path) {
                if let Ok(mut settings) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(hooks) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
                        for event in &["PreToolUse", "PostToolUse", "Notification", "Stop"] {
                            if let Some(arr) = hooks.get_mut(*event).and_then(|v| v.as_array_mut()) {
                                arr.retain(|e| e.get("command").and_then(|c| c.as_str())
                                    .map(|c| !c.contains("vibe-island")).unwrap_or(true));
                            }
                        }
                    }
                    let _ = fs::write(&settings_path, serde_json::to_string_pretty(&settings).unwrap_or_default());
                }
            }
        }
        results.push("Claude Code: uninstalled".into());

        let _ = fs::remove_file(home.join(".codex/vibe-island-hook.py"));
        results.push("Codex: uninstalled".into());

        let _ = fs::remove_file(home.join(".gemini/vibe-island-hook.py"));
        results.push("Gemini: uninstalled".into());

        let _ = fs::remove_file(home.join(".config/opencode/plugins/vibe-island.js"));
        results.push("OpenCode: uninstalled".into());

        let _ = fs::remove_file(home.join(".amp/plugins/vibe-island.js"));
        results.push("Amp: uninstalled".into());

        let kimi_config = home.join(".kimi/config.toml");
        if kimi_config.exists() {
            if let Ok(content) = fs::read_to_string(&kimi_config) {
                let cleaned = remove_managed_block(&content, "vibe-island Kimi hooks");
                let _ = fs::write(&kimi_config, cleaned);
            }
        }
        let _ = fs::remove_file(home.join(".kimi/vibe-island-hook.py"));
        results.push("Kimi: uninstalled".into());

        let _ = fs::remove_file(home.join(".kiro/agents/vibe-island.json"));
        let _ = fs::remove_file(home.join(".kiro/vibe-island-hook.py"));
        results.push("Kiro: uninstalled".into());

        let _ = fs::remove_file(home.join(".droid/vibe-island-hook.py"));
        results.push("Droid: uninstalled".into());

        let _ = fs::remove_file(home.join(".hermes/plugins/vibe-island"));
        results.push("Hermes: uninstalled".into());

        for cli in &["code", "cursor", "windsurf"] {
            let _ = std::process::Command::new(cli)
                .args(["--uninstall-extension", "vibe-island.terminal-focus"])
                .output();
        }
        results.push("VS Code extensions: uninstalled".into());

        Ok(results)
    }
}

/// Claude Code / Cursor hook script (Python)
const CLAUDE_HOOK_PY: &str = r#"#!/usr/bin/env python3
"""Vibe Island hook — forwards events to ~/.vibe-island/run/vibe-island.sock"""
import json, os, socket, sys, subprocess

_PROTO_REV = "vi-e7c4"
_COMPAT = 0x3A7F

SOCKET_PATH = os.path.expanduser("~/.vibe-island/run/vibe-island.sock")
FALLBACK_SOCKET = "/tmp/vibe-island.sock"
OSC2_DIR = os.path.expanduser("~/.vibe-island/cache/osc2-titles")

def get_tty():
    try:
        if os.isatty(0):
            return os.ttyname(0)
    except Exception:
        pass
    try:
        pid = os.getpid()
        out = subprocess.check_output(["ps", "-o", "tty=", "-p", str(pid)], timeout=1).decode().strip()
        if out and out not in ("??", "?"):
            return "/dev/" + out
    except Exception:
        pass
    return None

def write_osc2(session_id, cwd, user_text):
    try:
        os.makedirs(OSC2_DIR, exist_ok=True)
        prefix = session_id[:16]
        project = (cwd or "").rstrip("/").rsplit("/", 1)[-1] or "session"
        prompt_preview = (user_text or "")[:30]
        title = (project + ": " + prompt_preview) if prompt_preview else project
        with open(os.path.join(OSC2_DIR, prefix), "w") as f:
            f.write(title)
    except Exception:
        pass

def send_event(data, held=False):
    msg = json.dumps(data).encode()
    if sys.platform == "win32":
        return _send_named_pipe(msg, held)
    for path in [SOCKET_PATH, FALLBACK_SOCKET]:
        try:
            s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            s.settimeout(300 if held else 3)
            s.connect(path)
            s.sendall(msg)
            if held:
                chunks = []
                while True:
                    chunk = s.recv(4096)
                    if not chunk:
                        break
                    chunks.append(chunk)
                s.close()
                return b"".join(chunks).decode() if chunks else None
            s.close()
            return True
        except Exception:
            continue
    return None

def _send_named_pipe(msg, held):
    import ctypes
    k32 = ctypes.windll.kernel32
    PIPE = r"\\.\pipe\vibe-island"
    GENERIC_RW = 0xC0000000
    OPEN_EXISTING = 3
    h = k32.CreateFileW(PIPE, GENERIC_RW, 0, None, OPEN_EXISTING, 0, None)
    if h in (0, 0xFFFFFFFF): return None
    try:
        written = ctypes.c_ulong(0)
        k32.WriteFile(h, msg, len(msg), ctypes.byref(written), None)
        if held:
            buf = ctypes.create_string_buffer(65536)
            rd = ctypes.c_ulong(0)
            k32.ReadFile(h, buf, 65536, ctypes.byref(rd), None)
            return buf.raw[:rd.value].decode() if rd.value else None
        return True
    except Exception:
        return None
    finally:
        k32.CloseHandle(h)

def main():
    hook_event = os.environ.get("CLAUDE_HOOK_EVENT", os.environ.get("HOOK_EVENT", ""))
    session_id = os.environ.get("CLAUDE_SESSION_ID", os.environ.get("SESSION_ID", ""))
    cwd = os.environ.get("CLAUDE_CWD", os.environ.get("CWD", os.getcwd()))
    tool_name = os.environ.get("CLAUDE_TOOL_NAME", os.environ.get("TOOL_NAME", ""))
    tool_input_raw = os.environ.get("CLAUDE_TOOL_INPUT", "{}")
    user_prompt = os.environ.get("CLAUDE_USER_PROMPT", "")

    # Always try to read stdin — Claude Code sends JSON on stdin for every event.
    # Merge: stdin wins for fields that env vars don't provide.
    try:
        stdin_data = json.load(sys.stdin)
        hook_event = hook_event or stdin_data.get("hook_event_name", stdin_data.get("type", ""))
        session_id = session_id or stdin_data.get("session_id", "")
        cwd = cwd or stdin_data.get("cwd", os.getcwd())
        tool_name = tool_name or stdin_data.get("tool_name", "")
        if not tool_input_raw or tool_input_raw == "{}":
            tool_input_raw = json.dumps(stdin_data.get("tool_input", {}))
        user_prompt = user_prompt or stdin_data.get("prompt", "")
    except Exception:
        pass

    if not hook_event or not session_id:
        return

    try:
        tool_input = json.loads(tool_input_raw) if tool_input_raw else {}
    except Exception:
        tool_input = {}

    tty = get_tty()
    bypass = os.environ.get("CLAUDE_BYPASS_PERMISSIONS", "") == "1"
    is_held = hook_event in ("PreToolUse",) and tool_name == "AskUserQuestion"
    is_perm = hook_event == "PreToolUse" and tool_name not in ("", None, "AskUserQuestion") and not bypass

    if hook_event == "UserPromptSubmit" and user_prompt:
        write_osc2(session_id, cwd, user_prompt)

    env_snapshot = {k: v for k, v in os.environ.items()
                    if k.startswith(("TERM", "TMUX", "SSH_", "HYPRLAND", "WAYLAND", "XDG_", "DISPLAY", "COLORTERM", "CLAUDE_BYPASS", "BYPASS_PERM"))}

    # Remap to PermissionRequest so the Rust server holds the connection and
    # routes through the approval/question UI. Without this the server closes
    # the socket immediately and All clicks are silent no-ops.
    wire_event_name = "PermissionRequest" if (is_held or is_perm) else hook_event

    event = {
        "session_id": session_id,
        "hook_event_name": wire_event_name,
        "cwd": cwd,
        "tool_name": tool_name,
        "tool_input": tool_input,
        "prompt": user_prompt or None,
        "_proto_rev": _PROTO_REV,
        "_compat": _COMPAT,
        "_source": "claude",
        "_ppid": os.getpid(),
        "_tty": tty,
        "_env": env_snapshot,
    }

    result = send_event(event, held=(is_held or is_perm))
    if result and isinstance(result, str):
        try:
            resp = json.loads(result)
            decision = resp.get("hookSpecificOutput", {}).get("decision", {})
            if decision:
                print(json.dumps(resp))
        except Exception:
            pass

if __name__ == "__main__":
    main()
"#;

/// Codex CLI hook script
const CODEX_HOOK_PY: &str = r#"#!/usr/bin/env python3
"""Vibe Island hook for Codex CLI"""
import json, os, socket, sys

_PROTO_REV = "vi-e7c4"
_COMPAT = 0x3A7F
SOCKET_PATH = os.path.expanduser("~/.vibe-island/run/vibe-island.sock")
FALLBACK_SOCKET = "/tmp/vibe-island.sock"

def send_event(data):
    msg = json.dumps(data).encode()
    if sys.platform == "win32":
        try:
            import ctypes
            k32 = ctypes.windll.kernel32
            h = k32.CreateFileW(r"\\.\pipe\vibe-island", 0xC0000000, 0, None, 3, 0, None)
            if h not in (0, 0xFFFFFFFF):
                k32.WriteFile(h, msg, len(msg), ctypes.byref(ctypes.c_ulong(0)), None)
                k32.CloseHandle(h)
        except Exception: pass
        return
    for path in [SOCKET_PATH, FALLBACK_SOCKET]:
        try:
            s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            s.settimeout(3)
            s.connect(path)
            s.sendall(msg)
            s.close()
            return
        except Exception:
            continue

def main():
    try:
        data = json.load(sys.stdin)
    except Exception:
        return
    event_type = data.get("type", "")
    event_map = {"session.start": "SessionStart", "session.end": "SessionEnd",
                 "tool.start": "PreToolUse", "tool.end": "PostToolUse"}
    hook_event = event_map.get(event_type, event_type)
    event = {"session_id": data.get("session_id", ""), "hook_event_name": hook_event,
             "cwd": data.get("cwd", os.getcwd()), "tool_name": data.get("tool_name", ""),
             "_proto_rev": _PROTO_REV, "_compat": _COMPAT, "_source": "codex", "_ppid": os.getpid()}
    send_event(event)

if __name__ == "__main__":
    main()
"#;

/// Gemini CLI hook script
const GEMINI_HOOK_PY: &str = r#"#!/usr/bin/env python3
"""Vibe Island hook for Gemini CLI"""
import json, os, socket, sys

_PROTO_REV = "vi-e7c4"
_COMPAT = 0x3A7F
SOCKET_PATH = os.path.expanduser("~/.vibe-island/run/vibe-island.sock")
FALLBACK_SOCKET = "/tmp/vibe-island.sock"

def send_event(data):
    msg = json.dumps(data).encode()
    if sys.platform == "win32":
        try:
            import ctypes
            k32 = ctypes.windll.kernel32
            h = k32.CreateFileW(r"\\.\pipe\vibe-island", 0xC0000000, 0, None, 3, 0, None)
            if h not in (0, 0xFFFFFFFF):
                k32.WriteFile(h, msg, len(msg), ctypes.byref(ctypes.c_ulong(0)), None)
                k32.CloseHandle(h)
        except Exception: pass
        return
    for path in [SOCKET_PATH, FALLBACK_SOCKET]:
        try:
            s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            s.settimeout(3)
            s.connect(path)
            s.sendall(msg)
            s.close()
            return
        except Exception:
            continue

def main():
    try:
        data = json.load(sys.stdin)
    except Exception:
        return
    event = {"session_id": data.get("session_id", "gemini-" + str(os.getpid())),
             "hook_event_name": data.get("type", "SessionStart"),
             "cwd": data.get("cwd", os.getcwd()), "tool_name": data.get("tool_name", ""),
             "_proto_rev": _PROTO_REV, "_compat": _COMPAT, "_source": "gemini", "_ppid": os.getpid()}
    send_event(event)

if __name__ == "__main__":
    main()
"#;

/// OpenCode plugin (JavaScript/ESM)
const OPENCODE_PLUGIN_JS: &str = r#"// vibe-island-plugin — auto-generated by Vibe Island
import { connect } from "net";
import { homedir } from "os";

const SOCKET = homedir() + "/.vibe-island/run/vibe-island.sock";
const FALLBACK = process.platform === "win32" ? "\\\\.\\pipe\\vibe-island" : "/tmp/vibe-island.sock";
const SOCKETS = process.platform === "win32" ? [FALLBACK] : [SOCKET, FALLBACK];

function sendToSocket(json) {
  return new Promise((resolve) => {
    let tried = 0;
    function tryNext() {
      if (tried >= SOCKETS.length) return resolve(false);
      const path = SOCKETS[tried++];
      try {
        const sock = connect({ path }, () => {
          sock.write(JSON.stringify(json));
          sock.end();
          resolve(true);
        });
        sock.on("error", tryNext);
        sock.setTimeout(3000, () => { sock.destroy(); tryNext(); });
      } catch { tryNext(); }
    }
    tryNext();
  });
}

function sendAndWaitResponse(json, timeoutMs = 300000) {
  return new Promise((resolve) => {
    let tried = 0;
    function tryNext() {
      if (tried >= SOCKETS.length) return resolve(null);
      const path = SOCKETS[tried++];
      try {
        const sock = connect({ path }, () => {
          sock.write(JSON.stringify(json));
        });
        let buf = "";
        sock.on("data", (data) => { buf += data.toString(); });
        sock.on("end", () => {
          try { resolve(JSON.parse(buf)); } catch { resolve(null); }
        });
        sock.on("error", tryNext);
        sock.setTimeout(timeoutMs, () => { sock.destroy(); resolve(null); });
      } catch { tryNext(); }
    }
    tryNext();
  });
}

export default async ({ client, serverUrl }) => {
  const pid = process.pid;
  const serverPort = serverUrl ? parseInt(serverUrl.port) || 4096 : 4096;
  const internalFetch = client?._client?.getConfig?.()?.fetch || null;
  const sessions = new Map();

  function base(sessionId, extra) {
    return { session_id: sessionId, _source: "opencode", _ppid: pid,
      _server_port: serverPort, ...extra };
  }

  return {
    "event": async ({ event }) => {
      const t = event.type;
      const p = event.properties || {};
      let mapped = null;

      if (t === "session.created" && p.info) {
        mapped = base(`opencode-${p.info.id}`, { hook_event_name: "SessionStart", cwd: p.info.directory });
      } else if (t === "session.deleted" && p.info) {
        mapped = base(`opencode-${p.info.id}`, { hook_event_name: "SessionEnd" });
      } else if (t === "message.part.updated" && p.part?.type === "text" && p.part?.messageID) {
        return; // Text updates — skip for now
      } else if (t === "message.part.updated" && p.part?.type === "tool" && p.part?.sessionID) {
        const st = p.part.state?.status;
        const toolName = (p.part.tool || "").charAt(0).toUpperCase() + (p.part.tool || "").slice(1);
        if (st === "running" || st === "pending") {
          mapped = base(`opencode-${p.part.sessionID}`, { hook_event_name: "PreToolUse", tool_name: toolName });
        } else if (st === "completed" || st === "error") {
          mapped = base(`opencode-${p.part.sessionID}`, { hook_event_name: "PostToolUse", tool_name: toolName });
        }
      } else if (t === "permission.asked" && p.id && p.sessionID) {
        const toolName = (p.permission || "").charAt(0).toUpperCase() + (p.permission || "").slice(1);
        mapped = base(`opencode-${p.sessionID}`, {
          hook_event_name: "PermissionRequest", tool_name: toolName,
          tool_input: { patterns: p.patterns || [] },
          _opencode_request_id: p.id
        });

        if (internalFetch) {
          const resp = await sendAndWaitResponse(mapped);
          if (resp?.hookSpecificOutput?.decision?.behavior) {
            const behavior = resp.hookSpecificOutput.decision.behavior;
            const reply = behavior === "allow" ? "once" : behavior === "always" ? "always" : "reject";
            try {
              await internalFetch(new Request(`http://localhost:${serverPort}/permission/${p.id}/reply`, {
                method: "POST", headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ reply }),
              }));
            } catch {}
          }
          return;
        }
      }

      if (mapped) await sendToSocket(mapped);
    },
  };
};
"#;

const AMP_PLUGIN_JS: &str = r#"// vibe-island — Amp agent lifecycle bridge
import { connect } from "net";
import os from "os";

const SOCKET = os.homedir() + "/.vibe-island/run/vibe-island.sock";
const FALLBACK = "/tmp/vibe-island.sock";
const _PROTO_REV = "vi-e7c4";

function send(e, d = {}) {
    const p = JSON.stringify({ hook_event_name: e, _source: "amp", _ppid: process.pid,
        _proto_rev: _PROTO_REV, ...d });
    for (const path of [SOCKET, FALLBACK]) {
        try {
            const sock = connect({ path }, () => { sock.write(p); sock.end(); });
            sock.on("error", () => {});
            return;
        } catch (_) {}
    }
}

export default (amp) => {
    const sid = () => amp.threadId || ("amp-" + process.pid);
    amp.on("session.start", () => send("SessionStart", { session_id: sid(), cwd: process.cwd() }));
    amp.on("agent.start", () => send("PreToolUse", { session_id: sid(), tool_name: "Agent" }));
    amp.on("agent.end", () => send("Stop", { session_id: sid() }));
    amp.on("tool.call", (ev) => send("PreToolUse", { session_id: sid(), tool_name: ev.tool || "" }));
    amp.on("tool.result", (ev) => send("PostToolUse", { session_id: sid(), tool_name: ev.tool || "" }));
};
"#;

const HERMES_PLUGIN_JS: &str = r#"// vibe-island — Hermes agent lifecycle event bridge
import { connect } from "net";
import os from "os";

const SOCKET = os.homedir() + "/.vibe-island/run/vibe-island.sock";
const FALLBACK = "/tmp/vibe-island.sock";
const _PROTO_REV = "vi-e7c4";

function send(e, d = {}) {
    const p = JSON.stringify({ hook_event_name: e, _source: "hermes", _ppid: process.pid,
        _proto_rev: _PROTO_REV, ...d });
    for (const path of [SOCKET, FALLBACK]) {
        try {
            const sock = connect({ path }, () => { sock.write(p); sock.end(); });
            sock.on("error", () => {});
            return;
        } catch (_) {}
    }
}

export default (hermes) => {
    const sid = () => hermes.sessionId || ("hermes-" + process.pid);
    hermes.on("session.start", () => send("SessionStart", { session_id: sid(), cwd: process.cwd() }));
    hermes.on("session.end", () => send("SessionEnd", { session_id: sid() }));
    hermes.on("tool.call", (ev) => send("PreToolUse", { session_id: sid(), tool_name: ev.tool || "" }));
    hermes.on("tool.result", (ev) => send("PostToolUse", { session_id: sid(), tool_name: ev.tool || "" }));
    hermes.on("idle", () => send("Stop", { session_id: sid() }));
};
"#;

const CLINE_HOOK_PY: &str = r#"#!/usr/bin/env python3
"""vibe-island — CLINE hook (PreToolUse/PostToolUse/Stop)"""
import os, sys, json, socket, time

_PROTO_REV = "vi-e7c4"
_COMPAT = 0x3A7F
SOCKET = os.path.expanduser("~/.vibe-island/run/vibe-island.sock")
FALLBACK = "/tmp/vibe-island.sock"

def _send(payload: dict):
    payload.update({"_proto_rev": _PROTO_REV, "_compat": _COMPAT, "_source": "cline"})
    data = json.dumps(payload).encode()
    if sys.platform == "win32":
        try:
            import ctypes
            k32 = ctypes.windll.kernel32
            h = k32.CreateFileW(r"\\.\pipe\vibe-island", 0xC0000000, 0, None, 3, 0, None)
            if h not in (0, 0xFFFFFFFF):
                k32.WriteFile(h, data, len(data), ctypes.byref(ctypes.c_ulong(0)), None)
                k32.CloseHandle(h)
        except Exception: pass
        return
    for path in [SOCKET, FALLBACK]:
        try:
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
                s.settimeout(3)
                s.connect(path)
                s.sendall(data)
                return
        except Exception:
            pass

try:
    event = json.load(sys.stdin)
except Exception:
    sys.exit(0)

hook = event.get("hook_event_name", "")
sess = event.get("session_id") or f"cline-{os.getpid()}"
cwd = event.get("cwd") or os.getcwd()

if hook == "PreToolUse":
    _send({"hook_event_name": "PreToolUse", "session_id": sess, "cwd": cwd,
           "tool_name": event.get("tool_name",""), "tool_input": event.get("tool_input",{})})
elif hook == "PostToolUse":
    _send({"hook_event_name": "PostToolUse", "session_id": sess, "cwd": cwd,
           "tool_name": event.get("tool_name","")})
elif hook in ("Stop", "Notification"):
    _send({"hook_event_name": hook, "session_id": sess, "cwd": cwd,
           "message": event.get("message","")})
"#;

const PICLI_HOOK_PY: &str = r#"#!/usr/bin/env python3
"""vibe-island — Pi-CLI hook (session/tool events)"""
import os, sys, json, socket

_PROTO_REV = "vi-e7c4"
_COMPAT = 0x3A7F
SOCKET = os.path.expanduser("~/.vibe-island/run/vibe-island.sock")
FALLBACK = "/tmp/vibe-island.sock"

def _send(payload: dict):
    payload.update({"_proto_rev": _PROTO_REV, "_compat": _COMPAT, "_source": "picli"})
    data = json.dumps(payload).encode()
    if sys.platform == "win32":
        try:
            import ctypes
            k32 = ctypes.windll.kernel32
            h = k32.CreateFileW(r"\\.\pipe\vibe-island", 0xC0000000, 0, None, 3, 0, None)
            if h not in (0, 0xFFFFFFFF):
                k32.WriteFile(h, data, len(data), ctypes.byref(ctypes.c_ulong(0)), None)
                k32.CloseHandle(h)
        except Exception: pass
        return
    for path in [SOCKET, FALLBACK]:
        try:
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
                s.settimeout(3)
                s.connect(path)
                s.sendall(data)
                return
        except Exception:
            pass

try:
    event = json.load(sys.stdin)
except Exception:
    sys.exit(0)

hook = event.get("event", event.get("hook_event_name", ""))
sess = event.get("session_id") or f"picli-{os.getpid()}"
cwd = event.get("cwd") or os.getcwd()

MAP = {
    "session_start": "SessionStart",
    "session_end": "SessionEnd",
    "tool_call": "PreToolUse",
    "tool_result": "PostToolUse",
    "stop": "Stop",
    "PreToolUse": "PreToolUse",
    "PostToolUse": "PostToolUse",
    "Stop": "Stop",
}

vi_event = MAP.get(hook)
if vi_event:
    _send({"hook_event_name": vi_event, "session_id": sess, "cwd": cwd,
           "tool_name": event.get("tool_name",""), "tool_input": event.get("tool_input",{})})
"#;
