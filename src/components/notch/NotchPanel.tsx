import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useStore } from "../../store/useStore";
import { SessionRow } from "./SessionRow";
import { ApprovalCard } from "../approval/ApprovalCard";
import { PixelPetLarge } from "./PixelPet";

/**
 * NotchPanel — the Dynamic Island panel.
 * Matches the original Vibe Island v6 layout:
 * - Compact: pixel pet + title + session count badge
 * - Expanded: hero session card + collapsed cards + approval/question UI
 */
export function NotchPanel() {
  const { sessions, expanded, toggleExpanded, platform } = useStore();
  const [hovering, setHovering] = useState(false);

  const activeSessions = sessions.filter((s) => s.status !== "completed");
  const waitingSessions = sessions.filter(
    (s) => s.status === "waiting_for_approval" || s.status === "waiting_for_answer"
  );
  const hasWaiting = waitingSessions.length > 0;
  const isExpanded = expanded || hovering;

  // Determine primary session (most recent activity)
  const primarySession = activeSessions.length > 0
    ? activeSessions.reduce((a, b) =>
        new Date(b.last_activity) > new Date(a.last_activity) ? b : a
      )
    : null;

  const primaryStatus = primarySession?.status || "idle";
  const primaryTitle = primarySession?.title || primarySession?.codex_title ||
    primarySession?.last_user_text?.slice(0, 30) || "No sessions";

  return (
    <motion.div
      className="relative"
      onMouseEnter={() => setHovering(true)}
      onMouseLeave={() => setHovering(false)}
      layout
    >
      {/* The Notch Shell */}
      <motion.div
        className="notch-shell"
        style={{
          minWidth: isExpanded ? 340 : 180,
          maxWidth: 400,
        }}
        animate={{
          width: isExpanded ? 380 : "auto",
          borderRadius: isExpanded ? 18 : 22,
        }}
        transition={{ type: "spring", stiffness: 500, damping: 35 }}
      >
        {/* ── Compact Pill (always visible) ── */}
        <div
          className="compact-pill"
          data-tauri-drag-region
          onClick={() => !isExpanded && toggleExpanded()}
          style={{ cursor: isExpanded ? "grab" : "pointer" }}
        >
          {/* Pixel Pet */}
          <PixelPetLarge status={primaryStatus as any} />

          {/* Title */}
          <span className="idle-text">{primaryTitle}</span>

          {/* Session count badge */}
          {activeSessions.length > 0 && (
            <span className="idle-count">
              {activeSessions.length}
            </span>
          )}

          {/* Waiting indicator */}
          {hasWaiting && !isExpanded && (
            <div
              className="flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium"
              style={{
                background: "rgba(249, 115, 22, 0.15)",
                color: "var(--vi-alert)",
              }}
            >
              <span className="pulse-dot">●</span>
              {waitingSessions.length}
            </div>
          )}

          {/* Expand/collapse chevron */}
          {activeSessions.length > 0 && (
            <button
              className="w-5 h-5 flex items-center justify-center rounded-full
                         text-[10px] opacity-40 hover:opacity-80 transition-opacity"
              data-no-drag
              onClick={(e) => {
                e.stopPropagation();
                toggleExpanded();
              }}
              style={{ color: "var(--notch-text)" }}
            >
              {isExpanded ? "▲" : "▼"}
            </button>
          )}
        </div>

        {/* ── Expanded Content ── */}
        <AnimatePresence>
          {isExpanded && activeSessions.length > 0 && (
            <motion.div
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: "auto", opacity: 1 }}
              exit={{ height: 0, opacity: 0 }}
              transition={{ type: "spring", stiffness: 500, damping: 35 }}
              className="overflow-hidden"
            >
              {/* Divider */}
              <div className="mx-3" style={{ height: 1, background: "var(--notch-border)" }} />

              {/* Session List */}
              <div className="max-h-72 overflow-y-auto py-1 px-1.5 space-y-0.5">
                {activeSessions.map((session, i) => (
                  <div key={session.id}>
                    <SessionRow
                      session={session}
                      isHero={i === 0}
                    />

                    {/* Inline approval/question card */}
                    {(session.status === "waiting_for_approval" ||
                      session.status === "waiting_for_answer") && (
                      <ApprovalCard session={session} />
                    )}
                  </div>
                ))}
              </div>

              {/* Footer */}
              <div
                className="mx-3 mt-0.5 mb-1.5 pt-1.5 flex items-center justify-between"
                style={{ borderTop: "1px solid var(--notch-border)" }}
              >
                <span className="text-[9px]" style={{ color: "var(--notch-muted)" }}>
                  {activeSessions.length} session{activeSessions.length !== 1 ? "s" : ""}
                  {platform?.compositor && platform.compositor !== "unknown" && (
                    <span className="ml-1.5 opacity-50">· {platform.compositor}</span>
                  )}
                </span>
                <button
                  className="text-[9px] hover:underline transition-opacity"
                  style={{ color: "var(--vi-explore)" }}
                  data-no-drag
                >
                  Settings
                </button>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </motion.div>

      {/* Alert glow effect */}
      {hasWaiting && (
        <div
          className="absolute inset-0 -z-10 rounded-[22px] blur-2xl animate-pulse"
          style={{
            background: "var(--vi-alert)",
            opacity: 0.12,
          }}
        />
      )}
    </motion.div>
  );
}
