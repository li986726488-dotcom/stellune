import type {
  AppState,
  CompatibilityReading,
  DailyFortune,
  DailyReading,
  EmotionGuide,
  Mood,
  MoodEntry,
  Profile,
  ProfileInput,
  TrailResponse,
  ZodiacSlug,
} from "@/types";

export interface CommandClient {
  getAppState(): Promise<AppState>;
  getProfile(): Promise<Profile | null>;
  saveProfile(input: ProfileInput): Promise<Profile>;
  getDailyReading(): Promise<DailyReading>;
  saveMood(mood: Mood): Promise<MoodEntry>;
  getTodayMood(): Promise<MoodEntry | null>;
  getTrail(days: number): Promise<TrailResponse>;
  getDailyFortune(): Promise<DailyFortune | null>;
  drawDailyFortune(): Promise<DailyFortune>;
  getCompatibility(partnerSign: ZodiacSlug): Promise<CompatibilityReading>;
  getEmotionGuide(mood?: Mood | null): Promise<EmotionGuide | null>;
}
