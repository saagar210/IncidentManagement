use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuarterOverride {
    pub id: String,
    pub quarter_id: String,
    pub rule_key: String,
    pub incident_id: String,
    pub reason: String,
    pub approved_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuarterFinalization {
    pub quarter_id: String,
    pub finalized_at: String,
    pub finalized_by: String,
    pub snapshot_id: String,
    pub inputs_hash: String,
    pub notes: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuarterSnapshot {
    pub id: String,
    pub quarter_id: String,
    pub schema_version: i64,
    pub inputs_hash: String,
    pub snapshot_json: String,
    pub created_at: String,
}

pub async fn list_overrides_for_quarter(pool: &SqlitePool, quarter_id: &str) -> AppResult<Vec<QuarterOverride>> {
    let rows = sqlx::query(
        "SELECT * FROM quarter_readiness_overrides WHERE quarter_id = ? ORDER BY created_at DESC",
    )
    .bind(quarter_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_override).collect())
}

pub async fn upsert_override(
    pool: &SqlitePool,
    quarter_id: &str,
    rule_key: &str,
    incident_id: &str,
    reason: &str,
    approved_by: &str,
) -> AppResult<QuarterOverride> {
    if quarter_id.trim().is_empty() || rule_key.trim().is_empty() || incident_id.trim().is_empty() {
        return Err(AppError::Validation("quarter_id, rule_key, incident_id are required".into()));
    }
    if reason.trim().is_empty() {
        return Err(AppError::Validation("Override reason is required".into()));
    }

    let existing_id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM quarter_readiness_overrides WHERE quarter_id = ? AND rule_key = ? AND incident_id = ?",
    )
    .bind(quarter_id)
    .bind(rule_key)
    .bind(incident_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let exists = existing_id.is_some();
    let id = existing_id.unwrap_or_else(|| format!("qov-{}", uuid::Uuid::new_v4()));
    if exists {
        sqlx::query(
            "UPDATE quarter_readiness_overrides SET reason = ?, approved_by = ? WHERE id = ?",
        )
        .bind(reason.trim())
        .bind(approved_by)
        .bind(&id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    } else {
        sqlx::query(
            "INSERT INTO quarter_readiness_overrides (id, quarter_id, rule_key, incident_id, reason, approved_by) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(quarter_id)
        .bind(rule_key)
        .bind(incident_id)
        .bind(reason.trim())
        .bind(approved_by)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }

    let row = sqlx::query("SELECT * FROM quarter_readiness_overrides WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(parse_override(&row))
}

pub async fn delete_override(pool: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM quarter_readiness_overrides WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Override '{}' not found", id)));
    }
    Ok(())
}

pub async fn get_finalization(pool: &SqlitePool, quarter_id: &str) -> AppResult<Option<QuarterFinalization>> {
    let row = sqlx::query("SELECT * FROM quarter_finalizations WHERE quarter_id = ?")
        .bind(quarter_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(row.map(|r| QuarterFinalization {
        quarter_id: r.get("quarter_id"),
        finalized_at: r.get("finalized_at"),
        finalized_by: r.get("finalized_by"),
        snapshot_id: r.get("snapshot_id"),
        inputs_hash: r.get("inputs_hash"),
        notes: r.get("notes"),
    }))
}

pub async fn upsert_snapshot(
    pool: &SqlitePool,
    quarter_id: &str,
    inputs_hash: &str,
    snapshot_json: &str,
) -> AppResult<QuarterSnapshot> {
    let existing_id: Option<String> = sqlx::query_scalar("SELECT id FROM quarter_snapshots WHERE quarter_id = ?")
        .bind(quarter_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let exists = existing_id.is_some();
    let id = existing_id.unwrap_or_else(|| format!("qsn-{}", uuid::Uuid::new_v4()));
    if exists {
        sqlx::query(
            "UPDATE quarter_snapshots SET inputs_hash = ?, snapshot_json = ?, created_at = (strftime('%Y-%m-%dT%H:%M:%SZ','now')) WHERE id = ?",
        )
        .bind(inputs_hash)
        .bind(snapshot_json)
        .bind(&id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    } else {
        sqlx::query(
            "INSERT INTO quarter_snapshots (id, quarter_id, inputs_hash, snapshot_json) VALUES (?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(quarter_id)
        .bind(inputs_hash)
        .bind(snapshot_json)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }

    let row = sqlx::query("SELECT * FROM quarter_snapshots WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(QuarterSnapshot {
        id: row.get("id"),
        quarter_id: row.get("quarter_id"),
        schema_version: row.get("schema_version"),
        inputs_hash: row.get("inputs_hash"),
        snapshot_json: row.get("snapshot_json"),
        created_at: row.get("created_at"),
    })
}

pub async fn finalize_quarter(
    pool: &SqlitePool,
    quarter_id: &str,
    finalized_by: &str,
    snapshot_id: &str,
    inputs_hash: &str,
    notes: &str,
) -> AppResult<QuarterFinalization> {
    sqlx::query(
        "INSERT OR REPLACE INTO quarter_finalizations (quarter_id, finalized_at, finalized_by, snapshot_id, inputs_hash, notes)
         VALUES (?, (strftime('%Y-%m-%dT%H:%M:%SZ','now')), ?, ?, ?, ?)",
    )
    .bind(quarter_id)
    .bind(finalized_by)
    .bind(snapshot_id)
    .bind(inputs_hash)
    .bind(notes)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let row = sqlx::query("SELECT * FROM quarter_finalizations WHERE quarter_id = ?")
        .bind(quarter_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(QuarterFinalization {
        quarter_id: row.get("quarter_id"),
        finalized_at: row.get("finalized_at"),
        finalized_by: row.get("finalized_by"),
        snapshot_id: row.get("snapshot_id"),
        inputs_hash: row.get("inputs_hash"),
        notes: row.get("notes"),
    })
}

pub async fn unfinalize_quarter(pool: &SqlitePool, quarter_id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM quarter_finalizations WHERE quarter_id = ?")
        .bind(quarter_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Quarter '{}' is not finalized", quarter_id)));
    }
    Ok(())
}

pub async fn get_snapshot_for_quarter(pool: &SqlitePool, quarter_id: &str) -> AppResult<Option<QuarterSnapshot>> {
    let row = sqlx::query("SELECT * FROM quarter_snapshots WHERE quarter_id = ?")
        .bind(quarter_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(row.map(|r| QuarterSnapshot {
        id: r.get("id"),
        quarter_id: r.get("quarter_id"),
        schema_version: r.get("schema_version"),
        inputs_hash: r.get("inputs_hash"),
        snapshot_json: r.get("snapshot_json"),
        created_at: r.get("created_at"),
    }))
}

fn parse_override(row: &sqlx::sqlite::SqliteRow) -> QuarterOverride {
    QuarterOverride {
        id: row.get("id"),
        quarter_id: row.get("quarter_id"),
        rule_key: row.get("rule_key"),
        incident_id: row.get("incident_id"),
        reason: row.get("reason"),
        approved_by: row.get("approved_by"),
        created_at: row.get("created_at"),
    }
}
