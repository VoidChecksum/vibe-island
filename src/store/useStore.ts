import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Session, AppConfig, PlatformInfo } from "../types";

interface AppStore {
  sessions: Session[];
  expanded: boolean;
  selectedSessionId: string | null;
  config: AppConfig | null;
  platform: PlatformInfo | null;
  mutedSessions: Set<string>;
  bypassSessions: Set<string>;

  setExpanded: (expanded: boolean) => void;
  toggleExpanded: () => void;
  selectSession: (id: string | null) => void;
  refreshSessions: () => Promise<void>;
  approvePermission: (sessionId: string, decision: string, reason?: string) => Promise<void>;
  answerQuestion: (sessionId: string, answers: Record<string, unknown>) => Promise<void>;
  jumpToTerminal: (sessionId: string) => Promise<void>;
  uninstallHooks: () => Promise<string>;
  loadConfig: () => Promise<void>;
  updateConfig: (config: AppConfig) => Promise<void>;
  loadPlatform: () => Promise<void>;
  toggleBypass: (sessionId: string) => Promise<void>;
  init: () => Promise<void>;
}

export const useStore = create<AppStore>((set, get) => ({
  sessions: [],
  expanded: false,
  selectedSessionId: null,
  config: null,
  platform: null,
  mutedSessions: new Set(),
  bypassSessions: new Set(),

  setExpanded: (expanded) => set({ expanded }),
  toggleExpanded: () => set((s) => ({ expanded: !s.expanded })),
  selectSession: (id) => set({ selectedSessionId: id }),

  refreshSessions: async () => {
    try {
      const sessions = await invoke<Session[]>("get_sessions");
      set({ sessions });
    } catch (e) {
      console.error("Failed to refresh sessions:", e);
    }
  },

  approvePermission: async (sessionId, decision, reason) => {
    try {
      await invoke("approve_permission", { sessionId, decision, reason: reason || null });
      await get().refreshSessions();
    } catch (e) {
      console.error("Failed to approve:", e);
    }
  },

  answerQuestion: async (sessionId, answers) => {
    try {
      await invoke("answer_question", { sessionId, answers });
      await get().refreshSessions();
    } catch (e) {
      console.error("Failed to answer:", e);
    }
  },

  jumpToTerminal: async (sessionId) => {
    try {
      await invoke("jump_to_terminal", { sessionId });
    } catch (e) {
      console.error("Failed to jump:", e);
    }
  },

  uninstallHooks: async () => {
    try {
      return await invoke<string>("uninstall_hooks");
    } catch (e) {
      return `Error: ${e}`;
    }
  },

  loadConfig: async () => {
    try {
      const config = await invoke<AppConfig>("get_config");
      set({ config });
    } catch (e) {
      console.error("Failed to load config:", e);
    }
  },

  updateConfig: async (config) => {
    try {
      await invoke("update_config", { config });
      set({ config });
    } catch (e) {
      console.error("Failed to update config:", e);
    }
  },

  loadPlatform: async () => {
    try {
      const platform = await invoke<PlatformInfo>("get_platform_info");
      set({ platform });
    } catch (e) {
      console.error("Failed to load platform:", e);
    }
  },

  toggleBypass: async (sessionId) => {
    const { bypassSessions } = get();
    const next = new Set(bypassSessions);
    const enabled = !next.has(sessionId);
    if (enabled) next.add(sessionId); else next.delete(sessionId);
    set({ bypassSessions: next });
    try {
      await invoke("set_bypass_mode", { sessionId, enabled });
    } catch (e) {
      console.error("Failed to set bypass mode:", e);
    }
  },

  init: async () => {
    await get().loadPlatform();
    await get().loadConfig();
    await get().refreshSessions();

    listen("session-update", () => get().refreshSessions());
    listen("permission-asked", () => { get().refreshSessions(); set({ expanded: true }); });
    listen("question-asked", () => { get().refreshSessions(); set({ expanded: true }); });

    setInterval(() => get().refreshSessions(), 2000);
  },
}));
