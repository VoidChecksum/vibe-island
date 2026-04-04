#!/usr/bin/env python3
"""
Vibe Island — Waybar Module
Reads session data from /tmp/vibe-island.sock and outputs waybar-compatible JSON.
Displays as a Dynamic Island pill in the waybar top bar.
"""

import json
import os
import socket
import sys
import time
import subprocess

SOCKET_PATH = "/tmp/vibe-island.sock"
POLL_INTERVAL = 2  # seconds

# Tool colors for waybar CSS classes
TOOL_ICONS = {
    "claude": "󰋦",    # nerd font AI icon
    "codex": "󰧑",
    "gemini": "󱙺",
    "cursor": "󰆍",
    "opencode": "󰅩",
    "copilot": "",
    "windsurf": "󰖟",
    "gemini": "󰊤",
    "default": "󰚩",
}

STATUS_ICONS = {
    "active": "●",
    "idle": "○",
    "in_progress": "◉",
    "waiting_for_approval": "⏳",
    "waiting_for_answer": "❓",
    "pending": "◌",
    "completed": "✓",
}

STATUS_CLASSES = {
    "active": "active",
    "idle": "idle",
    "in_progress": "working",
    "waiting_for_approval": "alert",
    "waiting_for_answer": "question",
}


def query_sessions():
    """Query the vibe-island socket for current sessions."""
    try:
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.settimeout(2)
        sock.connect(SOCKET_PATH)
        # Send a query command
        sock.sendall(json.dumps({"hook_event_name": "query", "session_id": "_waybar"}).encode())
        sock.shutdown(socket.SHUT_WR)
        data = b""
        while True:
            chunk = sock.recv(4096)
            if not chunk:
                break
            data += chunk
        sock.close()
        if data:
            return json.loads(data)
    except Exception:
        pass
    return None


def get_sessions_from_tauri():
    """Fallback: read sessions by checking active socket connections."""
    sessions = []

    # Check for active Claude Code sessions
    claude_settings = os.path.expanduser("~/.claude/settings.json")
    if os.path.exists(claude_settings):
        # Check if claude is running
        try:
            result = subprocess.run(
                ["pgrep", "-f", "claude"],
                capture_output=True, text=True, timeout=2
            )
            if result.stdout.strip():
                pids = result.stdout.strip().split("\n")
                for pid in pids[:3]:  # max 3
                    sessions.append({
                        "source": "claude",
                        "status": "active",
                        "pid": pid.strip(),
                    })
        except Exception:
            pass

    # Check for Codex sessions
    try:
        result = subprocess.run(
            ["pgrep", "-f", "codex"],
            capture_output=True, text=True, timeout=2
        )
        if result.stdout.strip():
            for pid in result.stdout.strip().split("\n")[:3]:
                sessions.append({
                    "source": "codex",
                    "status": "active",
                    "pid": pid.strip(),
                })
    except Exception:
        pass

    # Check for Gemini sessions
    try:
        result = subprocess.run(
            ["pgrep", "-f", "gemini"],
            capture_output=True, text=True, timeout=2
        )
        if result.stdout.strip():
            for pid in result.stdout.strip().split("\n")[:3]:
                sessions.append({
                    "source": "gemini",
                    "status": "active",
                    "pid": pid.strip(),
                })
    except Exception:
        pass

    return sessions


def format_output(sessions):
    """Format sessions for waybar JSON output."""
    if not sessions:
        return {
            "text": "",
            "tooltip": "Vibe Island — No active sessions",
            "class": "empty",
            "alt": "empty",
        }

    # Count by status
    total = len(sessions)
    waiting = sum(1 for s in sessions if s.get("status") in ("waiting_for_approval", "waiting_for_answer"))
    working = sum(1 for s in sessions if s.get("status") == "in_progress")

    # Build the pill text
    dots = ""
    for s in sessions[:5]:
        source = s.get("source", "default")
        status = s.get("status", "active")
        icon = TOOL_ICONS.get(source, TOOL_ICONS["default"])
        dots += f"{icon} "

    if waiting > 0:
        text = f"{dots} {waiting}⏳"
        css_class = "alert"
    elif working > 0:
        text = f"{dots}"
        css_class = "working"
    else:
        text = f"{dots}"
        css_class = "idle"

    # Tooltip with full details
    tooltip_lines = [f"<b>Vibe Island</b> — {total} session{'s' if total != 1 else ''}\n"]
    for s in sessions:
        source = s.get("source", "?")
        status = s.get("status", "?")
        title = s.get("title", s.get("cwd", "").split("/")[-1] if s.get("cwd") else "")
        tool = s.get("tool_name", "")
        status_icon = STATUS_ICONS.get(status, "?")

        line = f"{status_icon} <b>{source}</b>"
        if title:
            line += f" · {title[:30]}"
        if tool:
            line += f" → {tool}"
        tooltip_lines.append(line)

    return {
        "text": text.strip(),
        "tooltip": "\n".join(tooltip_lines),
        "class": css_class,
        "alt": css_class,
    }


def main():
    """Main loop — output waybar JSON on each poll."""
    while True:
        # Try socket first, fallback to process detection
        sessions = get_sessions_from_tauri()

        output = format_output(sessions)
        print(json.dumps(output), flush=True)
        time.sleep(POLL_INTERVAL)


if __name__ == "__main__":
    main()
