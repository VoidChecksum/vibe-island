use std::fs;
use std::path::PathBuf;

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

        Ok(results)
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

        for event in &["PreToolUse", "PostToolUse", "Notification", "Stop"] {
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
}

/// Claude Code / Cursor hook script (Python)
const CLAUDE_HOOK_PY: &str = r#"#!/usr/bin/env python3
"""Vibe Island hook — forwards events to /tmp/vibe-island.sock"""
import json, os, socket, sys

SOCKET_PATH = "/tmp/vibe-island.sock"
if sys.platform == "win32":
    SOCKET_PATH = r"\\.\pipe\vibe-island"

def send_event(data):
    try:
        if sys.platform == "win32":
            import win32file
            handle = win32file.CreateFile(
                SOCKET_PATH, win32file.GENERIC_WRITE, 0, None,
                win32file.OPEN_EXISTING, 0, None)
            win32file.WriteFile(handle, json.dumps(data).encode())
            win32file.CloseHandle(handle)
        else:
            s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            s.settimeout(3)
            s.connect(SOCKET_PATH)
            s.sendall(json.dumps(data).encode())
            s.close()
    except Exception:
        pass

def main():
    hook_event = os.environ.get("CLAUDE_HOOK_EVENT", os.environ.get("HOOK_EVENT", ""))
    session_id = os.environ.get("CLAUDE_SESSION_ID", os.environ.get("SESSION_ID", ""))
    cwd = os.environ.get("CLAUDE_CWD", os.environ.get("CWD", os.getcwd()))
    tool_name = os.environ.get("CLAUDE_TOOL_NAME", os.environ.get("TOOL_NAME", ""))

    if not hook_event or not session_id:
        try:
            data = json.load(sys.stdin)
            hook_event = data.get("hook_event_name", data.get("type", ""))
            session_id = data.get("session_id", "")
            cwd = data.get("cwd", cwd)
            tool_name = data.get("tool_name", tool_name)
        except Exception:
            return

    event = {
        "session_id": session_id,
        "hook_event_name": hook_event,
        "cwd": cwd,
        "tool_name": tool_name,
        "_source": "claude",
        "_ppid": os.getpid(),
        "_tty": os.ttyname(0) if hasattr(os, "ttyname") and os.isatty(0) else None,
    }
    send_event(event)

if __name__ == "__main__":
    main()
"#;

/// Codex CLI hook script
const CODEX_HOOK_PY: &str = r#"#!/usr/bin/env python3
"""Vibe Island hook for Codex CLI"""
import json, os, socket, sys

SOCKET_PATH = "/tmp/vibe-island.sock"

def send_event(data):
    try:
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.settimeout(3)
        s.connect(SOCKET_PATH)
        s.sendall(json.dumps(data).encode())
        s.close()
    except Exception:
        pass

def main():
    try:
        data = json.load(sys.stdin)
    except Exception:
        return

    event_type = data.get("type", "")
    event_map = {
        "session.start": "SessionStart",
        "session.end": "SessionEnd",
        "tool.start": "PreToolUse",
        "tool.end": "PostToolUse",
    }
    hook_event = event_map.get(event_type, event_type)

    event = {
        "session_id": data.get("session_id", ""),
        "hook_event_name": hook_event,
        "cwd": data.get("cwd", os.getcwd()),
        "tool_name": data.get("tool_name", ""),
        "_source": "codex",
        "_ppid": os.getpid(),
    }
    send_event(event)

if __name__ == "__main__":
    main()
"#;

/// Gemini CLI hook script
const GEMINI_HOOK_PY: &str = r#"#!/usr/bin/env python3
"""Vibe Island hook for Gemini CLI"""
import json, os, socket, sys

SOCKET_PATH = "/tmp/vibe-island.sock"

def send_event(data):
    try:
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.settimeout(3)
        s.connect(SOCKET_PATH)
        s.sendall(json.dumps(data).encode())
        s.close()
    except Exception:
        pass

def main():
    try:
        data = json.load(sys.stdin)
    except Exception:
        return

    event = {
        "session_id": data.get("session_id", f"gemini-{os.getpid()}"),
        "hook_event_name": data.get("type", "SessionStart"),
        "cwd": data.get("cwd", os.getcwd()),
        "tool_name": data.get("tool_name", ""),
        "_source": "gemini",
        "_ppid": os.getpid(),
    }
    send_event(event)

if __name__ == "__main__":
    main()
"#;

/// OpenCode plugin (JavaScript/ESM)
const OPENCODE_PLUGIN_JS: &str = r#"// vibe-island-plugin — auto-generated by Vibe Island
import { connect } from "net";

const SOCKET = "/tmp/vibe-island.sock";

function sendToSocket(json) {
  return new Promise((resolve) => {
    try {
      const sock = connect({ path: SOCKET }, () => {
        sock.write(JSON.stringify(json));
        sock.end();
        resolve(true);
      });
      sock.on("error", () => resolve(false));
      sock.setTimeout(3000, () => { sock.destroy(); resolve(false); });
    } catch { resolve(false); }
  });
}

function sendAndWaitResponse(json, timeoutMs = 300000) {
  return new Promise((resolve) => {
    try {
      const sock = connect({ path: SOCKET }, () => {
        sock.write(JSON.stringify(json));
      });
      let buf = "";
      sock.on("data", (data) => { buf += data.toString(); });
      sock.on("end", () => {
        try { resolve(JSON.parse(buf)); } catch { resolve(null); }
      });
      sock.on("error", () => resolve(null));
      sock.setTimeout(timeoutMs, () => { sock.destroy(); resolve(null); });
    } catch { resolve(null); }
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
