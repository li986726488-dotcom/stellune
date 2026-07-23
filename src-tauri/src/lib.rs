mod commands;
mod domain;
mod error;
mod infrastructure;
mod services;

use tauri::Manager;

use services::AppRuntime;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Local development and packaged builds can read credentials from the
    // nearest ignored `.env`; explicit process environment values still win.
    let _ = dotenvy::dotenv();

    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let database_path = app_data_dir.join("stellune.db");
            let logger = infrastructure::logger::AppLogger::new(&app_data_dir);
            let pool =
                tauri::async_runtime::block_on(infrastructure::database::connect(&database_path))?;
            app.manage(AppRuntime::new(pool, logger));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::get_profile,
            commands::save_profile,
            commands::get_daily_reading,
            commands::save_mood,
            commands::get_trail,
            commands::get_daily_fortune,
            commands::draw_daily_fortune,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Stellune");
}
