import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../../store/useStore";

const STEPS = [
  {
    title: "A Dynamic Island for your AI coding tools",
    features: [
      "Permissions at a glance",
      "Click to jump to the right window",
      "Zero config, works out of the box",
    ],
  },
  {
    title: "Everything. One glance.",
    features: [
      "See all your AI agent sessions",
      "Approve permissions without switching",
      "Jump to the exact terminal tab",
    ],
  },
  {
    title: "Your Environment",
    features: [
      "Claude Code, Codex, Gemini, Cursor",
      "Windsurf, Copilot, OpenCode, and more",
      "Works on macOS, Windows, and Linux",
    ],
  },
];

interface Props {
  onComplete: () => void;
}

export function OnboardingScreen({ onComplete }: Props) {
  const [step, setStep] = useState(0);
  const [installing, setInstalling] = useState(false);
  const { platform } = useStore();

  const isLast = step === STEPS.length - 1;

  const handleFinish = async () => {
    setInstalling(true);
    try {
      await invoke("install_hooks");
    } catch (e) {
      console.error("Hook install error:", e);
    }
    setInstalling(false);
    onComplete();
  };

  return (
    <div className="fixed inset-0 bg-island-bg flex items-center justify-center">
      <div className="max-w-md w-full px-8">
        <AnimatePresence mode="wait">
          <motion.div
            key={step}
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
            className="text-center"
          >
            {/* Island preview */}
            <div className="mx-auto mb-8 w-48 h-12 rounded-island bg-island-surface border border-island-border flex items-center justify-center">
              <div className="flex gap-1.5">
                <div className="w-2.5 h-2.5 rounded-full bg-[#D97706]" />
                <div className="w-2.5 h-2.5 rounded-full bg-[#10B981]" />
                <div className="w-2.5 h-2.5 rounded-full bg-[#6366F1]" />
              </div>
            </div>

            <h2 className="text-xl font-semibold text-island-text mb-4">
              {STEPS[step].title}
            </h2>

            <ul className="space-y-3 mb-8">
              {STEPS[step].features.map((f, i) => (
                <motion.li
                  key={i}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: i * 0.1 }}
                  className="text-sm text-island-muted flex items-center gap-2"
                >
                  <span className="text-island-accent">✦</span>
                  {f}
                </motion.li>
              ))}
            </ul>

            {/* Platform-specific note */}
            {isLast && platform?.compositor === "hyprland" && (
              <div className="mb-4 p-3 rounded-lg bg-island-surface text-xs text-island-muted text-left">
                <span className="text-island-accent font-medium">Hyprland detected</span>
                <br />
                Window rules will be applied automatically via hyprctl.
              </div>
            )}
          </motion.div>
        </AnimatePresence>

        {/* Navigation */}
        <div className="flex items-center justify-between">
          {/* Step dots */}
          <div className="flex gap-2">
            {STEPS.map((_, i) => (
              <div
                key={i}
                className={`w-2 h-2 rounded-full transition-colors ${
                  i === step ? "bg-island-accent" : "bg-island-border"
                }`}
              />
            ))}
          </div>

          {/* Button */}
          {isLast ? (
            <button
              onClick={handleFinish}
              disabled={installing}
              className="px-6 py-2.5 rounded-full bg-island-accent text-island-bg
                         font-medium text-sm hover:opacity-90 transition-opacity
                         disabled:opacity-50"
            >
              {installing ? "Setting up..." : "Start Vibing"}
            </button>
          ) : (
            <button
              onClick={() => setStep(step + 1)}
              className="px-6 py-2.5 rounded-full bg-island-surface text-island-text
                         font-medium text-sm hover:bg-island-border transition-colors"
            >
              Next
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
