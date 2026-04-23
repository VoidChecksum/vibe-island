import { useState, useEffect } from "react";
import { useStore } from "../../store/useStore";
import type { AppConfig } from "../../types";
import { invoke } from "@tauri-apps/api/core";

export function SettingsPanel() {
  const { config, updateConfig, platform, uninstallHooks } = useStore();
  const [localConfig, setLocalConfig] = useState<AppConfig | null>(null);
  const [hookStatus, setHookStatus] = useState("");
  const [uninstallStatus, setUninstallStatus] = useState("");

  useEffect(() => {
    if (config) setLocalConfig(JSON.parse(JSON.stringify(config)));
  }, [config]);

  if (!localConfig) return <div className="p-4 text-sm" style={{ color: "var(--notch-muted)" }}>Loading…</div>;

  const save = () => { if (localConfig) updateConfig(localConfig); };

  const setLayout = <K extends keyof AppConfig["layout"]>(key: K, value: AppConfig["layout"][K]) =>
    setLocalConfig((c) => c ? { ...c, layout: { ...c.layout, [key]: value } } : c);

  const Toggle = ({ label, value, onChange, description }: {
    label: string; value: boolean; onChange: (v: boolean) => void; description?: string;
  }) => (
    <div className="flex items-start justify-between gap-4 py-2.5" style={{ borderBottom: "1px solid var(--notch-border)" }}>
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium" style={{ color: "var(--notch-text)" }}>{label}</p>
        {description && <p className="text-xs mt-0.5" style={{ color: "var(--notch-muted)" }}>{description}</p>}
      </div>
      <button
        onClick={() => onChange(!value)}
        className="flex-shrink-0 w-10 h-6 rounded-full transition-colors relative"
        style={{ background: value ? "var(--vi-work)" : "var(--notch-surface)", border: "1px solid var(--notch-border)" }}
        data-no-drag
      >
        <div
          className="absolute top-0.5 w-5 h-5 rounded-full transition-transform"
          style={{ background: "#fff", transform: value ? "translateX(16px)" : "translateX(2px)" }}
        />
      </button>
    </div>
  );

  return (
    <div className="h-full overflow-y-auto" style={{ background: "#0d0d0d", color: "#fff" }}>
      <div className="px-6 py-5 space-y-6 max-w-lg mx-auto">
        <h1 className="text-xl font-semibold tracking-tight">Vibe Island</h1>

        {platform && (
          <section>
            <h2 className="text-xs font-semibold uppercase tracking-wider mb-2" style={{ color: "var(--notch-muted)" }}>Environment</h2>
            <div className="rounded-xl p-3 space-y-1.5 text-sm" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
              {([["Platform", `${platform.os} · ${platform.compositor}`], ["Desktop", platform.desktop], ["Wayland", platform.wayland ? "Yes" : "No"]] as [string, string][]).map(([k, v]) => (
                <div key={k} className="flex justify-between">
                  <span style={{ color: "var(--notch-muted)" }}>{k}</span>
                  <span>{v}</span>
                </div>
              ))}
            </div>
          </section>
        )}

        <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2" style={{ color: "var(--notch-muted)" }}>Behavior</h2>
          <div className="rounded-xl px-4" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <Toggle label="Expand on hover" description="Expand the notch panel when mouse hovers over pill"
              value={localConfig.layout.expand_on_hover} onChange={(v) => setLayout("expand_on_hover", v)} />
            <Toggle label="Hide when empty" description="Hide the pill when no sessions are running"
              value={localConfig.layout.hide_when_empty} onChange={(v) => setLayout("hide_when_empty", v)} />
            <Toggle label="Expand on subagent done" description="Surface the panel when a teammate or subagent finishes"
              value={localConfig.layout.expand_on_subagent_done} onChange={(v) => setLayout("expand_on_subagent_done", v)} />
            <Toggle label="Click outside to dismiss" description="Immediately dismiss the panel when clicking outside"
              value={localConfig.layout.click_outside_dismisses} onChange={(v) => setLayout("click_outside_dismisses", v)} />
            <div className="py-2.5" style={{ borderBottom: "1px solid var(--notch-border)" }}>
              <div className="flex justify-between items-center mb-1.5">
                <p className="text-sm font-medium">Dwell time</p>
                <span className="text-xs" style={{ color: "var(--notch-muted)" }}>{localConfig.layout.dwell_time_secs}s</span>
              </div>
              <p className="text-xs mb-2" style={{ color: "var(--notch-muted)" }}>How long the notch stays expanded after a task completes</p>
              <input type="range" min="0" max="30" step="1"
                value={localConfig.layout.dwell_time_secs}
                onChange={(e) => setLayout("dwell_time_secs", parseFloat(e.target.value))}
                className="w-full" style={{ accentColor: "var(--vi-work)" }} data-no-drag />
            </div>
          </div>
        </section>

        <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2" style={{ color: "var(--notch-muted)" }}>Display</h2>
          <div className="rounded-xl px-4" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <Toggle label="Notch follows active window" description="Move the panel to the display where your active window is"
              value={localConfig.layout.notch_follows_active_window} onChange={(v) => setLayout("notch_follows_active_window", v)} />
            <Toggle label="Auto-configure terminal titles" description="Required for precise tab jumping in Ghostty and Warp"
              value={localConfig.layout.auto_configure_terminal_titles} onChange={(v) => setLayout("auto_configure_terminal_titles", v)} />
            <Toggle label="Show tool names" value={localConfig.layout.show_tool_names} onChange={(v) => setLayout("show_tool_names", v)} />
            <Toggle label="Show session time" value={localConfig.layout.show_session_time} onChange={(v) => setLayout("show_session_time", v)} />
            <div className="py-2.5" style={{ borderBottom: "1px solid var(--notch-border)" }}>
              <div className="flex justify-between items-center mb-1">
                <p className="text-sm font-medium">Opacity</p>
                <span className="text-xs" style={{ color: "var(--notch-muted)" }}>{Math.round(localConfig.display.opacity * 100)}%</span>
              </div>
              <input type="range" min="0.5" max="1" step="0.05"
                value={localConfig.display.opacity}
                onChange={(e) => setLocalConfig((c) => c ? { ...c, display: { ...c.display, opacity: parseFloat(e.target.value) } } : c)}
                className="w-full" style={{ accentColor: "var(--vi-work)" }} data-no-drag />
            </div>
          </div>
        </section>

        <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2" style={{ color: "var(--notch-muted)" }}>Sound</h2>
          <div className="rounded-xl px-4" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <Toggle label="Enable sounds" value={localConfig.sound.enabled}
              onChange={(v) => setLocalConfig((c) => c ? { ...c, sound: { ...c.sound, enabled: v } } : c)} />
            <div className="py-2.5" style={{ borderBottom: "1px solid var(--notch-border)" }}>
              <div className="flex justify-between items-center mb-1">
                <p className="text-sm font-medium">Volume</p>
                <span className="text-xs" style={{ color: "var(--notch-muted)" }}>{Math.round(localConfig.sound.volume * 100)}%</span>
              </div>
              <input type="range" min="0" max="1" step="0.05"
                value={localConfig.sound.volume}
                onChange={(e) => setLocalConfig((c) => c ? { ...c, sound: { ...c.sound, volume: parseFloat(e.target.value) } } : c)}
                className="w-full" style={{ accentColor: "var(--vi-work)" }} data-no-drag />
            </div>
            <Toggle label="Permission requests" value={localConfig.sound.events.permission_request}
              onChange={(v) => setLocalConfig((c) => c ? { ...c, sound: { ...c.sound, events: { ...c.sound.events, permission_request: v } } } : c)} />
            <Toggle label="Session start" value={localConfig.sound.events.session_start}
              onChange={(v) => setLocalConfig((c) => c ? { ...c, sound: { ...c.sound, events: { ...c.sound.events, session_start: v } } } : c)} />
          </div>
        </section>

        <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2" style={{ color: "var(--notch-muted)" }}>Integrations</h2>
          <div className="rounded-xl p-4 space-y-3" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <button onClick={async () => {
              setHookStatus("Installing…");
              try { const r = await invoke<string>("install_hooks"); setHookStatus(r || "Done"); }
              catch (e) { setHookStatus(`Error: ${e}`); }
            }} className="w-full py-2 rounded-lg text-sm font-medium transition-colors"
              style={{ background: "var(--notch-hover)", border: "1px solid var(--notch-border)" }} data-no-drag>
              Install / Reinstall Hooks
            </button>
            {hookStatus && <pre className="text-[10px] whitespace-pre-wrap leading-relaxed" style={{ color: "var(--notch-muted)" }}>{hookStatus}</pre>}
          </div>
        </section>

        <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2" style={{ color: "var(--notch-muted)" }}>Advanced</h2>
          <div className="rounded-xl px-4" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <Toggle label="Auto-install hooks on startup" value={localConfig.auto_install_hooks}
              onChange={(v) => setLocalConfig((c) => c ? { ...c, auto_install_hooks: v } : c)} />
            <Toggle label="Launch at login" value={localConfig.launch_at_login}
              onChange={(v) => setLocalConfig((c) => c ? { ...c, launch_at_login: v } : c)} />
          </div>
          <div className="mt-3 rounded-xl p-4" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <button onClick={async () => {
              setUninstallStatus("Uninstalling…");
              const r = await uninstallHooks();
              setUninstallStatus(r || "Done");
            }} className="w-full py-2 rounded-lg text-sm font-medium transition-colors"
              style={{ background: "rgba(255,95,86,0.1)", border: "1px solid rgba(255,95,86,0.2)", color: "#ff5f56" }} data-no-drag>
              Uninstall All Hooks
            </button>
            {uninstallStatus && <pre className="text-[10px] whitespace-pre-wrap leading-relaxed mt-2" style={{ color: "var(--notch-muted)" }}>{uninstallStatus}</pre>}
          </div>
        </section>

        <button onClick={save} className="w-full py-2.5 rounded-xl text-sm font-medium"
          style={{ background: "var(--vi-work)", color: "#fff" }} data-no-drag>
          Save Settings
        </button>

        <p className="text-center text-[10px] pb-4" style={{ color: "var(--notch-muted)" }}>
          Vibe Island · open source · vibeisland.app
        </p>
      </div>
    </div>
  );
}
