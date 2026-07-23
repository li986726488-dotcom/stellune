import type { CommandClient } from "../../../src/services/command-client";
import { zodiacFromBirthday } from "../../../src/services/zodiac";
import type {
  AppState,
  DailyFortune,
  DailyReading,
  Mood,
  MoodEntry,
  Profile,
  ProfileInput,
  TrailResponse,
} from "../../../src/types";
import dailyFortuneFixture from "../fixtures/daily-fortune.json";
import dailyReadingFixture from "../fixtures/daily-reading.json";
import profileFixture from "../fixtures/profile.json";
import trailFixture from "../fixtures/trail.json";

function clone<T>(value: T): T {
  return structuredClone(value);
}

function fixtureProfile(): Profile {
  return clone(profileFixture) as Profile;
}

export class MockCommandClient implements CommandClient {
  private profile: Profile | null;
  private fortune: DailyFortune | null = null;
  private trail = clone(trailFixture) as TrailResponse;
  private readonly scenario = new URLSearchParams(window.location.search).get(
    "scenario",
  );

  constructor() {
    this.profile = this.scenario === "onboarding" ? null : fixtureProfile();
  }

  private async waitForSlowTabRefresh() {
    if (this.scenario === "slow-tabs") {
      await new Promise((resolve) => window.setTimeout(resolve, 220));
    }
  }

  async getAppState(): Promise<AppState> {
    return {
      onboardingComplete: Boolean(this.profile),
      profile: clone(this.profile),
      todayReady: Boolean(this.profile),
    };
  }

  async getProfile(): Promise<Profile | null> {
    return clone(this.profile);
  }

  async saveProfile(input: ProfileInput): Promise<Profile> {
    const now = "2026-07-23T13:00:00+08:00";
    const calculationChanged =
      !this.profile ||
      this.profile.birthday !== input.birthday ||
      this.profile.birthTime !== (input.birthTime || null) ||
      this.profile.birthCity !== (input.birthCity?.trim() || null);
    const completeness =
      40 +
      (input.nickname?.trim() ? 20 : 0) +
      (input.birthTime ? 20 : 0) +
      (input.birthCity?.trim() ? 20 : 0);
    this.profile = {
      id: this.profile?.id ?? "profile-local",
      guestId: this.profile?.guestId ?? "guest-stellune-e2e",
      nickname: input.nickname?.trim() || "星际旅人",
      birthday: input.birthday,
      birthTime: input.birthTime || null,
      birthCity: input.birthCity?.trim() || null,
      zodiac: zodiacFromBirthday(input.birthday),
      personalizationMode:
        input.birthTime && input.birthCity ? "natal" : "sunSignLite",
      completeness,
      calculationVersion:
        (this.profile?.calculationVersion ?? 1) +
        (this.profile && calculationChanged ? 1 : 0),
      createdAt: this.profile?.createdAt ?? now,
      updatedAt: now,
    };
    return clone(this.profile);
  }

  async getDailyReading(): Promise<DailyReading> {
    if (this.scenario === "astrology-error") {
      throw {
        code: "ASTROLOGY_NETWORK_ERROR",
        message: "真实星相暂时没有抵达，请稍后再试。",
        retryable: true,
      };
    }
    const reading = clone(dailyReadingFixture) as DailyReading;
    if (this.profile) {
      reading.zodiac = clone(this.profile.zodiac);
      reading.personalizationMode = this.profile.personalizationMode;
      if (this.profile.calculationVersion > 1) {
        reading.hero.title = "新资料校准后的星图已抵达";
      }
    }
    return reading;
  }

  async saveMood(mood: Mood): Promise<MoodEntry> {
    const entry = {
      id: `mood-2026-07-23`,
      date: "2026-07-23",
      mood,
    };
    const today = this.trail.entries.find(
      (item) => item.date === "2026-07-23",
    );
    if (today) today.mood = mood;
    return entry;
  }

  async getTrail(): Promise<TrailResponse> {
    await this.waitForSlowTabRefresh();
    return clone(this.trail);
  }

  async getDailyFortune(): Promise<DailyFortune | null> {
    await this.waitForSlowTabRefresh();
    return clone(this.fortune);
  }

  async drawDailyFortune(): Promise<DailyFortune> {
    this.fortune ??= clone(dailyFortuneFixture) as DailyFortune;
    return clone(this.fortune);
  }
}
