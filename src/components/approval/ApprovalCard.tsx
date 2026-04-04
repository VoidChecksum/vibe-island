import { motion } from "framer-motion";
import { useStore } from "../../store/useStore";
import type { Session } from "../../types";

interface Props {
  session: Session;
}

export function ApprovalCard({ session }: Props) {
  const { approvePermission, answerQuestion } = useStore();
  const isQuestion = session.status === "waiting_for_answer";

  const toolName = session.tool_name || "Unknown";
  const toolInput = session.tool_input || {};

  // Extract readable info from tool input
  const getDetail = () => {
    if (toolInput.command) return toolInput.command as string;
    if (toolInput.file_path) return toolInput.file_path as string;
    if (toolInput.patterns && Array.isArray(toolInput.patterns)) {
      return (toolInput.patterns as string[]).join(", ");
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
        className="mx-1 mb-1 p-2 rounded-lg bg-island-purple/10 border border-island-purple/20"
      >
        <div className="flex items-center gap-1.5 mb-1.5">
          <span className="text-xs">❓</span>
          <span className="text-xs font-medium text-island-purple">
            Claude is asking a question
          </span>
        </div>

        <p className="text-[11px] text-island-text/70 mb-2">
          Please answer in the terminal
        </p>

        <button
          className="w-full text-xs py-1.5 rounded-md bg-island-surface
                     hover:bg-island-border text-island-text transition-colors"
          onClick={() => {
            // Jump to terminal
          }}
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
      className="mx-1 mb-1 p-2 rounded-lg bg-island-amber/10 border border-island-amber/20"
    >
      {/* Tool info */}
      <div className="flex items-center gap-1.5 mb-1">
        <span className="text-xs">🔒</span>
        <span className="text-xs font-medium text-island-amber">
          {toolName}
        </span>
      </div>

      {/* Detail */}
      {detail && (
        <div className="mb-2 p-1.5 rounded bg-island-bg/50 overflow-hidden">
          <code className="text-[10px] text-island-text/60 break-all line-clamp-3">
            {detail}
          </code>
        </div>
      )}

      {/* Action buttons */}
      <div className="flex gap-1.5">
        <button
          className="flex-1 text-xs py-1.5 rounded-md bg-island-red/20
                     hover:bg-island-red/30 text-island-red transition-colors"
          onClick={() => approvePermission(session.id, "deny")}
        >
          Deny
        </button>
        <button
          className="flex-1 text-xs py-1.5 rounded-md bg-island-surface
                     hover:bg-island-border text-island-text transition-colors"
          onClick={() => approvePermission(session.id, "allow")}
        >
          Allow Once
        </button>
        <button
          className="flex-1 text-xs py-1.5 rounded-md bg-island-green/20
                     hover:bg-island-green/30 text-island-green transition-colors"
          onClick={() => approvePermission(session.id, "always")}
        >
          Always
        </button>
      </div>
    </motion.div>
  );
}
