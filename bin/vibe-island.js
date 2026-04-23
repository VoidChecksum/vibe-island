#!/usr/bin/env node
import { execSync, spawnSync } from "child_process";
import { existsSync, mkdirSync, writeFileSync } from "fs";
import { homedir } from "os";
import { join } from "path";

const VERSION = "1.0.0";
const REPO = "VoidChecksum/vibe-island";
const SOCKET = join(homedir(), ".vibe-island/run/vibe-island.sock");

const CLAUDE_HOOK = `#!/usr/bin/env python3
"""Vibe Island hook — forwards Claude Code events to the notch."""
import json, os, socket, sys, subprocess

SOCKET_PATH = os.path.expanduser("~/.vibe-island/run/vibe-island.sock")
FALLBACK_SOCKET = "/tmp/vibe-island.sock"

def get_tty():
    try:
        if os.isatty(0): return os.ttyname(0)
    except Exception: pass
    try:
        pid = os.getpid()
        out = subprocess.check_output(["ps", "-o", "tty=", "-p", str(pid)], timeout=1).decode().strip()
        if out and out not in ("??", "?"): return "/dev/" + out
    except Exception: pass
    return None

def send_event(data, held=False):
    for path in [SOCKET_PATH, FALLBACK_SOCKET]:
        try:
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
        except Exception: continue
    return None

def main():
    hook_event = os.environ.get("CLAUDE_HOOK_EVENT", os.environ.get("HOOK_EVENT", ""))
    session_id = os.environ.get("CLAUDE_SESSION_ID", os.environ.get("SESSION_ID", ""))
    cwd = os.environ.get("CLAUDE_CWD", os.environ.get("CWD", os.getcwd()))
    tool_name = os.environ.get("CLAUDE_TOOL_NAME", os.environ.get("TOOL_NAME", ""))
    tool_input_raw = os.environ.get("CLAUDE_TOOL_INPUT", "{}")
    user_prompt = os.environ.get("CLAUDE_USER_PROMPT", "")

    try:
        stdin_data = json.load(sys.stdin)
        hook_event = hook_event or stdin_data.get("hook_event_name", stdin_data.get("type", ""))
        session_id = session_id or stdin_data.get("session_id", "")
        cwd = cwd or stdin_data.get("cwd", os.getcwd())
        tool_name = tool_name or stdin_data.get("tool_name", "")
        if not tool_input_raw or tool_input_raw == "{}":
            tool_input_raw = json.dumps(stdin_data.get("tool_input", {}))
        user_prompt = user_prompt or stdin_data.get("prompt", "")
    except Exception: pass

    if not hook_event or not session_id: return

    try: tool_input = json.loads(tool_input_raw) if tool_input_raw else {}
    except Exception: tool_input = {}

    tty = get_tty()
    bypass = os.environ.get("CLAUDE_BYPASS_PERMISSIONS", "") == "1"
    is_held = hook_event in ("PreToolUse",) and tool_name == "AskUserQuestion"
    is_perm = hook_event == "PreToolUse" and tool_name not in ("", None, "AskUserQuestion") and not bypass

    wire_event = "PermissionRequest" if (is_held or is_perm) else hook_event
    env_snap = {k: v for k, v in os.environ.items()
                if k.startswith(("TERM", "TMUX", "SSH_", "HYPRLAND", "WAYLAND", "XDG_", "DISPLAY", "COLORTERM", "CLAUDE_BYPASS", "BYPASS_PERM"))}

    event = {"session_id": session_id, "hook_event_name": wire_event, "cwd": cwd,
             "tool_name": tool_name, "tool_input": tool_input, "prompt": user_prompt or None,
             "_source": "claude", "_ppid": os.getpid(), "_tty": tty, "_env": env_snap}

    result = send_event(event, held=(is_held or is_perm))
    if result and isinstance(result, str):
        try:
            resp = json.loads(result)
            if resp.get("hookSpecificOutput", {}).get("decision"): print(json.dumps(resp))
        except Exception: pass

if __name__ == "__main__": main()
`;

const args = process.argv.slice(2);
const cmd = args[0];

function log(msg) { process.stdout.write(msg + "\n"); }
function err(msg) { process.stderr.write("\x1b[31m✗\x1b[0m " + msg + "\n"); }
function ok(msg)  { process.stdout.write("\x1b[32m✓\x1b[0m " + msg + "\n"); }

function platformDownloadUrl() {
  const p = process.platform;
  const a = process.arch;
  const base = `https://github.com/${REPO}/releases/latest/download`;
  if (p === "darwin") return `${base}/Vibe.Island_${VERSION}_universal.dmg`;
  if (p === "win32")  return `${base}/Vibe.Island_${VERSION}_x64-setup.exe`;
  if (a === "arm64")  return `${base}/vibe-island_${VERSION}_arm64.AppImage`;
  return `${base}/vibe-island_${VERSION}_amd64.AppImage`;
}

function installHooks() {
  log("\n\x1b[1mInstalling Vibe Island hooks...\x1b[0m\n");

  const home = homedir();
  const claudeDir = join(home, ".claude");
  const hookPath = join(claudeDir, "vibe-island-hook.py");

  if (!existsSync(claudeDir)) mkdirSync(claudeDir, { recursive: true });
  writeFileSync(hookPath, CLAUDE_HOOK, { mode: 0o755 });

  // Register in Claude Code settings.json
  const settingsPath = join(claudeDir, "settings.json");
  let settings = {};
  if (existsSync(settingsPath)) {
    try { settings = JSON.parse(require("fs").readFileSync(settingsPath, "utf8")); } catch {}
  }
  if (!settings.hooks) settings.hooks = {};
  const entry = [{ type: "command", command: `python3 ${hookPath}`, timeout: 300000 }];
  for (const ev of ["SessionStart", "UserPromptSubmit", "PreToolUse", "PostToolUse", "Notification", "Stop"]) {
    if (!settings.hooks[ev]) settings.hooks[ev] = entry;
  }
  writeFileSync(settingsPath, JSON.stringify(settings, null, 2));
  ok("Claude Code hooks installed");

  // Run dir
  const runDir = join(home, ".vibe-island/run");
  mkdirSync(runDir, { recursive: true });
  ok("~/.vibe-island/run created");

  log(`\n\x1b[1mDone!\x1b[0m Hook: \x1b[2m${hookPath}\x1b[0m`);
  log(`\nNow download and run the Vibe Island app:`);
  log(`\x1b[36m${platformDownloadUrl()}\x1b[0m`);
  log(`\nOr build from source: \x1b[2mnpx tauri build\x1b[0m\n`);
}

function showHelp() {
  log(`
\x1b[1mVibe Island\x1b[0m ${VERSION} — Dynamic Island for AI coding tools

\x1b[1mUsage:\x1b[0m
  npx vibe-island install     Install hooks for Claude Code (and other tools)
  npx vibe-island download    Show download URL for your platform
  npx vibe-island status      Check if socket is running
  npx vibe-island --help      Show this help

\x1b[1mSupported tools:\x1b[0m
  Claude Code, Codex, Gemini, Cursor, OpenCode, Amp,
  Kimi, Kiro, Droid, Hermes, CLINE, Pi-CLI

\x1b[1mGitHub:\x1b[0m https://github.com/${REPO}
`);
}

function checkStatus() {
  const { existsSync } = await import("fs").catch(() => ({ existsSync }));
  if (existsSync(SOCKET)) {
    ok(`Socket exists: ${SOCKET}`);
    log("App is running or was recently running.");
  } else {
    err(`Socket not found: ${SOCKET}`);
    log("App is not running. Start Vibe Island to activate the socket.");
  }
}

switch (cmd) {
  case "install":   installHooks(); break;
  case "download":  log(platformDownloadUrl()); break;
  case "status":    checkStatus(); break;
  default:          showHelp(); break;
}
