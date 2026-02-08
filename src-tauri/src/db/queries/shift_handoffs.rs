use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::shift_handoff::{CreateShiftHandoffRequest, ShiftHandoff};

pub async fn list_recent(db: &SqlitePool, limit: i64) -> AppResult<Vec<ShiftHandoff>> {
    let rows =
        sqlx::query("SELECT * FROM shift_handoffs ORDER BY created_at DESC LIMIT ?")
            .bind(limit)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_shift_handoff).collect())
}

pub async fn create(
    db: &SqlitePool,
    id: &str,
    req: &CreateShiftHandoffRequest,
) -> AppResult<ShiftHandoff> {
    sqlx::query(
        "INSERT INTO shift_handoffs (id, shift_end_time, content, created_by) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(&req.shift_end_time)
    .bind(&req.content)
    .bind(&req.created_by)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let row = sqlx::query("SELECT * FROM shift_handoffs WHERE id = ?")
        .bind(id)
        .fetch_one(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(parse_shift_handoff(&row))
}

pub async fn delete(db: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM shift_handoffs WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Shift handoff '{}' not found",
            id
        )));
    }
    Ok(())
}

fn parse_shift_handoff(row: &sqlx::sqlite::SqliteRow) -> ShiftHandoff {
    ShiftHandoff {
        id: row.get("id"),
        shift_end_time: row.get("shift_end_time"),
        content: row.get("content"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
    }
}
