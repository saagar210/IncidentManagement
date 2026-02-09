use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::{audit, stakeholder_updates};
use crate::error::AppError;
use crate::models::stakeholder_update::{CreateStakeholderUpdateRequest, StakeholderUpdate};

#[tauri::command]
pub async fn list_stakeholder_updates(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<Vec<StakeholderUpdate>, AppError> {
    stakeholder_updates::list_by_incident(&*db, &incident_id).await
}

#[tauri::command]
pub async fn create_stakeholder_update(
    db: State<'_, SqlitePool>,
    req: CreateStakeholderUpdateRequest,
) -> Result<StakeholderUpdate, AppError> {
    req.validate()?;
    let id = format!("su-{}", uuid::Uuid::new_v4());
    let result = stakeholder_updates::create(&*db, &id, &req).await?;
    if let Err(e) = audit::insert_audit_entry(
        &*db,
        "stakeholder_update",
        &id,
        "created",
        &format!(
            "Created {} stakeholder update for incident {}",
            &req.update_type, &req.incident_id
        ),
        "",
    )
    .await
    {
        eprintln!("Warning: failed to write audit entry for stakeholder update create: {}", e);
    }
    Ok(result)
}

#[tauri::command]
pub async fn delete_stakeholder_update(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    stakeholder_updates::delete(&*db, &id).await?;
    if let Err(e) = audit::insert_audit_entry(
        &*db,
        "stakeholder_update",
        &id,
        "deleted",
        "Deleted stakeholder update",
        "",
    )
    .await
    {
        eprintln!("Warning: failed to write audit entry for stakeholder update delete: {}", e);
    }
    Ok(())
}
