import type {
  AppState,
  DailyFortune,
  DailyReading,
  Mood,
  MoodEntry,
  Profile,
  ProfileInput,
  TrailResponse,
} from "@/types";

export interface CommandClient {
  getAppState(): Promise<AppState>;
  getProfile(): Promise<Profile | null>;
  saveProfile(input: ProfileInput): Promise<Profile>;
  getDailyReading(): Promise<DailyReading>;
  saveMood(mood: Mood): Promise<MoodEntry>;
  getTrail(days: number): Promise<TrailResponse>;
  getDailyFortune(): Promise<DailyFortune | null>;
  drawDailyFortune(): Promise<DailyFortune>;
}
