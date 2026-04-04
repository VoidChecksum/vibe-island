import { motion } from "framer-motion";
import type { Session } from "../../types";
import { TOOL_COLORS, TOOL_LABELS } from "../../types";

interface Props {
  session: Session;
}

const STATUS_ICONS: Record<string, { icon: string; color: string; label: string }> = {
  active: { icon: "●", color: "#34D399", label: "Active" },
  idle: { icon: "○", color: "#888888", label: "Idle" },
  in_progress: { icon: "◉", color: "#34D399", label: "Working" },
  waiting_for_approval: { icon: "⏳", color: "#FBBF24", label: "Needs approval" },
  waiting_for_answer: { icon: "❓", color: "#A78BFA", label: "Waiting for answer" },
  pending: { icon: "◌", color: "#888888", label: "Pending" },
  completed: { icon: "✓", color: "#888888", label: "Done" },
};

export function SessionRow({ session }: Props) {
  const toolColor = TOOL_COLORS[session.source] || "#888";
  const toolLabel = TOOL_LABELS[session.source] || session.source;
  const statusInfo = STATUS_ICONS[session.status] || STATUS_ICONS.active;

  const projectName = session.cwd?.split("/").pop() || session.cwd?.split("\\").pop() || "";
  const displayTitle = session.title || session.codex_title || session.last_user_text;

  const elapsed = () => {
    const start = new Date(session.started_at).getTime();
    const now = Date.now();
    const mins = Math.floor((now - start) / 60000);
    if (mins < 1) return "<1m";
    if (mins < 60) return `${mins}m`;
    const hrs = Math.floor(mins / 60);
    return `${hrs}h${mins % 60}m`;
  };

  return (
    <motion.div
      className="flex items-center gap-2 px-2 py-1.5 rounded-lg
                 hover:bg-island-surface/50 transition-colors cursor-pointer group"
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      title={`${toolLabel} — ${statusInfo.label}\n${session.cwd || ""}`}
    >
      {/* Tool indicator */}
      <div className="flex-shrink-0 flex items-center gap-1.5">
        <div
          className="w-3 h-3 rounded-full"
          style={{ backgroundColor: toolColor }}
        />
        <span className="text-xs font-medium text-island-text w-12 truncate">
          {toolLabel}
        </span>
      </div>

      {/* Session info */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1">
          {projectName && (
            <span className="text-xs text-island-muted truncate max-w-[80px]">
              {projectName}
            </span>
          )}
          {displayTitle && (
            <>
              {projectName && <span className="text-island-border text-[10px]">·</span>}
              <span className="text-xs text-island-text/70 truncate">
                {displayTitle.slice(0, 40)}
              </span>
            </>
          )}
        </div>

        {/* Current tool use */}
        {session.tool_name && session.status === "in_progress" && (
          <div className="flex items-center gap-1 mt-0.5">
            <div className="w-1 h-1 rounded-full bg-island-green animate-pulse" />
            <span className="text-[10px] text-island-green/70 truncate">
              {session.tool_name}
            </span>
          </div>
        )}
      </div>

      {/* Status + time */}
      <div className="flex-shrink-0 flex items-center gap-1.5">
        <span className="text-[10px] text-island-muted">{elapsed()}</span>
        <span
          className="text-xs"
          style={{ color: statusInfo.color }}
          title={statusInfo.label}
        >
          {statusInfo.icon}
        </span>
      </div>
    </motion.div>
  );
}
