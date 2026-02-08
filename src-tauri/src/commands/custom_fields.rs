use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::custom_fields;
use crate::error::AppError;
use crate::models::custom_field::{
    CreateCustomFieldRequest, CustomFieldDefinition, CustomFieldValue, UpdateCustomFieldRequest,
};

#[tauri::command]
pub async fn list_custom_fields(
    db: State<'_, SqlitePool>,
) -> Result<Vec<CustomFieldDefinition>, AppError> {
    custom_fields::list_custom_fields(&*db).await
}

#[tauri::command]
pub async fn create_custom_field(
    db: State<'_, SqlitePool>,
    field: CreateCustomFieldRequest,
) -> Result<CustomFieldDefinition, AppError> {
    field.validate()?;
    let id = format!("cf-{}", uuid::Uuid::new_v4());
    custom_fields::create_custom_field(&*db, &id, &field).await
}

#[tauri::command]
pub async fn update_custom_field(
    db: State<'_, SqlitePool>,
    id: String,
    field: UpdateCustomFieldRequest,
) -> Result<CustomFieldDefinition, AppError> {
    field.validate()?;
    custom_fields::update_custom_field(&*db, &id, &field).await
}

#[tauri::command]
pub async fn delete_custom_field(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    custom_fields::delete_custom_field(&*db, &id).await
}

#[tauri::command]
pub async fn get_incident_custom_fields(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<Vec<CustomFieldValue>, AppError> {
    custom_fields::get_incident_custom_fields(&*db, &incident_id).await
}

#[tauri::command]
pub async fn set_incident_custom_fields(
    db: State<'_, SqlitePool>,
    incident_id: String,
    values: Vec<CustomFieldValue>,
) -> Result<Vec<CustomFieldValue>, AppError> {
    custom_fields::set_incident_custom_fields(&*db, &incident_id, &values).await
}
