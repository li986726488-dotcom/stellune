import { defineStore } from "pinia";
import { useCommandClient } from "@/services/commands";
import { toAppError } from "@/services/errors";
import type {
  AppErrorShape,
  DailyReading,
  Mood,
  MoodEntry,
} from "@/types";

export const useDailyReadingStore = defineStore("dailyReading", {
  state: () => ({
    reading: null as DailyReading | null,
    selectedMood: null as Mood | null,
    moodEntry: null as MoodEntry | null,
    loading: false,
    savingMood: false,
    requiresRecalibration: false,
    error: null as AppErrorShape | null,
    loadGeneration: 0,
  }),
  actions: {
    async load(force = false) {
      if (this.reading && !force && !this.requiresRecalibration) {
        return this.reading;
      }
      const generation = ++this.loadGeneration;
      this.loading = true;
      this.error = null;
      try {
        const reading = await useCommandClient().getDailyReading();
        if (generation !== this.loadGeneration) return this.reading;
        this.reading = reading;
        this.requiresRecalibration = reading.source.stale;
        return reading;
      } catch (error) {
        if (generation !== this.loadGeneration) return this.reading;
        this.error = toAppError(error);
        return null;
      } finally {
        if (generation === this.loadGeneration) {
          this.loading = false;
        }
      }
    },
    chooseMood(mood: Mood) {
      this.selectedMood = mood;
    },
    async saveMood() {
      if (!this.selectedMood) return;
      this.savingMood = true;
      try {
        this.moodEntry = await useCommandClient().saveMood(this.selectedMood);
      } catch (error) {
        this.error = toAppError(error);
      } finally {
        this.savingMood = false;
      }
    },
    invalidate() {
      this.loadGeneration += 1;
      this.reading = null;
      this.requiresRecalibration = false;
      this.error = null;
    },
    markNeedsRecalibration() {
      this.requiresRecalibration = true;
      this.error = null;
    },
  },
});
