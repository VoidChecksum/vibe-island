import { useEffect } from "react";
import { useStore } from "./store/useStore";
import { NotchPanel } from "./components/notch/NotchPanel";

export default function App() {
  const init = useStore((s) => s.init);
  const sessions = useStore((s) => s.sessions);
  const approvePermission = useStore((s) => s.approvePermission);
  const answerQuestion = useStore((s) => s.answerQuestion);

  useEffect(() => {
    init();
  }, [init]);

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

  return (
    <div className="w-screen h-screen flex justify-center">
      <NotchPanel />
    </div>
  );
}
