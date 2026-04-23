import { useEffect, useState } from "react";
import { useStore } from "./store/useStore";
import { NotchPanel } from "./components/notch/NotchPanel";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

interface UpdateInfo { version: string; body?: string; }

export default function App() {
  const init = useStore((s) => s.init);
  const sessions = useStore((s) => s.sessions);
  const approvePermission = useStore((s) => s.approvePermission);
  const answerQuestion = useStore((s) => s.answerQuestion);
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  const [installing, setInstalling] = useState(false);

  useEffect(() => { init(); }, [init]);

  // Listen for background update check result
  useEffect(() => {
    const unlisten = listen<UpdateInfo>("update-available", (e) => setUpdate(e.payload));
    return () => { unlisten.then(fn => fn()); };
  }, []);

  // Keyboard shortcuts for approval/question
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;
      if (!mod) return;
      const approvalSession = sessions.find((s) => s.status === "waiting_for_approval");
      const questionSession = sessions.find((s) => s.status === "waiting_for_answer");
      if (e.key === "y" || e.key === "Y") {
        if (approvalSession) { e.preventDefault(); approvePermission(approvalSession.id, "allow"); }
      } else if (e.key === "n" || e.key === "N") {
        if (approvalSession) { e.preventDefault(); approvePermission(approvalSession.id, "deny"); }
      } else {
        const num = parseInt(e.key, 10);
        if (!isNaN(num) && num >= 1 && num <= 9 && questionSession) {
          const toolInput = (questionSession.tool_input || {}) as Record<string, unknown>;
          const questions = (toolInput.questions as Array<{ header: string; options?: string[] }>) || [];
          const topQ = questions[0];
          if (topQ?.options && topQ.options[num - 1] !== undefined) {
            e.preventDefault();
            answerQuestion(questionSession.id, { [topQ.header]: [String(num)] });
          }
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [sessions, approvePermission, answerQuestion]);

  const handleInstall = async () => {
    setInstalling(true);
    try {
      await invoke("install_update");
    } catch (e) {
      console.error("Update failed:", e);
      setInstalling(false);
    }
  };

  return (
    <div className="w-screen h-screen flex justify-center">
      <NotchPanel />

      {/* Update notification — floats below the notch */}
      {update && !installing && (
        <div
          style={{
            position: "fixed",
            top: 52,
            left: "50%",
            transform: "translateX(-50%)",
            background: "rgba(0,0,0,0.92)",
            border: "1px solid rgba(255,255,255,0.12)",
            borderRadius: 10,
            padding: "8px 14px",
            display: "flex",
            alignItems: "center",
            gap: 10,
            fontSize: 11,
            color: "rgba(255,255,255,0.8)",
            boxShadow: "0 4px 20px rgba(0,0,0,0.5), 0 0 12px rgba(6,182,212,0.1)",
            zIndex: 9999,
            whiteSpace: "nowrap",
          }}
          data-no-drag
        >
          <span style={{ color: "var(--vi-explore)", fontSize: 13 }}>↑</span>
          <span>
            <span style={{ color: "#fff", fontWeight: 500 }}>v{update.version}</span> available
          </span>
          <button
            onClick={handleInstall}
            style={{
              background: "rgba(255,255,255,0.9)",
              color: "#000",
              border: "none",
              borderRadius: 5,
              padding: "3px 10px",
              fontSize: 10,
              fontWeight: 600,
              cursor: "pointer",
            }}
            data-no-drag
          >
            Install &amp; Restart
          </button>
          <button
            onClick={() => setUpdate(null)}
            style={{
              background: "none",
              border: "none",
              color: "rgba(255,255,255,0.35)",
              fontSize: 13,
              cursor: "pointer",
              padding: "0 2px",
            }}
            data-no-drag
          >
            ×
          </button>
        </div>
      )}

      {installing && (
        <div
          style={{
            position: "fixed",
            top: 52,
            left: "50%",
            transform: "translateX(-50%)",
            background: "rgba(0,0,0,0.92)",
            border: "1px solid rgba(255,255,255,0.12)",
            borderRadius: 10,
            padding: "8px 14px",
            fontSize: 11,
            color: "rgba(255,255,255,0.6)",
            boxShadow: "0 4px 20px rgba(0,0,0,0.5)",
            zIndex: 9999,
          }}
        >
          <span className="pulse-dot" style={{ color: "var(--vi-explore)", marginRight: 6 }}>●</span>
          Downloading update…
        </div>
      )}
    </div>
  );
}
