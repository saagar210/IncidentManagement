use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FieldProvenance {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub field_name: String,
    pub source_type: String, // manual | import | computed | ai
    pub source_ref: String,
    pub source_version: String,
    pub input_hash: String,
    pub meta_json: String,
    pub recorded_at: String,
}

pub async fn insert_field_provenance(
    pool: &SqlitePool,
    entity_type: &str,
    entity_id: &str,
    field_name: &str,
    source_type: &str,
    source_ref: &str,
    source_version: &str,
    input_hash: &str,
    meta_json: &str,
) -> AppResult<()> {
    if entity_type.trim().is_empty() || entity_id.trim().is_empty() || field_name.trim().is_empty() {
        return Err(AppError::Validation("Provenance entity_type/entity_id/field_name are required".into()));
    }
    if source_type.trim().is_empty() {
        return Err(AppError::Validation("Provenance source_type is required".into()));
    }

    let id = format!("prv-{}", uuid::Uuid::new_v4());
    sqlx::query(
        "INSERT INTO field_provenance (id, entity_type, entity_id, field_name, source_type, source_ref, source_version, input_hash, meta_json)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(entity_type)
    .bind(entity_id)
    .bind(field_name)
    .bind(source_type)
    .bind(source_ref)
    .bind(source_version)
    .bind(input_hash)
    .bind(meta_json)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

pub async fn list_field_provenance_for_entity(
    pool: &SqlitePool,
    entity_type: &str,
    entity_id: &str,
) -> AppResult<Vec<FieldProvenance>> {
    let rows = sqlx::query(
        "SELECT * FROM field_provenance WHERE entity_type = ? AND entity_id = ? ORDER BY recorded_at DESC",
    )
    .bind(entity_type)
    .bind(entity_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows
        .iter()
        .map(|row| FieldProvenance {
            id: row.get("id"),
            entity_type: row.get("entity_type"),
            entity_id: row.get("entity_id"),
            field_name: row.get("field_name"),
            source_type: row.get("source_type"),
            source_ref: row.get("source_ref"),
            source_version: row.get("source_version"),
            input_hash: row.get("input_hash"),
            meta_json: row.get("meta_json"),
            recorded_at: row.get("recorded_at"),
        })
        .collect())
}
