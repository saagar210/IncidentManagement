use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::sla;
use crate::error::AppError;
use crate::models::sla::{
    CreateSlaDefinitionRequest, SlaDefinition, SlaStatus, UpdateSlaDefinitionRequest,
};

#[tauri::command]
pub async fn list_sla_definitions(
    db: State<'_, SqlitePool>,
) -> Result<Vec<SlaDefinition>, AppError> {
    sla::list_sla_definitions(&*db).await
}

#[tauri::command]
pub async fn create_sla_definition(
    db: State<'_, SqlitePool>,
    req: CreateSlaDefinitionRequest,
) -> Result<SlaDefinition, AppError> {
    req.validate()?;
    sla::create_sla_definition(&*db, &req).await
}

#[tauri::command]
pub async fn update_sla_definition(
    db: State<'_, SqlitePool>,
    id: String,
    req: UpdateSlaDefinitionRequest,
) -> Result<SlaDefinition, AppError> {
    req.validate()?;
    sla::update_sla_definition(&*db, &id, &req).await
}

#[tauri::command]
pub async fn delete_sla_definition(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    sla::delete_sla_definition(&*db, &id).await
}

#[tauri::command]
pub async fn compute_sla_status(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<SlaStatus, AppError> {
    sla::compute_sla_status(&*db, &incident_id).await
}
