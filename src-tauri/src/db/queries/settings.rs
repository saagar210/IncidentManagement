use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::quarter::{QuarterConfig, UpsertQuarterRequest};

pub async fn get_quarter_configs(db: &SqlitePool) -> AppResult<Vec<QuarterConfig>> {
    let rows = sqlx::query(
        "SELECT * FROM quarter_config ORDER BY fiscal_year DESC, quarter_number DESC"
    )
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_quarter).collect())
}

pub async fn get_quarter_by_id(db: &SqlitePool, id: &str) -> AppResult<QuarterConfig> {
    let row = sqlx::query("SELECT * FROM quarter_config WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Quarter '{}' not found", id)))?;

    Ok(parse_quarter(&row))
}

pub async fn get_previous_quarter(
    db: &SqlitePool,
    fiscal_year: i64,
    quarter_number: i64,
) -> AppResult<Option<QuarterConfig>> {
    let (prev_fy, prev_q) = if quarter_number == 1 {
        (fiscal_year - 1, 4)
    } else {
        (fiscal_year, quarter_number - 1)
    };

    let row = sqlx::query(
        "SELECT * FROM quarter_config WHERE fiscal_year = ? AND quarter_number = ?"
    )
    .bind(prev_fy)
    .bind(prev_q)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(row.map(|r| parse_quarter(&r)))
}

pub async fn upsert_quarter(db: &SqlitePool, req: &UpsertQuarterRequest) -> AppResult<QuarterConfig> {
    let id = req
        .id
        .clone()
        .unwrap_or_else(|| format!("fy{}-q{}", req.fiscal_year, req.quarter_number));

    sqlx::query(
        "INSERT INTO quarter_config (id, fiscal_year, quarter_number, start_date, end_date, label) VALUES (?, ?, ?, ?, ?, ?) ON CONFLICT(fiscal_year, quarter_number) DO UPDATE SET start_date = excluded.start_date, end_date = excluded.end_date, label = excluded.label"
    )
    .bind(&id)
    .bind(req.fiscal_year)
    .bind(req.quarter_number)
    .bind(&req.start_date)
    .bind(&req.end_date)
    .bind(&req.label)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_quarter_by_id(db, &id).await
}

pub async fn delete_quarter(db: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM quarter_config WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Quarter '{}' not found", id)));
    }
    Ok(())
}

pub async fn get_setting(db: &SqlitePool, key: &str) -> AppResult<Option<String>> {
    let row = sqlx::query("SELECT value FROM app_settings WHERE key = ?")
        .bind(key)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(row.map(|r| r.get::<String, _>("value")))
}

pub async fn set_setting(db: &SqlitePool, key: &str, value: &str) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value"
    )
    .bind(key)
    .bind(value)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

fn parse_quarter(row: &sqlx::sqlite::SqliteRow) -> QuarterConfig {
    QuarterConfig {
        id: row.get("id"),
        fiscal_year: row.get("fiscal_year"),
        quarter_number: row.get("quarter_number"),
        start_date: row.get("start_date"),
        end_date: row.get("end_date"),
        label: row.get("label"),
        created_at: row.get("created_at"),
    }
}
