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

  // Actions
  setExpanded: (expanded: boolean) => void;
  toggleExpanded: () => void;
  selectSession: (id: string | null) => void;
  refreshSessions: () => Promise<void>;
  approvePermission: (sessionId: string, decision: string, reason?: string) => Promise<void>;
  answerQuestion: (sessionId: string, answers: Record<string, unknown>) => Promise<void>;
  loadConfig: () => Promise<void>;
  updateConfig: (config: AppConfig) => Promise<void>;
  loadPlatform: () => Promise<void>;
  init: () => Promise<void>;
}

export const useStore = create<AppStore>((set, get) => ({
  sessions: [],
  expanded: false,
  selectedSessionId: null,
  config: null,
  platform: null,

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
      await invoke("approve_permission", {
        sessionId,
        decision,
        reason: reason || null,
      });
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

  init: async () => {
    await get().loadPlatform();
    await get().loadConfig();
    await get().refreshSessions();

    // Listen for session updates from backend
    listen("session-update", () => {
      get().refreshSessions();
    });

    listen("permission-asked", () => {
      get().refreshSessions();
      set({ expanded: true });
    });

    listen("question-asked", () => {
      get().refreshSessions();
      set({ expanded: true });
    });

    // Poll sessions every 2 seconds as fallback
    setInterval(() => get().refreshSessions(), 2000);
  },
}));
