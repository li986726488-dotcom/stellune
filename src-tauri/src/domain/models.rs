use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZodiacSign {
    pub slug: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInput {
    #[serde(default)]
    pub nickname: Option<String>,
    pub birthday: String,
    #[serde(default)]
    pub birth_time: Option<String>,
    #[serde(default)]
    pub birth_city: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub guest_id: String,
    pub nickname: String,
    pub birthday: String,
    pub birth_time: Option<String>,
    pub birth_city: Option<String>,
    pub zodiac: ZodiacSign,
    pub personalization_mode: String,
    pub completeness: i32,
    #[serde(default = "default_profile_version")]
    pub calculation_version: i64,
    pub created_at: String,
    pub updated_at: String,
}

fn default_profile_version() -> i64 {
    1
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStateDto {
    pub onboarding_complete: bool,
    pub profile: Option<Profile>,
    pub today_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanetPosition {
    pub planet: String,
    pub longitude: f64,
    pub degree_in_sign: f64,
    pub sign: String,
    pub speed: f64,
    pub retrograde: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScoreDimensions {
    pub love: i32,
    pub work: i32,
    pub wealth: i32,
    pub social: i32,
    pub inner: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingScores {
    pub overall: i32,
    pub dimensions: ScoreDimensions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingBrief {
    pub key: String,
    pub label: String,
    pub text: String,
    pub based_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LuckyHint {
    pub value: String,
    pub basis: String,
    pub certainty: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LuckyColor {
    pub value: String,
    pub hex: String,
    pub basis: String,
    pub certainty: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LuckyHints {
    pub color: LuckyColor,
    pub number: LuckyHint,
    pub time: LuckyHint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingHero {
    pub title: String,
    pub subtitle: String,
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingSource {
    pub provider: String,
    pub rules_version: String,
    pub narrative_provider: String,
    #[serde(default)]
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyReading {
    pub id: String,
    pub date: String,
    pub weekday: String,
    pub zodiac: ZodiacSign,
    pub personalization_mode: String,
    pub hero: ReadingHero,
    pub scores: ReadingScores,
    pub briefs: Vec<ReadingBrief>,
    pub echo: String,
    pub lucky: LuckyHints,
    pub source: ReadingSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoodEntry {
    pub id: String,
    pub date: String,
    pub mood: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailEntry {
    pub date: String,
    pub weekday: String,
    pub theme: String,
    pub scores: ReadingScores,
    pub mood: Option<String>,
    pub echo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailSummary {
    pub average: i32,
    pub primary_mood: Option<String>,
    pub primary_mood_count: i32,
    pub rising_dimension: String,
    pub rising_delta: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailResponse {
    pub summary: TrailSummary,
    pub entries: Vec<TrailEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyFortune {
    pub id: String,
    pub date: String,
    pub number: i32,
    pub catalog: String,
    pub grade: String,
    pub title: String,
    pub verse: String,
    pub interpretation: String,
    pub advice: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityFactor {
    pub key: String,
    pub label: String,
    pub score: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityReading {
    pub mode: String,
    pub primary_sign: ZodiacSign,
    pub partner_sign: ZodiacSign,
    pub score: i32,
    pub level: String,
    pub title: String,
    pub summary: String,
    pub factors: Vec<CompatibilityFactor>,
    pub certainty: String,
    pub rules_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmotionGuide {
    pub mood: String,
    pub title: String,
    pub summary: String,
    pub action: String,
    pub need: String,
    pub based_on: Vec<String>,
    pub generator: String,
    pub certainty: String,
    pub rules_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleImpact {
    pub id: String,
    pub planet: String,
    pub aspect: String,
    pub orb: f64,
    pub impact: ScoreDimensions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingTrace {
    pub reference_degree: f64,
    pub rules_version: String,
    pub impacts: Vec<RuleImpact>,
}
