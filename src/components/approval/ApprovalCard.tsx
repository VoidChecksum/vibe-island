import { motion } from "framer-motion";
import { useStore } from "../../store/useStore";
import type { Session } from "../../types";

interface Props {
  session: Session;
}

/**
 * ApprovalCard — matches the original Vibe Island approval UI.
 * Shows tool name, command/file preview, and Deny/Allow/Always buttons.
 * For questions, shows "Claude is asking a question" with Go to Terminal button.
 */
export function ApprovalCard({ session }: Props) {
  const { approvePermission } = useStore();
  const isQuestion = session.status === "waiting_for_answer";
  const toolName = session.tool_name || "Unknown";
  const toolInput = session.tool_input || {};

  const getDetail = () => {
    if (typeof toolInput === "object") {
      const ti = toolInput as Record<string, unknown>;
      if (ti.command) return String(ti.command);
      if (ti.file_path) return String(ti.file_path);
      if (Array.isArray(ti.patterns) && ti.patterns.length > 0) {
        return (ti.patterns as string[]).join(" && ");
      }
    }
    return null;
  };

  const detail = getDetail();

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
            Claude is waiting for an answer
          </span>
        </div>
        <p className="text-[10px] mb-2" style={{ color: "rgba(255,255,255,0.4)" }}>
          Please answer in the terminal
        </p>
        <button
          className="approve-btn allow"
          style={{ width: "100%" }}
          data-no-drag
        >
          Go to Terminal
        </button>
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
      {/* Tool name */}
      <div className="flex items-center gap-1.5 mb-1.5">
        <span className="text-[11px] font-medium" style={{ color: "var(--vi-alert)" }}>
          {toolName}
        </span>
      </div>

      {/* Command/file preview */}
      {detail && (
        <div
          className="mb-2 px-2 py-1.5 rounded-[6px] overflow-hidden"
          style={{ background: "rgba(0,0,0,0.3)" }}
        >
          <code
            className="text-[10px] break-all leading-relaxed"
            style={{
              color: "rgba(255,255,255,0.5)",
              fontFamily: "'SF Mono', 'Fira Code', 'Cascadia Code', monospace",
              display: "-webkit-box",
              WebkitLineClamp: 3,
              WebkitBoxOrient: "vertical",
              overflow: "hidden",
            }}
          >
            {detail}
          </code>
        </div>
      )}

      {/* Action buttons — matching original v6-approve-bar */}
      <div className="approve-bar">
        <button
          className="approve-btn deny"
          data-no-drag
          onClick={() => approvePermission(session.id, "deny")}
        >
          Deny
        </button>
        <button
          className="approve-btn allow"
          data-no-drag
          onClick={() => approvePermission(session.id, "allow")}
        >
          Allow Once
        </button>
        <button
          className="approve-btn always"
          data-no-drag
          onClick={() => approvePermission(session.id, "always")}
        >
          Always Allow
        </button>
      </div>
    </motion.div>
  );
}
