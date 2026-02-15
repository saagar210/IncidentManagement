use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::provenance;
use crate::error::AppError;

#[tauri::command]
pub async fn list_field_provenance_for_entity(
    db: State<'_, SqlitePool>,
    entity_type: String,
    entity_id: String,
) -> Result<Vec<provenance::FieldProvenance>, AppError> {
    provenance::list_field_provenance_for_entity(&*db, &entity_type, &entity_id).await
}

