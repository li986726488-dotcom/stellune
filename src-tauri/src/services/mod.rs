use std::collections::HashMap;

use chrono::{Local, NaiveDate};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    domain::{
        models::{
            AppStateDto, DailyFortune, DailyReading, MoodEntry, Profile, ProfileInput,
            ScoreDimensions, TrailResponse, TrailSummary,
        },
        rules::{
            FORTUNE_RULES_VERSION, RULES_VERSION, calculate_reading, fortune_by_index,
            zodiac_from_birthday,
        },
    },
    error::AppError,
    infrastructure::{
        astrology::{ASTROLOGY_ENDPOINT, AstrologyProvider},
        database::{self, SnapshotRecord},
        logger::AppLogger,
    },
};

const ASTROLOGY_PROVIDER_VERSION: &str = "astrologyapi-planets-tropical-v1";
const NARRATIVE_PROMPT_VERSION: &str = "template-2026.07.1";

#[derive(Clone)]
pub struct AppRuntime {
    pub pool: SqlitePool,
    pub astrology: AstrologyProvider,
    pub logger: AppLogger,
}

impl AppRuntime {
    pub fn new(pool: SqlitePool, logger: AppLogger) -> Self {
        Self {
            pool,
            astrology: AstrologyProvider::new(logger.clone()),
            logger,
        }
    }

    pub async fn app_state(&self) -> Result<AppStateDto, AppError> {
        let profile = database::get_profile(&self.pool).await?;
        let today_ready = if let Some(profile) = &profile {
            database::get_reading(&self.pool, &reading_cache_key(profile, today()))
                .await?
                .is_some()
        } else {
            false
        };
        Ok(AppStateDto {
            onboarding_complete: profile.is_some(),
            profile,
            today_ready,
        })
    }

    pub async fn profile(&self) -> Result<Option<Profile>, AppError> {
        database::get_profile(&self.pool).await
    }

    pub async fn save_profile(&self, input: ProfileInput) -> Result<Profile, AppError> {
        let zodiac = zodiac_from_birthday(&input.birthday)?;
        let existing = database::get_profile(&self.pool).await?;
        let now = Local::now().to_rfc3339();
        let provided_nickname = clean_optional(input.nickname);
        let nickname = provided_nickname
            .clone()
            .unwrap_or_else(|| "星际旅人".into());
        let birth_time = clean_optional(input.birth_time);
        let birth_city = clean_optional(input.birth_city);
        validate_birth_time(birth_time.as_deref())?;
        let calculation_changed = existing.as_ref().is_none_or(|profile| {
            profile.birthday != input.birthday
                || profile.birth_time != birth_time
                || profile.birth_city != birth_city
        });
        let personalization_mode = if birth_time.is_some() && birth_city.is_some() {
            "natal"
        } else {
            "sunSignLite"
        };
        let completeness = 40
            + i32::from(provided_nickname.is_some()) * 20
            + i32::from(birth_time.is_some()) * 20
            + i32::from(birth_city.is_some()) * 20;
        let profile = Profile {
            id: existing
                .as_ref()
                .map(|value| value.id.clone())
                .unwrap_or_else(|| "profile-local".into()),
            guest_id: existing
                .as_ref()
                .map(|value| value.guest_id.clone())
                .unwrap_or_else(|| format!("guest-{}", Uuid::new_v4())),
            nickname,
            birthday: input.birthday,
            birth_time,
            birth_city,
            zodiac,
            personalization_mode: personalization_mode.into(),
            completeness,
            calculation_version: existing
                .as_ref()
                .map(|profile| profile.calculation_version + i64::from(calculation_changed))
                .unwrap_or(1),
            created_at: existing
                .as_ref()
                .map(|value| value.created_at.clone())
                .unwrap_or_else(|| now.clone()),
            updated_at: now,
        };
        database::save_profile(&self.pool, &profile).await?;
        Ok(profile)
    }

    pub async fn daily_reading(&self) -> Result<DailyReading, AppError> {
        let date = today();
        for _ in 0..3 {
            let correlation_id = format!("reading-{}", Uuid::new_v4());
            let profile = self
                .profile()
                .await?
                .ok_or_else(AppError::profile_incomplete)?;
            let cache_key = reading_cache_key(&profile, date);
            if let Some(reading) = database::get_reading(&self.pool, &cache_key).await? {
                self.logger.info(
                    "reading.cache.hit",
                    &correlation_id,
                    json!({
                        "date": reading.date,
                        "readingId": reading.id,
                        "profileVersion": profile.calculation_version,
                        "cacheKey": cache_key,
                    }),
                );
                return Ok(reading);
            }

            self.logger.info(
                "reading.cache.miss",
                &correlation_id,
                json!({
                    "date": date.format("%Y-%m-%d").to_string(),
                    "profileVersion": profile.calculation_version,
                    "personalizationMode": profile.personalization_mode,
                    "zodiac": profile.zodiac.slug,
                    "cacheKey": cache_key,
                }),
            );
            let (latitude, longitude, timezone) = coordinates(profile.birth_city.as_deref());
            let sky = match self
                .astrology
                .fetch(&correlation_id, date, latitude, longitude, timezone)
                .await
            {
                Ok(sky) => sky,
                Err(error) => {
                    if let Some(mut fallback) = database::get_current_reading_for_date(
                        &self.pool,
                        &profile.id,
                        &date.format("%Y-%m-%d").to_string(),
                    )
                    .await?
                    {
                        fallback.source.stale = true;
                        self.logger.error(
                            "reading.fallback",
                            &correlation_id,
                            json!({
                                "reason": error.code,
                                "message": error.message,
                                "fallbackReadingId": fallback.id,
                            }),
                        );
                        return Ok(fallback);
                    }
                    return Err(error);
                }
            };
            let snapshot_key = format!(
                "{}:{latitude:.4}:{longitude:.4}:{timezone}",
                date.format("%Y-%m-%d")
            );
            let snapshot_id = format!("snapshot-{}", stable_hash(&snapshot_key));
            database::save_snapshot(
                &self.pool,
                SnapshotRecord {
                    id: &snapshot_id,
                    snapshot_key: &snapshot_key,
                    provider: "AstrologyAPI",
                    endpoint: ASTROLOGY_ENDPOINT,
                    request_json: &sky.request,
                    raw_response_json: &sky.raw,
                    normalized_json: &sky.positions,
                },
            )
            .await?;
            let (mut reading, trace) = calculate_reading(&profile, &sky.positions, date);
            reading.id = format!(
                "reading-{}-{}",
                reading.date,
                cache_key.get(..16).unwrap_or(&cache_key)
            );
            self.logger.info(
                "rules.calculated",
                &correlation_id,
                json!({
                    "profileVersion": profile.calculation_version,
                    "zodiac": profile.zodiac.slug,
                    "positions": sky.positions,
                    "trace": trace,
                    "scores": reading.scores,
                    "lucky": reading.lucky,
                }),
            );
            self.logger.generation_input(
                &correlation_id,
                "template",
                json!({
                    "date": reading.date,
                    "zodiac": reading.zodiac,
                    "personalizationMode": reading.personalization_mode,
                    "theme": reading.hero.theme,
                    "scores": reading.scores,
                    "trace": trace,
                }),
            );
            self.logger.generation_output(
                &correlation_id,
                "template",
                json!({
                    "hero": reading.hero,
                    "briefs": reading.briefs,
                    "echo": reading.echo,
                    "source": reading.source,
                }),
            );
            if database::save_reading(&self.pool, &cache_key, &profile, &reading, &trace).await? {
                self.logger.info(
                    "reading.persisted",
                    &correlation_id,
                    json!({
                        "readingId": reading.id,
                        "date": reading.date,
                        "profileVersion": profile.calculation_version,
                    }),
                );
                return Ok(reading);
            }
            self.logger.info(
                "reading.discarded",
                &correlation_id,
                json!({
                    "reason": "profile-version-changed",
                    "profileVersion": profile.calculation_version,
                }),
            );
        }

        Err(AppError::internal("个人资料连续发生变化，请稍后重新校准。"))
    }

    pub async fn save_mood(&self, mood: String) -> Result<MoodEntry, AppError> {
        const MOODS: [&str; 5] = ["开心", "平静", "迷茫", "疲惫", "低落"];
        if !MOODS.contains(&mood.as_str()) {
            return Err(AppError::validation("请选择列表中的心情。"));
        }
        let date = today().format("%Y-%m-%d").to_string();
        let entry = MoodEntry {
            id: format!("mood-{date}"),
            date,
            mood,
        };
        database::save_mood(&self.pool, &entry).await?;
        Ok(entry)
    }

    pub async fn trail(&self, days: i64) -> Result<TrailResponse, AppError> {
        let days = days.clamp(1, 90);
        let entries = database::get_trail_entries(&self.pool, days).await?;
        let summary = summarize_trail(&entries);
        Ok(TrailResponse { summary, entries })
    }

    pub async fn fortune(&self) -> Result<Option<DailyFortune>, AppError> {
        let profile = self
            .profile()
            .await?
            .ok_or_else(AppError::profile_incomplete)?;
        let date = today().format("%Y-%m-%d").to_string();
        database::get_fortune(&self.pool, &date, &profile.guest_id, FORTUNE_RULES_VERSION).await
    }

    pub async fn draw_fortune(&self) -> Result<DailyFortune, AppError> {
        let profile = self
            .profile()
            .await?
            .ok_or_else(AppError::profile_incomplete)?;
        if let Some(fortune) = self.fortune().await? {
            return Ok(fortune);
        }
        let date = today().format("%Y-%m-%d").to_string();
        let seed = stable_hash(&format!(
            "{}:{}:{}",
            date, profile.guest_id, FORTUNE_RULES_VERSION
        ));
        let index = usize::from_str_radix(&seed[..8], 16).unwrap_or_default() % 7;
        let fortune = fortune_by_index(index, &date);
        database::save_fortune(
            &self.pool,
            &fortune,
            &profile.guest_id,
            FORTUNE_RULES_VERSION,
        )
        .await?;
        Ok(fortune)
    }
}

fn today() -> NaiveDate {
    Local::now().date_naive()
}

fn reading_cache_key(profile: &Profile, date: NaiveDate) -> String {
    let (latitude, longitude, timezone) = coordinates(profile.birth_city.as_deref());
    stable_hash(&format!(
        "date={};calculation_version={};birthday={};birth_time={};birth_city={};mode={};latitude={latitude:.4};longitude={longitude:.4};timezone={timezone};rules={};astrology={};narrative={}",
        date.format("%Y-%m-%d"),
        profile.calculation_version,
        profile.birthday,
        profile.birth_time.as_deref().unwrap_or_default(),
        profile.birth_city.as_deref().unwrap_or_default(),
        profile.personalization_mode,
        RULES_VERSION,
        ASTROLOGY_PROVIDER_VERSION,
        NARRATIVE_PROMPT_VERSION
    ))
}

fn stable_hash(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    format!("{digest:x}")
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn validate_birth_time(value: Option<&str>) -> Result<(), AppError> {
    if let Some(value) = value {
        chrono::NaiveTime::parse_from_str(value, "%H:%M")
            .map_err(|_| AppError::validation("出生时间格式应为 HH:mm。"))?;
    }
    Ok(())
}

fn coordinates(city: Option<&str>) -> (f64, f64, f64) {
    match city.unwrap_or_default() {
        city if city.contains("北京") => (39.9042, 116.4074, 8.0),
        city if city.contains("深圳") => (22.5431, 114.0579, 8.0),
        city if city.contains("广州") => (23.1291, 113.2644, 8.0),
        city if city.contains("杭州") => (30.2741, 120.1551, 8.0),
        city if city.contains("成都") => (30.5728, 104.0668, 8.0),
        _ => (31.2304, 121.4737, 8.0),
    }
}

fn summarize_trail(entries: &[crate::domain::models::TrailEntry]) -> TrailSummary {
    if entries.is_empty() {
        return TrailSummary {
            average: 0,
            primary_mood: None,
            primary_mood_count: 0,
            rising_dimension: "social".into(),
            rising_delta: 0,
        };
    }
    let average = (entries
        .iter()
        .map(|entry| entry.scores.overall)
        .sum::<i32>() as f64
        / entries.len() as f64)
        .round() as i32;
    let mut moods = HashMap::<String, i32>::new();
    for mood in entries.iter().filter_map(|entry| entry.mood.as_ref()) {
        *moods.entry(mood.clone()).or_default() += 1;
    }
    let (primary_mood, primary_mood_count) = moods
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(mood, count)| (Some(mood), count))
        .unwrap_or((None, 0));
    let first = &entries.first().expect("non-empty").scores.dimensions;
    let last = &entries.last().expect("non-empty").scores.dimensions;
    let deltas = dimension_deltas(first, last);
    let (rising_dimension, rising_delta) = deltas
        .into_iter()
        .max_by_key(|(_, delta)| *delta)
        .unwrap_or(("social", 0));
    TrailSummary {
        average,
        primary_mood,
        primary_mood_count,
        rising_dimension: rising_dimension.into(),
        rising_delta,
    }
}

fn dimension_deltas(first: &ScoreDimensions, last: &ScoreDimensions) -> [(&'static str, i32); 5] {
    [
        ("love", last.love - first.love),
        ("work", last.work - first.work),
        ("wealth", last.wealth - first.wealth),
        ("social", last.social - first.social),
        ("inner", last.inner - first.inner),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn runtime() -> AppRuntime {
        let directory = tempfile::tempdir().expect("temporary directory");
        let directory = directory.keep();
        let database_path = directory.join("stellune-test.db");
        let logger = AppLogger::new(&directory);
        let pool = database::connect(&database_path)
            .await
            .expect("test database");
        AppRuntime::new(pool, logger)
    }

    #[tokio::test]
    async fn birthday_only_profile_uses_light_mode_and_40_percent() {
        let profile = runtime()
            .await
            .save_profile(ProfileInput {
                nickname: None,
                birthday: "1998-10-08".into(),
                birth_time: None,
                birth_city: None,
            })
            .await
            .expect("profile");
        assert_eq!(profile.personalization_mode, "sunSignLite");
        assert_eq!(profile.completeness, 40);
    }

    #[tokio::test]
    async fn complete_profile_uses_natal_mode_and_100_percent() {
        let profile = runtime()
            .await
            .save_profile(ProfileInput {
                nickname: Some("月光旅人".into()),
                birthday: "1998-10-08".into(),
                birth_time: Some("08:30".into()),
                birth_city: Some("杭州".into()),
            })
            .await
            .expect("profile");
        assert_eq!(profile.personalization_mode, "natal");
        assert_eq!(profile.completeness, 100);
    }

    #[tokio::test]
    async fn nickname_change_keeps_calculation_version() {
        let runtime = runtime().await;
        let initial = runtime
            .save_profile(ProfileInput {
                nickname: Some("星际旅人".into()),
                birthday: "1998-10-08".into(),
                birth_time: None,
                birth_city: None,
            })
            .await
            .expect("initial profile");
        let renamed = runtime
            .save_profile(ProfileInput {
                nickname: Some("月光旅人".into()),
                birthday: initial.birthday.clone(),
                birth_time: initial.birth_time.clone(),
                birth_city: initial.birth_city.clone(),
            })
            .await
            .expect("renamed profile");

        assert_eq!(renamed.calculation_version, initial.calculation_version);
        let date = NaiveDate::from_ymd_opt(2026, 7, 23).expect("date");
        assert_eq!(
            reading_cache_key(&renamed, date),
            reading_cache_key(&initial, date)
        );
    }

    #[tokio::test]
    async fn calculation_input_change_increments_version() {
        let runtime = runtime().await;
        let initial = runtime
            .save_profile(ProfileInput {
                nickname: Some("星际旅人".into()),
                birthday: "1998-10-08".into(),
                birth_time: None,
                birth_city: None,
            })
            .await
            .expect("initial profile");
        let recalibrated = runtime
            .save_profile(ProfileInput {
                nickname: Some("星际旅人".into()),
                birthday: initial.birthday.clone(),
                birth_time: Some("08:30".into()),
                birth_city: Some("杭州".into()),
            })
            .await
            .expect("recalibrated profile");

        assert_eq!(
            recalibrated.calculation_version,
            initial.calculation_version + 1
        );
        let date = NaiveDate::from_ymd_opt(2026, 7, 23).expect("date");
        assert_ne!(
            reading_cache_key(&recalibrated, date),
            reading_cache_key(&initial, date)
        );
    }

    #[tokio::test]
    async fn outdated_reading_cannot_replace_current_profile_version() {
        let runtime = runtime().await;
        let initial = runtime
            .save_profile(ProfileInput {
                nickname: Some("星际旅人".into()),
                birthday: "1998-10-08".into(),
                birth_time: None,
                birth_city: None,
            })
            .await
            .expect("initial profile");
        let date = NaiveDate::from_ymd_opt(2026, 7, 23).expect("date");
        let (mut first_reading, first_trace) = calculate_reading(&initial, &[], date);
        first_reading.id = "reading-profile-v1".into();
        first_reading.hero.theme = "旧版本".into();
        assert!(
            database::save_reading(
                &runtime.pool,
                "cache-profile-v1",
                &initial,
                &first_reading,
                &first_trace,
            )
            .await
            .expect("save first reading")
        );

        let recalibrated = runtime
            .save_profile(ProfileInput {
                nickname: Some("星际旅人".into()),
                birthday: initial.birthday.clone(),
                birth_time: Some("08:30".into()),
                birth_city: Some("杭州".into()),
            })
            .await
            .expect("recalibrated profile");
        let (mut second_reading, second_trace) = calculate_reading(&recalibrated, &[], date);
        second_reading.id = "reading-profile-v2".into();
        second_reading.hero.theme = "新版本".into();
        assert!(
            database::save_reading(
                &runtime.pool,
                "cache-profile-v2",
                &recalibrated,
                &second_reading,
                &second_trace,
            )
            .await
            .expect("save second reading")
        );

        assert!(
            !database::save_reading(
                &runtime.pool,
                "cache-profile-v1-late",
                &initial,
                &first_reading,
                &first_trace,
            )
            .await
            .expect("reject outdated reading")
        );

        let current =
            database::get_current_reading_for_date(&runtime.pool, &recalibrated.id, "2026-07-23")
                .await
                .expect("current reading")
                .expect("reading exists");
        assert_eq!(current.id, second_reading.id);
        let stored_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM daily_readings WHERE profile_id = ? AND local_date = ?",
        )
        .bind(&recalibrated.id)
        .bind("2026-07-23")
        .fetch_one(&runtime.pool)
        .await
        .expect("stored count");
        let current_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM daily_readings
             WHERE profile_id = ? AND local_date = ? AND is_current = 1",
        )
        .bind(&recalibrated.id)
        .bind("2026-07-23")
        .fetch_one(&runtime.pool)
        .await
        .expect("current count");
        assert_eq!(stored_count, 2);
        assert_eq!(current_count, 1);

        let trail = database::get_trail_entries(&runtime.pool, 7)
            .await
            .expect("trail");
        assert_eq!(trail.len(), 1);
        assert_eq!(trail[0].theme, "新版本");
    }
}
