# Vibe Island — DMG Parity Design

**Date:** 2026-04-23  
**Goal:** Make the open-source repo functionally and visually identical to VibeIsland.dmg (reverse-engineered).

---

## 1. Architecture (unchanged)

Tauri v2 + React 19 + TypeScript frontend, Rust backend, Zustand state, Tailwind CSS + Framer Motion.

Data flow: `AI Tool → hook script → ~/.vibe-island/run/vibe-island.sock → SessionStore → Tauri emit → React UI`

---

## 2. Layer 1 — Build Fixes (blockers)

### 2.1 Socket path
- **Change:** `/tmp/vibe-island.sock` → `~/.vibe-island/run/vibe-island.sock`
- **Affects:** `src-tauri/src/socket/mod.rs` (server bind path), all hook scripts in `hooks/mod.rs`
- **Add:** `~/.vibe-island/run/` directory creation on startup

### 2.2 Protocol negotiation in hook scripts
- Add `_PROTO_REV = "vi-e7c4"` and `_COMPAT = 0x3A7F` constants to all hook scripts
- Add `_negotiateFallback` path: if primary socket unavailable, try `/tmp/vibe-island.sock`
- Affects all embedded Python/JS hook string constants in `hooks/mod.rs`

### 2.3 Tauri window config
- `tauri.conf.json` window `notch`: transparent, no decorations, always-on-top, skip taskbar, visible: true, resizable: false
- `~/.vibe-island/run/` dir created at app startup in `lib.rs`

---

## 3. Layer 2 — Rust Backend Additions

### 3.1 New hook integrations (`hooks/mod.rs`)

| Tool | Method | Config path |
|------|--------|------------|
| Amp | JS plugin (`amp.on` API) | `~/.amp/plugins/vibe-island.js` |
| Kimi | TOML config inject | `~/.kimi/config.toml` |
| Kiro | JSON agent file | `~/.kiro/agents/vibe-island.json` |
| Droid | Config injection | `~/.droid/config.json` |
| Hermes | JS plugin | `~/.hermes/plugins/vibe-island.js` |

- Amp plugin uses `amp.on("session.start", "agent.start", "agent.end", "tool.call", "tool.result")` — sends to socket
- Kimi: inject `[vibe-island]` block into TOML with hook command (managed block, sentinel comments)
- Kiro: write `~/.kiro/agents/vibe-island.json` with session/tool event hooks
- Uninstall removes all managed blocks and plugin files

### 3.2 Terminal jump (`src-tauri/src/platform/mod.rs`)

New `jump_to_terminal(session_id: String)` Tauri command:

**macOS:**
- Read TTY from session → find matching iTerm2 tab via AppleScript (embedded JXA string)
- Fallback: Terminal.app AppleScript
- OSC2 title cache: read `~/.vibe-island/cache/osc2-titles/<session_prefix>` for display title matching
- Ghostty: set tab title via `/tmp/vibe-island-ghostty-title-<id>` sentinel file
- Warp: query `~/Library/Application Support/dev.warp.Warp-Stable/warp.sqlite` for active window

**Linux:**
- `xdotool` for X11 window focusing by name/class
- Hyprland: `hyprctl dispatch focuswindow` by TTY class
- tmux: `tmux select-window -t <pane_id>` if `TMUX` env detected

**Windows:**
- PowerShell `SetForegroundWindow` via named pipe

### 3.3 OSC2 title caching
- Dir: `~/.vibe-island/cache/osc2-titles/`
- Hook scripts write `<session_id_prefix>` → display title (project/cwd basename + prompt preview)
- `jump_to_terminal` reads cache to match terminal window titles

### 3.4 Uninstall support
- New `uninstall_hooks() -> Result<Vec<String>, String>` Tauri command
- Removes: hook scripts, removes managed blocks from settings JSONs, removes Kimi TOML blocks, removes OpenCode plugin, removes Amp/Hermes plugins, uninstalls VS Code extension

### 3.5 New config fields (`config/mod.rs`)

```rust
pub struct LayoutConfig {
    // existing...
    pub dwell_time_secs: f32,           // 0–30s, how long to stay expanded after idle
    pub expand_on_hover: bool,
    pub hide_when_empty: bool,
    pub expand_on_subagent_done: bool,
    pub notch_follows_active_window: bool,
    pub auto_configure_terminal_titles: bool,  // Ghostty/Warp title config
    pub click_outside_dismisses: bool,
}
```

---

## 4. Layer 3 — React UI Additions (fidelity gaps)

### 4.1 Types (`src/types/index.ts`)
Add to `TOOL_COLORS`:
```ts
amp: "#8B5CF6", kimi: "#EC4899", kiro: "#F59E0B", hermes: "#10B981", droid: "#84CC16"
```
Add same keys to `TOOL_LABELS`.

### 4.2 NotchPanel dwell-time auto-collapse
- After primary session transitions to `idle`/`completed`, start `dwell_time_secs` countdown
- If no new waiting sessions during countdown, auto-collapse to pill
- Cancel countdown if user hovers or new approval arrives

### 4.3 SessionRow bypass indicator
- When `session.env.CLAUDE_BYPASS_PERMISSIONS === "1"` or `codex_permission_mode === "full-auto"`, show orange pill "bypass" on session card

### 4.4 ApprovalCard terminal jump
- "Go to Terminal" button invokes `jump_to_terminal(session.id)` Tauri command (not a no-op)
- Question card: show actual questions from `session.tool_input.questions[]` if present
- Allow answering inline (text inputs per question) → `answer_question` invoke

### 4.5 SettingsPanel complete implementation
New sections:
- **Behavior:** expand on hover toggle, hide when empty toggle, dwell time slider (0–30s), expand on subagent done toggle, click outside dismisses toggle
- **Display:** notch follows active window toggle, auto-configure terminal titles toggle  
- **Integrations:** install status per tool (Claude ✓/✗, Codex ✓/✗, Gemini ✓/✗, etc.), reinstall button per tool
- **Advanced:** Uninstall all hooks button (calls `uninstall_hooks`)

---

## 5. Layer 4 — VS Code Extension

Directory: `src-tauri/resources/terminal-focus/`

Files:
- `package.json` — publisher: `vibe-island`, name: `terminal-focus`, engines vscode `^1.85.0`
- `extension.js` — `activate(context)`: registers URI handler `vscode://vibe-island.terminal-focus/focus?pid=<pid>`, on trigger focuses terminal whose `processId` matches
- `deactivate()` — no-op

HookInstaller installs via `code --install-extension <vsix_path>` for each detected IDE CLI (code, code-insiders, cursor, windsurf, kiro).

Extension is bundled as a `.vsix` file embedded in Tauri resources.

---

## 6. Layer 5 — Tauri Config

`src-tauri/tauri.conf.json` window `notch`:
```json
{
  "label": "notch",
  "title": "Vibe Island",
  "width": 400, "height": 400,
  "x": null, "y": 0,
  "decorations": false,
  "transparent": true,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "resizable": false,
  "visible": true,
  "center": true
}
```

---

## 7. Implementation Order

1. Fix socket path + `~/.vibe-island/run/` creation + protocol negotiation in hook scripts
2. Add Amp/Kimi/Kiro/Droid/Hermes hooks + uninstall command
3. Add new config fields (Rust)
4. Build + verify compilation
5. Add `jump_to_terminal` command (macOS AppleScript + Linux xdotool)
6. Add OSC2 title caching to hook scripts
7. React: types, NotchPanel dwell, SessionRow bypass pill, ApprovalCard jump + question inline, SettingsPanel complete
8. VS Code extension bundle + install via HookInstaller
9. Tauri window config
10. End-to-end test: start Claude Code, verify session appears, approve permission, jump to terminal

---

## 8. Out of Scope (proprietary-only)

- License management / LicenseLease
- SSH remote deploy (deferred — most complex)
- PeonPing sound pack registry (use local packs only)
- Sentry crash reporting
- Subscription usage limits display
- Confetti cannon (SwiftUI native)

---

## 9. Files Changed

| File | Change |
|------|--------|
| `src-tauri/src/socket/mod.rs` | Socket path |
| `src-tauri/src/hooks/mod.rs` | 5 new tools + protocol rev + uninstall + OSC2 |
| `src-tauri/src/config/mod.rs` | 6 new config fields |
| `src-tauri/src/platform/mod.rs` | `jump_to_terminal` command |
| `src-tauri/src/lib.rs` | Register new commands, create run dir |
| `src-tauri/tauri.conf.json` | Window config |
| `src-tauri/resources/terminal-focus/` | VS Code extension |
| `src/types/index.ts` | 5 new tools |
| `src/components/notch/NotchPanel.tsx` | Dwell-time collapse |
| `src/components/notch/SessionRow.tsx` | Bypass pill |
| `src/components/approval/ApprovalCard.tsx` | Jump + inline Q&A |
| `src/components/settings/SettingsPanel.tsx` | All missing toggles + uninstall |
