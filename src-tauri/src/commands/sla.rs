use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::{audit, sla};
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
    let result = sla::create_sla_definition(&*db, &req).await?;
    let _ = audit::insert_audit_entry(
        &*db,
        "sla_definition",
        &result.id,
        "created",
        &format!("Created SLA definition: {} ({})", &req.name, &req.priority),
        "",
    )
    .await;
    Ok(result)
}

#[tauri::command]
pub async fn update_sla_definition(
    db: State<'_, SqlitePool>,
    id: String,
    req: UpdateSlaDefinitionRequest,
) -> Result<SlaDefinition, AppError> {
    req.validate()?;
    let result = sla::update_sla_definition(&*db, &id, &req).await?;
    let _ = audit::insert_audit_entry(&*db, "sla_definition", &id, "updated", "Updated SLA definition", "").await;
    Ok(result)
}

#[tauri::command]
pub async fn delete_sla_definition(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    sla::delete_sla_definition(&*db, &id).await?;
    let _ = audit::insert_audit_entry(&*db, "sla_definition", &id, "deleted", "Deleted SLA definition", "").await;
    Ok(())
}

#[tauri::command]
pub async fn compute_sla_status(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<SlaStatus, AppError> {
    sla::compute_sla_status(&*db, &incident_id).await
}
