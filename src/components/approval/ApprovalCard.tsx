import { useState } from "react";
import { motion } from "framer-motion";
import { useStore } from "../../store/useStore";
import type { Session } from "../../types";

interface Props {
  session: Session;
}

interface DiffLine {
  type: "context" | "add" | "remove" | "header";
  content: string;
  lineNum?: number;
}

function parseDiff(old_string: string, new_string: string): DiffLine[] {
  if (!old_string && !new_string) return [];
  const oldLines = (old_string || "").split("\n");
  const newLines = (new_string || "").split("\n");
  const lines: DiffLine[] = [];
  let ln = 1;
  for (let i = 0; i < Math.max(oldLines.length, newLines.length) && i < 14; i++) {
    const o = oldLines[i];
    const n = newLines[i];
    if (o !== undefined && n !== undefined && o !== n) {
      lines.push({ type: "remove", content: o, lineNum: ln });
      lines.push({ type: "add", content: n, lineNum: ln });
    } else if (o !== undefined) {
      lines.push({ type: "context", content: o, lineNum: ln });
    } else if (n !== undefined) {
      lines.push({ type: "add", content: n, lineNum: ln });
    }
    ln++;
  }
  return lines.slice(0, 12);
}

export function ApprovalCard({ session }: Props) {
  const { approvePermission, answerQuestion, jumpToTerminal, platform } = useStore();
  const [selectedOption, setSelectedOption] = useState<number | null>(null);
  const [answeredText, setAnsweredText] = useState<string | null>(null);
  const [freeText, setFreeText] = useState("");
  const [approved, setApproved] = useState(false);

  // Platform-appropriate modifier key label
  const mod = platform?.os === "macos" ? "⌘" : "Ctrl+";

  const isQuestion = session.status === "waiting_for_answer";
  const toolName = session.tool_name || "Unknown";
  const toolInput = (session.tool_input || {}) as Record<string, unknown>;

  const questions = (toolInput.questions as Array<{ header: string; question: string; options?: string[] }>) || [];
  const topQuestion = questions[0];
  const options: string[] = topQuestion?.options || [];

  const getFilePath = () => {
    if (toolInput.file_path) return String(toolInput.file_path);
    if (toolInput.path) return String(toolInput.path);
    return null;
  };

  const getDetail = () => {
    if (toolInput.command) return String(toolInput.command);
    if (toolInput.file_path) return String(toolInput.file_path);
    if (Array.isArray(toolInput.patterns) && toolInput.patterns.length > 0) {
      return (toolInput.patterns as string[]).join(", ");
    }
    return null;
  };

  const isEditTool = ["Edit", "Write", "MultiEdit"].includes(toolName);
  const diffLines = isEditTool
    ? parseDiff(String(toolInput.old_string || ""), String(toolInput.new_string || ""))
    : [];
  const addCount = diffLines.filter((l) => l.type === "add").length;
  const removeCount = diffLines.filter((l) => l.type === "remove").length;
  const filePath = getFilePath();
  const detail = getDetail();

  const handleOptionClick = (i: number) => {
    setSelectedOption(i);
    setAnsweredText(options[i]);
    const answerMap: Record<string, string[]> = {};
    if (topQuestion) answerMap[topQuestion.header] = [String(i + 1)];
    answerQuestion(session.id, answerMap);
  };

  const handleFreeTextSubmit = () => {
    if (!freeText.trim() || !topQuestion) return;
    setAnsweredText(freeText.trim());
    answerQuestion(session.id, { [topQuestion.header]: [freeText.trim()] });
  };

  const handleAllow = () => {
    setApproved(true);
    approvePermission(session.id, "allow");
  };

  const handleAlways = () => {
    setApproved(true);
    approvePermission(session.id, "always");
  };

  // ── Question card ──────────────────────────────────────────
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
          <svg width="12" height="12" viewBox="0 0 24 24" fill="var(--vi-question)" opacity="0.9">
            <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" />
          </svg>
          <span className="text-[11px] font-medium" style={{ color: "var(--vi-question)" }}>
            {session.source === "claude" ? "Claude" : session.source} asks
          </span>
        </div>

        {topQuestion && !answeredText && (
          <p className="text-[11px] mb-2 font-medium" style={{ color: "rgba(255,255,255,0.85)" }}>
            {topQuestion.question}
          </p>
        )}

        {/* Confirmation state after answering */}
        {answeredText ? (
          <div className="flex items-center gap-2 py-1">
            <span style={{ color: "var(--vi-idle)", fontSize: "16px" }}>✓</span>
            <span style={{ color: "var(--vi-idle)", fontWeight: 600, fontSize: "11px" }}>
              {answeredText}
            </span>
          </div>
        ) : options.length > 0 ? (
          /* Numbered option buttons */
          <div className="space-y-1" data-no-drag>
            {options.map((opt, i) => (
              <button
                key={i}
                className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-[6px] text-left transition-all"
                style={{
                  background: selectedOption === i ? "rgba(6,182,212,0.25)" : "rgba(6,182,212,0.15)",
                  cursor: "pointer",
                  fontSize: "10px",
                  fontWeight: 400,
                  color: "rgba(255,255,255,0.9)",
                }}
                data-no-drag
                onClick={() => handleOptionClick(i)}
              >
                <span className="font-mono text-[9px] opacity-50">{mod}{i + 1}</span>
                <span>{opt}</span>
              </button>
            ))}
          </div>
        ) : topQuestion ? (
          /* Free-text input */
          <div className="space-y-1.5" data-no-drag>
            <input
              type="text"
              value={freeText}
              autoFocus
              placeholder="Type your answer…"
              className="w-full px-2.5 py-1.5 rounded-[6px] text-[11px]"
              style={{
                background: "rgba(255,255,255,0.06)",
                border: "1px solid rgba(255,255,255,0.14)",
                color: "rgba(255,255,255,0.9)",
                outline: "none",
              }}
              data-no-drag
              onChange={(e) => setFreeText(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter") handleFreeTextSubmit(); }}
            />
            <button
              className="approve-btn allow"
              style={{ width: "100%", opacity: freeText.trim() ? 1 : 0.4 }}
              data-no-drag
              onClick={handleFreeTextSubmit}
            >
              Send ↵
            </button>
          </div>
        ) : (
          /* No question data — jump to terminal */
          <button
            className="approve-btn allow"
            style={{ width: "100%" }}
            data-no-drag
            onClick={() => jumpToTerminal(session.id)}
          >
            Go to Terminal
          </button>
        )}
      </motion.div>
    );
  }

  // ── Permission approval card ───────────────────────────────
  return (
    <motion.div
      initial={{ opacity: 0, height: 0 }}
      animate={{ opacity: 1, height: "auto" }}
      exit={{ opacity: 0, height: 0 }}
      className="mx-1.5 mb-1 rounded-[10px] overflow-hidden"
      style={{ background: "rgba(249,115,22,0.06)", border: "1px solid rgba(249,115,22,0.12)" }}
    >
      {/* Confirmation state after approving */}
      {approved ? (
        <div className="flex items-center gap-2 px-3 py-3">
          <span style={{ color: "var(--vi-idle)", fontSize: "16px" }}>✓</span>
          <span style={{ color: "var(--vi-idle)", fontWeight: 600, fontSize: "11px" }}>Allowed</span>
        </div>
      ) : (
        <>
          {/* Deny / Allow row */}
          <div className="flex gap-1.5 p-2 pb-0">
            <button
              className="approve-btn deny flex-1"
              data-no-drag
              onClick={() => approvePermission(session.id, "deny")}
            >
              Deny <span style={{ opacity: 0.5 }}>{mod}N</span>
            </button>
            <button
              className="approve-btn allow flex-1"
              data-no-drag
              onClick={handleAllow}
            >
              Allow <span style={{ opacity: 0.5 }}>{mod}Y</span>
            </button>
          </div>

          <div className="px-2.5 pt-2 pb-2.5">
            <div className="flex items-center gap-1.5 mb-1.5">
              <span className="text-[11px] font-medium" style={{ color: "var(--vi-alert)" }}>{toolName}</span>
              {filePath && (
                <span className="text-[10px]" style={{ color: "#6b7280" }}>
                  {filePath.split("/").slice(-2).join("/")}
                </span>
              )}
            </div>

            {/* Diff view for Edit/Write/MultiEdit tools */}
            {isEditTool && diffLines.length > 0 && (
              <div
                className="rounded-[6px] overflow-hidden mb-1.5"
                style={{ background: "rgba(0,0,0,0.4)", fontSize: "10px", fontFamily: "monospace" }}
              >
                {filePath && (
                  <div
                    className="px-2 py-1 flex items-center justify-between"
                    style={{ background: "rgba(255,255,255,0.04)", color: "#6b7280" }}
                  >
                    <span className="truncate">{filePath}</span>
                    {(addCount > 0 || removeCount > 0) && (
                      <span style={{ flexShrink: 0, marginLeft: 8 }}>
                        {addCount > 0 && <span style={{ color: "#34d399" }}>+{addCount}</span>}
                        {addCount > 0 && removeCount > 0 && " "}
                        {removeCount > 0 && <span style={{ color: "#f87171" }}>-{removeCount}</span>}
                      </span>
                    )}
                  </div>
                )}
                {diffLines.map((line, i) => (
                  <div
                    key={i}
                    className="px-2 py-0.5 flex gap-2"
                    style={{
                      background:
                        line.type === "add" ? "rgba(52,211,153,0.1)" :
                        line.type === "remove" ? "rgba(248,113,113,0.1)" :
                        "transparent",
                      color:
                        line.type === "add" ? "#34d399" :
                        line.type === "remove" ? "#f87171" :
                        "rgba(255,255,255,0.4)",
                    }}
                  >
                    <span style={{ opacity: 0.4, width: "10px", flexShrink: 0 }}>
                      {line.type === "add" ? "+" : line.type === "remove" ? "-" : " "}
                    </span>
                    <span className="truncate">{line.content}</span>
                  </div>
                ))}
              </div>
            )}

            {/* Command / detail for non-Edit tools */}
            {!isEditTool && detail && (
              <div className="mb-1.5 px-2 py-1.5 rounded-[6px] overflow-hidden" style={{ background: "rgba(0,0,0,0.3)" }}>
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

            {/* Always Allow — behavior "always" so hook auto-approves future calls */}
            <button
              className="w-full text-[9px] text-center mt-0.5"
              style={{ color: "var(--vi-explore)", opacity: 0.6 }}
              data-no-drag
              onClick={handleAlways}
            >
              Always Allow
            </button>
          </div>
        </>
      )}
    </motion.div>
  );
}
