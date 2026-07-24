import type { CommandClient } from "../../../src/services/command-client";
import { zodiacFromBirthday } from "../../../src/services/zodiac";
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
  ZodiacSign,
  ZodiacSlug,
} from "../../../src/types";
import dailyFortuneFixture from "../fixtures/daily-fortune.json";
import dailyReadingFixture from "../fixtures/daily-reading.json";
import exploreInsightsFixture from "../fixtures/explore-insights.json";
import profileFixture from "../fixtures/profile.json";
import trailFixture from "../fixtures/trail.json";

const redrawnFortuneFixture: DailyFortune = {
  id: "fortune-2026-07-23-048",
  date: "2026-07-23",
  number: 48,
  catalog: "星轨百签",
  grade: "中上签",
  title: "春水绕石",
  verse: "春水绕石不争路，曲处仍能向海流。",
  interpretation: "柔软的处理方式会比正面碰撞更有效，关系和事务都有转圜。",
  advice: "把一个对抗式表达改成共同解决问题的邀请。",
};

function clone<T>(value: T): T {
  return structuredClone(value);
}

function fixtureProfile(): Profile {
  return clone(profileFixture) as Profile;
}

const zodiacSigns: Record<ZodiacSlug, ZodiacSign> = {
  aries: { slug: "aries", name: "白羊座", symbol: "♈" },
  taurus: { slug: "taurus", name: "金牛座", symbol: "♉" },
  gemini: { slug: "gemini", name: "双子座", symbol: "♊" },
  cancer: { slug: "cancer", name: "巨蟹座", symbol: "♋" },
  leo: { slug: "leo", name: "狮子座", symbol: "♌" },
  virgo: { slug: "virgo", name: "处女座", symbol: "♍" },
  libra: { slug: "libra", name: "天秤座", symbol: "♎" },
  scorpio: { slug: "scorpio", name: "天蝎座", symbol: "♏" },
  sagittarius: { slug: "sagittarius", name: "射手座", symbol: "♐" },
  capricorn: { slug: "capricorn", name: "摩羯座", symbol: "♑" },
  aquarius: { slug: "aquarius", name: "水瓶座", symbol: "♒" },
  pisces: { slug: "pisces", name: "双鱼座", symbol: "♓" },
};

export class MockCommandClient implements CommandClient {
  private profile: Profile | null;
  private fortune: DailyFortune | null = null;
  private moodEntry: MoodEntry | null = null;
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
    if (this.scenario === "rapid-mood" && mood === "疲惫") {
      await new Promise((resolve) => window.setTimeout(resolve, 120));
    }
    const entry = {
      id: `mood-2026-07-23`,
      date: "2026-07-23",
      mood,
    };
    const today = this.trail.entries.find(
      (item) => item.date === "2026-07-23",
    );
    if (today) today.mood = mood;
    this.moodEntry = entry;
    return clone(entry);
  }

  async getTodayMood(): Promise<MoodEntry | null> {
    return clone(this.moodEntry);
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
    this.fortune = this.fortune
      ? clone(redrawnFortuneFixture)
      : (clone(dailyFortuneFixture) as DailyFortune);
    return clone(this.fortune);
  }

  async getCompatibility(
    partnerSign: ZodiacSlug,
  ): Promise<CompatibilityReading> {
    await this.waitForSlowTabRefresh();
    if (this.scenario === "explore-error" && partnerSign !== "gemini") {
      throw {
        code: "INTERNAL_ERROR",
        message: "关系共鸣暂时没有回应。",
        retryable: true,
      };
    }
    const fixture =
      exploreInsightsFixture.compatibilityBySign[partnerSign] ??
      exploreInsightsFixture.compatibilityBySign.gemini;
    return {
      mode: "sunSignCompatibility",
      primarySign: clone(this.profile?.zodiac ?? zodiacSigns.libra),
      partnerSign: clone(zodiacSigns[partnerSign]),
      score: fixture.score,
      level: fixture.level,
      title: fixture.title,
      summary: fixture.summary,
      factors: [],
      certainty: "sun-sign-entertainment",
      rulesVersion: "2026.07.2",
    };
  }

  async getEmotionGuide(mood?: Mood | null): Promise<EmotionGuide | null> {
    await this.waitForSlowTabRefresh();
    if (!mood && !this.moodEntry) return null;
    if (this.scenario === "rapid-mood-failure" && mood === "疲惫") {
      await new Promise((resolve) => window.setTimeout(resolve, 500));
    }
    if (this.scenario === "rapid-mood-failure" && mood === "低落") {
      throw {
        code: "INTERNAL_ERROR",
        message: "最新心情暂时保存失败。",
        retryable: true,
      };
    }
    if (this.scenario === "explore-error" && mood) {
      throw {
        code: "INTERNAL_ERROR",
        message: "情绪说明书暂时没有回应。",
        retryable: true,
      };
    }
    const selectedMood = mood ?? this.moodEntry?.mood;
    if (!selectedMood) return null;
    const fixture = exploreInsightsFixture.emotionByMood[selectedMood];
    return {
      mood: selectedMood,
      title: fixture.title,
      summary: fixture.summary,
      action: fixture.action,
      need: fixture.need,
      basedOn: [
        `mood:${selectedMood}`,
        "theme:校准",
        "social:high",
        "inner:medium",
      ],
      generator: "template",
      certainty: "emotional-companion",
      rulesVersion: "2026.07.1",
    };
  }
}
