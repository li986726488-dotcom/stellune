import { invoke } from "@tauri-apps/api/core";
import type { CommandClient } from "@/services/command-client";
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

export class TauriCommandClient implements CommandClient {
  getAppState(): Promise<AppState> {
    return invoke("get_app_state");
  }

  getProfile(): Promise<Profile | null> {
    return invoke("get_profile");
  }

  saveProfile(input: ProfileInput): Promise<Profile> {
    return invoke("save_profile", { input });
  }

  getDailyReading(): Promise<DailyReading> {
    return invoke("get_daily_reading");
  }

  saveMood(mood: Mood): Promise<MoodEntry> {
    return invoke("save_mood", { mood });
  }

  getTrail(days: number): Promise<TrailResponse> {
    return invoke("get_trail", { days });
  }

  getDailyFortune(): Promise<DailyFortune | null> {
    return invoke("get_daily_fortune");
  }

  drawDailyFortune(): Promise<DailyFortune> {
    return invoke("draw_daily_fortune");
  }
}
