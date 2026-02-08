use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::stakeholder_update::{CreateStakeholderUpdateRequest, StakeholderUpdate};

pub async fn list_by_incident(
    db: &SqlitePool,
    incident_id: &str,
) -> AppResult<Vec<StakeholderUpdate>> {
    let rows = sqlx::query(
        "SELECT * FROM stakeholder_updates WHERE incident_id = ? ORDER BY created_at DESC",
    )
    .bind(incident_id)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_stakeholder_update).collect())
}

pub async fn create(
    db: &SqlitePool,
    id: &str,
    req: &CreateStakeholderUpdateRequest,
) -> AppResult<StakeholderUpdate> {
    sqlx::query(
        "INSERT INTO stakeholder_updates (id, incident_id, content, update_type, generated_by) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(&req.incident_id)
    .bind(&req.content)
    .bind(&req.update_type)
    .bind(&req.generated_by)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let row = sqlx::query("SELECT * FROM stakeholder_updates WHERE id = ?")
        .bind(id)
        .fetch_one(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(parse_stakeholder_update(&row))
}

pub async fn delete(db: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM stakeholder_updates WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Stakeholder update '{}' not found",
            id
        )));
    }
    Ok(())
}

fn parse_stakeholder_update(row: &sqlx::sqlite::SqliteRow) -> StakeholderUpdate {
    StakeholderUpdate {
        id: row.get("id"),
        incident_id: row.get("incident_id"),
        content: row.get("content"),
        update_type: row.get("update_type"),
        generated_by: row.get("generated_by"),
        created_at: row.get("created_at"),
    }
}
