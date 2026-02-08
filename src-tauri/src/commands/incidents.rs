use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::{incidents, settings};
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
    incidents::insert_incident(&*db, &id, &incident).await
}

#[tauri::command]
pub async fn update_incident(
    db: State<'_, SqlitePool>,
    id: String,
    incident: UpdateIncidentRequest,
) -> Result<Incident, AppError> {
    incidents::update_incident(&*db, &id, &incident).await
}

#[tauri::command]
pub async fn delete_incident(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    incidents::delete_incident(&*db, &id).await
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

// Action Item commands

#[tauri::command]
pub async fn create_action_item(
    db: State<'_, SqlitePool>,
    item: CreateActionItemRequest,
) -> Result<ActionItem, AppError> {
    item.validate()?;
    let id = format!("ai-{}", uuid::Uuid::new_v4());
    incidents::insert_action_item(&*db, &id, &item).await
}

#[tauri::command]
pub async fn update_action_item(
    db: State<'_, SqlitePool>,
    id: String,
    item: UpdateActionItemRequest,
) -> Result<ActionItem, AppError> {
    incidents::update_action_item(&*db, &id, &item).await
}

#[tauri::command]
pub async fn delete_action_item(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    incidents::delete_action_item(&*db, &id).await
}

#[tauri::command]
pub async fn list_action_items(
    db: State<'_, SqlitePool>,
    incident_id: Option<String>,
) -> Result<Vec<ActionItem>, AppError> {
    incidents::list_action_items(&*db, incident_id.as_deref()).await
}
