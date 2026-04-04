import { useState, useEffect } from "react";
import { useStore } from "../../store/useStore";
import type { AppConfig } from "../../types";
import { invoke } from "@tauri-apps/api/core";

export function SettingsPanel() {
  const { config, updateConfig, platform } = useStore();
  const [localConfig, setLocalConfig] = useState<AppConfig | null>(null);
  const [hookStatus, setHookStatus] = useState<string>("");

  useEffect(() => {
    if (config) setLocalConfig({ ...config });
  }, [config]);

  if (!localConfig) return <div className="p-4 text-island-muted">Loading...</div>;

  const save = () => {
    if (localConfig) updateConfig(localConfig);
  };

  const installHooks = async () => {
    setHookStatus("Installing...");
    try {
      const result = await invoke<string>("install_hooks");
      setHookStatus(result);
    } catch (e) {
      setHookStatus(`Error: ${e}`);
    }
  };

  return (
    <div className="h-full bg-island-bg text-island-text overflow-y-auto">
      <div className="p-6 space-y-6">
        <h1 className="text-xl font-semibold">Vibe Island Settings</h1>

        {/* Platform info */}
        {platform && (
          <div className="p-3 rounded-lg bg-island-surface text-xs space-y-1">
            <div className="flex justify-between">
              <span className="text-island-muted">Platform</span>
              <span>{platform.os} / {platform.compositor}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-island-muted">Desktop</span>
              <span>{platform.desktop}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-island-muted">Wayland</span>
              <span>{platform.wayland ? "Yes" : "No"}</span>
            </div>
          </div>
        )}

        {/* Display */}
        <Section title="Display">
          <Row label="Position">
            <select
              value={localConfig.display.position}
              onChange={(e) => {
                setLocalConfig({
                  ...localConfig,
                  display: { ...localConfig.display, position: e.target.value },
                });
                save();
              }}
              className="bg-island-surface border border-island-border rounded px-2 py-1 text-sm"
            >
              <option value="top-center">Top Center</option>
              <option value="top-left">Top Left</option>
              <option value="top-right">Top Right</option>
            </select>
          </Row>
          <Row label="Opacity">
            <input
              type="range"
              min="0.5"
              max="1"
              step="0.05"
              value={localConfig.display.opacity}
              onChange={(e) => {
                setLocalConfig({
                  ...localConfig,
                  display: { ...localConfig.display, opacity: parseFloat(e.target.value) },
                });
                save();
              }}
              className="w-24"
            />
            <span className="text-xs text-island-muted ml-2">
              {Math.round(localConfig.display.opacity * 100)}%
            </span>
          </Row>
        </Section>

        {/* Layout */}
        <Section title="Layout">
          <Row label="Style">
            <select
              value={localConfig.layout.style}
              onChange={(e) => {
                setLocalConfig({
                  ...localConfig,
                  layout: { ...localConfig.layout, style: e.target.value },
                });
                save();
              }}
              className="bg-island-surface border border-island-border rounded px-2 py-1 text-sm"
            >
              <option value="clean">Clean</option>
              <option value="detailed">Detailed</option>
              <option value="compact">Compact</option>
            </select>
          </Row>
          <Toggle
            label="Show tool names"
            value={localConfig.layout.show_tool_names}
            onChange={(v) => {
              setLocalConfig({
                ...localConfig,
                layout: { ...localConfig.layout, show_tool_names: v },
              });
              save();
            }}
          />
          <Toggle
            label="Show working directory"
            value={localConfig.layout.show_cwd}
            onChange={(v) => {
              setLocalConfig({
                ...localConfig,
                layout: { ...localConfig.layout, show_cwd: v },
              });
              save();
            }}
          />
        </Section>

        {/* Sound */}
        <Section title="Sound">
          <Toggle
            label="Enable sounds"
            value={localConfig.sound.enabled}
            onChange={(v) => {
              setLocalConfig({
                ...localConfig,
                sound: { ...localConfig.sound, enabled: v },
              });
              save();
            }}
          />
          <Row label="Volume">
            <input
              type="range"
              min="0"
              max="1"
              step="0.1"
              value={localConfig.sound.volume}
              onChange={(e) => {
                setLocalConfig({
                  ...localConfig,
                  sound: { ...localConfig.sound, volume: parseFloat(e.target.value) },
                });
                save();
              }}
              className="w-24"
            />
          </Row>
        </Section>

        {/* Hooks */}
        <Section title="CLI Hooks">
          <p className="text-xs text-island-muted mb-3">
            Install event hooks for your AI coding tools. This enables Vibe Island
            to monitor sessions and handle permissions.
          </p>
          <button
            onClick={installHooks}
            className="w-full py-2 rounded-lg bg-island-accent/20 text-island-accent
                       hover:bg-island-accent/30 transition-colors text-sm font-medium"
          >
            Install / Repair Hooks
          </button>
          {hookStatus && (
            <pre className="mt-2 p-2 rounded bg-island-surface text-[10px] text-island-muted whitespace-pre-wrap">
              {hookStatus}
            </pre>
          )}
        </Section>

        {/* Hyprland Config */}
        {platform?.compositor === "hyprland" && (
          <Section title="Hyprland">
            <p className="text-xs text-island-muted mb-2">
              Add these rules to your <code>~/.config/hypr/hyprland.conf</code>:
            </p>
            <pre className="p-2 rounded bg-island-surface text-[10px] text-island-accent font-mono whitespace-pre">
{`windowrulev2 = float, class:^(vibe-island)$
windowrulev2 = pin, class:^(vibe-island)$
windowrulev2 = noborder, class:^(vibe-island)$
windowrulev2 = noshadow, class:^(vibe-island)$
windowrulev2 = noanim, class:^(vibe-island)$
windowrulev2 = move 33% 0, class:^(vibe-island)$`}
            </pre>
          </Section>
        )}

        {/* About */}
        <Section title="About">
          <p className="text-xs text-island-muted">
            Vibe Island v1.0.0 — A Dynamic Island for your AI coding tools.
          </p>
          <p className="text-xs text-island-muted mt-1">
            Cross-platform • Open Source • No license required
          </p>
        </Section>
      </div>
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h2 className="text-sm font-medium text-island-text mb-2">{title}</h2>
      <div className="space-y-2">{children}</div>
    </div>
  );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-xs text-island-muted">{label}</span>
      <div className="flex items-center">{children}</div>
    </div>
  );
}

function Toggle({
  label,
  value,
  onChange,
}: {
  label: string;
  value: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-xs text-island-muted">{label}</span>
      <button
        className={`w-9 h-5 rounded-full transition-colors ${
          value ? "bg-island-accent" : "bg-island-border"
        }`}
        onClick={() => onChange(!value)}
      >
        <div
          className={`w-3.5 h-3.5 rounded-full bg-white transition-transform ${
            value ? "translate-x-4" : "translate-x-0.5"
          }`}
        />
      </button>
    </div>
  );
}
