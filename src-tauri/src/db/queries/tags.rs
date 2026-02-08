use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};

pub async fn get_incident_tags(db: &SqlitePool, incident_id: &str) -> AppResult<Vec<String>> {
    let rows = sqlx::query("SELECT tag FROM incident_tags WHERE incident_id = ? ORDER BY tag ASC")
        .bind(incident_id)
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(|r| r.get::<String, _>("tag")).collect())
}

pub async fn set_incident_tags(
    db: &SqlitePool,
    incident_id: &str,
    tags: &[String],
) -> AppResult<Vec<String>> {
    let mut tx = db
        .begin()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    sqlx::query("DELETE FROM incident_tags WHERE incident_id = ?")
        .bind(incident_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    for tag in tags {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            continue;
        }
        sqlx::query("INSERT OR IGNORE INTO incident_tags (incident_id, tag) VALUES (?, ?)")
            .bind(incident_id)
            .bind(trimmed)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    get_incident_tags(db, incident_id).await
}

pub async fn get_all_tags(db: &SqlitePool) -> AppResult<Vec<String>> {
    let rows = sqlx::query("SELECT DISTINCT tag FROM incident_tags ORDER BY tag ASC")
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(|r| r.get::<String, _>("tag")).collect())
}
