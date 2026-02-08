use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::saved_filter::{
    CreateSavedFilterRequest, SavedFilter, UpdateSavedFilterRequest,
};

pub async fn list_saved_filters(db: &SqlitePool) -> AppResult<Vec<SavedFilter>> {
    let rows = sqlx::query("SELECT * FROM saved_filters ORDER BY name ASC")
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_saved_filter).collect())
}

pub async fn get_saved_filter(db: &SqlitePool, id: &str) -> AppResult<SavedFilter> {
    let row = sqlx::query("SELECT * FROM saved_filters WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Saved filter '{}' not found", id)))?;

    Ok(parse_saved_filter(&row))
}

pub async fn create_saved_filter(
    db: &SqlitePool,
    id: &str,
    req: &CreateSavedFilterRequest,
) -> AppResult<SavedFilter> {
    // If this filter is set as default, clear other defaults
    if req.is_default {
        sqlx::query("UPDATE saved_filters SET is_default = 0 WHERE is_default = 1")
            .execute(db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    sqlx::query(
        "INSERT INTO saved_filters (id, name, filters, is_default) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(&req.name)
    .bind(&req.filters)
    .bind(req.is_default)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_saved_filter(db, id).await
}

pub async fn update_saved_filter(
    db: &SqlitePool,
    id: &str,
    req: &UpdateSavedFilterRequest,
) -> AppResult<SavedFilter> {
    let existing = get_saved_filter(db, id).await?;

    let name = req.name.as_ref().unwrap_or(&existing.name);
    let filters = req.filters.as_ref().unwrap_or(&existing.filters);
    let is_default = req.is_default.unwrap_or(existing.is_default);

    // If this filter is being set as default, clear other defaults
    if is_default && !existing.is_default {
        sqlx::query("UPDATE saved_filters SET is_default = 0 WHERE is_default = 1 AND id != ?")
            .bind(id)
            .execute(db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    sqlx::query(
        "UPDATE saved_filters SET name=?, filters=?, is_default=?, updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?",
    )
    .bind(name)
    .bind(filters)
    .bind(is_default)
    .bind(id)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_saved_filter(db, id).await
}

pub async fn delete_saved_filter(db: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM saved_filters WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Saved filter '{}' not found",
            id
        )));
    }
    Ok(())
}

fn parse_saved_filter(row: &sqlx::sqlite::SqliteRow) -> SavedFilter {
    SavedFilter {
        id: row.get("id"),
        name: row.get("name"),
        filters: row.get("filters"),
        is_default: row.get::<bool, _>("is_default"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
