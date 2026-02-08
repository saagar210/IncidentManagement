use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::{audit, roles};
use crate::error::AppError;
use crate::models::role::{AssignRoleRequest, IncidentRole};

#[tauri::command]
pub async fn assign_role(
    db: State<'_, SqlitePool>,
    req: AssignRoleRequest,
) -> Result<IncidentRole, AppError> {
    req.validate()?;
    let id = format!("role-{}", uuid::Uuid::new_v4());
    let result = roles::assign_role(
        &*db,
        &id,
        &req.incident_id,
        &req.role,
        &req.assignee,
        req.is_primary,
    )
    .await?;
    let _ = audit::insert_audit_entry(
        &*db,
        "incident",
        &req.incident_id,
        "role_assigned",
        &format!("Assigned {} as {}", &req.assignee, &req.role),
        "",
    )
    .await;
    Ok(result)
}

#[tauri::command]
pub async fn unassign_role(db: State<'_, SqlitePool>, id: String) -> Result<(), AppError> {
    roles::unassign_role(&*db, &id).await?;
    let _ = audit::insert_audit_entry(
        &*db,
        "incident_role",
        &id,
        "role_unassigned",
        "Unassigned role",
        "",
    )
    .await;
    Ok(())
}

#[tauri::command]
pub async fn list_incident_roles(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<Vec<IncidentRole>, AppError> {
    roles::list_roles_for_incident(&*db, &incident_id).await
}
