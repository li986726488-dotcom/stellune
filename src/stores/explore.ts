import { defineStore } from "pinia";
import { useCommandClient } from "@/services/commands";
import { toAppError } from "@/services/errors";
import { enqueueMoodMutation } from "@/services/mood-mutations";
import type {
  AppErrorShape,
  CompatibilityReading,
  DailyFortune,
  EmotionGuide,
  Mood,
  ZodiacSlug,
} from "@/types";

export const useExploreStore = defineStore("explore", {
  state: () => ({
    fortune: null as DailyFortune | null,
    partnerSign: "gemini" as ZodiacSlug,
    compatibility: null as CompatibilityReading | null,
    selectedMood: null as Mood | null,
    emotionGuide: null as EmotionGuide | null,
    loading: false,
    initialized: false,
    drawing: false,
    compatibilityLoading: false,
    emotionLoading: false,
    compatibilityRequestId: 0,
    emotionRequestId: 0,
    error: null as AppErrorShape | null,
    compatibilityError: null as AppErrorShape | null,
    emotionError: null as AppErrorShape | null,
  }),
  actions: {
    async load() {
      this.loading = true;
      this.error = null;
      this.compatibilityError = null;
      this.emotionError = null;
      const client = useCommandClient();
      const fortune = client
        .getDailyFortune()
        .then((value) => {
          this.fortune = value;
        })
        .catch((error) => {
          this.error = toAppError(error);
        });
      await Promise.all([
        fortune,
        this.loadCompatibility(this.partnerSign),
        this.loadEmotionGuide(),
      ]);
      this.loading = false;
      this.initialized = true;
    },
    async draw() {
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
    async loadCompatibility(partnerSign: ZodiacSlug) {
      const requestId = ++this.compatibilityRequestId;
      const committedSign =
        this.compatibility?.partnerSign.slug ?? this.partnerSign;
      this.partnerSign = partnerSign;
      this.compatibilityLoading = true;
      this.compatibilityError = null;
      try {
        const result =
          await useCommandClient().getCompatibility(partnerSign);
        if (requestId === this.compatibilityRequestId) {
          this.partnerSign = result.partnerSign.slug;
          this.compatibility = result;
        }
      } catch (error) {
        if (requestId === this.compatibilityRequestId) {
          this.partnerSign = committedSign;
          this.compatibilityError = toAppError(error);
        }
      } finally {
        if (requestId === this.compatibilityRequestId) {
          this.compatibilityLoading = false;
        }
      }
    },
    async loadEmotionGuide(mood: Mood | null = null) {
      const requestId = ++this.emotionRequestId;
      this.emotionLoading = true;
      this.emotionError = null;
      try {
        const result = await useCommandClient().getEmotionGuide(mood);
        if (requestId === this.emotionRequestId) {
          this.selectedMood = result?.mood ?? null;
          this.emotionGuide = result;
        }
      } catch (error) {
        if (requestId === this.emotionRequestId) {
          this.emotionError = toAppError(error);
        }
      } finally {
        if (requestId === this.emotionRequestId) {
          this.emotionLoading = false;
        }
      }
    },
    async selectMood(mood: Mood) {
      const requestId = ++this.emotionRequestId;
      const committedMood = this.emotionGuide?.mood ?? null;
      this.selectedMood = mood;
      this.emotionLoading = true;
      this.emotionError = null;
      try {
        const client = useCommandClient();
        const result = await enqueueMoodMutation(async () => {
          if (requestId !== this.emotionRequestId) return null;
          const guide = await client.getEmotionGuide(mood);
          if (!guide || requestId !== this.emotionRequestId) return null;
          await client.saveMood(mood);
          return guide;
        });
        if (requestId === this.emotionRequestId) {
          if (!result) {
            throw new Error("保存心情后未能生成情绪说明书。");
          }
          this.selectedMood = result.mood;
          this.emotionGuide = result;
        }
      } catch (error) {
        if (requestId === this.emotionRequestId) {
          this.selectedMood = committedMood;
          this.emotionError = toAppError(error);
        }
      } finally {
        if (requestId === this.emotionRequestId) {
          this.emotionLoading = false;
        }
      }
    },
  },
});
