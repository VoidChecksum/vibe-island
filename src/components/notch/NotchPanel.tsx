import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useStore } from "../../store/useStore";
import { SessionRow } from "./SessionRow";
import { ApprovalCard } from "../approval/ApprovalCard";
import { TOOL_COLORS } from "../../types";

export function NotchPanel() {
  const { sessions, expanded, toggleExpanded, platform } = useStore();
  const [hovering, setHovering] = useState(false);

  const activeSessions = sessions.filter(
    (s) => s.status !== "completed"
  );
  const waitingSessions = sessions.filter(
    (s) => s.status === "waiting_for_approval" || s.status === "waiting_for_answer"
  );
  const hasWaiting = waitingSessions.length > 0;

  const isExpanded = expanded || hovering;

  // Collapsed pill content
  const pillContent = () => {
    if (activeSessions.length === 0) {
      return (
        <div className="flex items-center gap-2 px-4 text-island-muted text-sm">
          <div className="w-2 h-2 rounded-full bg-island-muted/30" />
          <span>No sessions</span>
        </div>
      );
    }

    return (
      <div className="flex items-center gap-1.5 px-3">
        {activeSessions.slice(0, 6).map((s) => (
          <div
            key={s.id}
            className="flex items-center gap-1"
            title={`${s.source}: ${s.status}`}
          >
            <div
              className="w-2.5 h-2.5 rounded-full"
              style={{ backgroundColor: TOOL_COLORS[s.source] || "#888" }}
            />
            {s.status === "waiting_for_approval" && (
              <div className="w-1.5 h-1.5 rounded-full bg-island-amber animate-pulse" />
            )}
            {s.status === "in_progress" && (
              <div className="w-1.5 h-1.5 rounded-full bg-island-green animate-pulse-slow" />
            )}
          </div>
        ))}
        {activeSessions.length > 6 && (
          <span className="text-xs text-island-muted ml-1">
            +{activeSessions.length - 6}
          </span>
        )}
        {hasWaiting && (
          <div className="ml-2 px-2 py-0.5 bg-island-amber/20 text-island-amber text-xs rounded-full">
            {waitingSessions.length} waiting
          </div>
        )}
      </div>
    );
  };

  return (
    <motion.div
      data-tauri-drag-region
      className="relative"
      onMouseEnter={() => setHovering(true)}
      onMouseLeave={() => setHovering(false)}
      layout
    >
      {/* The Island */}
      <motion.div
        className="bg-island-bg border border-island-border overflow-hidden cursor-pointer"
        style={{
          borderRadius: isExpanded ? 20 : 24,
          minWidth: isExpanded ? 380 : 200,
          maxWidth: 420,
        }}
        onClick={() => !isExpanded && toggleExpanded()}
        animate={{
          width: isExpanded ? 400 : "auto",
        }}
        transition={{ type: "spring", stiffness: 400, damping: 30 }}
      >
        {/* Collapsed pill */}
        <motion.div
          className="flex items-center justify-between h-12"
          data-tauri-drag-region
        >
          {pillContent()}

          {/* Session count badge */}
          {activeSessions.length > 0 && (
            <div className="flex items-center gap-1 pr-3">
              <span className="text-xs text-island-muted">
                {activeSessions.length}
              </span>
              <button
                className="w-5 h-5 flex items-center justify-center rounded-full
                           hover:bg-island-surface text-island-muted hover:text-island-text
                           transition-colors text-xs"
                onClick={(e) => {
                  e.stopPropagation();
                  toggleExpanded();
                }}
              >
                {isExpanded ? "▲" : "▼"}
              </button>
            </div>
          )}
        </motion.div>

        {/* Expanded content */}
        <AnimatePresence>
          {isExpanded && activeSessions.length > 0 && (
            <motion.div
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: "auto", opacity: 1 }}
              exit={{ height: 0, opacity: 0 }}
              transition={{ type: "spring", stiffness: 400, damping: 30 }}
              className="overflow-hidden"
            >
              <div className="border-t border-island-border" />

              {/* Session list */}
              <div className="max-h-80 overflow-y-auto p-2 space-y-1">
                {activeSessions.map((session) => (
                  <div key={session.id}>
                    <SessionRow session={session} />

                    {/* Inline approval card */}
                    {(session.status === "waiting_for_approval" ||
                      session.status === "waiting_for_answer") && (
                      <ApprovalCard session={session} />
                    )}
                  </div>
                ))}
              </div>

              {/* Footer */}
              <div className="border-t border-island-border px-3 py-1.5 flex items-center justify-between">
                <span className="text-[10px] text-island-muted">
                  {activeSessions.length} session{activeSessions.length !== 1 ? "s" : ""}
                  {platform?.compositor && (
                    <span className="ml-2 opacity-50">{platform.compositor}</span>
                  )}
                </span>
                <button
                  className="text-[10px] text-island-accent hover:underline"
                  onClick={() => {
                    // TODO: open settings window
                  }}
                >
                  Settings
                </button>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </motion.div>

      {/* Glow effect when waiting */}
      {hasWaiting && (
        <div
          className="absolute inset-0 -z-10 rounded-island blur-xl opacity-20 animate-pulse"
          style={{ backgroundColor: "#FBBF24" }}
        />
      )}
    </motion.div>
  );
}
