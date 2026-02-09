use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnrichmentJob {
    pub id: String,
    pub job_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub status: String,
    pub input_hash: String,
    pub output_json: String,
    pub model_id: String,
    pub prompt_version: String,
    pub error: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

pub async fn create_job_running(
    pool: &SqlitePool,
    job_type: &str,
    entity_type: &str,
    entity_id: &str,
    input_hash: &str,
    model_id: &str,
    prompt_version: &str,
) -> AppResult<EnrichmentJob> {
    if job_type.trim().is_empty() || entity_type.trim().is_empty() || entity_id.trim().is_empty() {
        return Err(AppError::Validation("job_type/entity_type/entity_id are required".into()));
    }
    if input_hash.trim().is_empty() {
        return Err(AppError::Validation("input_hash is required".into()));
    }

    let id = format!("enj-{}", uuid::Uuid::new_v4());
    sqlx::query(
        "INSERT INTO enrichment_jobs (id, job_type, entity_type, entity_id, status, input_hash, model_id, prompt_version)
         VALUES (?, ?, ?, ?, 'running', ?, ?, ?)",
    )
    .bind(&id)
    .bind(job_type)
    .bind(entity_type)
    .bind(entity_id)
    .bind(input_hash)
    .bind(model_id)
    .bind(prompt_version)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_job(pool, &id).await?.ok_or_else(|| AppError::Database("Failed to load created job".into()))
}

pub async fn complete_job_success(
    pool: &SqlitePool,
    id: &str,
    output_json: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE enrichment_jobs
         SET status = 'succeeded', output_json = ?, completed_at = (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
         WHERE id = ?",
    )
    .bind(output_json)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub async fn complete_job_failure(pool: &SqlitePool, id: &str, error: &str) -> AppResult<()> {
    sqlx::query(
        "UPDATE enrichment_jobs
         SET status = 'failed', error = ?, completed_at = (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
         WHERE id = ?",
    )
    .bind(error)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub async fn get_job(pool: &SqlitePool, id: &str) -> AppResult<Option<EnrichmentJob>> {
    let row = sqlx::query("SELECT * FROM enrichment_jobs WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(row.map(|r| parse_row(&r)))
}

pub async fn list_jobs_for_entity(pool: &SqlitePool, entity_type: &str, entity_id: &str) -> AppResult<Vec<EnrichmentJob>> {
    let rows = sqlx::query(
        "SELECT * FROM enrichment_jobs WHERE entity_type = ? AND entity_id = ? ORDER BY created_at DESC",
    )
    .bind(entity_type)
    .bind(entity_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(rows.iter().map(parse_row).collect())
}

fn parse_row(row: &sqlx::sqlite::SqliteRow) -> EnrichmentJob {
    EnrichmentJob {
        id: row.get("id"),
        job_type: row.get("job_type"),
        entity_type: row.get("entity_type"),
        entity_id: row.get("entity_id"),
        status: row.get("status"),
        input_hash: row.get("input_hash"),
        output_json: row.get("output_json"),
        model_id: row.get("model_id"),
        prompt_version: row.get("prompt_version"),
        error: row.get("error"),
        created_at: row.get("created_at"),
        completed_at: row.get("completed_at"),
    }
}
