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

  const isBypass =
    session.env?.CLAUDE_BYPASS_PERMISSIONS === "1" ||
    session.codex_permission_mode === "full-auto" ||
    session.env?.BYPASS_PERMISSIONS === "1";

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
          <span className="sess-tag" style={{ color: toolColor }}>{toolLabel}</span>
          {session.tty && (
            <span className="sess-tag">
              {session.env?.TERM_PROGRAM || session.env?.TERM || "Terminal"}
            </span>
          )}
          {isBypass && (
            <span
              className="sess-tag"
              style={{ background: "rgba(249,115,22,0.15)", color: "var(--vi-alert)", fontSize: "9px" }}
            >
              bypass
            </span>
          )}
          <span className="sess-dur">{elapsed()}</span>
        </div>

        {isHero && session.last_user_text && (
          <div className="sess-you">You: {session.last_user_text.slice(0, 60)}</div>
        )}

        {isHero && session.tool_name && (
          <div className="sess-you" style={{ color: "rgba(255,255,255,0.3)" }}>
            ⚡ {session.tool_name}
            {session.tool_input && typeof session.tool_input === "object" &&
             (session.tool_input as Record<string, unknown>).command
              ? `: ${String((session.tool_input as Record<string, unknown>).command).slice(0, 40)}`
              : ""}
          </div>
        )}
      </div>
    </div>
  );
}
