use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IncidentEnrichment {
    pub incident_id: String,
    pub executive_summary: String,
    pub last_job_id: Option<String>,
    pub generated_by: String,
    pub updated_at: String,
}

pub async fn get_incident_enrichment(pool: &SqlitePool, incident_id: &str) -> AppResult<Option<IncidentEnrichment>> {
    let row = sqlx::query("SELECT * FROM incident_enrichments WHERE incident_id = ?")
        .bind(incident_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(row.map(|r| parse_row(&r)))
}

pub async fn upsert_incident_executive_summary(
    pool: &SqlitePool,
    incident_id: &str,
    executive_summary: &str,
    generated_by: &str,
    last_job_id: Option<&str>,
) -> AppResult<()> {
    if incident_id.trim().is_empty() {
        return Err(AppError::Validation("Incident ID is required".into()));
    }
    if executive_summary.len() > 50_000 {
        return Err(AppError::Validation("Executive summary too long (max 50000 chars)".into()));
    }

    sqlx::query(
        r#"
        INSERT INTO incident_enrichments (incident_id, executive_summary, last_job_id, generated_by, updated_at)
        VALUES (?, ?, ?, ?, (strftime('%Y-%m-%dT%H:%M:%SZ','now')))
        ON CONFLICT(incident_id) DO UPDATE SET
          executive_summary = excluded.executive_summary,
          last_job_id = excluded.last_job_id,
          generated_by = excluded.generated_by,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(incident_id)
    .bind(executive_summary)
    .bind(last_job_id)
    .bind(generated_by)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

fn parse_row(row: &sqlx::sqlite::SqliteRow) -> IncidentEnrichment {
    IncidentEnrichment {
        incident_id: row.get("incident_id"),
        executive_summary: row.get("executive_summary"),
        last_job_id: row.get("last_job_id"),
        generated_by: row.get("generated_by"),
        updated_at: row.get("updated_at"),
    }
}
