import { useState, useEffect } from "react";
import { useStore } from "../../store/useStore";
import type { AppConfig } from "../../types";
import { TOOL_LABELS } from "../../types";
import { invoke } from "@tauri-apps/api/core";
import extensionIconUrl from "../../assets/brand/extension-icon.png";
import wechatQrUrl from "../../assets/brand/wechat-group-qr.jpg";
import feishuQrUrl from "../../assets/brand/feishu-group-qr.png";

export function SettingsPanel() {
  const { config, updateConfig, platform, uninstallHooks } = useStore();
  const [localConfig, setLocalConfig] = useState<AppConfig | null>(null);
  const [hookStatus, setHookStatus] = useState("");
  const [uninstallStatus, setUninstallStatus] = useState("");
  const [section, setSection] = useState("behavior");

  useEffect(() => {
    if (config) setLocalConfig(JSON.parse(JSON.stringify(config)));
  }, [config]);

  if (!localConfig) return <div className="p-4 text-sm" style={{ color: "var(--notch-muted)" }}>Loading…</div>;

  const save = () => { if (localConfig) updateConfig(localConfig); };

  const setLayout = <K extends keyof AppConfig["layout"]>(key: K, value: AppConfig["layout"][K]) =>
    setLocalConfig((c) => c ? { ...c, layout: { ...c.layout, [key]: value } } : c);
  const setUsage = <K extends keyof AppConfig["usage"]>(key: K, value: AppConfig["usage"][K]) =>
    setLocalConfig((c) => c ? { ...c, usage: { ...c.usage, [key]: value } } : c);
  const setLabs = <K extends keyof AppConfig["labs"]>(key: K, value: AppConfig["labs"][K]) =>
    setLocalConfig((c) => c ? { ...c, labs: { ...c.labs, [key]: value } } : c);
  const setTerminal = <K extends keyof AppConfig["terminal"]>(key: K, value: AppConfig["terminal"][K]) =>
    setLocalConfig((c) => c ? { ...c, terminal: { ...c.terminal, [key]: value } } : c);
  const toggleTool = (tool: string) =>
    setLocalConfig((c) => {
      if (!c) return c;
      const monitored = new Set(c.monitored_tools);
      if (monitored.has(tool)) monitored.delete(tool); else monitored.add(tool);
      return { ...c, monitored_tools: [...monitored] };
    });

  const Select = ({ label, value, options, onChange }: {
    label: string; value: string; options: Array<[string, string]>; onChange: (v: string) => void;
  }) => (
    <div className="flex items-center justify-between gap-4 py-2.5" style={{ borderBottom: "1px solid var(--notch-border)" }}>
      <span className="text-sm font-medium" style={{ color: "var(--notch-text)" }}>{label}</span>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="rounded-lg px-2 py-1 text-xs outline-none"
        style={{ background: "#050505", border: "1px solid var(--notch-border)", color: "var(--notch-text)" }}
        data-no-drag
      >
        {options.map(([v, l]) => <option key={v} value={v}>{l}</option>)}
      </select>
    </div>
  );

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
    <div className="h-full flex overflow-hidden" style={{ background: "#0d0d0d", color: "#fff" }}>
      <aside className="w-44 flex-shrink-0 p-4 space-y-2" style={{ background: "#080808", borderRight: "1px solid var(--notch-border)" }}>
        <div className="flex items-center gap-2 mb-4">
          <img src={extensionIconUrl} className="w-7 h-7 rounded-lg" alt="" />
          <div>
            <h1 className="text-sm font-semibold tracking-tight">Vibe Island</h1>
            <p className="text-[10px]" style={{ color: "var(--notch-muted)" }}>open source</p>
          </div>
        </div>
        {[
          ["behavior", "Behaviour"],
          ["display", "Display"],
          ["usage", "Usage"],
          ["sound", "Sound"],
          ["tools", "CLI Hooks"],
          ["terminal", "Terminal"],
          ["labs", "Labs"],
          ["community", "Community"],
          ["advanced", "Advanced"],
        ].map(([id, label]) => (
          <button
            key={id}
            onClick={() => setSection(id)}
            className="w-full text-left rounded-lg px-3 py-2 text-xs"
            style={{
              background: section === id ? "rgba(255,255,255,0.10)" : "transparent",
              color: section === id ? "#fff" : "rgba(255,255,255,0.55)",
            }}
            data-no-drag
          >
            {label}
          </button>
        ))}
      </aside>

      <div className="flex-1 overflow-y-auto">
      <div className="px-6 py-5 space-y-6 max-w-xl mx-auto">

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

        {section === "behavior" && <section>
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
            <Toggle label="Auto-collapse on leave" description="Collapse when pointer leaves the island"
              value={localConfig.layout.auto_collapse_on_leave} onChange={(v) => setLayout("auto_collapse_on_leave", v)} />
            <Toggle label="Disable click to jump" description="Keep session rows informational only"
              value={localConfig.layout.disable_click_to_jump} onChange={(v) => setLayout("disable_click_to_jump", v)} />
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
            <div className="py-2.5">
              <div className="flex justify-between items-center mb-1.5">
                <p className="text-sm font-medium">Session dismissal timeout</p>
                <span className="text-xs" style={{ color: "var(--notch-muted)" }}>
                  {localConfig.layout.session_idle_cleanup_secs === 0 ? "Off" : `${Math.round(localConfig.layout.session_idle_cleanup_secs / 60)}m`}
                </span>
              </div>
              <p className="text-xs mb-2" style={{ color: "var(--notch-muted)" }}>Auto-remove idle sessions after this period (0 = never)</p>
              <input type="range" min="0" max="1800" step="60"
                value={localConfig.layout.session_idle_cleanup_secs}
                onChange={(e) => setLayout("session_idle_cleanup_secs", parseInt(e.target.value))}
                className="w-full" style={{ accentColor: "var(--vi-work)" }} data-no-drag />
            </div>
          </div>
        </section>}

        {section === "display" && <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2" style={{ color: "var(--notch-muted)" }}>Display</h2>
          <div className="rounded-xl px-4" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <Toggle label="Notch follows active window" description="Move the panel to the display where your active window is"
              value={localConfig.layout.notch_follows_active_window} onChange={(v) => setLayout("notch_follows_active_window", v)} />
            <Toggle label="Auto-configure terminal titles" description="Required for precise tab jumping in Ghostty and Warp"
              value={localConfig.layout.auto_configure_terminal_titles} onChange={(v) => setLayout("auto_configure_terminal_titles", v)} />
            <Toggle label="Show tool names" value={localConfig.layout.show_tool_names} onChange={(v) => setLayout("show_tool_names", v)} />
            <Toggle label="Show session time" value={localConfig.layout.show_session_time} onChange={(v) => setLayout("show_session_time", v)} />
            <Select label="Display" value={localConfig.display.monitor} options={[["auto", "Follow focus"], ["primary", "Main display"], ["builtin", "Built-in display"]]}
              onChange={(v) => setLocalConfig((c) => c ? { ...c, display: { ...c.display, monitor: v } } : c)} />
            <Select label="Layout" value={localConfig.layout.style} options={[["clean", "Clean"], ["detailed", "Detailed"], ["compact", "Compact"]]}
              onChange={(v) => setLayout("style", v)} />
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
        </section>}

        {section === "sound" && <section>
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
            <Toggle label="Input required" value={localConfig.sound.events.input_required}
              onChange={(v) => setLocalConfig((c) => c ? { ...c, sound: { ...c.sound, events: { ...c.sound.events, input_required: v } } } : c)} />
            <Toggle label="Quiet hours" value={localConfig.sound.quiet_hours.enabled}
              onChange={(v) => setLocalConfig((c) => c ? { ...c, sound: { ...c.sound, quiet_hours: { ...c.sound.quiet_hours, enabled: v } } } : c)} />
          </div>
        </section>}

        {section === "usage" && <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2" style={{ color: "var(--notch-muted)" }}>Usage</h2>
          <div className="rounded-xl px-4" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <Toggle label="Show usage limits" description="Surface provider quota and remaining context where hooks can detect it"
              value={localConfig.usage.show_usage_limits} onChange={(v) => setUsage("show_usage_limits", v)} />
            <Select label="Provider" value={localConfig.usage.provider} options={[["auto", "Auto-detect"], ["claude", "Claude"], ["codex", "Codex"], ["gemini", "Gemini"]]}
              onChange={(v) => setUsage("provider", v)} />
            <Select label="Value mode" value={localConfig.usage.value_mode} options={[["remaining", "Remaining"], ["used", "Used"]]}
              onChange={(v) => setUsage("value_mode", v)} />
          </div>
        </section>}

        {section === "tools" && <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2" style={{ color: "var(--notch-muted)" }}>Integrations</h2>
          <div className="rounded-xl p-4 space-y-3" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <div className="flex flex-wrap gap-2">
              {Object.entries(TOOL_LABELS).map(([tool, label]) => {
                const active = localConfig.monitored_tools.includes(tool);
                return (
                  <button
                    key={tool}
                    onClick={() => toggleTool(tool)}
                    className="rounded-full px-3 py-1 text-[11px]"
                    style={{
                      background: active ? "rgba(34,197,94,0.16)" : "rgba(255,255,255,0.06)",
                      border: active ? "1px solid rgba(34,197,94,0.35)" : "1px solid rgba(255,255,255,0.08)",
                      color: active ? "#bbf7d0" : "rgba(255,255,255,0.55)",
                    }}
                    data-no-drag
                  >
                    {label}
                  </button>
                );
              })}
            </div>
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
        </section>}

        {section === "terminal" && <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2" style={{ color: "var(--notch-muted)" }}>Terminal</h2>
          <div className="rounded-xl px-4" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <Toggle label="Warp tab jump" description="Use Warp-specific tab focus when metadata is available"
              value={localConfig.terminal.warp_tab_jump} onChange={(v) => setTerminal("warp_tab_jump", v)} />
            <Toggle label="Disable Claude Code Native Terminal Title" description="Let Vibe Island own terminal title tracking"
              value={localConfig.terminal.disable_claude_native_title} onChange={(v) => setTerminal("disable_claude_native_title", v)} />
            <Toggle label="Disable click to jump" value={localConfig.terminal.disable_click_to_jump}
              onChange={(v) => setTerminal("disable_click_to_jump", v)} />
            <div className="py-2.5">
              <p className="text-sm font-medium">Custom jump rules</p>
              <p className="text-xs mt-0.5" style={{ color: "var(--notch-muted)" }}>
                {localConfig.terminal.custom_jump_rules.length || "No"} custom rules configured.
              </p>
            </div>
          </div>
        </section>}

        {section === "labs" && <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2" style={{ color: "var(--notch-muted)" }}>Labs</h2>
          <div className="rounded-xl px-4" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <Toggle label="Beta updates" value={localConfig.labs.beta_updates} onChange={(v) => setLabs("beta_updates", v)} />
            <Toggle label="Auto Mode" description="Expose bypass/auto-approve controls in session menus"
              value={localConfig.labs.auto_mode} onChange={(v) => setLabs("auto_mode", v)} />
            <Toggle label="Cursor approval" value={localConfig.labs.cursor_approval} onChange={(v) => setLabs("cursor_approval", v)} />
            <Toggle label="Codex Desktop approval alert" value={localConfig.labs.codex_desktop_alerts} onChange={(v) => setLabs("codex_desktop_alerts", v)} />
            <Toggle label="Kiro hints" value={localConfig.labs.kiro_hints} onChange={(v) => setLabs("kiro_hints", v)} />
          </div>
        </section>}

        {section === "community" && <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-2" style={{ color: "var(--notch-muted)" }}>Community</h2>
          <div className="rounded-xl p-4 grid grid-cols-2 gap-4" style={{ background: "var(--notch-surface)", border: "1px solid var(--notch-border)" }}>
            <div>
              <img src={wechatQrUrl} className="rounded-lg w-full" alt="WeChat group QR" />
              <p className="text-center text-[10px] mt-2" style={{ color: "var(--notch-muted)" }}>WeChat group</p>
            </div>
            <div>
              <img src={feishuQrUrl} className="rounded-lg w-full bg-white p-2" alt="Feishu group QR" />
              <p className="text-center text-[10px] mt-2" style={{ color: "var(--notch-muted)" }}>Feishu group</p>
            </div>
          </div>
        </section>}

        {section === "advanced" && <section>
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
        </section>}

        <button onClick={save} className="w-full py-2.5 rounded-xl text-sm font-medium"
          style={{ background: "var(--vi-work)", color: "#fff" }} data-no-drag>
          Save Settings
        </button>

        <p className="text-center text-[10px] pb-4" style={{ color: "var(--notch-muted)" }}>
          Vibe Island · open source · vibeisland.app
        </p>
      </div>
      </div>
    </div>
  );
}
