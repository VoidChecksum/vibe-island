<p align="center">
  <img src="assets/banner.svg" alt="Vibe Island" width="900" />
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-v2-FFC131?style=for-the-badge&logo=tauri&logoColor=white&labelColor=0D1117" alt="Tauri" />
  <img src="https://img.shields.io/badge/React-19-61DAFB?style=for-the-badge&logo=react&logoColor=white&labelColor=0D1117" alt="React" />
  <img src="https://img.shields.io/badge/Rust-2021-DEA584?style=for-the-badge&logo=rust&logoColor=white&labelColor=0D1117" alt="Rust" />
  <img src="https://img.shields.io/badge/AI_Tools-16+-00D4FF?style=for-the-badge&labelColor=0D1117" alt="Tools" />
  <img src="https://img.shields.io/badge/Terminals-13+-34D399?style=for-the-badge&labelColor=0D1117" alt="Terminals" />
  <a href="LICENSE"><img src="https://img.shields.io/badge/GPL--3.0-license-8B949E?style=for-the-badge&labelColor=0D1117" alt="License" /></a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/macOS-supported-000000?style=flat-square&logo=apple&logoColor=white" alt="macOS" />
  <img src="https://img.shields.io/badge/Windows-supported-0078D6?style=flat-square&logo=windows&logoColor=white" alt="Windows" />
  <img src="https://img.shields.io/badge/Linux-supported-FCC624?style=flat-square&logo=linux&logoColor=black" alt="Linux" />
  <img src="https://img.shields.io/badge/Hyprland-native-58E1FF?style=flat-square&logo=wayland&logoColor=white" alt="Hyprland" />
  <img src="https://img.shields.io/badge/Sway-supported-68B723?style=flat-square&logo=sway&logoColor=white" alt="Sway" />
  <img src="https://img.shields.io/badge/GNOME-supported-4A86CF?style=flat-square&logo=gnome&logoColor=white" alt="GNOME" />
  <img src="https://img.shields.io/badge/KDE-supported-1D99F3?style=flat-square&logo=kde&logoColor=white" alt="KDE" />
</p>

---

## What is Vibe Island?

<p align="center">
  <img src="assets/og.png" alt="Vibe Island Preview" width="860" />
</p>

<p align="center">
  <img src="src/assets/brand/onboarding-wallpaper.jpg" alt="Onboarding wallpaper" width="430" />
  <img src="assets/terminal.png" alt="Terminal integration preview" width="360" />
</p>

A floating notch panel that sits at the top of your screen and monitors all your AI coding sessions — Claude Code, Codex, Gemini, Cursor, Amp, Kimi, Kiro, and more. See which agents are running, approve permissions without switching windows, and jump to the exact terminal tab with one click.

<table>
<tr>
<td width="50%">

### The Problem

You're running 4 AI agents across different terminals. One needs permission approval. Another finished 10 minutes ago. You're constantly switching windows to check on them.

**That's 15 hours a week in context switches.**

</td>
<td width="50%">

### The Solution

A single floating pill at the top of your screen shows everything at a glance:

- 🟠 Claude is working on `auth-module`
- 🟢 Codex is idle
- 🟣 Gemini needs permission to run `rm -rf /tmp`
- 🔵 Cursor is writing tests

**Approve, deny, or jump — without leaving your flow.**

</td>
</tr>
</table>

### Visual Workflow

<table>
<tr>
<td align="center" width="25%">
  <img src="src/assets/brand/extension-icon.png" width="52" alt="Launch" /><br/>
  <b>1. Launch</b><br/>
  <sub>First-run setup installs hooks and shows a DMG-style guided onboarding.</sub>
</td>
<td align="center" width="25%">
  <img src="src/assets/brand/cursor-pixel.png" width="52" alt="Watch" /><br/>
  <b>2. Watch</b><br/>
  <sub>The top notch tracks sessions, subagents, tools, terminal, and elapsed time.</sub>
</td>
<td align="center" width="25%">
  <span style="font-size:44px">🔐</span><br/>
  <b>3. Decide</b><br/>
  <sub>Approve once, always allow, deny, answer questions, or enable Auto Mode.</sub>
</td>
<td align="center" width="25%">
  <span style="font-size:44px">🎯</span><br/>
  <b>4. Jump</b><br/>
  <sub>Click a row to focus the exact terminal, tab, tmux pane, IDE, or compositor window.</sub>
</td>
</tr>
</table>

<p align="center">
  <b>Dynamic Island shape</b> · <b>inline permissions</b> · <b>terminal focus</b> · <b>sound alerts</b> · <b>labs controls</b> · <b>usage telemetry UI</b>
</p>

---

## Features

<table>
<tr>
<td width="33%" align="center">

### 👁️ Session Monitoring

See all active AI coding sessions in a compact pill. Pixel-art pet shows status. Per-session tool + terminal badges, elapsed time, and live tool activity.

</td>
<td width="33%" align="center">

### 🔐 Permission Approval

Approve or deny tool use directly from the panel. Diff rendering for Edit/Write tools. Numbered options for Ask questions. Keyboard shortcuts `⌘Y` / `⌘N`.

</td>
<td width="33%" align="center">

### 🎯 Jump to Terminal

Click any session to jump to the exact terminal — iTerm2, Ghostty, Warp, Kitty, Alacritty, GNOME Terminal, Konsole, WezTerm, tmux, VS Code, Cursor, and more.

</td>
</tr>
<tr>
<td width="33%" align="center">

### 🔊 Sound Effects

Audible notifications when agents need attention. Custom sound packs supported.

</td>
<td width="33%" align="center">

### ⚙️ Smart Behavior

Dwell-time auto-collapse, expand on hover, hide when empty, bypass permission pill, click-outside dismiss.

</td>
<td width="33%" align="center">

### 🔧 Zero Config

Auto-installs hooks for all supported AI tools on first launch. Protocol `vi-e7c4`, OSC2 title caching, fallback socket path, and first-run onboarding assets from the desktop app.

</td>
</tr>
</table>

---

## Supported AI Tools

<p align="center">
  <img src="src/assets/brand/extension-icon.png" alt="Extension icon" width="80" />
</p>

| Tool | Integration | Hook Type | Status |
|------|-------------|-----------|--------|
| **Claude Code** | Full (sessions, permissions, questions, OSC2) | Python hook | ✅ |
| **OpenAI Codex** | Full (sessions, permissions, app-server) | Python hook | ✅ |
| **Google Gemini CLI** | Full (sessions, permissions) | Python hook | ✅ |
| **Cursor** | Full (sessions, permissions) | Python hook | ✅ |
| **OpenCode** | Full (sessions, permissions, questions) | JS plugin | ✅ |
| **Amp** | Full (session/agent/tool events) | JS plugin | ✅ |
| **Kimi Code** | Full (sessions, tool events) | TOML inject | ✅ |
| **Kiro** | Full (session/tool events) | JSON agent | ✅ |
| **Droid** | Session monitoring | Config inject | ✅ |
| **Hermes** | Full (session/tool events) | JS plugin | ✅ |
| **Windsurf** | Session monitoring | URI handler | ✅ |
| **Copilot** | Session monitoring | Config inject | ✅ |
| **CodeBuddy** | Session monitoring | Config inject | ✅ |
| **Qoder** | Session monitoring | Config inject | ✅ |
| **Cline** | Session monitoring | Config inject | ✅ |
| **π CLI** | Session monitoring | Config inject | ✅ |

---

## Supported Terminals (Jump-to-Terminal)

Terminal focus uses TTY→PID resolution — works for **any** terminal without per-app configuration.

| Terminal | macOS | Linux X11 | Linux Wayland |
|----------|-------|-----------|---------------|
| **iTerm2** | AppleScript by TTY | — | — |
| **Terminal.app** | AppleScript by TTY | — | — |
| **Ghostty** | Sentinel file | `xdotool --pid` | `hyprctl` / `swaymsg` |
| **Warp** | Bundle ID activate | `xdotool --pid` | `swaymsg` |
| **Kitty** | `kitty@` remote | `xdotool --pid` | `swaymsg` |
| **Alacritty** | — | `xdotool --pid` | `swaymsg` / `hyprctl` |
| **WezTerm** | — | `xdotool --pid` | `swaymsg` / `hyprctl` |
| **GNOME Terminal** | — | `xdotool --pid` | XWayland / `swaymsg` |
| **Konsole** | — | `xdotool --pid` / `wmctrl` | KDE XWayland |
| **foot** | — | — | `swaymsg` |
| **tmux** | pane by TTY | pane by TTY | pane by TTY |
| **VS Code / Cursor** | URI handler extension | URI handler extension | URI handler extension |
| **Hyper / xterm** | — | `xdotool --pid` | XWayland |

### How Jump Works

```
Session TTY path
      │
      ▼
/proc/*/fd/0 readlink  ──► terminal emulator PID
      │
      ▼
Compositor dispatch:
  Hyprland  → hyprctl dispatch focuswindow pid:<pid>
  Sway      → swaymsg '[pid=<pid>] focus'
  KDE       → wmctrl -ip <pid>
  X11       → xdotool search --pid <pid> windowactivate
  tmux      → tmux select-pane -t <pane>
  macOS     → AppleScript / bundle-id activate
```

---

## Platform Support

<table>
<tr>
<td width="25%" align="center">

### macOS

Native notch-aware positioning. Floats above all windows. `.dmg` installer. iTerm2, Ghostty, Warp, Terminal.app, Kitty support.

</td>
<td width="25%" align="center">

### Windows

Always-on-top floating panel. Named pipe IPC. System tray. `.msi` / `.exe` installers.

</td>
<td width="25%" align="center">

### Linux X11

`xdotool --pid` for universal terminal focus. AppIndicator tray. `.deb`, `.rpm`, `.AppImage`.

</td>
<td width="25%" align="center">

### Linux Wayland

Hyprland (`hyprctl`), Sway (`swaymsg`), KDE (`wmctrl`), GNOME (XWayland). Auto-detected at runtime.

</td>
</tr>
</table>

### Hyprland Setup

Vibe Island auto-applies window rules on startup. To make them permanent:

```ini
# ~/.config/hypr/hyprland.conf
windowrulev2 = float, class:^(vibe-island)$
windowrulev2 = pin, class:^(vibe-island)$
windowrulev2 = noborder, class:^(vibe-island)$
windowrulev2 = noshadow, class:^(vibe-island)$
windowrulev2 = noanim, class:^(vibe-island)$
windowrulev2 = move 33% 0, class:^(vibe-island)$
```

---

## Quick Start

### Option 1 — npx (hooks only, no build required)

Install hooks for all your AI tools in one command:

```bash
npx @voidchecksum/vibe-island install
```

Then [download the app](#download) for your platform.

### Option 2 — Download

| Platform | Link |
|----------|------|
| macOS (universal) | [Vibe Island.dmg](https://github.com/VoidChecksum/vibe-island/releases/latest) |
| Windows | [Vibe Island Setup.exe](https://github.com/VoidChecksum/vibe-island/releases/latest) |
| Linux (AppImage) | [vibe-island.AppImage](https://github.com/VoidChecksum/vibe-island/releases/latest) |
| Linux (.deb) | [vibe-island.deb](https://github.com/VoidChecksum/vibe-island/releases/latest) |

### Option 3 — Build from source

```bash
git clone https://github.com/VoidChecksum/vibe-island.git
cd vibe-island
npm install
npx tauri build
```

### Platform Dependencies

<details>
<summary><b>Linux (Debian/Ubuntu)</b></summary>

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev \
  wmctrl xdotool
```

</details>

<details>
<summary><b>Linux (Arch)</b></summary>

```bash
sudo pacman -S webkit2gtk-4.1 base-devel curl wget file libxdotool \
  openssl libayatana-appindicator librsvg alsa-lib wmctrl
```

</details>

<details>
<summary><b>macOS</b></summary>

```bash
xcode-select --install
```

</details>

<details>
<summary><b>Windows</b></summary>

Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++" workload. WebView2 is included in Windows 10/11.

</details>

### Development

```bash
npx tauri dev
# Hook scripts auto-install on first launch
# Socket: ~/.vibe-island/run/vibe-island.sock
```

### Build Output

| Platform | Output |
|----------|--------|
| macOS | `src-tauri/target/release/bundle/dmg/Vibe Island.dmg` |
| Windows | `src-tauri/target/release/bundle/msi/Vibe Island.msi` |
| Linux | `src-tauri/target/release/bundle/deb/vibe-island.deb` |
| Linux | `src-tauri/target/release/bundle/appimage/vibe-island.AppImage` |

---

## UI Anatomy

<table>
<tr>
<td width="50%">

```text
┌──────────────────────────────────────┐
│  ▣ Vibe Island        ● 1 waiting    │  compact pill
├──────────────────────────────────────┤
│  🟠 auth-module   Claude  iTerm  12m │
│     You: refactor login flow          │
│     Edit(src/auth/session.ts)         │
│                                      │
│  ⚠ Permission Request                 │
│    Bash npm run test                  │
│    [Deny ⌘N]     [Allow ⌘Y]          │
│           Always Allow                │
└──────────────────────────────────────┘
```

</td>
<td width="50%">

- Compact notch when idle.
- Expands on hover or when a tool asks.
- Parent sessions and subagents group together.
- Inline approval cards render command details and file diffs.
- Settings expose Behaviour, Display, Usage, Sound, CLI Hooks, Terminal, Labs, Community, Advanced.

</td>
</tr>
</table>

<p align="center">
  <img src="src/assets/brand/wechat-group-qr.jpg" alt="WeChat group QR" width="130" />
  &nbsp;&nbsp;
  <img src="src/assets/brand/feishu-group-qr.png" alt="Feishu group QR" width="130" />
</p>

## Architecture

<p align="center">
  <img src="assets/arch.svg" alt="Architecture" width="700" />
</p>

### Data Flow

```
AI Tool ── hook.py/js ── ~/.vibe-island/run/vibe-island.sock ── SessionStore ── React UI
            (proto vi-e7c4)    (fallback /tmp/vibe-island.sock)       │
                                                                       ▼
                                                              Tauri emit("session-update")
                                                                       │
                                                              OSC2 cache → jump_to_terminal()
```

### Event Types

| Event | Direction | Description |
|-------|-----------|-------------|
| `SessionStart` | Hook → App | New AI session started |
| `SessionEnd` | Hook → App | Session terminated |
| `UserPromptSubmit` | Hook → App | User sent a prompt |
| `PreToolUse` | Hook → App | Tool about to execute |
| `PostToolUse` | Hook → App | Tool completed |
| `PermissionRequest` | Hook ↔ App | Tool needs approval (held connection) |
| `Stop` | Hook → App | Session went idle |

---

## Project Structure

```
vibe-island/
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs              App entry, Tauri commands
│   │   ├── sessions/mod.rs     Session state machine + models
│   │   ├── socket/mod.rs       Unix socket server
│   │   ├── hooks/mod.rs        Hook installer (14 tools, proto vi-e7c4)
│   │   ├── config/mod.rs       Persistent config: display, labs, usage, terminal, sound
│   │   ├── sound/mod.rs        Audio (rodio) + bundled ceremony sound
│   │   └── platform/mod.rs     jump_to_terminal(), 13+ terminals
│   ├── resources/
│   │   ├── Sounds/             Built-in notification sounds
│   │   └── terminal-focus/     VS Code extension (URI handler)
│   └── Cargo.toml
├── src/
│   ├── assets/brand/           Wallpaper, QR, icon, Departure Mono font
│   ├── components/
│   │   ├── notch/
│   │   │   ├── NotchPanel.tsx      Dynamic Island, dwell-time collapse
│   │   │   ├── SessionRow.tsx      Session row, terminal badge, bypass pill
│   │   │   └── PixelPet.tsx        Pixel-art status character
│   │   ├── approval/
│   │   │   └── ApprovalCard.tsx    Diff view, inline Q&A, ⌘Y/⌘N
│   │   ├── onboarding/
│   │   │   └── OnboardingScreen.tsx DMG-style first-run setup
│   │   └── settings/
│   │       └── SettingsPanel.tsx   Sidebar settings: usage, labs, sound, terminal, hooks
│   ├── store/useStore.ts
│   ├── types/index.ts          16 tools, all config types
│   └── styles/index.css
└── package.json
```

---

## Configuration

`~/.config/vibe-island/config.json`:

```json
{
  "display": { "monitor": "primary", "position": "top-center", "opacity": 0.95 },
  "layout": {
    "style": "clean",
    "show_tool_names": true,
    "dwell_time_secs": 4.0,
    "expand_on_hover": true,
    "hide_when_empty": false,
    "expand_on_subagent_done": false,
    "click_outside_dismisses": false,
    "notch_follows_active_window": false,
    "auto_configure_terminal_titles": false
  },
  "sound": {
    "enabled": true,
    "volume": 0.5,
    "pack": "builtin-8bit",
    "events": { "permission_request": true, "input_required": true }
  },
  "usage": { "show_usage_limits": true, "provider": "auto", "value_mode": "remaining" },
  "labs": { "auto_mode": false, "cursor_approval": true, "codex_desktop_alerts": true },
  "terminal": { "warp_tab_jump": true, "disable_click_to_jump": false }
}
```

---

## Tech Stack

| Layer | Technology | Why |
|-------|-----------|-----|
| Backend | **Rust** + Tauri v2 | ~5MB binary, native performance, cross-platform |
| Frontend | **React 19** + TypeScript | Component model, strict types |
| Styling | **Tailwind CSS** + Framer Motion | Utility-first, spring animations |
| State | **Zustand** | Lightweight, no boilerplate |
| Audio | **rodio** (Rust) + bundled WAV | Cross-platform, built-in notification pack |
| IPC | **Unix socket** / Named pipe | Protocol `vi-e7c4`, `0x3A7F` compat |
| Terminal Focus | **TTY→PID→compositor** | Universal — works for all 13+ terminals |
| Build | **Vite** + Cargo | Fast HMR, incremental Rust builds |

---

## Contributing

```bash
git clone https://github.com/YOUR_USERNAME/vibe-island.git
cd vibe-island
npm install
cargo check --manifest-path src-tauri/Cargo.toml
npx tauri dev

# Type checks
npx tsc --noEmit
```

---

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/footer.svg" />
    <source media="(prefers-color-scheme: light)" srcset="assets/footer.svg" />
    <img alt="Footer" src="assets/footer.svg" width="900" />
  </picture>
</p>

<p align="center">
  <sub>Made with ☕ by <a href="https://github.com/VoidChecksum">VoidChecksum</a> · <a href="https://github.com/VoidChecksum/vibe-island/issues">Report Bug</a> · <a href="https://github.com/VoidChecksum/vibe-island/issues">Request Feature</a></sub>
</p>
