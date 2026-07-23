import { defineStore } from "pinia";
import { useCommandClient } from "@/services/commands";
import { toAppError } from "@/services/errors";
import type { AppErrorShape, AppState } from "@/types";

export const useAppStore = defineStore("app", {
  state: () => ({
    ready: false,
    loading: false,
    state: null as AppState | null,
    error: null as AppErrorShape | null,
  }),
  getters: {
    onboardingComplete: (state) =>
      Boolean(state.state?.onboardingComplete),
  },
  actions: {
    async bootstrap() {
      this.loading = true;
      this.error = null;
      try {
        this.state = await useCommandClient().getAppState();
      } catch (error) {
        this.error = toAppError(error);
      } finally {
        this.loading = false;
        this.ready = true;
      }
    },
    setProfileReady(profile: AppState["profile"]) {
      this.state = {
        onboardingComplete: Boolean(profile),
        profile,
        todayReady: false,
      };
    },
  },
});
