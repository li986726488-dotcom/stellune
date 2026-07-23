import { defineStore } from "pinia";
import { useCommandClient } from "@/services/commands";
import { toAppError } from "@/services/errors";
import type {
  AppErrorShape,
  TrailEntry,
  TrailResponse,
} from "@/types";

export const useTrailStore = defineStore("trail", {
  state: () => ({
    trail: null as TrailResponse | null,
    selectedDate: null as string | null,
    loading: false,
    error: null as AppErrorShape | null,
  }),
  getters: {
    selectedEntry(state): TrailEntry | null {
      if (!state.trail?.entries.length) return null;
      return (
        state.trail.entries.find(
          (entry) => entry.date === state.selectedDate,
        ) ?? state.trail.entries[state.trail.entries.length - 1]
      );
    },
  },
  actions: {
    async load() {
      this.loading = true;
      this.error = null;
      try {
        this.trail = await useCommandClient().getTrail(7);
        this.selectedDate =
          this.trail.entries[this.trail.entries.length - 1]?.date ?? null;
      } catch (error) {
        this.error = toAppError(error);
      } finally {
        this.loading = false;
      }
    },
    select(date: string) {
      this.selectedDate = date;
    },
  },
});
