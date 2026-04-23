import { useState } from "react";
import { PixelPet } from "./PixelPet";
import type { Session } from "../../types";
import { TOOL_LABELS } from "../../types";
import { useStore } from "../../store/useStore";

interface Props {
  session: Session;
  isHero?: boolean;
}

const TERMINAL_NAMES: Record<string, string> = {
  "com.googlecode.iterm2": "iTerm",
  "com.apple.Terminal": "Terminal",
  "com.mitchellh.ghostty": "Ghostty",
  "dev.warp.Warp-Stable": "Warp",
  "com.microsoft.VSCode": "VS Code",
  "com.todesktop.230313mzl4w4u92": "Cursor",
  "com.microsoft.VSCodeInsiders": "Insiders",
  "net.kovidgoyal.kitty": "Kitty",
  "io.alacritty": "Alacritty",
  "org.tabby.tabby": "Tabby",
  "com.github.wez.wezterm": "WezTerm",
};

function terminalName(session: Session): string | null {
  if (session.terminal_bundle_id) {
    const mapped = TERMINAL_NAMES[session.terminal_bundle_id];
    if (mapped) return mapped;
  }
  const termProg = session.env?.TERM_PROGRAM;
  if (termProg) {
    const map: Record<string, string> = {
      "iTerm.app": "iTerm",
      "vscode": "VS Code",
      "ghostty": "Ghostty",
      "WezTerm": "WezTerm",
      "tmux": "tmux",
      "Hyper": "Hyper",
    };
    return map[termProg] || termProg;
  }
  if (session.tty) return "Terminal";
  return null;
}

export function SessionRow({ session, isHero = false }: Props) {
  const { jumpToTerminal, toggleBypass, bypassSessions } = useStore();
  const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number } | null>(null);
  const toolLabel = TOOL_LABELS[session.source] || session.source;
  const projectName = session.cwd?.split("/").pop() || session.cwd?.split("\\").pop() || "";
  const displayTitle = session.title
    || session.codex_title
    || session.last_user_text?.slice(0, 30)
    || projectName
    || "session";

  const isDone = session.status === "completed" || session.status === "idle";

  const isBypass =
    bypassSessions.has(session.id) ||
    session.env?.CLAUDE_BYPASS_PERMISSIONS === "1" ||
    session.codex_permission_mode === "full-auto" ||
    session.env?.BYPASS_PERMISSIONS === "1";

  const termName = terminalName(session);

  const elapsed = () => {
    const ms = Date.now() - new Date(session.started_at).getTime();
    const mins = Math.floor(ms / 60000);
    if (mins < 1) return "<1m";
    if (mins < 60) return `${mins}m`;
    return `${Math.floor(mins / 60)}h`;
  };

  const dotColor = (() => {
    if (session.status === "waiting_for_approval" || session.status === "waiting_for_answer") return "var(--vi-alert)";
    if (isDone) return "var(--vi-idle)";
    return "var(--vi-work)";
  })();

  const handleCtxMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setCtxMenu({ x: e.clientX, y: e.clientY });
  };

  return (
    <div
      className={`sess-card${!isHero ? " sess-mini" : ""}${isDone && isHero ? " sess-done" : ""}`}
      onClick={() => { setCtxMenu(null); jumpToTerminal(session.id); }}
      onContextMenu={handleCtxMenu}
      title={`Click to jump to ${toolLabel} terminal`}
    >
      {ctxMenu && (
        <div
          style={{
            position: "fixed",
            left: ctxMenu.x,
            top: ctxMenu.y,
            background: "rgba(20,20,25,0.97)",
            border: "1px solid rgba(255,255,255,0.08)",
            borderRadius: 8,
            padding: "4px 0",
            zIndex: 9999,
            minWidth: 160,
            boxShadow: "0 8px 32px rgba(0,0,0,0.5)",
          }}
          onClick={e => e.stopPropagation()}
          onMouseLeave={() => setCtxMenu(null)}
        >
          {[
            { label: "Jump to Terminal", action: () => { jumpToTerminal(session.id); setCtxMenu(null); } },
            { label: isBypass ? "Disable Auto Mode" : "Enable Auto Mode", action: () => { toggleBypass(session.id); setCtxMenu(null); } },
          ].map(item => (
            <button
              key={item.label}
              data-no-drag
              onClick={item.action}
              style={{
                display: "block",
                width: "100%",
                padding: "7px 14px",
                background: "none",
                border: "none",
                color: "rgba(255,255,255,0.85)",
                fontSize: 11,
                textAlign: "left",
                cursor: "pointer",
              }}
              onMouseEnter={e => (e.currentTarget.style.background = "rgba(255,255,255,0.06)")}
              onMouseLeave={e => (e.currentTarget.style.background = "none")}
            >
              {item.label}
            </button>
          ))}
        </div>
      )}

      <div className="sess-pet" style={{ display: "flex", alignItems: isHero ? "flex-start" : "center" }}>
        {isHero
          ? <PixelPet status={session.status} size={16} />
          : <div style={{ width: 6, height: 6, borderRadius: "50%", background: dotColor, flexShrink: 0 }} />
        }
      </div>

      <div className="sess-info">
        <div className="sess-r1">
          <span className="sess-name">{displayTitle}</span>
          <span className="sess-tag">{toolLabel}</span>
          {termName && <span className="sess-tag">{termName}</span>}
          {isBypass && (
            <span className="sess-tag" style={{ background: "rgba(249,115,22,0.15)", color: "var(--vi-alert)" }}>
              bypass
            </span>
          )}
          <span className="sess-dur">{elapsed()}</span>
        </div>

        {isHero && isDone && (
          <div className="sess-you" style={{ color: "var(--vi-idle)" }}>Done — click to jump</div>
        )}

        {isHero && !isDone && session.last_user_text && (
          <div className="sess-you">You: {session.last_user_text.slice(0, 60)}</div>
        )}

        {isHero && !isDone && session.tool_name && (
          <div className="sess-status">
            {session.tool_name}
            {session.tool_input && typeof session.tool_input === "object" &&
             (session.tool_input as Record<string, unknown>).file_path
              ? `(${String((session.tool_input as Record<string, unknown>).file_path).split("/").pop()})`
              : session.tool_input && (session.tool_input as Record<string, unknown>).command
              ? ` ${String((session.tool_input as Record<string, unknown>).command).slice(0, 40)}`
              : ""}
          </div>
        )}
      </div>
    </div>
  );
}
