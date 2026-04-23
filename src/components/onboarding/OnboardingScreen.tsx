import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../../store/useStore";
import wallpaperUrl from "../../assets/brand/onboarding-wallpaper.jpg";
import extensionIconUrl from "../../assets/brand/extension-icon.png";
import ceremonyUrl from "../../assets/sounds/onboarding-ceremony.wav";

const STEPS = [
  {
    title: "A Dynamic Island for your AI coding tools",
    eyebrow: "Vibe Island",
    features: [
      "Permissions at a glance",
      "Click to jump to the right window",
      "Zero config, works out of the box",
    ],
  },
  {
    title: "Everything. One glance.",
    eyebrow: "15 hours saved",
    features: [
      "You have 4 agents running",
      "One of them has been waiting for you",
      "Approve without switching tabs",
    ],
  },
  {
    title: "Your Environment",
    eyebrow: "Zero config",
    features: [
      "Claude Code, Codex, Gemini, Cursor, Kiro",
      "Windsurf, Copilot, OpenCode, Droid, Amp, Kimi",
      "Restart running sessions, or start a new one",
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
      new Audio(ceremonyUrl).play().catch(() => {});
      await invoke("install_hooks");
    } catch (e) {
      console.error("Hook install error:", e);
    }
    setInstalling(false);
    onComplete();
  };

  return (
    <div
      className="fixed inset-0 flex items-center justify-center overflow-hidden"
      style={{
        background: `linear-gradient(180deg, rgba(0,0,0,.50), rgba(0,0,0,.94)), url(${wallpaperUrl}) center/cover`,
      }}
    >
      <div className="absolute inset-x-0 top-0 h-20 bg-gradient-to-b from-black/80 to-transparent" />
      <div className="max-w-md w-full px-8 relative">
        <AnimatePresence mode="wait">
          <motion.div
            key={step}
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
            className="text-center"
          >
            <div className="mx-auto mb-7 w-56 rounded-[22px] border border-white/10 bg-black/90 shadow-2xl shadow-black/60 overflow-hidden">
              <div className="h-10 px-4 flex items-center gap-2">
                <img src={extensionIconUrl} alt="" className="w-5 h-5 rounded-md" />
                <span className="text-[11px] font-semibold text-white/90">Vibe Island</span>
                <span className="ml-auto flex items-center gap-1 text-[9px] text-orange-300">
                  <span className="w-1.5 h-1.5 rounded-full bg-orange-400 animate-pulse" />
                  1 waiting
                </span>
              </div>
              <div className="border-t border-white/10 px-3 py-2">
                <div className="flex items-center gap-2 text-[10px] text-white/70">
                  <span className="w-2 h-2 rounded-full bg-emerald-400" />
                  <span>Claude · Codex · Gemini · Cursor</span>
                </div>
              </div>
            </div>

            <p className="text-[11px] uppercase tracking-[0.22em] text-white/45 mb-3">
              {STEPS[step].eyebrow}
            </p>
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
