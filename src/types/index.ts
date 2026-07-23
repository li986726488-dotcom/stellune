export type ZodiacSlug =
  | "aries"
  | "taurus"
  | "gemini"
  | "cancer"
  | "leo"
  | "virgo"
  | "libra"
  | "scorpio"
  | "sagittarius"
  | "capricorn"
  | "aquarius"
  | "pisces";

export interface ZodiacSign {
  slug: ZodiacSlug;
  name: string;
  symbol: string;
}

export type PersonalizationMode = "sunSignLite" | "natal";
export type Mood = "开心" | "平静" | "迷茫" | "疲惫" | "低落";

export interface ProfileInput {
  nickname?: string;
  birthday: string;
  birthTime?: string | null;
  birthCity?: string | null;
}

export interface Profile {
  id: string;
  guestId: string;
  nickname: string;
  birthday: string;
  birthTime: string | null;
  birthCity: string | null;
  zodiac: ZodiacSign;
  personalizationMode: PersonalizationMode;
  completeness: number;
  calculationVersion: number;
  createdAt: string;
  updatedAt: string;
}

export interface AppState {
  onboardingComplete: boolean;
  profile: Profile | null;
  todayReady: boolean;
}

export interface ScoreDimensions {
  love: number;
  work: number;
  wealth: number;
  social: number;
  inner: number;
}

export interface ReadingScores {
  overall: number;
  dimensions: ScoreDimensions;
}

export interface ReadingBrief {
  key: "relationship" | "work" | "self";
  label: string;
  text: string;
  basedOn: string[];
}

export interface LuckyHint {
  value: string;
  basis: string;
  certainty: "entertainment-rule";
}

export interface DailyReading {
  id: string;
  date: string;
  weekday: string;
  zodiac: ZodiacSign;
  personalizationMode: PersonalizationMode;
  hero: {
    title: string;
    subtitle: string;
    theme: string;
  };
  scores: ReadingScores;
  briefs: ReadingBrief[];
  echo: string;
  lucky: {
    color: LuckyHint & { hex: string };
    number: LuckyHint;
    time: LuckyHint;
  };
  source: {
    provider: string;
    rulesVersion: string;
    narrativeProvider: string;
    stale: boolean;
  };
}

export interface MoodEntry {
  id: string;
  date: string;
  mood: Mood;
}

export interface TrailEntry {
  date: string;
  weekday: string;
  theme: string;
  scores: ReadingScores;
  mood: Mood | null;
  echo: string;
}

export interface TrailSummary {
  average: number;
  primaryMood: Mood | null;
  primaryMoodCount: number;
  risingDimension: keyof ScoreDimensions;
  risingDelta: number;
}

export interface TrailResponse {
  summary: TrailSummary;
  entries: TrailEntry[];
}

export type FortuneGrade =
  | "上上签"
  | "上签"
  | "中上签"
  | "中签"
  | "中下签"
  | "下签"
  | "下下签";

export interface DailyFortune {
  id: string;
  date: string;
  grade: FortuneGrade;
  title: string;
  verse: string;
  interpretation: string;
  advice: string;
}

export interface AppErrorShape {
  code: string;
  message: string;
  retryable: boolean;
}
