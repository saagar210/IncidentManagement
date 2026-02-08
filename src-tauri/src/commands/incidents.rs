use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::{audit, incidents, settings, tags};
use crate::error::AppError;
use crate::models::incident::{
    ActionItem, CreateActionItemRequest, CreateIncidentRequest, Incident, IncidentFilters,
    UpdateActionItemRequest, UpdateIncidentRequest,
};

#[tauri::command]
pub async fn create_incident(
    db: State<'_, SqlitePool>,
    incident: CreateIncidentRequest,
) -> Result<Incident, AppError> {
    incident.validate()?;
    let id = format!("inc-{}", uuid::Uuid::new_v4());
    let result = incidents::insert_incident(&*db, &id, &incident).await?;
    let _ = audit::insert_audit_entry(
        &*db,
        "incident",
        &id,
        "created",
        &format!("Created incident: {}", &incident.title),
        "",
    )
    .await;
    Ok(result)
}

#[tauri::command]
pub async fn update_incident(
    db: State<'_, SqlitePool>,
    id: String,
    incident: UpdateIncidentRequest,
) -> Result<Incident, AppError> {
    incident.validate()?;
    let result = incidents::update_incident(&*db, &id, &incident).await?;
    let summary = if let Some(ref status) = incident.status {
        format!("Updated incident status to {}", status)
    } else {
        "Updated incident".to_string()
    };
    let _ = audit::insert_audit_entry(&*db, "incident", &id, "updated", &summary, "").await;
    Ok(result)
}

#[tauri::command]
pub async fn delete_incident(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    incidents::delete_incident(&*db, &id).await?;
    let _ = audit::insert_audit_entry(&*db, "incident", &id, "deleted", "Moved incident to trash", "").await;
    Ok(())
}

#[tauri::command]
pub async fn get_incident(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<Incident, AppError> {
    incidents::get_incident_by_id(&*db, &id).await
}

#[tauri::command]
pub async fn list_incidents(
    db: State<'_, SqlitePool>,
    filters: IncidentFilters,
) -> Result<Vec<Incident>, AppError> {
    // Resolve quarter_id to date range if provided
    let quarter_dates = if let Some(ref qid) = filters.quarter_id {
        let q = settings::get_quarter_by_id(&*db, qid).await?;
        Some((q.start_date, q.end_date))
    } else {
        None
    };

    incidents::list_incidents(&*db, &filters, quarter_dates).await
}

#[tauri::command]
pub async fn search_incidents(
    db: State<'_, SqlitePool>,
    query: String,
) -> Result<Vec<Incident>, AppError> {
    if query.len() > 500 {
        return Err(AppError::Validation(
            "Search query too long (max 500 characters)".into(),
        ));
    }
    incidents::search_incidents(&*db, &query).await
}

#[tauri::command]
pub async fn bulk_update_status(
    db: State<'_, SqlitePool>,
    ids: Vec<String>,
    status: String,
) -> Result<(), AppError> {
    incidents::bulk_update_status(&*db, &ids, &status).await
}

#[tauri::command]
pub async fn bulk_delete_incidents(
    db: State<'_, SqlitePool>,
    ids: Vec<String>,
) -> Result<i64, AppError> {
    incidents::bulk_delete_incidents(&*db, &ids).await
}

// Action Item commands

#[tauri::command]
pub async fn create_action_item(
    db: State<'_, SqlitePool>,
    item: CreateActionItemRequest,
) -> Result<ActionItem, AppError> {
    item.validate()?;
    let id = format!("ai-{}", uuid::Uuid::new_v4());
    let result = incidents::insert_action_item(&*db, &id, &item).await?;
    let _ = audit::insert_audit_entry(
        &*db,
        "action_item",
        &id,
        "created",
        &format!("Created action item: {}", &item.title),
        &format!("incident_id: {}", &item.incident_id),
    )
    .await;
    Ok(result)
}

#[tauri::command]
pub async fn update_action_item(
    db: State<'_, SqlitePool>,
    id: String,
    item: UpdateActionItemRequest,
) -> Result<ActionItem, AppError> {
    item.validate()?;
    let result = incidents::update_action_item(&*db, &id, &item).await?;
    let summary = if let Some(ref status) = item.status {
        format!("Updated action item status to {}", status)
    } else {
        "Updated action item".to_string()
    };
    let _ = audit::insert_audit_entry(&*db, "action_item", &id, "updated", &summary, "").await;
    Ok(result)
}

#[tauri::command]
pub async fn delete_action_item(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    incidents::delete_action_item(&*db, &id).await?;
    let _ = audit::insert_audit_entry(&*db, "action_item", &id, "deleted", "Deleted action item", "").await;
    Ok(())
}

#[tauri::command]
pub async fn list_action_items(
    db: State<'_, SqlitePool>,
    incident_id: Option<String>,
) -> Result<Vec<ActionItem>, AppError> {
    incidents::list_action_items(&*db, incident_id.as_deref()).await
}

// Tags

#[tauri::command]
pub async fn get_incident_tags(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<Vec<String>, AppError> {
    tags::get_incident_tags(&*db, &incident_id).await
}

#[tauri::command]
pub async fn set_incident_tags(
    db: State<'_, SqlitePool>,
    incident_id: String,
    tag_list: Vec<String>,
) -> Result<Vec<String>, AppError> {
    if tag_list.len() > 50 {
        return Err(AppError::Validation("Too many tags (max 50)".into()));
    }
    for tag in &tag_list {
        if tag.len() > 100 {
            return Err(AppError::Validation("Tag too long (max 100 characters)".into()));
        }
    }
    tags::set_incident_tags(&*db, &incident_id, &tag_list).await
}

#[tauri::command]
pub async fn get_all_tags(
    db: State<'_, SqlitePool>,
) -> Result<Vec<String>, AppError> {
    tags::get_all_tags(&*db).await
}

// Soft delete / Trash

#[tauri::command]
pub async fn list_deleted_incidents(
    db: State<'_, SqlitePool>,
) -> Result<Vec<Incident>, AppError> {
    incidents::list_deleted_incidents(&*db).await
}

#[tauri::command]
pub async fn restore_incident(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<Incident, AppError> {
    incidents::restore_incident(&*db, &id).await
}

#[tauri::command]
pub async fn permanent_delete_incident(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    incidents::permanent_delete_incident(&*db, &id).await
}

#[tauri::command]
pub async fn count_deleted_incidents(
    db: State<'_, SqlitePool>,
) -> Result<i64, AppError> {
    incidents::count_deleted_incidents(&*db).await
}

#[tauri::command]
pub async fn count_overdue_action_items(
    db: State<'_, SqlitePool>,
) -> Result<i64, AppError> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM action_items ai
         JOIN incidents i ON ai.incident_id = i.id
         WHERE ai.status != 'Done'
         AND ai.due_date IS NOT NULL
         AND ai.due_date < strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         AND i.deleted_at IS NULL"
    )
    .fetch_one(&*db)
    .await
    .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

    Ok(count)
}
