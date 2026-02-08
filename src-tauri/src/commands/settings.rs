use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::settings;
use crate::error::AppError;
use crate::models::quarter::{QuarterConfig, UpsertQuarterRequest};

#[tauri::command]
pub async fn get_quarter_configs(
    db: State<'_, SqlitePool>,
) -> Result<Vec<QuarterConfig>, AppError> {
    settings::get_quarter_configs(&*db).await
}

#[tauri::command]
pub async fn upsert_quarter_config(
    db: State<'_, SqlitePool>,
    config: UpsertQuarterRequest,
) -> Result<QuarterConfig, AppError> {
    config.validate()?;
    settings::upsert_quarter(&*db, &config).await
}

#[tauri::command]
pub async fn delete_quarter_config(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    settings::delete_quarter(&*db, &id).await
}

#[tauri::command]
pub async fn get_setting(
    db: State<'_, SqlitePool>,
    key: String,
) -> Result<Option<String>, AppError> {
    settings::get_setting(&*db, &key).await
}

#[tauri::command]
pub async fn set_setting(
    db: State<'_, SqlitePool>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    settings::set_setting(&*db, &key, &value).await
}
