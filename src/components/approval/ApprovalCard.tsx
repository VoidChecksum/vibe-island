import { useState } from "react";
import { motion } from "framer-motion";
import { useStore } from "../../store/useStore";
import type { Session } from "../../types";

interface Props {
  session: Session;
}

export function ApprovalCard({ session }: Props) {
  const { approvePermission, answerQuestion, jumpToTerminal } = useStore();
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const isQuestion = session.status === "waiting_for_answer";
  const toolName = session.tool_name || "Unknown";
  const toolInput = (session.tool_input || {}) as Record<string, unknown>;

  const questions = (toolInput.questions as Array<{ header: string; question: string }>) || [];

  const getDetail = () => {
    if (toolInput.command) return String(toolInput.command);
    if (toolInput.file_path) return String(toolInput.file_path);
    if (Array.isArray(toolInput.patterns) && toolInput.patterns.length > 0) {
      return (toolInput.patterns as string[]).join(" && ");
    }
    return null;
  };

  const detail = getDetail();

  const handleAnswerSubmit = () => {
    const answerMap: Record<string, string[]> = {};
    for (const q of questions) {
      if (answers[q.header]) answerMap[q.header] = [answers[q.header]];
    }
    answerQuestion(session.id, answerMap);
  };

  if (isQuestion) {
    return (
      <motion.div
        initial={{ opacity: 0, height: 0 }}
        animate={{ opacity: 1, height: "auto" }}
        exit={{ opacity: 0, height: 0 }}
        className="mx-1.5 mb-1 p-2.5 rounded-[10px]"
        style={{ background: "rgba(192,132,252,0.08)", border: "1px solid rgba(192,132,252,0.15)" }}
      >
        <div className="flex items-center gap-1.5 mb-2">
          <span className="text-[11px] font-medium" style={{ color: "var(--vi-question)" }}>
            {session.source === "claude" ? "Claude" : session.source} is waiting for an answer
          </span>
        </div>

        {questions.length > 0 ? (
          <div className="space-y-1.5 mb-2">
            {questions.map((q, i) => (
              <div key={i}>
                <p className="text-[10px] mb-1" style={{ color: "rgba(255,255,255,0.5)" }}>
                  {q.question}
                </p>
                <input
                  type="text"
                  value={answers[q.header] || ""}
                  onChange={(e) => setAnswers((a) => ({ ...a, [q.header]: e.target.value }))}
                  onKeyDown={(e) => e.key === "Enter" && handleAnswerSubmit()}
                  placeholder="Your answer…"
                  className="w-full px-2 py-1 rounded-[6px] text-[11px] outline-none"
                  style={{ background: "rgba(0,0,0,0.3)", border: "1px solid rgba(192,132,252,0.2)", color: "#fff" }}
                  data-no-drag
                />
              </div>
            ))}
            <button
              className="approve-btn allow mt-1"
              style={{ width: "100%" }}
              data-no-drag
              onClick={handleAnswerSubmit}
            >
              Submit Answer
            </button>
          </div>
        ) : (
          <>
            <p className="text-[10px] mb-2" style={{ color: "rgba(255,255,255,0.4)" }}>
              Please answer in the terminal
            </p>
            <button
              className="approve-btn allow"
              style={{ width: "100%" }}
              data-no-drag
              onClick={() => jumpToTerminal(session.id)}
            >
              Go to Terminal
            </button>
          </>
        )}
      </motion.div>
    );
  }

  return (
    <motion.div
      initial={{ opacity: 0, height: 0 }}
      animate={{ opacity: 1, height: "auto" }}
      exit={{ opacity: 0, height: 0 }}
      className="mx-1.5 mb-1 p-2.5 rounded-[10px]"
      style={{ background: "rgba(249,115,22,0.06)", border: "1px solid rgba(249,115,22,0.12)" }}
    >
      <div className="flex items-center gap-1.5 mb-1.5">
        <span className="text-[11px] font-medium" style={{ color: "var(--vi-alert)" }}>{toolName}</span>
      </div>

      {detail && (
        <div className="mb-2 px-2 py-1.5 rounded-[6px] overflow-hidden" style={{ background: "rgba(0,0,0,0.3)" }}>
          <code
            className="text-[10px] break-all leading-relaxed"
            style={{
              color: "rgba(255,255,255,0.5)",
              fontFamily: "'SF Mono','Fira Code','Cascadia Code',monospace",
              display: "-webkit-box",
              WebkitLineClamp: 3,
              WebkitBoxOrient: "vertical",
              overflow: "hidden",
            } as React.CSSProperties}
          >
            {detail}
          </code>
        </div>
      )}

      <div className="approve-bar">
        <button className="approve-btn deny" data-no-drag onClick={() => approvePermission(session.id, "deny")}>Deny</button>
        <button className="approve-btn allow" data-no-drag onClick={() => approvePermission(session.id, "allow")}>Allow Once</button>
        <button className="approve-btn always" data-no-drag onClick={() => approvePermission(session.id, "always")}>Always Allow</button>
      </div>
    </motion.div>
  );
}
