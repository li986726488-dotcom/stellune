import { defineStore } from "pinia";
import { useCommandClient } from "@/services/commands";
import { toAppError } from "@/services/errors";
import type { AppErrorShape, DailyFortune } from "@/types";

export const useExploreStore = defineStore("explore", {
  state: () => ({
    fortune: null as DailyFortune | null,
    loading: false,
    initialized: false,
    drawing: false,
    error: null as AppErrorShape | null,
  }),
  actions: {
    async load() {
      this.loading = true;
      this.error = null;
      try {
        this.fortune = await useCommandClient().getDailyFortune();
      } catch (error) {
        this.error = toAppError(error);
      } finally {
        this.loading = false;
        this.initialized = true;
      }
    },
    async draw() {
      if (this.fortune) return this.fortune;
      this.drawing = true;
      this.error = null;
      try {
        this.fortune = await useCommandClient().drawDailyFortune();
        return this.fortune;
      } catch (error) {
        this.error = toAppError(error);
        return null;
      } finally {
        this.drawing = false;
      }
    },
  },
});
