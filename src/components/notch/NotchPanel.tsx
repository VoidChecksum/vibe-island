import { useState, useEffect, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useStore } from "../../store/useStore";
import { SessionRow } from "./SessionRow";
import { ApprovalCard } from "../approval/ApprovalCard";
import { PixelPetLarge } from "./PixelPet";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

export function NotchPanel() {
  const { sessions, expanded, toggleExpanded, setExpanded, platform, config } = useStore();
  const [hovering, setHovering] = useState(false);
  const dwellTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const prevPrimaryStatus = useRef<string>("");

  const activeSessions = sessions.filter((s) => s.status !== "completed");
  const waitingSessions = sessions.filter(
    (s) => s.status === "waiting_for_approval" || s.status === "waiting_for_answer"
  );
  const hasWaiting = waitingSessions.length > 0;
  const expandOnHover = config?.layout?.expand_on_hover ?? true;
  const dwellSecs = config?.layout?.dwell_time_secs ?? 4;
  const hideWhenEmpty = config?.layout?.hide_when_empty ?? false;
  const isExpanded = expanded || (hovering && expandOnHover);

  const primarySession = activeSessions.length > 0
    ? activeSessions.reduce((a, b) =>
        new Date(b.last_activity) > new Date(a.last_activity) ? b : a)
    : null;

  const primaryStatus = primarySession?.status ?? "idle";
  const primaryTitle = primarySession?.title
    ?? primarySession?.codex_title
    ?? primarySession?.last_user_text?.slice(0, 30)
    ?? "No sessions";

  // Dwell-time auto-collapse
  useEffect(() => {
    const justWentIdle = prevPrimaryStatus.current !== "idle" && primaryStatus === "idle";
    if (justWentIdle && expanded && !hasWaiting) {
      dwellTimer.current = setTimeout(() => setExpanded(false), dwellSecs * 1000);
    }
    if (hasWaiting && dwellTimer.current) {
      clearTimeout(dwellTimer.current);
      dwellTimer.current = null;
    }
    prevPrimaryStatus.current = primaryStatus;
    return () => { if (dwellTimer.current) clearTimeout(dwellTimer.current); };
  }, [primaryStatus, hasWaiting, expanded, dwellSecs, setExpanded]);

  useEffect(() => {
    if (hovering && dwellTimer.current) {
      clearTimeout(dwellTimer.current);
      dwellTimer.current = null;
    }
  }, [hovering]);

  const openSettings = async () => {
    try {
      const win = await WebviewWindow.getByLabel("settings");
      if (win) {
        await win.show();
        await win.setFocus();
      }
    } catch (e) {
      console.error("Failed to open settings:", e);
    }
  };

  if (hideWhenEmpty && activeSessions.length === 0) return null;

  return (
    <motion.div
      className="relative"
      onMouseEnter={() => setHovering(true)}
      onMouseLeave={() => setHovering(false)}
      layout
    >
      <motion.div
        className="notch-shell"
        animate={{ width: isExpanded ? 380 : 240 }}
        transition={{ type: "spring", stiffness: 500, damping: 35 }}
        style={{ maxWidth: 400 }}
      >
        <div
          className="compact-pill"
          data-tauri-drag-region
          onClick={() => !isExpanded && toggleExpanded()}
          style={{ cursor: isExpanded ? "grab" : "pointer" }}
        >
          <PixelPetLarge status={primaryStatus as any} />
          <span className="idle-text">{primaryTitle}</span>

          {activeSessions.length > 0 && (
            <span className="idle-count">{activeSessions.length}</span>
          )}

          {hasWaiting && !isExpanded && (
            <div
              className="flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium"
              style={{ background: "rgba(249, 115, 22, 0.15)", color: "var(--vi-alert)" }}
            >
              <span className="pulse-dot">●</span>
              {waitingSessions.length}
            </div>
          )}

          {activeSessions.length > 0 && (
            <button
              className="w-5 h-5 flex items-center justify-center rounded-full text-[10px] opacity-40 hover:opacity-80 transition-opacity"
              data-no-drag
              onClick={(e) => { e.stopPropagation(); toggleExpanded(); }}
              style={{ color: "var(--notch-text)" }}
            >
              {isExpanded ? "▲" : "▼"}
            </button>
          )}
        </div>

        <AnimatePresence>
          {isExpanded && activeSessions.length > 0 && (
            <motion.div
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: "auto", opacity: 1 }}
              exit={{ height: 0, opacity: 0 }}
              transition={{ type: "spring", stiffness: 500, damping: 35 }}
              className="overflow-hidden"
            >
              <div className="mx-3" style={{ height: 1, background: "var(--notch-border)" }} />

              <div className="max-h-72 overflow-y-auto py-1 px-1.5 space-y-0.5">
                {activeSessions.map((session, i) => (
                  <div key={session.id}>
                    <SessionRow session={session} isHero={i === 0} />
                    {(session.status === "waiting_for_approval" || session.status === "waiting_for_answer") && (
                      <ApprovalCard session={session} />
                    )}
                  </div>
                ))}
              </div>

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
                  onClick={openSettings}
                >
                  Settings
                </button>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </motion.div>

      {hasWaiting && (
        <div
          className="absolute inset-0 -z-10 rounded-[22px] blur-2xl animate-pulse"
          style={{ background: "var(--vi-alert)", opacity: 0.12 }}
        />
      )}
    </motion.div>
  );
}
