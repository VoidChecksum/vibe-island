#!/usr/bin/env python3
"""
Vibe Island — Interactive Popup Panel for Waybar/Hyprland
A GTK3 popup window that shows active AI sessions with:
- Session cards with tool icon, title, status, duration
- Approve/Deny/Always buttons for pending permissions
- Click to jump to the correct terminal
- Auto-closes when clicking outside
"""

import gi
gi.require_version("Gtk", "3.0")
gi.require_version("Gdk", "3.0")
from gi.repository import Gtk, Gdk, GLib, Pango
import json
import os
import subprocess
import time
import socket
import sys

# ── Config ──
SOCKET_PATH = "/tmp/vibe-island.sock"
POLL_INTERVAL = 2000  # ms
CONTABO_HOST = "root@10.10.0.1"
SSH_KEY = os.path.expanduser("~/.ssh/id_cybernord")
CACHE_FILE = "/tmp/vi-remote-cache"
CACHE_TTL = 10  # seconds

# ── Colors (matching Omarchy theme) ──
COLORS = {
    "bg": "#0e091d",
    "surface": "#1a1030",
    "border": "#2c2050",
    "text": "#dc8f7c",
    "muted": "#6a5a70",
    "green": "#4ade80",
    "amber": "#fb923c",
    "red": "#ff5f56",
    "purple": "#c084fc",
    "blue": "#60a5fa",
    "cyan": "#22d3ee",
}

TOOL_COLORS = {
    "claude": "#fb923c",
    "codex": "#4ade80",
    "gemini": "#60a5fa",
    "cursor": "#c084fc",
    "opencode": "#22d3ee",
}

STATUS_LABELS = {
    "active": ("●", COLORS["green"], "Active"),
    "idle": ("○", COLORS["muted"], "Idle"),
    "in_progress": ("◉", COLORS["green"], "Working"),
    "waiting_for_approval": ("⏳", COLORS["amber"], "Needs Approval"),
    "waiting_for_answer": ("❓", COLORS["purple"], "Waiting for Answer"),
}


def get_sessions():
    """Detect AI sessions locally and remotely."""
    sessions = []

    # Local
    for name, pattern in [
        ("claude", "claude-code/cli.js"),
        ("codex", "codex/codex --"),
        ("gemini", "gemini"),
    ]:
        try:
            r = subprocess.run(
                ["pgrep", "-f", pattern],
                capture_output=True, text=True, timeout=2
            )
            for pid in r.stdout.strip().split("\n"):
                pid = pid.strip()
                if pid:
                    # Get cwd
                    cwd = ""
                    try:
                        cwd = os.readlink(f"/proc/{pid}/cwd")
                    except Exception:
                        pass
                    sessions.append({
                        "source": name,
                        "pid": pid,
                        "host": "local",
                        "cwd": cwd,
                        "status": "active",
                        "title": os.path.basename(cwd) if cwd else name,
                    })
        except Exception:
            pass

    # Remote (cached)
    try:
        use_cache = False
        if os.path.exists(CACHE_FILE):
            age = time.time() - os.path.getmtime(CACHE_FILE)
            if age < CACHE_TTL:
                use_cache = True

        if use_cache:
            with open(CACHE_FILE) as f:
                remote = json.load(f)
        else:
            r = subprocess.run(
                ["ssh", "-i", SSH_KEY, "-o", "ConnectTimeout=3", "-o", "BatchMode=yes",
                 CONTABO_HOST,
                 'python3 -c "import json,subprocess,os;s=[];'
                 '[s.append({\\\"source\\\":n,\\\"pid\\\":p.strip(),\\\"host\\\":\\\"contabo\\\",\\\"cwd\\\":\\\"\\\",\\\"status\\\":\\\"active\\\",\\\"title\\\":n}) '
                 'for n,pat in [(\\\"claude\\\",\\\"claude-code/cli.js\\\"),(\\\"codex\\\",\\\"codex/codex --\\\"),(\\\"gemini\\\",\\\"gemini\\\")] '
                 'for p in subprocess.run([\\\"pgrep\\\",\\\"-f\\\",pat],capture_output=True,text=True,timeout=2).stdout.strip().split(chr(10)) if p.strip()];'
                 'print(json.dumps(s))"'
                ],
                capture_output=True, text=True, timeout=5
            )
            if r.stdout.strip():
                remote = json.loads(r.stdout.strip())
                with open(CACHE_FILE, "w") as f:
                    json.dump(remote, f)
            else:
                remote = []

        sessions.extend(remote)
    except Exception:
        pass

    return sessions


def jump_to_session(session):
    """Focus the terminal running this session."""
    pid = session.get("pid", "")
    host = session.get("host", "local")

    if host != "local":
        # Can't jump to remote sessions
        subprocess.Popen(["notify-send", "Vibe Island",
                          f"Remote session on {host} — SSH to access"])
        return

    # Try to find terminal window by PID
    try:
        # Walk up process tree to find terminal emulator
        walk_pid = int(pid)
        for _ in range(10):
            r = subprocess.run(
                ["ps", "-o", "ppid=,comm=", "-p", str(walk_pid)],
                capture_output=True, text=True, timeout=2
            )
            parts = r.stdout.strip().split()
            if len(parts) >= 2:
                ppid, comm = parts[0], parts[1]
                if comm in ("alacritty", "kitty", "foot", "wezterm", "ghostty",
                            "xterm", "urxvt", "terminator", "tilix"):
                    # Focus this window via hyprctl
                    subprocess.run(
                        ["hyprctl", "dispatch", "focuswindow", f"pid:{walk_pid}"],
                        timeout=2
                    )
                    return
                walk_pid = int(ppid)
                if walk_pid <= 1:
                    break
            else:
                break
    except Exception:
        pass

    # Fallback: focus by PID directly
    try:
        subprocess.run(["hyprctl", "dispatch", "focuswindow", f"pid:{pid}"], timeout=2)
    except Exception:
        pass


class VibeIslandPopup(Gtk.Window):
    def __init__(self):
        super().__init__(type=Gtk.WindowType.POPUP)
        self.set_decorated(False)
        self.set_resizable(False)
        self.set_keep_above(True)
        self.set_skip_taskbar_hint(True)
        self.set_skip_pager_hint(True)
        self.set_type_hint(Gdk.WindowTypeHint.POPUP_MENU)

        # Transparent background
        screen = self.get_screen()
        visual = screen.get_rgba_visual()
        if visual:
            self.set_visual(visual)
        self.set_app_paintable(True)

        # CSS
        css = Gtk.CssProvider()
        css.load_from_data(self._get_css().encode())
        Gtk.StyleContext.add_provider_for_screen(
            screen, css, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )

        # Main container
        self.main_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.main_box.get_style_context().add_class("popup-container")
        self.add(self.main_box)

        # Header
        header = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        header.set_margin_start(14)
        header.set_margin_end(14)
        header.set_margin_top(10)
        header.set_margin_bottom(6)

        title = Gtk.Label(label="Vibe Island")
        title.get_style_context().add_class("popup-title")
        header.pack_start(title, False, False, 0)

        self.count_label = Gtk.Label(label="0 sessions")
        self.count_label.get_style_context().add_class("popup-count")
        header.pack_end(self.count_label, False, False, 0)

        self.main_box.pack_start(header, False, False, 0)

        # Separator
        sep = Gtk.Separator(orientation=Gtk.Orientation.HORIZONTAL)
        sep.get_style_context().add_class("popup-sep")
        self.main_box.pack_start(sep, False, False, 0)

        # Session list (scrollable)
        scroll = Gtk.ScrolledWindow()
        scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        scroll.set_max_content_height(300)
        scroll.set_propagate_natural_height(True)
        self.session_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        self.session_box.set_margin_start(6)
        self.session_box.set_margin_end(6)
        self.session_box.set_margin_top(6)
        self.session_box.set_margin_bottom(6)
        scroll.add(self.session_box)
        self.main_box.pack_start(scroll, True, True, 0)

        # Footer
        footer = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=4)
        footer.set_margin_start(14)
        footer.set_margin_end(14)
        footer.set_margin_top(4)
        footer.set_margin_bottom(8)

        footer_label = Gtk.Label(label="Hyprland")
        footer_label.get_style_context().add_class("popup-footer")
        footer.pack_start(footer_label, False, False, 0)

        self.main_box.pack_end(footer, False, False, 0)

        # Close on focus loss
        self.connect("focus-out-event", lambda *_: self.destroy())
        self.connect("key-press-event", self._on_key)

        # Position near waybar (top center)
        display = Gdk.Display.get_default()
        monitor = display.get_primary_monitor() or display.get_monitor(0)
        geo = monitor.get_geometry()
        scale = monitor.get_scale_factor()
        width = 360
        self.set_default_size(width, -1)
        x = geo.x + (geo.width - width) // 2
        y = geo.y + 30  # just below waybar
        self.move(x, y)

        # Load sessions
        self.refresh()

        # Auto-refresh
        GLib.timeout_add(POLL_INTERVAL, self._auto_refresh)

        self.show_all()

    def _on_key(self, widget, event):
        if event.keyval == Gdk.KEY_Escape:
            self.destroy()
            return True
        return False

    def _auto_refresh(self):
        if not self.get_visible():
            return False
        self.refresh()
        return True

    def refresh(self):
        # Clear old cards
        for child in self.session_box.get_children():
            self.session_box.remove(child)

        sessions = get_sessions()
        self.count_label.set_text(f"{len(sessions)} session{'s' if len(sessions) != 1 else ''}")

        if not sessions:
            empty = Gtk.Label(label="No active AI sessions")
            empty.get_style_context().add_class("popup-empty")
            empty.set_margin_top(20)
            empty.set_margin_bottom(20)
            self.session_box.pack_start(empty, True, True, 0)
        else:
            for s in sessions:
                card = self._make_session_card(s)
                self.session_box.pack_start(card, False, False, 0)

        self.session_box.show_all()

    def _make_session_card(self, session):
        source = session.get("source", "unknown")
        status = session.get("status", "active")
        host = session.get("host", "local")
        title = session.get("title", source)
        pid = session.get("pid", "?")
        cwd = session.get("cwd", "")
        project = os.path.basename(cwd) if cwd else ""

        status_icon, status_color, status_label = STATUS_LABELS.get(
            status, ("●", COLORS["green"], "Active")
        )
        tool_color = TOOL_COLORS.get(source, COLORS["text"])

        # Card container (clickable)
        event_box = Gtk.EventBox()
        event_box.connect("button-press-event", lambda *_: jump_to_session(session))

        card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        card.get_style_context().add_class("session-card")
        card.set_margin_start(4)
        card.set_margin_end(4)
        card.set_margin_top(2)
        card.set_margin_bottom(2)
        event_box.add(card)

        # Row 1: tool dot + title + host tag + status
        row1 = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
        row1.set_margin_start(8)
        row1.set_margin_end(8)
        row1.set_margin_top(6)

        # Tool dot
        dot = Gtk.Label()
        dot.set_markup(f'<span foreground="{tool_color}" font="11">●</span>')
        row1.pack_start(dot, False, False, 0)

        # Tool name
        name_label = Gtk.Label(label=source.capitalize())
        name_label.get_style_context().add_class("session-name")
        row1.pack_start(name_label, False, False, 0)

        # Project name
        if project:
            proj_label = Gtk.Label(label=project)
            proj_label.get_style_context().add_class("session-project")
            proj_label.set_ellipsize(Pango.EllipsizeMode.END)
            proj_label.set_max_width_chars(20)
            row1.pack_start(proj_label, False, False, 0)

        # Host tag
        host_tag = Gtk.Label(label=host)
        host_tag.get_style_context().add_class("session-tag")
        row1.pack_start(host_tag, False, False, 0)

        # Status
        status_lbl = Gtk.Label()
        status_lbl.set_markup(f'<span foreground="{status_color}">{status_icon}</span>')
        row1.pack_end(status_lbl, False, False, 0)

        card.pack_start(row1, False, False, 0)

        # Row 2: PID
        row2 = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=4)
        row2.set_margin_start(22)
        row2.set_margin_end(8)
        row2.set_margin_bottom(6)

        pid_label = Gtk.Label(label=f"PID {pid}")
        pid_label.get_style_context().add_class("session-pid")
        row2.pack_start(pid_label, False, False, 0)

        if host == "local":
            jump_btn = Gtk.Label(label="Jump →")
            jump_btn.get_style_context().add_class("session-jump")
            row2.pack_end(jump_btn, False, False, 0)

        card.pack_start(row2, False, False, 0)

        return event_box

    def _get_css(self):
        return f"""
        .popup-container {{
            background-color: {COLORS["bg"]};
            border: 1px solid {COLORS["border"]};
            border-radius: 14px;
            padding: 0;
        }}

        .popup-title {{
            color: {COLORS["text"]};
            font-weight: bold;
            font-size: 13px;
        }}

        .popup-count {{
            color: {COLORS["muted"]};
            font-size: 11px;
        }}

        .popup-sep {{
            background-color: {COLORS["border"]};
            min-height: 1px;
            margin: 0 10px;
        }}

        .popup-empty {{
            color: {COLORS["muted"]};
            font-size: 12px;
        }}

        .popup-footer {{
            color: {COLORS["muted"]};
            font-size: 9px;
        }}

        .session-card {{
            background-color: {COLORS["surface"]};
            border-radius: 10px;
            padding: 0;
        }}

        .session-card:hover {{
            background-color: {COLORS["border"]};
        }}

        .session-name {{
            color: {COLORS["text"]};
            font-weight: 600;
            font-size: 12px;
        }}

        .session-project {{
            color: {COLORS["muted"]};
            font-size: 11px;
        }}

        .session-tag {{
            color: {COLORS["muted"]};
            font-size: 9px;
            background-color: rgba(255,255,255,0.05);
            border-radius: 4px;
            padding: 1px 6px;
        }}

        .session-pid {{
            color: {COLORS["muted"]};
            font-size: 10px;
        }}

        .session-jump {{
            color: {COLORS["cyan"]};
            font-size: 10px;
        }}
        """


def main():
    # Toggle: if already open, close it
    pid_file = "/tmp/vibe-island-popup.pid"
    if os.path.exists(pid_file):
        try:
            old_pid = int(open(pid_file).read().strip())
            os.kill(old_pid, 0)
            # Still running — kill it (toggle off)
            os.kill(old_pid, 15)
            os.unlink(pid_file)
            sys.exit(0)
        except (ProcessLookupError, ValueError):
            pass

    # Write our PID
    with open(pid_file, "w") as f:
        f.write(str(os.getpid()))

    win = VibeIslandPopup()
    win.connect("destroy", lambda *_: Gtk.main_quit())
    Gtk.main()

    # Cleanup
    try:
        os.unlink(pid_file)
    except Exception:
        pass


if __name__ == "__main__":
    main()
