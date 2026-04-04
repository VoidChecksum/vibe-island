import { PixelPet } from "./PixelPet";
import type { Session } from "../../types";
import { TOOL_LABELS } from "../../types";

interface Props {
  session: Session;
  isHero?: boolean;
}

/**
 * SessionRow — matches the original v6-sess layout:
 * - Pet icon (16x16 pixel art)
 * - Session name (project/title)
 * - Tool tag + terminal tag
 * - Duration
 * - "You:" last prompt preview (hero only)
 */
export function SessionRow({ session, isHero = false }: Props) {
  const toolLabel = TOOL_LABELS[session.source] || session.source;
  const projectName = session.cwd?.split("/").pop() || session.cwd?.split("\\").pop() || "";
  const displayTitle = session.title || session.codex_title ||
    session.last_user_text?.slice(0, 30) || projectName || "session";

  const elapsed = () => {
    const ms = Date.now() - new Date(session.started_at).getTime();
    const mins = Math.floor(ms / 60000);
    if (mins < 1) return "<1m";
    if (mins < 60) return `${mins}m`;
    return `${Math.floor(mins / 60)}h${mins % 60}m`;
  };

  return (
    <div className="sess-card">
      {/* Pixel Pet */}
      <div className="sess-pet">
        <PixelPet status={session.status} size={16} />
      </div>

      {/* Session Info */}
      <div className="sess-info">
        <div className="sess-r1">
          <span className="sess-name">{displayTitle}</span>
          <span className="sess-tag">{toolLabel}</span>
          {session.tty && (
            <span className="sess-tag">
              {session.env?.TERM_PROGRAM || "Terminal"}
            </span>
          )}
          <span className="sess-dur">{elapsed()}</span>
        </div>

        {/* Hero: show last user prompt */}
        {isHero && session.last_user_text && (
          <div className="sess-you">
            You: {session.last_user_text.slice(0, 50)}
          </div>
        )}

        {/* Current tool use indicator */}
        {session.tool_name && session.status === "in_progress" && (
          <div className="flex items-center gap-1 mt-0.5">
            <span
              className="w-1 h-1 rounded-full pulse-dot"
              style={{ background: "var(--vi-work-bright)" }}
            />
            <span className="text-[10px]" style={{ color: "var(--vi-work)", opacity: 0.7 }}>
              {session.tool_name}
            </span>
          </div>
        )}
      </div>
    </div>
  );
}
