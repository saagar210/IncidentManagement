use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::{audit, saved_filters};
use crate::error::AppError;
use crate::models::saved_filter::{
    CreateSavedFilterRequest, SavedFilter, UpdateSavedFilterRequest,
};

#[tauri::command]
pub async fn list_saved_filters(
    db: State<'_, SqlitePool>,
) -> Result<Vec<SavedFilter>, AppError> {
    saved_filters::list_saved_filters(&*db).await
}

#[tauri::command]
pub async fn create_saved_filter(
    db: State<'_, SqlitePool>,
    req: CreateSavedFilterRequest,
) -> Result<SavedFilter, AppError> {
    req.validate()?;
    let id = format!("sf-{}", uuid::Uuid::new_v4());
    let result = saved_filters::create_saved_filter(&*db, &id, &req).await?;
    if let Err(e) = audit::insert_audit_entry(
        &*db,
        "saved_filter",
        &id,
        "created",
        &format!("Created saved filter: {}", &req.name),
        "",
    )
    .await
    {
        eprintln!("Warning: failed to write audit entry for saved filter create: {}", e);
    }
    Ok(result)
}

#[tauri::command]
pub async fn update_saved_filter(
    db: State<'_, SqlitePool>,
    id: String,
    req: UpdateSavedFilterRequest,
) -> Result<SavedFilter, AppError> {
    req.validate()?;
    let result = saved_filters::update_saved_filter(&*db, &id, &req).await?;
    if let Err(e) = audit::insert_audit_entry(
        &*db,
        "saved_filter",
        &id,
        "updated",
        "Updated saved filter",
        "",
    )
    .await
    {
        eprintln!("Warning: failed to write audit entry for saved filter update: {}", e);
    }
    Ok(result)
}

#[tauri::command]
pub async fn delete_saved_filter(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    saved_filters::delete_saved_filter(&*db, &id).await?;
    if let Err(e) = audit::insert_audit_entry(
        &*db,
        "saved_filter",
        &id,
        "deleted",
        "Deleted saved filter",
        "",
    )
    .await
    {
        eprintln!("Warning: failed to write audit entry for saved filter delete: {}", e);
    }
    Ok(())
}
