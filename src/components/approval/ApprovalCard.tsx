import { useState } from "react";
import { motion } from "framer-motion";
import { useStore } from "../../store/useStore";
import type { Session } from "../../types";

interface Props { session: Session; }

interface DiffLine {
  type: "context" | "add" | "remove";
  content: string;
  lineNum: number;
}

function parseDiff(old_string: string, new_string: string): DiffLine[] {
  if (!old_string && !new_string) return [];
  const oldLines = (old_string || "").split("\n");
  const newLines = (new_string || "").split("\n");
  const out: DiffLine[] = [];
  let ln = 1;
  for (let i = 0; i < Math.max(oldLines.length, newLines.length) && i < 14; i++) {
    const o = oldLines[i], n = newLines[i];
    if (o !== undefined && n !== undefined && o !== n) {
      out.push({ type: "remove", content: o, lineNum: ln });
      out.push({ type: "add",    content: n, lineNum: ln });
    } else if (o !== undefined) {
      out.push({ type: "context", content: o, lineNum: ln });
    } else if (n !== undefined) {
      out.push({ type: "add", content: n, lineNum: ln });
    }
    ln++;
  }
  return out.slice(0, 12);
}

export function ApprovalCard({ session }: Props) {
  const { approvePermission, answerQuestion, jumpToTerminal, platform } = useStore();
  const [selectedOption, setSelectedOption] = useState<number | null>(null);
  const [answeredText, setAnsweredText] = useState<string | null>(null);
  const [freeText, setFreeText] = useState("");
  const [approved, setApproved] = useState(false);

  const mod = platform?.os === "macos" ? "⌘" : "Ctrl+";
  const isQuestion = session.status === "waiting_for_answer";
  const toolName = session.tool_name || "Unknown";
  const toolInput = (session.tool_input || {}) as Record<string, unknown>;

  const questions = (toolInput.questions as Array<{ header: string; question: string; options?: string[] }>) || [];
  const topQuestion = questions[0];
  const options: string[] = topQuestion?.options || [];

  const filePath = (() => {
    if (toolInput.file_path) return String(toolInput.file_path);
    if (toolInput.path) return String(toolInput.path);
    return null;
  })();

  const detail = (() => {
    if (toolInput.command) return String(toolInput.command);
    if (toolInput.file_path) return String(toolInput.file_path);
    if (Array.isArray(toolInput.patterns) && toolInput.patterns.length > 0)
      return (toolInput.patterns as string[]).join(", ");
    return null;
  })();

  const isEditTool = ["Edit", "Write", "MultiEdit"].includes(toolName);
  const diffLines = isEditTool
    ? parseDiff(String(toolInput.old_string || ""), String(toolInput.new_string || ""))
    : [];
  const addCount    = diffLines.filter(l => l.type === "add").length;
  const removeCount = diffLines.filter(l => l.type === "remove").length;

  const handleOptionClick = (i: number) => {
    setSelectedOption(i);
    setAnsweredText(options[i]);
    const m: Record<string, string[]> = {};
    if (topQuestion) m[topQuestion.header] = [String(i + 1)];
    answerQuestion(session.id, m);
  };

  const handleFreeTextSubmit = () => {
    if (!freeText.trim() || !topQuestion) return;
    setAnsweredText(freeText.trim());
    answerQuestion(session.id, { [topQuestion.header]: [freeText.trim()] });
  };

  const handleAllow  = () => { setApproved(true); approvePermission(session.id, "allow"); };
  const handleAlways = () => { setApproved(true); approvePermission(session.id, "always"); };
  const handleDeny   = () => approvePermission(session.id, "deny");

  // ── Question card ──────────────────────────────────────────────────────────
  if (isQuestion) {
    return (
      <motion.div
        initial={{ opacity: 0, height: 0 }}
        animate={{ opacity: 1, height: "auto" }}
        exit={{ opacity: 0, height: 0 }}
        className="mx-1.5 mb-1 rounded-[10px] overflow-hidden"
        style={{
          background: "rgba(6,182,212,0.06)",
          border: "1px solid rgba(6,182,212,0.12)",
          padding: "10px 12px",
        }}
      >
        {/* Header: chat bubble + "Claude asks" */}
        <div className="flex items-center mb-1.5" style={{ gap: 5 }}>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="var(--vi-explore)">
            <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" />
          </svg>
          <span style={{ color: "var(--vi-explore)", fontSize: 10, fontWeight: 600 }}>
            {session.source === "claude" ? "Claude" : session.source} asks
          </span>
        </div>

        {/* Question text */}
        {topQuestion && !answeredText && (
          <p style={{ color: "rgba(255,255,255,0.9)", fontSize: 11, fontWeight: 500, marginBottom: 6 }}>
            {topQuestion.question}
          </p>
        )}

        {/* Answered confirmation */}
        {answeredText ? (
          <div className="flex items-center" style={{ gap: 8, paddingTop: 4 }}>
            <span style={{ color: "var(--vi-idle)", fontSize: 16 }}>✓</span>
            <span style={{ color: "var(--vi-idle)", fontWeight: 600, fontSize: 11 }}>{answeredText}</span>
          </div>
        ) : options.length > 0 ? (
          /* Numbered option buttons */
          <div style={{ display: "flex", flexDirection: "column", gap: 3 }} data-no-drag>
            {options.map((opt, i) => (
              <button
                key={i}
                data-no-drag
                onClick={() => handleOptionClick(i)}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                  padding: "5px 10px",
                  borderRadius: 6,
                  background: selectedOption === i ? "rgba(6,182,212,0.25)" : "rgba(6,182,212,0.15)",
                  color: "rgba(255,255,255,0.9)",
                  fontSize: 10,
                  fontWeight: 400,
                  border: "none",
                  cursor: "pointer",
                  textAlign: "left",
                  transition: "background 0.15s",
                }}
              >
                {/* Cyan square number badge — 18×18 */}
                <span style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  width: 18,
                  height: 18,
                  borderRadius: 4,
                  background: "rgba(6,182,212,0.6)",
                  color: "#fff",
                  fontSize: 10,
                  fontWeight: 700,
                  fontFamily: "var(--font-mono)",
                  flexShrink: 0,
                  opacity: 0.5,
                }}>
                  {mod}{i + 1}
                </span>
                <span>{opt}</span>
              </button>
            ))}
          </div>
        ) : topQuestion ? (
          /* Free-text input */
          <div style={{ display: "flex", flexDirection: "column", gap: 6 }} data-no-drag>
            <input
              type="text"
              value={freeText}
              autoFocus
              placeholder="Type your answer…"
              data-no-drag
              onChange={e => setFreeText(e.target.value)}
              onKeyDown={e => { if (e.key === "Enter") handleFreeTextSubmit(); }}
              style={{
                background: "rgba(255,255,255,0.06)",
                border: "1px solid rgba(255,255,255,0.14)",
                borderRadius: 6,
                padding: "5px 10px",
                color: "rgba(255,255,255,0.9)",
                fontSize: 11,
                outline: "none",
                width: "100%",
              }}
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
          <button className="approve-btn allow" style={{ width: "100%" }} data-no-drag onClick={() => jumpToTerminal(session.id)}>
            Go to Terminal
          </button>
        )}
      </motion.div>
    );
  }

  // ── Permission approval card ───────────────────────────────────────────────
  return (
    <motion.div
      initial={{ opacity: 0, height: 0 }}
      animate={{ opacity: 1, height: "auto" }}
      exit={{ opacity: 0, height: 0 }}
      className="mx-1.5 mb-1 rounded-[10px] overflow-hidden"
      style={{
        background: "rgba(249,115,22,0.04)",
        border: "1px solid rgba(249,115,22,0.1)",
        padding: "10px 12px",
      }}
    >
      {approved ? (
        /* Confirmation state */
        <div className="flex items-center" style={{ gap: 8 }}>
          <span style={{ color: "var(--vi-idle)", fontSize: 16 }}>✓</span>
          <span style={{ color: "var(--vi-idle)", fontWeight: 600, fontSize: 11 }}>Allowed</span>
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: 3 }}>
          {/* Header: orange dot + "Permission Request" */}
          <div className="flex items-center" style={{ gap: 6, marginBottom: 2 }}>
            <div style={{ width: 6, height: 6, borderRadius: "50%", background: "var(--vi-alert)", flexShrink: 0 }} />
            <span style={{ color: "rgba(255,255,255,0.4)", fontSize: 11 }}>Permission Request</span>
          </div>

          {/* Tool line: ⚠ + toolName (orange, bold) + file path (white) */}
          <div className="flex items-center" style={{ gap: 6, marginBottom: 4 }}>
            <span style={{ color: "var(--vi-alert)", fontSize: 11 }}>⚠</span>
            <span style={{ color: "var(--vi-alert)", fontSize: 11, fontWeight: 600 }}>{toolName}</span>
            {filePath && (
              <span style={{ color: "rgba(255,255,255,0.85)", fontSize: 11 }}>
                {filePath.split("/").slice(-2).join("/")}
              </span>
            )}
          </div>

          {/* Diff view for Edit/Write/MultiEdit */}
          {isEditTool && diffLines.length > 0 && (
            <div style={{ marginBottom: 4 }}>
              <div style={{
                background: "rgba(255,255,255,0.04)",
                borderRadius: 4,
                padding: "3px 0",
                fontSize: 9,
                fontFamily: "var(--font-mono)",
                overflow: "hidden",
              }}>
                {diffLines.map((line, i) => (
                  <div
                    key={i}
                    style={{
                      display: "flex",
                      gap: 4,
                      padding: "0 6px",
                      background:
                        line.type === "add"    ? "rgba(34,197,94,0.08)" :
                        line.type === "remove" ? "rgba(249,115,22,0.08)" :
                        "transparent",
                      color:
                        line.type === "add"    ? "rgb(34,197,94)" :
                        line.type === "remove" ? "rgb(252,165,165)" :
                        "rgba(255,255,255,0.35)",
                    }}
                  >
                    <span style={{ color: "rgba(255,255,255,0.2)", width: 14, flexShrink: 0 }}>
                      {line.lineNum}
                    </span>
                    <span style={{ opacity: 0.6, width: 10, flexShrink: 0 }}>
                      {line.type === "add" ? "+" : line.type === "remove" ? "-" : " "}
                    </span>
                    <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                      {line.content}
                    </span>
                  </div>
                ))}
              </div>
              {/* Diff stats */}
              {(addCount > 0 || removeCount > 0) && (
                <div style={{ color: "rgba(255,255,255,0.3)", fontSize: 9, marginTop: 2 }}>
                  {addCount > 0 && <span style={{ color: "rgb(34,197,94)" }}>+{addCount}</span>}
                  {addCount > 0 && removeCount > 0 && " "}
                  {removeCount > 0 && <span style={{ color: "rgb(252,165,165)" }}>-{removeCount}</span>}
                </div>
              )}
            </div>
          )}

          {/* Command / detail for non-Edit tools */}
          {!isEditTool && detail && (
            <div style={{
              background: "rgba(0,0,0,0.3)",
              borderRadius: 4,
              padding: "4px 8px",
              marginBottom: 4,
            }}>
              <code style={{
                color: "rgba(255,255,255,0.5)",
                fontSize: 10,
                fontFamily: "var(--font-mono)",
                display: "-webkit-box",
                WebkitLineClamp: 3,
                WebkitBoxOrient: "vertical",
                overflow: "hidden",
              } as React.CSSProperties}>
                {detail}
              </code>
            </div>
          )}

          {/* Deny / Allow row at BOTTOM */}
          <div style={{ display: "flex", gap: 6, paddingTop: 4 }}>
            <button className="approve-btn deny flex-1" data-no-drag onClick={handleDeny}>
              Deny <span style={{ opacity: 0.5 }}>{mod}N</span>
            </button>
            <button className="approve-btn allow flex-1" data-no-drag onClick={handleAllow}>
              Allow <span style={{ opacity: 0.5 }}>{mod}Y</span>
            </button>
          </div>

          {/* Always Allow */}
          <button
            data-no-drag
            onClick={handleAlways}
            style={{
              color: "var(--vi-explore)",
              fontSize: 9,
              opacity: 0.6,
              textAlign: "center",
              marginTop: 2,
              background: "none",
              border: "none",
              cursor: "pointer",
            }}
          >
            Always Allow
          </button>
        </div>
      )}
    </motion.div>
  );
}
