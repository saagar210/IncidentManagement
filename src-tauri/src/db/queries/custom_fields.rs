use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::custom_field::{
    CreateCustomFieldRequest, CustomFieldDefinition, CustomFieldValue, UpdateCustomFieldRequest,
};

pub async fn list_custom_fields(db: &SqlitePool) -> AppResult<Vec<CustomFieldDefinition>> {
    let rows = sqlx::query("SELECT * FROM custom_field_definitions ORDER BY display_order ASC, name ASC")
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_field_def).collect())
}

pub async fn create_custom_field(
    db: &SqlitePool,
    id: &str,
    req: &CreateCustomFieldRequest,
) -> AppResult<CustomFieldDefinition> {
    sqlx::query(
        "INSERT INTO custom_field_definitions (id, name, field_type, options, display_order) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(id)
    .bind(&req.name)
    .bind(&req.field_type)
    .bind(&req.options)
    .bind(req.display_order)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_custom_field(db, id).await
}

pub async fn get_custom_field(db: &SqlitePool, id: &str) -> AppResult<CustomFieldDefinition> {
    let row = sqlx::query("SELECT * FROM custom_field_definitions WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Custom field '{}' not found", id)))?;

    Ok(parse_field_def(&row))
}

pub async fn update_custom_field(
    db: &SqlitePool,
    id: &str,
    req: &UpdateCustomFieldRequest,
) -> AppResult<CustomFieldDefinition> {
    let existing = get_custom_field(db, id).await?;

    let name = req.name.as_ref().unwrap_or(&existing.name);
    let field_type = req.field_type.as_ref().unwrap_or(&existing.field_type);
    let options = req.options.as_ref().unwrap_or(&existing.options);
    let display_order = req.display_order.unwrap_or(existing.display_order);

    sqlx::query(
        "UPDATE custom_field_definitions SET name=?, field_type=?, options=?, display_order=?, updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?"
    )
    .bind(name)
    .bind(field_type)
    .bind(options)
    .bind(display_order)
    .bind(id)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_custom_field(db, id).await
}

pub async fn delete_custom_field(db: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM custom_field_definitions WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Custom field '{}' not found", id)));
    }
    Ok(())
}

pub async fn get_incident_custom_fields(
    db: &SqlitePool,
    incident_id: &str,
) -> AppResult<Vec<CustomFieldValue>> {
    let rows = sqlx::query("SELECT * FROM custom_field_values WHERE incident_id = ?")
        .bind(incident_id)
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows
        .iter()
        .map(|r| CustomFieldValue {
            incident_id: r.get("incident_id"),
            field_id: r.get("field_id"),
            value: r.get("value"),
        })
        .collect())
}

pub async fn set_incident_custom_fields(
    db: &SqlitePool,
    incident_id: &str,
    values: &[CustomFieldValue],
) -> AppResult<Vec<CustomFieldValue>> {
    let mut tx = db
        .begin()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    sqlx::query("DELETE FROM custom_field_values WHERE incident_id = ?")
        .bind(incident_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    for v in values {
        sqlx::query(
            "INSERT INTO custom_field_values (incident_id, field_id, value) VALUES (?, ?, ?)"
        )
        .bind(incident_id)
        .bind(&v.field_id)
        .bind(&v.value)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    get_incident_custom_fields(db, incident_id).await
}

fn parse_field_def(row: &sqlx::sqlite::SqliteRow) -> CustomFieldDefinition {
    CustomFieldDefinition {
        id: row.get("id"),
        name: row.get("name"),
        field_type: row.get("field_type"),
        options: row.get::<Option<String>, _>("options").unwrap_or_default(),
        display_order: row.get::<Option<i64>, _>("display_order").unwrap_or(0),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
