use tauri::State;

use crate::{
    domain::models::{
        AppStateDto, CompatibilityReading, DailyFortune, DailyReading, EmotionGuide, MoodEntry,
        Profile, ProfileInput, TrailResponse,
    },
    error::AppError,
    services::AppRuntime,
};

#[tauri::command]
pub async fn get_app_state(state: State<'_, AppRuntime>) -> Result<AppStateDto, AppError> {
    state.app_state().await
}

#[tauri::command]
pub async fn get_profile(state: State<'_, AppRuntime>) -> Result<Option<Profile>, AppError> {
    state.profile().await
}

#[tauri::command]
pub async fn save_profile(
    state: State<'_, AppRuntime>,
    input: ProfileInput,
) -> Result<Profile, AppError> {
    state.save_profile(input).await
}

#[tauri::command]
pub async fn get_daily_reading(state: State<'_, AppRuntime>) -> Result<DailyReading, AppError> {
    state.daily_reading().await
}

#[tauri::command]
pub async fn save_mood(state: State<'_, AppRuntime>, mood: String) -> Result<MoodEntry, AppError> {
    state.save_mood(mood).await
}

#[tauri::command]
pub async fn get_today_mood(state: State<'_, AppRuntime>) -> Result<Option<MoodEntry>, AppError> {
    state.today_mood().await
}

#[tauri::command]
pub async fn get_trail(state: State<'_, AppRuntime>, days: i64) -> Result<TrailResponse, AppError> {
    state.trail(days).await
}

#[tauri::command]
pub async fn get_daily_fortune(
    state: State<'_, AppRuntime>,
) -> Result<Option<DailyFortune>, AppError> {
    state.fortune().await
}

#[tauri::command]
pub async fn draw_daily_fortune(state: State<'_, AppRuntime>) -> Result<DailyFortune, AppError> {
    state.draw_fortune().await
}

#[tauri::command]
pub async fn get_compatibility(
    state: State<'_, AppRuntime>,
    partner_sign: String,
) -> Result<CompatibilityReading, AppError> {
    state.compatibility(partner_sign).await
}

#[tauri::command]
pub async fn get_emotion_guide(
    state: State<'_, AppRuntime>,
    mood: Option<String>,
) -> Result<Option<EmotionGuide>, AppError> {
    state.emotion_guide(mood).await
}
