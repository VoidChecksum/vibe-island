# Vibe Island DMG Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make vibe-island repo a full 1:1 open-source replica of VibeIsland.dmg — matching performance, visual design, and all features.

**Architecture:** Tauri v2 (Rust backend) + React 19 (TypeScript frontend). Two windows: `notch` (420×48, transparent, always-on-top pill) and `settings` (560×640, regular window). Data flows: hook scripts → `~/.vibe-island/run/vibe-island.sock` → SessionStore → Tauri events → React.

**Tech Stack:** Rust/Tokio, Tauri v2, React 19, Framer Motion, Zustand, Tailwind CSS, rodio (audio), dirs crate.

---

## File Map

| File | Change |
|------|--------|
| `src-tauri/src/socket/mod.rs` | Socket path → `~/.vibe-island/run/vibe-island.sock` |
| `src-tauri/src/hooks/mod.rs` | Protocol rev, Amp/Kimi/Kiro/Droid/Hermes hooks, OSC2 caching, uninstall |
| `src-tauri/src/config/mod.rs` | 7 new layout config fields |
| `src-tauri/src/platform/mod.rs` | `jump_to_terminal` + TTY walk + AppleScript/xdotool |
| `src-tauri/src/lib.rs` | Register new commands, create `~/.vibe-island/run/` on startup |
| `src-tauri/tauri.conf.json` | Already correct — verify only |
| `src-tauri/resources/terminal-focus/package.json` | VS Code extension manifest |
| `src-tauri/resources/terminal-focus/extension.js` | VS Code extension implementation |
| `src/types/index.ts` | Add amp/kimi/kiro/hermes/droid colors+labels |
| `src/store/useStore.ts` | Add `jumpToTerminal`, `uninstallHooks`, `dwellTimer` |
| `src/components/notch/NotchPanel.tsx` | Dwell-time collapse, settings button opens window |
| `src/components/notch/SessionRow.tsx` | Bypass pill, click → jump |
| `src/components/approval/ApprovalCard.tsx` | Inline Q&A inputs, jump invocation |
| `src/components/settings/SettingsPanel.tsx` | All 8 missing toggle/slider sections |

---

### Task 1: Fix socket path and create run directory

**Files:**
- Modify: `src-tauri/src/socket/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Update socket path in socket/mod.rs**

Replace `socket_path()`:
```rust
fn socket_path() -> PathBuf {
    let run_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".vibe-island/run");
    std::fs::create_dir_all(&run_dir).ok();
    run_dir.join("vibe-island.sock")
}
```

- [ ] **Step 2: Create run dir on startup in lib.rs**

In `run()`, before `tauri::Builder`, add:
```rust
// Ensure run dir exists
let run_dir = dirs::home_dir()
    .unwrap_or_else(|| PathBuf::from("/tmp"))
    .join(".vibe-island/run");
std::fs::create_dir_all(&run_dir).ok();
```

- [ ] **Step 3: Commit**
```bash
git add src-tauri/src/socket/mod.rs src-tauri/src/lib.rs
git commit -m "fix: socket path → ~/.vibe-island/run/vibe-island.sock"
```

---

### Task 2: Protocol negotiation + OSC2 caching in all hook scripts

**Files:**
- Modify: `src-tauri/src/hooks/mod.rs` (CLAUDE_HOOK_PY, CODEX_HOOK_PY, GEMINI_HOOK_PY constants)

- [ ] **Step 1: Update CLAUDE_HOOK_PY with protocol rev + OSC2 + fallback socket**

Replace `CLAUDE_HOOK_PY` constant:
```rust
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
            return f"/dev/{out}"
    except Exception:
        pass
    return None

def write_osc2(session_id, cwd, user_text):
    try:
        os.makedirs(OSC2_DIR, exist_ok=True)
        prefix = session_id[:16]
        project = (cwd or "").rstrip("/").rsplit("/", 1)[-1] or "session"
        prompt_preview = (user_text or "")[:30]
        title = f"{project}: {prompt_preview}" if prompt_preview else project
        with open(os.path.join(OSC2_DIR, prefix), "w") as f:
            f.write(title)
    except Exception:
        pass

def send_event(data, held=False):
    for path in [SOCKET_PATH, FALLBACK_SOCKET]:
        try:
            if sys.platform == "win32":
                import win32file
                handle = win32file.CreateFile(
                    r"\\.\pipe\vibe-island", win32file.GENERIC_WRITE | (win32file.GENERIC_READ if held else 0),
                    0, None, win32file.OPEN_EXISTING, 0, None)
                win32file.WriteFile(handle, json.dumps(data).encode())
                if held:
                    result = b""
                    while True:
                        _, chunk = win32file.ReadFile(handle, 4096)
                        if not chunk: break
                        result += chunk
                    win32file.CloseHandle(handle)
                    return result.decode() if result else None
                win32file.CloseHandle(handle)
            else:
                s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                s.settimeout(300 if held else 3)
                s.connect(path)
                s.sendall(json.dumps(data).encode())
                if held:
                    chunks = []
                    while True:
                        chunk = s.recv(4096)
                        if not chunk: break
                        chunks.append(chunk)
                    s.close()
                    return b"".join(chunks).decode() if chunks else None
                s.close()
            return True
        except Exception:
            continue
    return None

def main():
    hook_event = os.environ.get("CLAUDE_HOOK_EVENT", os.environ.get("HOOK_EVENT", ""))
    session_id = os.environ.get("CLAUDE_SESSION_ID", os.environ.get("SESSION_ID", ""))
    cwd = os.environ.get("CLAUDE_CWD", os.environ.get("CWD", os.getcwd()))
    tool_name = os.environ.get("CLAUDE_TOOL_NAME", os.environ.get("TOOL_NAME", ""))
    tool_input_raw = os.environ.get("CLAUDE_TOOL_INPUT", "{}")
    user_prompt = os.environ.get("CLAUDE_USER_PROMPT", "")

    stdin_data = {}
    if not hook_event or not session_id:
        try:
            stdin_data = json.load(sys.stdin)
            hook_event = stdin_data.get("hook_event_name", stdin_data.get("type", ""))
            session_id = stdin_data.get("session_id", "")
            cwd = stdin_data.get("cwd", cwd)
            tool_name = stdin_data.get("tool_name", tool_name)
            tool_input_raw = json.dumps(stdin_data.get("tool_input", {}))
            user_prompt = stdin_data.get("prompt", user_prompt)
        except Exception:
            return

    if not hook_event or not session_id:
        return

    try:
        tool_input = json.loads(tool_input_raw) if tool_input_raw else {}
    except Exception:
        tool_input = {}

    tty = get_tty()

    is_held = hook_event in ("PreToolUse",) and tool_name == "AskUserQuestion"
    is_perm = hook_event == "PreToolUse" and tool_name not in ("", None) and tool_name != "AskUserQuestion"

    if hook_event == "UserPromptSubmit" and user_prompt:
        write_osc2(session_id, cwd, user_prompt)

    env_snapshot = {k: v for k, v in os.environ.items()
                    if k.startswith(("TERM", "TMUX", "SSH_", "HYPRLAND", "WAYLAND", "XDG_", "DISPLAY", "COLORTERM"))}

    event = {
        "session_id": session_id,
        "hook_event_name": hook_event,
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
```

- [ ] **Step 2: Update CODEX_HOOK_PY with same socket path + protocol rev**

Replace `CODEX_HOOK_PY`:
```rust
const CODEX_HOOK_PY: &str = r#"#!/usr/bin/env python3
"""Vibe Island hook for Codex CLI"""
import json, os, socket, sys

_PROTO_REV = "vi-e7c4"
_COMPAT = 0x3A7F
SOCKET_PATH = os.path.expanduser("~/.vibe-island/run/vibe-island.sock")
FALLBACK_SOCKET = "/tmp/vibe-island.sock"

def send_event(data):
    for path in [SOCKET_PATH, FALLBACK_SOCKET]:
        try:
            s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            s.settimeout(3)
            s.connect(path)
            s.sendall(json.dumps(data).encode())
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
        "_proto_rev": _PROTO_REV,
        "_compat": _COMPAT,
        "_source": "codex",
        "_ppid": os.getpid(),
    }
    send_event(event)

if __name__ == "__main__":
    main()
"#;
```

- [ ] **Step 3: Update GEMINI_HOOK_PY with same socket path + protocol rev**

Replace `GEMINI_HOOK_PY`:
```rust
const GEMINI_HOOK_PY: &str = r#"#!/usr/bin/env python3
"""Vibe Island hook for Gemini CLI"""
import json, os, socket, sys

_PROTO_REV = "vi-e7c4"
_COMPAT = 0x3A7F
SOCKET_PATH = os.path.expanduser("~/.vibe-island/run/vibe-island.sock")
FALLBACK_SOCKET = "/tmp/vibe-island.sock"

def send_event(data):
    for path in [SOCKET_PATH, FALLBACK_SOCKET]:
        try:
            s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            s.settimeout(3)
            s.connect(path)
            s.sendall(json.dumps(data).encode())
            s.close()
            return
        except Exception:
            continue

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
        "_proto_rev": _PROTO_REV,
        "_compat": _COMPAT,
        "_source": "gemini",
        "_ppid": os.getpid(),
    }
    send_event(event)

if __name__ == "__main__":
    main()
"#;
```

- [ ] **Step 4: Commit**
```bash
git add src-tauri/src/hooks/mod.rs
git commit -m "feat: protocol rev vi-e7c4, OSC2 caching, fallback socket in hook scripts"
```

---

### Task 3: Add Amp, Kimi, Kiro, Droid, Hermes hook integrations + uninstall

**Files:**
- Modify: `src-tauri/src/hooks/mod.rs`

- [ ] **Step 1: Add new tool install methods to HookInstaller**

After `install_opencode_plugin`, add:

```rust
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
        fs::write(&hook_path, GEMINI_HOOK_PY)?; // same format

        let existing = if config_path.exists() {
            fs::read_to_string(&config_path)?
        } else {
            String::new()
        };

        if !existing.contains("vibe-island Kimi hooks START") {
            let block = format!(
                "\n# --- vibe-island Kimi hooks START (managed, do not edit) ---\n\
                 [hooks]\n\
                 pre_tool_call = \"python3 {}\"\n\
                 post_tool_call = \"python3 {}\"\n\
                 # --- vibe-island Kimi hooks END ---\n",
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
        let agent_path = agents_dir.join("vibe-island.json");
        let hook_path = home.join(".kiro/vibe-island-hook.py");
        fs::write(&hook_path, CLAUDE_HOOK_PY)?;
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
        let config_path = home.join(".droid/config.json");
        if !home.join(".droid").exists() {
            return Ok("Droid: not installed, skipped".into());
        }
        let hook_path = home.join(".droid/vibe-island-hook.py");
        fs::write(&hook_path, CLAUDE_HOOK_PY)?;

        let mut config: serde_json::Value = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        let obj = config.as_object_mut().ok_or("not an object")?;
        obj.entry("vibe_island_hook").or_insert(serde_json::json!(
            format!("python3 {}", hook_path.display())
        ));
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

    pub fn uninstall_all() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::new();
        let home = dirs::home_dir().ok_or("No home directory")?;

        // Claude
        let _ = fs::remove_file(home.join(".claude/vibe-island-hook.py"));
        let settings_path = home.join(".claude/settings.json");
        if settings_path.exists() {
            if let Ok(content) = fs::read_to_string(&settings_path) {
                if let Ok(mut settings) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(hooks) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
                        for event in &["PreToolUse", "PostToolUse", "Notification", "Stop"] {
                            if let Some(arr) = hooks.get_mut(*event).and_then(|v| v.as_array_mut()) {
                                arr.retain(|e| {
                                    e.get("command").and_then(|c| c.as_str())
                                        .map(|c| !c.contains("vibe-island"))
                                        .unwrap_or(true)
                                });
                            }
                        }
                    }
                    let _ = fs::write(&settings_path, serde_json::to_string_pretty(&settings)?);
                }
            }
        }
        results.push("Claude Code: uninstalled".into());

        // Codex
        let _ = fs::remove_file(home.join(".codex/vibe-island-hook.py"));
        results.push("Codex: uninstalled".into());

        // Gemini
        let _ = fs::remove_file(home.join(".gemini/vibe-island-hook.py"));
        results.push("Gemini: uninstalled".into());

        // OpenCode
        let _ = fs::remove_file(home.join(".config/opencode/plugins/vibe-island.js"));
        results.push("OpenCode: uninstalled".into());

        // Amp
        let _ = fs::remove_file(home.join(".amp/plugins/vibe-island.js"));
        results.push("Amp: uninstalled".into());

        // Kimi — remove managed block
        let kimi_config = home.join(".kimi/config.toml");
        if kimi_config.exists() {
            if let Ok(content) = fs::read_to_string(&kimi_config) {
                let cleaned = remove_managed_block(&content, "vibe-island Kimi hooks");
                let _ = fs::write(&kimi_config, cleaned);
            }
        }
        let _ = fs::remove_file(home.join(".kimi/vibe-island-hook.py"));
        results.push("Kimi: uninstalled".into());

        // Kiro
        let _ = fs::remove_file(home.join(".kiro/agents/vibe-island.json"));
        let _ = fs::remove_file(home.join(".kiro/vibe-island-hook.py"));
        results.push("Kiro: uninstalled".into());

        // Droid
        let _ = fs::remove_file(home.join(".droid/vibe-island-hook.py"));
        results.push("Droid: uninstalled".into());

        // Hermes
        let _ = fs::remove_file(home.join(".hermes/plugins/vibe-island"));
        results.push("Hermes: uninstalled".into());

        // VS Code extension
        for cli in &["code", "cursor", "windsurf"] {
            let _ = std::process::Command::new(cli)
                .args(["--uninstall-extension", "vibe-island.terminal-focus"])
                .output();
        }
        results.push("VS Code extensions: uninstalled".into());

        Ok(results)
    }
```

- [ ] **Step 2: Add helper `remove_managed_block` function**

Before `impl HookInstaller`:
```rust
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
```

- [ ] **Step 3: Add new plugin constants after OPENCODE_PLUGIN_JS**

```rust
const AMP_PLUGIN_JS: &str = r#"// vibe-island-plugin — Amp agent lifecycle bridge
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
        } catch {}
    }
}

export default (amp) => {
    const sid = () => amp.threadId || `amp-${process.pid}`;
    amp.on("session.start", () => {
        send("SessionStart", { session_id: sid(), cwd: process.cwd() });
    });
    amp.on("agent.start", () => {
        send("PreToolUse", { session_id: sid(), tool_name: "Agent" });
    });
    amp.on("agent.end", () => {
        send("Stop", { session_id: sid() });
    });
    amp.on("tool.call", (ev) => {
        send("PreToolUse", { session_id: sid(), tool_name: ev.tool || "" });
    });
    amp.on("tool.result", (ev) => {
        send("PostToolUse", { session_id: sid(), tool_name: ev.tool || "" });
    });
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
        } catch {}
    }
}

export default (hermes) => {
    const sid = () => hermes.sessionId || `hermes-${process.pid}`;
    hermes.on("session.start", () => send("SessionStart", { session_id: sid(), cwd: process.cwd() }));
    hermes.on("session.end", () => send("SessionEnd", { session_id: sid() }));
    hermes.on("tool.call", (ev) => send("PreToolUse", { session_id: sid(), tool_name: ev.tool || "" }));
    hermes.on("tool.result", (ev) => send("PostToolUse", { session_id: sid(), tool_name: ev.tool || "" }));
    hermes.on("idle", () => send("Stop", { session_id: sid() }));
};
"#;
```

- [ ] **Step 4: Add new tools to `install_all` and wire uninstall**

In `install_all`, after `install_opencode_plugin` block:
```rust
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
```

- [ ] **Step 5: Commit**
```bash
git add src-tauri/src/hooks/mod.rs
git commit -m "feat: add Amp, Kimi, Kiro, Droid, Hermes hooks + uninstall support"
```

---

### Task 4: New config fields

**Files:**
- Modify: `src-tauri/src/config/mod.rs`

- [ ] **Step 1: Add new fields to LayoutConfig**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub style: String,
    pub show_tool_names: bool,
    pub show_session_time: bool,
    pub show_cwd: bool,
    pub max_visible_sessions: usize,
    // New fields matching DMG
    pub dwell_time_secs: f32,
    pub expand_on_hover: bool,
    pub hide_when_empty: bool,
    pub expand_on_subagent_done: bool,
    pub click_outside_dismisses: bool,
    pub notch_follows_active_window: bool,
    pub auto_configure_terminal_titles: bool,
}
```

- [ ] **Step 2: Update Default impl to include new fields**

In `Default for AppConfig`, update `layout`:
```rust
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
```

- [ ] **Step 3: Commit**
```bash
git add src-tauri/src/config/mod.rs
git commit -m "feat: add 7 new layout config fields for DMG parity"
```

---

### Task 5: Terminal jump command

**Files:**
- Modify: `src-tauri/src/platform/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add jump_to_terminal function to platform/mod.rs**

Add after `setup_window`:
```rust
/// Walk process tree to find TTY from a PID
fn get_tty_for_pid(pid: u32) -> Option<String> {
    let output = std::process::Command::new("ps")
        .args(["-o", "tty=,ppid=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    let line = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = line.trim().splitn(2, ' ').collect();
    if parts.len() >= 1 {
        let tty = parts[0].trim();
        if !tty.is_empty() && tty != "??" && tty != "?" {
            return Some(format!("/dev/{}", tty));
        }
    }
    None
}

/// Read OSC2 cached title for a session
fn read_osc2_title(session_id: &str) -> Option<String> {
    let dir = dirs::home_dir()?.join(".vibe-island/cache/osc2-titles");
    let prefix = &session_id[..session_id.len().min(16)];
    std::fs::read_to_string(dir.join(prefix)).ok()
}

pub fn jump_to_terminal(session: &crate::sessions::Session) {
    let session_id = session.id.clone();
    let tty = session.tty.clone();
    let source = session.source.clone();
    let title_hint = read_osc2_title(&session_id)
        .or_else(|| session.title.clone())
        .or_else(|| session.last_user_text.clone().map(|t| t[..t.len().min(20)].to_string()));

    #[cfg(target_os = "macos")]
    {
        // Try iTerm2 first via AppleScript
        if let Some(ref tty_path) = tty {
            let tty_name = tty_path.trim_start_matches("/dev/");
            let script = format!(r#"
tell application "iTerm2"
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
end tell
"#, tty = tty_name);
            let result = std::process::Command::new("osascript")
                .args(["-e", &script])
                .output();
            if let Ok(out) = result {
                let s = String::from_utf8_lossy(&out.stdout);
                if s.trim() == "ok" { return; }
            }
        }

        // Try Terminal.app
        if let Some(ref tty_path) = tty {
            let tty_name = tty_path.trim_start_matches("/dev/");
            let script = format!(r#"
tell application "Terminal"
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
end tell
"#, tty = tty_name);
            let result = std::process::Command::new("osascript")
                .args(["-e", &script])
                .output();
            if let Ok(out) = result {
                let s = String::from_utf8_lossy(&out.stdout);
                if s.trim() == "ok" { return; }
            }
        }

        // Ghostty: write sentinel file
        let sentinel = format!("/tmp/vibe-island-ghostty-title-{}", &session_id[..session_id.len().min(8)]);
        if let Some(ref title) = title_hint {
            let _ = std::fs::write(&sentinel, title);
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Hyprland
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            if let Some(ref tty_path) = tty {
                let _ = std::process::Command::new("hyprctl")
                    .args(["dispatch", "focuswindow", &format!("title:{}", tty_path)])
                    .output();
                return;
            }
        }

        // tmux: find pane by TTY
        if std::env::var("TMUX").is_ok() || std::path::Path::new("/tmp/tmux-1000").exists() {
            if let Some(ref tty_path) = tty {
                let tty_name = tty_path.trim_start_matches("/dev/");
                let output = std::process::Command::new("tmux")
                    .args(["list-panes", "-a", "-F", "#{pane_tty} #{pane_active} #{window_active} #{session_name}:#{window_index}.#{pane_index}"])
                    .output();
                if let Ok(out) = output {
                    for line in String::from_utf8_lossy(&out.stdout).lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 4 && parts[0].contains(tty_name) {
                            let target = parts[3];
                            let _ = std::process::Command::new("tmux")
                                .args(["select-pane", "-t", target])
                                .output();
                            return;
                        }
                    }
                }
            }
        }

        // xdotool fallback
        if let Some(ref title) = title_hint {
            let _ = std::process::Command::new("xdotool")
                .args(["search", "--name", title, "windowactivate"])
                .output();
        }
    }

    #[cfg(target_os = "windows")]
    {
        // PowerShell: bring window with matching title to foreground
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
```

- [ ] **Step 2: Add jump_to_terminal Tauri command to lib.rs**

Add command:
```rust
#[tauri::command]
async fn jump_to_terminal(
    state: tauri::State<'_, SharedState>,
    session_id: String,
) -> Result<(), String> {
    let s = state.read().await;
    let sessions = s.sessions.list_async().await;
    drop(s);
    if let Some(session) = sessions.into_iter().find(|s| s.id == session_id) {
        platform::jump_to_terminal(&session);
    }
    Ok(())
}

#[tauri::command]
async fn uninstall_hooks() -> Result<String, String> {
    let results = hooks::HookInstaller::uninstall_all().map_err(|e| e.to_string())?;
    Ok(results.join("\n"))
}
```

- [ ] **Step 3: Register new commands in invoke_handler**

In `invoke_handler!(...)` add:
```rust
jump_to_terminal,
uninstall_hooks,
```

- [ ] **Step 4: Commit**
```bash
git add src-tauri/src/platform/mod.rs src-tauri/src/lib.rs
git commit -m "feat: jump_to_terminal command (iTerm2/Terminal.app/Hyprland/tmux/xdotool) + uninstall_hooks"
```

---

### Task 6: VS Code terminal-focus extension

**Files:**
- Create: `src-tauri/resources/terminal-focus/package.json`
- Create: `src-tauri/resources/terminal-focus/extension.js`

- [ ] **Step 1: Create extension package.json**

```json
{
  "name": "terminal-focus",
  "displayName": "Vibe Island Terminal Focus",
  "description": "Allows Vibe Island to jump to the correct terminal tab in VS Code, Cursor, and other IDEs.",
  "version": "1.0.0",
  "publisher": "vibe-island",
  "engines": { "vscode": "^1.85.0" },
  "main": "./extension.js",
  "activationEvents": ["onStartupFinished"],
  "contributes": {},
  "categories": ["Other"]
}
```

- [ ] **Step 2: Create extension.js**

```javascript
const vscode = require('vscode');

function activate(context) {
    context.subscriptions.push(
        vscode.window.registerUriHandler({
            handleUri(uri) {
                if (uri.path === '/setup') return;
                const params = new URLSearchParams(uri.query);
                const pid = params.get('pid') ? parseInt(params.get('pid')) : null;
                const tty = params.get('tty');
                const title = params.get('title');

                // Focus terminal by PID
                if (pid) {
                    for (const terminal of vscode.window.terminals) {
                        terminal.processId.then(termPid => {
                            if (termPid === pid) {
                                terminal.show(false);
                            }
                        });
                    }
                    return;
                }

                // Focus terminal by TTY
                if (tty) {
                    for (const terminal of vscode.window.terminals) {
                        if (terminal.name && terminal.name.includes(tty)) {
                            terminal.show(false);
                            return;
                        }
                    }
                }

                // Focus terminal by title hint
                if (title) {
                    for (const t of vscode.window.terminals) {
                        if (t.name && t.name.includes(title)) {
                            t.show(false);
                            return;
                        }
                    }
                    // Fall back to active terminal
                    if (vscode.window.activeTerminal) {
                        vscode.window.activeTerminal.show(false);
                    }
                }
            }
        })
    );
}

function deactivate() {}

module.exports = { activate, deactivate };
```

- [ ] **Step 3: Add VSIX install to HookInstaller (hooks/mod.rs)**

In `install_all`, at end, add VS Code extension install:
```rust
        // VS Code extension (best-effort)
        for cli in &["code", "code-insiders", "cursor", "windsurf"] {
            if std::process::Command::new(cli).arg("--version").output().is_ok() {
                let ext_dir = home.join(".vibe-island/extensions/terminal-focus");
                std::fs::create_dir_all(&ext_dir).ok();
                // Extension files are bundled at ~/.vibe-island/extensions/terminal-focus/
                // HookInstaller writes them there and installs via CLI
                let pkg = ext_dir.join("package.json");
                let ext = ext_dir.join("extension.js");
                if !pkg.exists() {
                    std::fs::write(&pkg, VSCODE_EXT_PACKAGE_JSON).ok();
                    std::fs::write(&ext, VSCODE_EXT_JS).ok();
                }
                let _ = std::process::Command::new(cli)
                    .args(["--install-extension", &ext_dir.to_string_lossy()])
                    .output();
                results.push(format!("{}: extension installed", cli));
                break; // Only install for first found IDE
            }
        }
```

Add constants:
```rust
const VSCODE_EXT_PACKAGE_JSON: &str = include_str!("../../resources/terminal-focus/package.json");
const VSCODE_EXT_JS: &str = include_str!("../../resources/terminal-focus/extension.js");
```

- [ ] **Step 4: Commit**
```bash
git add src-tauri/resources/ src-tauri/src/hooks/mod.rs
git commit -m "feat: VS Code terminal-focus extension for IDE jump support"
```

---

### Task 7: React — types update + store additions

**Files:**
- Modify: `src/types/index.ts`
- Modify: `src/store/useStore.ts`

- [ ] **Step 1: Add new tool colors and labels to types/index.ts**

```typescript
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
};
```

Also update `AppConfig` to include new layout fields:
```typescript
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
  };
```

- [ ] **Step 2: Add jumpToTerminal + uninstallHooks to store**

In `useStore.ts`, add to interface:
```typescript
  jumpToTerminal: (sessionId: string) => Promise<void>;
  uninstallHooks: () => Promise<string>;
```

Add implementations:
```typescript
  jumpToTerminal: async (sessionId) => {
    try {
      await invoke("jump_to_terminal", { sessionId });
    } catch (e) {
      console.error("Failed to jump:", e);
    }
  },

  uninstallHooks: async () => {
    try {
      return await invoke<string>("uninstall_hooks");
    } catch (e) {
      return `Error: ${e}`;
    }
  },
```

- [ ] **Step 3: Commit**
```bash
git add src/types/index.ts src/store/useStore.ts
git commit -m "feat: add 5 new tool types + jumpToTerminal + uninstallHooks to store"
```

---

### Task 8: NotchPanel — dwell-time collapse + settings window open

**Files:**
- Modify: `src/components/notch/NotchPanel.tsx`

- [ ] **Step 1: Replace NotchPanel.tsx**

```tsx
import { useState, useEffect, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useStore } from "../../store/useStore";
import { SessionRow } from "./SessionRow";
import { ApprovalCard } from "../approval/ApprovalCard";
import { PixelPetLarge } from "./PixelPet";
import { invoke } from "@tauri-apps/api/core";

export function NotchPanel() {
  const { sessions, expanded, toggleExpanded, setExpanded, platform, config } = useStore();
  const [hovering, setHovering] = useState(false);
  const dwellTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const prevHasActive = useRef(false);

  const activeSessions = sessions.filter((s) => s.status !== "completed");
  const waitingSessions = sessions.filter(
    (s) => s.status === "waiting_for_approval" || s.status === "waiting_for_answer"
  );
  const hasWaiting = waitingSessions.length > 0;
  const expandOnHover = config?.layout?.expand_on_hover ?? true;
  const dwellSecs = config?.layout?.dwell_time_secs ?? 4;
  const hideWhenEmpty = config?.layout?.hide_when_empty ?? false;
  const isExpanded = expanded || (hovering && expandOnHover);

  const primarySession = activeSessions.length > 0
    ? activeSessions.reduce((a, b) =>
        new Date(b.last_activity) > new Date(a.last_activity) ? b : a
      )
    : null;

  const primaryStatus = primarySession?.status ?? "idle";
  const primaryTitle = primarySession?.title
    ?? primarySession?.codex_title
    ?? primarySession?.last_user_text?.slice(0, 30)
    ?? "No sessions";

  // Dwell-time: auto-collapse after primary session goes idle (no waiting)
  useEffect(() => {
    const hasActive = activeSessions.length > 0;
    const justWentIdle = prevHasActive.current && !hasWaiting &&
      (primarySession?.status === "idle" || primarySession?.status === "completed");

    if (justWentIdle && expanded) {
      dwellTimer.current = setTimeout(() => {
        setExpanded(false);
      }, dwellSecs * 1000);
    }

    if (hasWaiting && dwellTimer.current) {
      clearTimeout(dwellTimer.current);
      dwellTimer.current = null;
    }

    prevHasActive.current = hasActive;
    return () => {
      if (dwellTimer.current) clearTimeout(dwellTimer.current);
    };
  }, [primarySession?.status, hasWaiting, expanded, dwellSecs]);

  // Cancel dwell on hover
  useEffect(() => {
    if (hovering && dwellTimer.current) {
      clearTimeout(dwellTimer.current);
      dwellTimer.current = null;
    }
  }, [hovering]);

  const openSettings = () => {
    invoke("plugin:window|setFocus", { label: "settings" }).catch(() => {});
    const { WebviewWindow } = require("@tauri-apps/api/webviewWindow");
    WebviewWindow.getByLabel("settings")?.show().catch(() => {});
  };

  if (hideWhenEmpty && activeSessions.length === 0) {
    return null;
  }

  return (
    <motion.div
      className="relative"
      onMouseEnter={() => setHovering(true)}
      onMouseLeave={() => setHovering(false)}
      layout
    >
      <motion.div
        className="notch-shell"
        animate={{
          width: isExpanded ? 380 : "auto",
          borderRadius: isExpanded ? 18 : 22,
        }}
        transition={{ type: "spring", stiffness: 500, damping: 35 }}
        style={{ minWidth: isExpanded ? 340 : 180, maxWidth: 400 }}
      >
        {/* Compact Pill */}
        <div
          className="compact-pill"
          data-tauri-drag-region
          onClick={() => !isExpanded && toggleExpanded()}
          style={{ cursor: isExpanded ? "grab" : "pointer" }}
        >
          <PixelPetLarge status={primaryStatus as any} />
          <span className="idle-text">{primaryTitle}</span>

          {activeSessions.length > 0 && (
            <span className="idle-count">{activeSessions.length}</span>
          )}

          {hasWaiting && !isExpanded && (
            <div
              className="flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium"
              style={{ background: "rgba(249, 115, 22, 0.15)", color: "var(--vi-alert)" }}
            >
              <span className="pulse-dot">●</span>
              {waitingSessions.length}
            </div>
          )}

          {activeSessions.length > 0 && (
            <button
              className="w-5 h-5 flex items-center justify-center rounded-full text-[10px] opacity-40 hover:opacity-80 transition-opacity"
              data-no-drag
              onClick={(e) => { e.stopPropagation(); toggleExpanded(); }}
              style={{ color: "var(--notch-text)" }}
            >
              {isExpanded ? "▲" : "▼"}
            </button>
          )}
        </div>

        {/* Expanded Content */}
        <AnimatePresence>
          {isExpanded && activeSessions.length > 0 && (
            <motion.div
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: "auto", opacity: 1 }}
              exit={{ height: 0, opacity: 0 }}
              transition={{ type: "spring", stiffness: 500, damping: 35 }}
              className="overflow-hidden"
            >
              <div className="mx-3" style={{ height: 1, background: "var(--notch-border)" }} />

              <div className="max-h-72 overflow-y-auto py-1 px-1.5 space-y-0.5">
                {activeSessions.map((session, i) => (
                  <div key={session.id}>
                    <SessionRow session={session} isHero={i === 0} />
                    {(session.status === "waiting_for_approval" || session.status === "waiting_for_answer") && (
                      <ApprovalCard session={session} />
                    )}
                  </div>
                ))}
              </div>

              <div
                className="mx-3 mt-0.5 mb-1.5 pt-1.5 flex items-center justify-between"
                style={{ borderTop: "1px solid var(--notch-border)" }}
              >
                <span className="text-[9px]" style={{ color: "var(--notch-muted)" }}>
                  {activeSessions.length} session{activeSessions.length !== 1 ? "s" : ""}
                  {platform?.compositor && platform.compositor !== "unknown" && (
                    <span className="ml-1.5 opacity-50">· {platform.compositor}</span>
                  )}
                </span>
                <button
                  className="text-[9px] hover:underline transition-opacity"
                  style={{ color: "var(--vi-explore)" }}
                  data-no-drag
                  onClick={openSettings}
                >
                  Settings
                </button>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </motion.div>

      {hasWaiting && (
        <div
          className="absolute inset-0 -z-10 rounded-[22px] blur-2xl animate-pulse"
          style={{ background: "var(--vi-alert)", opacity: 0.12 }}
        />
      )}
    </motion.div>
  );
}
```

- [ ] **Step 2: Commit**
```bash
git add src/components/notch/NotchPanel.tsx
git commit -m "feat: dwell-time auto-collapse, expand-on-hover, hide-when-empty, settings window open"
```

---

### Task 9: SessionRow — bypass pill + terminal jump on click

**Files:**
- Modify: `src/components/notch/SessionRow.tsx`

- [ ] **Step 1: Replace SessionRow.tsx**

```tsx
import { PixelPet } from "./PixelPet";
import type { Session } from "../../types";
import { TOOL_LABELS, TOOL_COLORS } from "../../types";
import { useStore } from "../../store/useStore";

interface Props {
  session: Session;
  isHero?: boolean;
}

export function SessionRow({ session, isHero = false }: Props) {
  const { jumpToTerminal } = useStore();
  const toolLabel = TOOL_LABELS[session.source] || session.source;
  const toolColor = TOOL_COLORS[session.source] || "#888";
  const projectName = session.cwd?.split("/").pop() || session.cwd?.split("\\").pop() || "";
  const displayTitle = session.title
    || session.codex_title
    || session.last_user_text?.slice(0, 30)
    || projectName
    || "session";

  const isBypass = session.env?.CLAUDE_BYPASS_PERMISSIONS === "1"
    || session.codex_permission_mode === "full-auto"
    || session.env?.BYPASS_PERMISSIONS === "1";

  const elapsed = () => {
    const ms = Date.now() - new Date(session.started_at).getTime();
    const mins = Math.floor(ms / 60000);
    if (mins < 1) return "<1m";
    if (mins < 60) return `${mins}m`;
    return `${Math.floor(mins / 60)}h${mins % 60}m`;
  };

  return (
    <div
      className="sess-card"
      onClick={() => jumpToTerminal(session.id)}
      title={`Click to jump to ${toolLabel} terminal`}
    >
      <div className="sess-pet">
        <PixelPet status={session.status} size={16} />
      </div>

      <div className="sess-info">
        <div className="sess-r1">
          <span className="sess-name">{displayTitle}</span>
          <span className="sess-tag" style={{ color: toolColor }}>
            {toolLabel}
          </span>
          {session.tty && (
            <span className="sess-tag">
              {session.env?.TERM_PROGRAM || session.env?.TERM || "Terminal"}
            </span>
          )}
          {isBypass && (
            <span
              className="sess-tag"
              style={{
                background: "rgba(249, 115, 22, 0.15)",
                color: "var(--vi-alert)",
                fontSize: "9px",
              }}
            >
              bypass
            </span>
          )}
          <span className="sess-dur">{elapsed()}</span>
        </div>

        {isHero && session.last_user_text && (
          <div className="sess-you">
            You: {session.last_user_text.slice(0, 60)}
          </div>
        )}

        {isHero && session.tool_name && (
          <div className="sess-you" style={{ color: "rgba(255,255,255,0.3)" }}>
            ⚡ {session.tool_name}
            {session.tool_input && typeof session.tool_input === "object" && (session.tool_input as any).command
              ? `: ${String((session.tool_input as any).command).slice(0, 40)}`
              : ""}
          </div>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**
```bash
git add src/components/notch/SessionRow.tsx
git commit -m "feat: bypass indicator pill, terminal jump on click, tool color per source"
```

---

### Task 10: ApprovalCard — inline Q&A + terminal jump

**Files:**
- Modify: `src/components/approval/ApprovalCard.tsx`

- [ ] **Step 1: Replace ApprovalCard.tsx**

```tsx
import { useState } from "react";
import { motion } from "framer-motion";
import { useStore } from "../../store/useStore";
import type { Session } from "../../types";

interface Props {
  session: Session;
}

export function ApprovalCard({ session }: Props) {
  const { approvePermission, answerQuestion, jumpToTerminal } = useStore();
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const isQuestion = session.status === "waiting_for_answer";
  const toolName = session.tool_name || "Unknown";
  const toolInput = session.tool_input || {};

  const questions: Array<{ header: string; question: string }> =
    (toolInput as any).questions || [];

  const getDetail = () => {
    const ti = toolInput as Record<string, unknown>;
    if (ti.command) return String(ti.command);
    if (ti.file_path) return String(ti.file_path);
    if (Array.isArray(ti.patterns) && ti.patterns.length > 0) {
      return (ti.patterns as string[]).join(" && ");
    }
    return null;
  };

  const detail = getDetail();

  const handleAnswerSubmit = () => {
    const answerMap: Record<string, string[]> = {};
    for (const q of questions) {
      if (answers[q.header]) {
        answerMap[q.header] = [answers[q.header]];
      }
    }
    answerQuestion(session.id, answerMap);
  };

  if (isQuestion) {
    return (
      <motion.div
        initial={{ opacity: 0, height: 0 }}
        animate={{ opacity: 1, height: "auto" }}
        exit={{ opacity: 0, height: 0 }}
        className="mx-1.5 mb-1 p-2.5 rounded-[10px]"
        style={{
          background: "rgba(192, 132, 252, 0.08)",
          border: "1px solid rgba(192, 132, 252, 0.15)",
        }}
      >
        <div className="flex items-center gap-1.5 mb-2">
          <span className="text-[11px] font-medium" style={{ color: "var(--vi-question)" }}>
            {session.source === "claude" ? "Claude" : session.source} is waiting for an answer
          </span>
        </div>

        {questions.length > 0 ? (
          <div className="space-y-1.5 mb-2">
            {questions.map((q, i) => (
              <div key={i}>
                <p className="text-[10px] mb-1" style={{ color: "rgba(255,255,255,0.5)" }}>
                  {q.question}
                </p>
                <input
                  type="text"
                  value={answers[q.header] || ""}
                  onChange={(e) => setAnswers((a) => ({ ...a, [q.header]: e.target.value }))}
                  onKeyDown={(e) => e.key === "Enter" && handleAnswerSubmit()}
                  placeholder="Your answer…"
                  className="w-full px-2 py-1 rounded-[6px] text-[11px] outline-none"
                  style={{
                    background: "rgba(0,0,0,0.3)",
                    border: "1px solid rgba(192,132,252,0.2)",
                    color: "#fff",
                  }}
                  data-no-drag
                />
              </div>
            ))}
            <button
              className="approve-btn allow mt-1"
              style={{ width: "100%" }}
              data-no-drag
              onClick={handleAnswerSubmit}
            >
              Submit Answer
            </button>
          </div>
        ) : (
          <>
            <p className="text-[10px] mb-2" style={{ color: "rgba(255,255,255,0.4)" }}>
              Please answer in the terminal
            </p>
            <button
              className="approve-btn allow"
              style={{ width: "100%" }}
              data-no-drag
              onClick={() => jumpToTerminal(session.id)}
            >
              Go to Terminal
            </button>
          </>
        )}
      </motion.div>
    );
  }

  return (
    <motion.div
      initial={{ opacity: 0, height: 0 }}
      animate={{ opacity: 1, height: "auto" }}
      exit={{ opacity: 0, height: 0 }}
      className="mx-1.5 mb-1 p-2.5 rounded-[10px]"
      style={{
        background: "rgba(249, 115, 22, 0.06)",
        border: "1px solid rgba(249, 115, 22, 0.12)",
      }}
    >
      <div className="flex items-center gap-1.5 mb-1.5">
        <span className="text-[11px] font-medium" style={{ color: "var(--vi-alert)" }}>
          {toolName}
        </span>
      </div>

      {detail && (
        <div className="mb-2 px-2 py-1.5 rounded-[6px] overflow-hidden"
             style={{ background: "rgba(0,0,0,0.3)" }}>
          <code
            className="text-[10px] break-all leading-relaxed"
            style={{
              color: "rgba(255,255,255,0.5)",
              fontFamily: "'SF Mono', 'Fira Code', 'Cascadia Code', monospace",
              display: "-webkit-box",
              WebkitLineClamp: 3,
              WebkitBoxOrient: "vertical",
              overflow: "hidden",
            } as React.CSSProperties}
          >
            {detail}
          </code>
        </div>
      )}

      <div className="approve-bar">
        <button className="approve-btn deny" data-no-drag
          onClick={() => approvePermission(session.id, "deny")}>
          Deny
        </button>
        <button className="approve-btn allow" data-no-drag
          onClick={() => approvePermission(session.id, "allow")}>
          Allow Once
        </button>
        <button className="approve-btn always" data-no-drag
          onClick={() => approvePermission(session.id, "always")}>
          Always Allow
        </button>
      </div>
    </motion.div>
  );
}
```

- [ ] **Step 2: Commit**
```bash
git add src/components/approval/ApprovalCard.tsx
git commit -m "feat: inline Q&A inputs, terminal jump button, per-source label in approval card"
```

---

### Task 11: SettingsPanel — complete with all DMG toggles

**Files:**
- Modify: `src/components/settings/SettingsPanel.tsx`

- [ ] **Step 1: Replace SettingsPanel.tsx**

```tsx
import { useState, useEffect } from "react";
import { useStore } from "../../store/useStore";
import type { AppConfig } from "../../types";
import { invoke } from "@tauri-apps/api/core";

export function SettingsPanel() {
  const { config, updateConfig, platform, uninstallHooks } = useStore();
  const [localConfig, setLocalConfig] = useState<AppConfig | null>(null);
  const [hookStatus, setHookStatus] = useState("");
  const [uninstallStatus, setUninstallStatus] = useState("");

  useEffect(() => {
    if (config) setLocalConfig({ ...config, layout: { ...config.layout } });
  }, [config]);

  if (!localConfig) return <div className="p-4 text-island-muted text-sm">Loading...</div>;

  const save = () => { if (localConfig) updateConfig(localConfig); };

  const setLayout = (key: keyof AppConfig["layout"], value: unknown) =>
    setLocalConfig((c) => c ? { ...c, layout: { ...c.layout, [key]: value } } : c);

  const setSound = (key: string, value: unknown) =>
    setLocalConfig((c) => c ? { ...c, sound: { ...c.sound, [key]: value } } : c);

  const Toggle = ({ label, value, onChange, description }: {
    label: string; value: boolean; onChange: (v: boolean) => void; description?: string;
  }) => (
    <div className="flex items-start justify-between gap-4 py-2.5"
         style={{ borderBottom: "1px solid var(--notch-border)" }}>
      <div>
        <p className="text-sm font-medium" style={{ color: "var(--notch-text)" }}>{label}</p>
        {description && <p className="text-xs mt-0.5" style={{ color: "var(--notch-muted)" }}>{description}</p>}
      </div>
      <button
        onClick={() => onChange(!value)}
        className="flex-shrink-0 w-10 h-6 rounded-full transition-colors relative"
        style={{ background: value ? "var(--vi-work)" : "var(--notch-surface)", border: "1px solid var(--notch-border)" }}
        data-no-drag
      >
        <div className="absolute top-0.5 w-5 h-5 rounded-full transition-transform"
             style={{ background: "#fff", transform: value ? "translateX(16px)" : "translateX(2px)" }} />
      </button>
    </div>
  );

  return (
    <div className="h-full overflow-y-auto" style={{ background: "#0d0d0d", color: "#fff" }}>
      <div className="px-6 py-5 space-y-6 max-w-lg mx-auto">
        <h1 className="text-xl font-semibold tracking-tight">Vibe Island</h1>

        {/* Platform info */}
        {platform && (
          <section>
            <h2 className="text-xs font-semibold uppercase tracking-wider mb-2"
                style={{ color: "var(--notch-muted)" }}>Environment</h2>
            <div className="rounded-xl p-3 space-y-1.5 text-sm"
                 style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
              {[
                ["Platform", `${platform.os} · ${platform.compositor}`],
                ["Desktop", platform.desktop],
                ["Wayland", platform.wayland ? "Yes" : "No"],
                ["Notch", platform.has_notch ? "Yes" : "No"],
              ].map(([k, v]) => (
                <div key={k} className="flex justify-between">
                  <span style={{ color: "var(--notch-muted)" }}>{k}</span>
                  <span>{v}</span>
                </div>
              ))}
            </div>
          </section>
        )}

        {/* Behavior */}
        <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2"
              style={{ color: "var(--notch-muted)" }}>Behavior</h2>
          <div className="rounded-xl px-4" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <Toggle
              label="Expand on hover"
              description="Expand the notch panel when mouse hovers over pill"
              value={localConfig.layout.expand_on_hover}
              onChange={(v) => setLayout("expand_on_hover", v)}
            />
            <Toggle
              label="Hide when empty"
              description="Hide the pill when no sessions are running"
              value={localConfig.layout.hide_when_empty}
              onChange={(v) => setLayout("hide_when_empty", v)}
            />
            <Toggle
              label="Expand on subagent done"
              description="Surface the panel when a teammate or subagent finishes"
              value={localConfig.layout.expand_on_subagent_done}
              onChange={(v) => setLayout("expand_on_subagent_done", v)}
            />
            <Toggle
              label="Click outside to dismiss"
              description="Immediately dismiss the panel when clicking outside"
              value={localConfig.layout.click_outside_dismisses}
              onChange={(v) => setLayout("click_outside_dismisses", v)}
            />
            <div className="py-2.5" style={{ borderBottom: "1px solid var(--notch-border)" }}>
              <div className="flex justify-between items-center mb-1.5">
                <p className="text-sm font-medium">Dwell time</p>
                <span className="text-xs" style={{ color: "var(--notch-muted)" }}>
                  {localConfig.layout.dwell_time_secs}s
                </span>
              </div>
              <p className="text-xs mb-2" style={{ color: "var(--notch-muted)" }}>
                How long the notch stays expanded after a task completes
              </p>
              <input
                type="range" min="0" max="30" step="1"
                value={localConfig.layout.dwell_time_secs}
                onChange={(e) => setLayout("dwell_time_secs", parseFloat(e.target.value))}
                className="w-full accent-blue-500"
                data-no-drag
              />
            </div>
          </div>
        </section>

        {/* Display */}
        <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2"
              style={{ color: "var(--notch-muted)" }}>Display</h2>
          <div className="rounded-xl px-4" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <Toggle
              label="Notch follows active window"
              description="Move the panel to the display where your active window is"
              value={localConfig.layout.notch_follows_active_window}
              onChange={(v) => setLayout("notch_follows_active_window", v)}
            />
            <Toggle
              label="Auto-configure terminal titles"
              description="Required for precise tab jumping in Ghostty and Warp. Configures terminal title format."
              value={localConfig.layout.auto_configure_terminal_titles}
              onChange={(v) => setLayout("auto_configure_terminal_titles", v)}
            />
            <Toggle
              label="Show tool names"
              value={localConfig.layout.show_tool_names}
              onChange={(v) => setLayout("show_tool_names", v)}
            />
            <Toggle
              label="Show session time"
              value={localConfig.layout.show_session_time}
              onChange={(v) => setLayout("show_session_time", v)}
            />
            <div className="py-2.5" style={{ borderBottom: "1px solid var(--notch-border)" }}>
              <div className="flex justify-between items-center mb-1">
                <p className="text-sm font-medium">Opacity</p>
                <span className="text-xs" style={{ color: "var(--notch-muted)" }}>
                  {Math.round(localConfig.display.opacity * 100)}%
                </span>
              </div>
              <input
                type="range" min="0.5" max="1" step="0.05"
                value={localConfig.display.opacity}
                onChange={(e) => setLocalConfig((c) => c ? {
                  ...c, display: { ...c.display, opacity: parseFloat(e.target.value) }
                } : c)}
                className="w-full accent-blue-500"
                data-no-drag
              />
            </div>
          </div>
        </section>

        {/* Sound */}
        <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2"
              style={{ color: "var(--notch-muted)" }}>Sound</h2>
          <div className="rounded-xl px-4" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <Toggle
              label="Enable sounds"
              value={localConfig.sound.enabled}
              onChange={(v) => setSound("enabled", v)}
            />
            <div className="py-2.5" style={{ borderBottom: "1px solid var(--notch-border)" }}>
              <div className="flex justify-between items-center mb-1">
                <p className="text-sm font-medium">Volume</p>
                <span className="text-xs" style={{ color: "var(--notch-muted)" }}>
                  {Math.round(localConfig.sound.volume * 100)}%
                </span>
              </div>
              <input
                type="range" min="0" max="1" step="0.05"
                value={localConfig.sound.volume}
                onChange={(e) => setSound("volume", parseFloat(e.target.value))}
                className="w-full accent-blue-500"
                data-no-drag
              />
            </div>
            <Toggle label="Permission requests" value={localConfig.sound.events.permission_request}
              onChange={(v) => setLocalConfig((c) => c ? { ...c, sound: { ...c.sound, events: { ...c.sound.events, permission_request: v } } } : c)} />
            <Toggle label="Session start" value={localConfig.sound.events.session_start}
              onChange={(v) => setLocalConfig((c) => c ? { ...c, sound: { ...c.sound, events: { ...c.sound.events, session_start: v } } } : c)} />
          </div>
        </section>

        {/* Integrations */}
        <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2"
              style={{ color: "var(--notch-muted)" }}>Integrations</h2>
          <div className="rounded-xl p-4 space-y-3"
               style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <button
              onClick={async () => {
                setHookStatus("Installing…");
                try {
                  const r = await invoke<string>("install_hooks");
                  setHookStatus(r || "Done");
                } catch (e) { setHookStatus(`Error: ${e}`); }
              }}
              className="w-full py-2 rounded-lg text-sm font-medium transition-colors"
              style={{ background: "var(--notch-hover)", border: "1px solid var(--notch-border)" }}
              data-no-drag
            >
              Install / Reinstall Hooks
            </button>
            {hookStatus && (
              <pre className="text-[10px] whitespace-pre-wrap leading-relaxed"
                   style={{ color: "var(--notch-muted)" }}>{hookStatus}</pre>
            )}
          </div>
        </section>

        {/* Advanced */}
        <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2"
              style={{ color: "var(--notch-muted)" }}>Advanced</h2>
          <div className="rounded-xl p-4 space-y-3"
               style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <Toggle
              label="Auto-install hooks on startup"
              value={localConfig.auto_install_hooks}
              onChange={(v) => setLocalConfig((c) => c ? { ...c, auto_install_hooks: v } : c)}
            />
            <Toggle
              label="Launch at login"
              value={localConfig.launch_at_login}
              onChange={(v) => setLocalConfig((c) => c ? { ...c, launch_at_login: v } : c)}
            />
            <button
              onClick={async () => {
                setUninstallStatus("Uninstalling…");
                const r = await uninstallHooks();
                setUninstallStatus(r || "Done");
              }}
              className="w-full py-2 rounded-lg text-sm font-medium transition-colors"
              style={{ background: "rgba(255,95,86,0.1)", border: "1px solid rgba(255,95,86,0.2)", color: "#ff5f56" }}
              data-no-drag
            >
              Uninstall All Hooks
            </button>
            {uninstallStatus && (
              <pre className="text-[10px] whitespace-pre-wrap leading-relaxed"
                   style={{ color: "var(--notch-muted)" }}>{uninstallStatus}</pre>
            )}
          </div>
        </section>

        {/* Save */}
        <button
          onClick={save}
          className="w-full py-2.5 rounded-xl text-sm font-medium transition-colors"
          style={{ background: "var(--vi-work)", color: "#fff" }}
          data-no-drag
        >
          Save Settings
        </button>

        <p className="text-center text-[10px] pb-4" style={{ color: "var(--notch-muted)" }}>
          Vibe Island · open source · vibeisland.app
        </p>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**
```bash
git add src/components/settings/SettingsPanel.tsx
git commit -m "feat: complete settings panel with all DMG toggles, dwell slider, uninstall"
```

---

### Task 12: Build verification

**Files:** None

- [ ] **Step 1: Install Rust + system deps**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential libxdo-dev \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev
```

- [ ] **Step 2: Install npm deps**
```bash
cd /home/void/vibe-island && npm install
```

- [ ] **Step 3: Type check**
```bash
npx tsc --noEmit
```
Expected: 0 errors.

- [ ] **Step 4: Cargo check**
```bash
cargo check --manifest-path src-tauri/Cargo.toml
```
Expected: 0 errors.

- [ ] **Step 5: Build**
```bash
npx tauri build 2>&1 | tail -20
```
Expected: `src-tauri/target/release/bundle/` artifacts.

- [ ] **Step 6: Commit if clean**
```bash
git add -A && git commit -m "build: verified full build passes"
```
