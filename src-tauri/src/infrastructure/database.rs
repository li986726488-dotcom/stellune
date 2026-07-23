use std::path::Path;

use chrono::Local;
use serde::{Serialize, de::DeserializeOwned};
use sqlx::{
    Row, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

use crate::{
    domain::models::{DailyFortune, DailyReading, MoodEntry, Profile, ReadingTrace, TrailEntry},
    error::AppError,
};

pub async fn connect(path: &Path) -> Result<SqlitePool, AppError> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(AppError::database)?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(AppError::database)?;
    Ok(pool)
}

pub async fn get_profile(pool: &SqlitePool) -> Result<Option<Profile>, AppError> {
    let row = sqlx::query("SELECT profile_json FROM profiles ORDER BY updated_at DESC LIMIT 1")
        .fetch_optional(pool)
        .await
        .map_err(AppError::database)?;
    row.map(|row| from_json(row.get("profile_json")))
        .transpose()
}

pub async fn save_profile(pool: &SqlitePool, profile: &Profile) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO profiles (id, profile_json, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET profile_json = excluded.profile_json, updated_at = excluded.updated_at",
    )
    .bind(&profile.id)
    .bind(to_json(profile)?)
    .bind(&profile.updated_at)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub async fn get_reading(
    pool: &SqlitePool,
    cache_key: &str,
) -> Result<Option<DailyReading>, AppError> {
    let row = sqlx::query("SELECT reading_json FROM daily_readings WHERE cache_key = ?")
        .bind(cache_key)
        .fetch_optional(pool)
        .await
        .map_err(AppError::database)?;
    row.map(|row| from_json(row.get("reading_json")))
        .transpose()
}

pub struct SnapshotRecord<'a> {
    pub id: &'a str,
    pub snapshot_key: &'a str,
    pub provider: &'a str,
    pub endpoint: &'a str,
    pub request_json: &'a serde_json::Value,
    pub raw_response_json: &'a serde_json::Value,
    pub normalized_json: &'a [crate::domain::models::PlanetPosition],
}

pub async fn save_snapshot(
    pool: &SqlitePool,
    snapshot: SnapshotRecord<'_>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT OR REPLACE INTO sky_snapshots
         (id, snapshot_key, provider, endpoint, request_json, raw_response_json, normalized_json, captured_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(snapshot.id)
    .bind(snapshot.snapshot_key)
    .bind(snapshot.provider)
    .bind(snapshot.endpoint)
    .bind(to_json(snapshot.request_json)?)
    .bind(to_json(snapshot.raw_response_json)?)
    .bind(to_json(snapshot.normalized_json)?)
    .bind(Local::now().to_rfc3339())
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub async fn save_reading(
    pool: &SqlitePool,
    cache_key: &str,
    profile: &Profile,
    reading: &DailyReading,
    trace: &ReadingTrace,
) -> Result<bool, AppError> {
    let now = Local::now().to_rfc3339();
    let mut transaction = pool.begin().await.map_err(AppError::database)?;

    // Acquire SQLite's write lock before checking the profile version so a
    // newer profile save cannot commit between the check and current switch.
    sqlx::query("UPDATE profiles SET updated_at = updated_at WHERE id = ?")
        .bind(&profile.id)
        .execute(&mut *transaction)
        .await
        .map_err(AppError::database)?;
    let current_profile = sqlx::query("SELECT profile_json FROM profiles WHERE id = ?")
        .bind(&profile.id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(AppError::database)?
        .map(|row| from_json::<Profile>(row.get("profile_json")))
        .transpose()?;
    if current_profile
        .as_ref()
        .is_none_or(|current| current.calculation_version != profile.calculation_version)
    {
        transaction.rollback().await.map_err(AppError::database)?;
        return Ok(false);
    }

    sqlx::query(
        "UPDATE daily_readings
         SET is_current = 0, superseded_at = ?
         WHERE profile_id = ? AND local_date = ? AND is_current = 1",
    )
    .bind(&now)
    .bind(&profile.id)
    .bind(&reading.date)
    .execute(&mut *transaction)
    .await
    .map_err(AppError::database)?;

    sqlx::query(
        "INSERT INTO daily_readings
         (
           id,
           cache_key,
           local_date,
           profile_id,
           reading_json,
           trace_json,
           created_at,
           profile_version,
           profile_snapshot_json,
           is_current,
           superseded_at
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1, NULL)
         ON CONFLICT DO UPDATE SET
           reading_json = excluded.reading_json,
           trace_json = excluded.trace_json,
           created_at = excluded.created_at,
           profile_version = excluded.profile_version,
           profile_snapshot_json = excluded.profile_snapshot_json,
           is_current = 1,
           superseded_at = NULL",
    )
    .bind(&reading.id)
    .bind(cache_key)
    .bind(&reading.date)
    .bind(&profile.id)
    .bind(to_json(reading)?)
    .bind(to_json(trace)?)
    .bind(&now)
    .bind(profile.calculation_version)
    .bind(to_json(profile)?)
    .execute(&mut *transaction)
    .await
    .map_err(AppError::database)?;
    transaction.commit().await.map_err(AppError::database)?;
    Ok(true)
}

pub async fn get_current_reading_for_date(
    pool: &SqlitePool,
    profile_id: &str,
    local_date: &str,
) -> Result<Option<DailyReading>, AppError> {
    let row = sqlx::query(
        "SELECT reading_json
         FROM daily_readings
         WHERE profile_id = ? AND local_date = ? AND is_current = 1
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(profile_id)
    .bind(local_date)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;
    row.map(|row| from_json(row.get("reading_json")))
        .transpose()
}

pub async fn save_mood(pool: &SqlitePool, mood: &MoodEntry) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO mood_entries (id, local_date, mood, updated_at) VALUES (?, ?, ?, ?)
         ON CONFLICT(local_date) DO UPDATE SET mood = excluded.mood, updated_at = excluded.updated_at",
    )
    .bind(&mood.id)
    .bind(&mood.date)
    .bind(&mood.mood)
    .bind(Local::now().to_rfc3339())
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub async fn get_mood(pool: &SqlitePool, date: &str) -> Result<Option<MoodEntry>, AppError> {
    let row = sqlx::query(
        "SELECT id, local_date, mood
         FROM mood_entries
         WHERE local_date = ?",
    )
    .bind(date)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;
    row.map(|row| {
        Ok(MoodEntry {
            id: row.try_get("id").map_err(AppError::database)?,
            date: row.try_get("local_date").map_err(AppError::database)?,
            mood: row.try_get("mood").map_err(AppError::database)?,
        })
    })
    .transpose()
}

pub async fn get_trail_entries(pool: &SqlitePool, days: i64) -> Result<Vec<TrailEntry>, AppError> {
    let rows = sqlx::query(
        "SELECT r.reading_json, m.mood
         FROM daily_readings r
         LEFT JOIN mood_entries m ON m.local_date = r.local_date
         WHERE r.is_current = 1
         ORDER BY r.local_date DESC
         LIMIT ?",
    )
    .bind(days)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut entries = rows
        .into_iter()
        .map(|row| {
            let reading: DailyReading = from_json(row.get("reading_json"))?;
            Ok(TrailEntry {
                date: reading.date,
                weekday: reading.weekday,
                theme: reading.hero.theme,
                scores: reading.scores,
                mood: row.try_get("mood").ok(),
                echo: reading.echo,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    entries.reverse();
    Ok(entries)
}

pub async fn get_fortune(
    pool: &SqlitePool,
    date: &str,
    guest_id: &str,
    rules_version: &str,
) -> Result<Option<DailyFortune>, AppError> {
    let row = sqlx::query(
        "SELECT fortune_json FROM daily_fortunes
         WHERE local_date = ? AND guest_id = ? AND rules_version = ?",
    )
    .bind(date)
    .bind(guest_id)
    .bind(rules_version)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;
    row.map(|row| from_json(row.get("fortune_json")))
        .transpose()
}

pub async fn save_fortune(
    pool: &SqlitePool,
    fortune: &DailyFortune,
    guest_id: &str,
    rules_version: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO daily_fortunes
         (id, local_date, guest_id, rules_version, fortune_json, created_at)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(local_date, guest_id, rules_version) DO UPDATE SET
           id = excluded.id,
           fortune_json = excluded.fortune_json,
           created_at = excluded.created_at",
    )
    .bind(&fortune.id)
    .bind(&fortune.date)
    .bind(guest_id)
    .bind(rules_version)
    .bind(to_json(fortune)?)
    .bind(Local::now().to_rfc3339())
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn to_json<T: Serialize + ?Sized>(value: &T) -> Result<String, AppError> {
    serde_json::to_string(value).map_err(AppError::internal)
}

fn from_json<T: DeserializeOwned>(value: String) -> Result<T, AppError> {
    serde_json::from_str(&value).map_err(AppError::internal)
}
