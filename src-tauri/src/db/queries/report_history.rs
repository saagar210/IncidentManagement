use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::report_history::ReportHistory;

pub async fn list_report_history(db: &SqlitePool) -> AppResult<Vec<ReportHistory>> {
    let records = sqlx::query_as::<_, ReportHistory>(
        "SELECT id, title, quarter_id, format, generated_at, file_path, config_json, file_size_bytes, inputs_hash, report_version, quarter_finalized_at
         FROM report_history
         ORDER BY generated_at DESC"
    )
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(records)
}

pub async fn insert_report_history(
    db: &SqlitePool,
    title: &str,
    quarter_id: Option<&str>,
    format: &str,
    file_path: &str,
    config_json: &str,
    file_size_bytes: i64,
    inputs_hash: &str,
    report_version: i64,
    quarter_finalized_at: Option<&str>,
) -> AppResult<ReportHistory> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO report_history (id, title, quarter_id, format, file_path, config_json, file_size_bytes, inputs_hash, report_version, quarter_finalized_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(title)
    .bind(quarter_id)
    .bind(format)
    .bind(file_path)
    .bind(config_json)
    .bind(file_size_bytes)
    .bind(inputs_hash)
    .bind(report_version)
    .bind(quarter_finalized_at)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let record = sqlx::query_as::<_, ReportHistory>(
        "SELECT id, title, quarter_id, format, generated_at, file_path, config_json, file_size_bytes, inputs_hash, report_version, quarter_finalized_at
         FROM report_history WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(record)
}

pub async fn delete_report_history(db: &SqlitePool, id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM report_history WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}
