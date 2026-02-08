use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::{audit, shift_handoffs};
use crate::error::AppError;
use crate::models::shift_handoff::{CreateShiftHandoffRequest, ShiftHandoff};

#[tauri::command]
pub async fn list_shift_handoffs(
    db: State<'_, SqlitePool>,
    limit: Option<i64>,
) -> Result<Vec<ShiftHandoff>, AppError> {
    let limit = limit.unwrap_or(20).min(100);
    shift_handoffs::list_recent(&*db, limit).await
}

#[tauri::command]
pub async fn create_shift_handoff(
    db: State<'_, SqlitePool>,
    req: CreateShiftHandoffRequest,
) -> Result<ShiftHandoff, AppError> {
    req.validate()?;
    let id = format!("sh-{}", uuid::Uuid::new_v4());
    let result = shift_handoffs::create(&*db, &id, &req).await?;
    let _ = audit::insert_audit_entry(
        &*db,
        "shift_handoff",
        &id,
        "created",
        &format!(
            "Created shift handoff by '{}'",
            if req.created_by.is_empty() {
                "unknown"
            } else {
                &req.created_by
            }
        ),
        "",
    )
    .await;
    Ok(result)
}

#[tauri::command]
pub async fn delete_shift_handoff(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    shift_handoffs::delete(&*db, &id).await?;
    let _ = audit::insert_audit_entry(
        &*db,
        "shift_handoff",
        &id,
        "deleted",
        "Deleted shift handoff",
        "",
    )
    .await;
    Ok(())
}
