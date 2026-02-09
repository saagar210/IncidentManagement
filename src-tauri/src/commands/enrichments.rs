use sqlx::SqlitePool;
use tauri::State;

use base64::Engine;
use sha2::{Digest, Sha256};

use crate::ai::{self, OllamaState};
use crate::db::queries::{enrichment_jobs, incident_enrichments, incidents, postmortems, stakeholder_updates, provenance};
use crate::error::AppError;
use crate::models::stakeholder_update::CreateStakeholderUpdateRequest;
use crate::models::postmortem::{CreatePostmortemRequest, UpdatePostmortemRequest, CreateContributingFactorRequest};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunEnrichmentCmd {
    pub job_type: String,
    pub incident_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcceptEnrichmentCmd {
    pub job_id: String,
}

#[tauri::command]
pub async fn run_incident_enrichment(
    db: State<'_, SqlitePool>,
    ollama: State<'_, OllamaState>,
    req: RunEnrichmentCmd,
) -> Result<enrichment_jobs::EnrichmentJob, AppError> {
    let inc = incidents::get_incident_by_id(&*db, &req.incident_id).await?;

    let input_obj = serde_json::json!({
        "incident_id": inc.id,
        "title": inc.title,
        "severity": inc.severity,
        "impact": inc.impact,
        "status": inc.status,
        "service": inc.service_name,
        "started_at": inc.started_at,
        "detected_at": inc.detected_at,
        "root_cause": inc.root_cause,
        "resolution": inc.resolution,
        "lessons_learned": inc.lessons_learned,
        "notes": inc.notes,
        "reopen_count": inc.reopen_count
    });
    let input_hash = hash_json(&input_obj)?;

    // Some enrichment job types are deterministic (computed) rather than AI-driven.
    let (model_id, prompt_version) = match req.job_type.as_str() {
        "factor_categorization" => ("".to_string(), "computed-v1".to_string()),
        _ => (ollama.primary_model.clone(), "v1".to_string()),
    };

    let mut job = enrichment_jobs::create_job_running(
        &*db,
        &req.job_type,
        "incident",
        &req.incident_id,
        &input_hash,
        &model_id,
        &prompt_version,
    )
    .await?;

    // If AI isn't available, produce deterministic fallback output for some jobs.
    let ai_available = *ollama.available.read().await;

    let output = match req.job_type.as_str() {
        "incident_executive_summary" => {
            if !ai_available {
                Err(AppError::Validation("AI unavailable".into()))
            } else {
                let summary = ai::summarize::generate_summary(
                    &*ollama,
                    &inc.title,
                    &inc.severity,
                    &inc.status,
                    &inc.service_name,
                    &inc.root_cause,
                    &inc.resolution,
                    &inc.notes,
                )
                .await?;
                Ok(serde_json::json!({ "summary": summary }))
            }
        }
        "stakeholder_update" => {
            if !ai_available {
                Err(AppError::Validation("AI unavailable".into()))
            } else {
                let content = ai::stakeholder::generate_stakeholder_update(
                    &*ollama,
                    &inc.title,
                    &inc.severity,
                    &inc.status,
                    &inc.service_name,
                    &inc.impact,
                    &inc.notes,
                )
                .await?;
                Ok(serde_json::json!({ "content": content, "update_type": "status" }))
            }
        }
        "postmortem_draft" => {
            if !ai_available {
                Err(AppError::Validation("AI unavailable".into()))
            } else {
                let factors = postmortems::list_contributing_factors(&*db, &inc.id).await?;
                let factor_lines: Vec<String> = factors
                    .iter()
                    .map(|f| format!("[{}] {}", f.category, f.description))
                    .collect();
                let markdown = ai::postmortem::generate_postmortem_draft(
                    &*ollama,
                    &inc.title,
                    &inc.severity,
                    &inc.service_name,
                    &inc.root_cause,
                    &inc.resolution,
                    &inc.lessons_learned,
                    &factor_lines,
                )
                .await?;
                Ok(serde_json::json!({ "markdown": markdown }))
            }
        }
        "factor_categorization" => {
            // Deterministic fallback: map root_cause into a Process factor if present.
            if inc.root_cause.trim().is_empty() {
                Ok(serde_json::json!({ "factors": [] }))
            } else {
                Ok(serde_json::json!({
                    "factors": [
                        { "category": "Process", "description": inc.root_cause, "is_root": true }
                    ]
                }))
            }
        }
        _ => Err(AppError::Validation(format!("Unknown job_type '{}'", req.job_type))),
    };

    match output {
        Ok(val) => {
            let out_str = serde_json::to_string(&val)
                .map_err(|e| AppError::Report(format!("Failed to serialize enrichment output: {}", e)))?;
            enrichment_jobs::complete_job_success(&*db, &job.id, &out_str).await?;
        }
        Err(e) => {
            enrichment_jobs::complete_job_failure(&*db, &job.id, &format!("{}", e)).await?;
        }
    }

    job = enrichment_jobs::get_job(&*db, &job.id).await?.ok_or_else(|| AppError::Database("Job disappeared".into()))?;
    Ok(job)
}

#[tauri::command]
pub async fn accept_enrichment_job(
    db: State<'_, SqlitePool>,
    req: AcceptEnrichmentCmd,
) -> Result<(), AppError> {
    accept_job(&*db, &req.job_id).await
}

async fn accept_job(db: &SqlitePool, job_id: &str) -> Result<(), AppError> {
    let job = enrichment_jobs::get_job(db, job_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Job '{}' not found", job_id)))?;

    if job.status != "succeeded" {
        return Err(AppError::Validation("Only succeeded jobs can be accepted".into()));
    }

    if job.entity_type != "incident" {
        return Err(AppError::Validation("Only incident jobs are supported".into()));
    }

    let meta = serde_json::json!({
        "job_id": job.id,
        "model_id": job.model_id,
        "prompt_version": job.prompt_version,
        "job_type": job.job_type
    })
    .to_string();

    match job.job_type.as_str() {
        "incident_executive_summary" => {
            let v: serde_json::Value = serde_json::from_str(&job.output_json)
                .map_err(|e| AppError::Report(format!("Invalid job output JSON: {}", e)))?;
            let summary = v.get("summary").and_then(|x| x.as_str()).unwrap_or("").to_string();
            incident_enrichments::upsert_incident_executive_summary(
                db,
                &job.entity_id,
                &summary,
                "ai",
                Some(&job.id),
            )
            .await?;
            provenance::insert_field_provenance(
                db,
                "incident",
                &job.entity_id,
                "executive_summary",
                "ai",
                &job.model_id,
                &job.prompt_version,
                &job.input_hash,
                &meta,
            )
            .await?;
        }
        "stakeholder_update" => {
            let v: serde_json::Value = serde_json::from_str(&job.output_json)
                .map_err(|e| AppError::Report(format!("Invalid job output JSON: {}", e)))?;
            let content = v.get("content").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let update_type = v.get("update_type").and_then(|x| x.as_str()).unwrap_or("status").to_string();
            let id = format!("stu-{}", uuid::Uuid::new_v4());
            let req = CreateStakeholderUpdateRequest {
                incident_id: job.entity_id.clone(),
                content: content.clone(),
                update_type,
                generated_by: "ai".into(),
            };
            req.validate()?;
            let created = stakeholder_updates::create(db, &id, &req).await?;

            provenance::insert_field_provenance(
                db,
                "stakeholder_update",
                &created.id,
                "content",
                "ai",
                &job.model_id,
                &job.prompt_version,
                &job.input_hash,
                &meta,
            )
            .await?;
        }
        "postmortem_draft" => {
            let v: serde_json::Value = serde_json::from_str(&job.output_json)
                .map_err(|e| AppError::Report(format!("Invalid job output JSON: {}", e)))?;
            let markdown = v.get("markdown").and_then(|x| x.as_str()).unwrap_or("").to_string();

            // Ensure postmortem exists.
            let existing = postmortems::get_postmortem_by_incident(db, &job.entity_id).await?;
            if existing.is_none() {
                let create = CreatePostmortemRequest { incident_id: job.entity_id.clone(), template_id: None, content: "{}".into() };
                create.validate()?;
                let _pm = postmortems::create_postmortem(db, &format!("pm-{}", uuid::Uuid::new_v4()), &create).await?;
            }
            let pm = postmortems::get_postmortem_by_incident(db, &job.entity_id).await?
                .ok_or_else(|| AppError::Database("Postmortem missing after create".into()))?;
            let update = UpdatePostmortemRequest { content: Some(serde_json::json!({ "markdown": markdown }).to_string()), status: None, reminder_at: None, no_action_items_justified: None, no_action_items_justification: None };
            update.validate()?;
            postmortems::update_postmortem(db, &pm.id, &update).await?;

            provenance::insert_field_provenance(
                db,
                "postmortem",
                &pm.id,
                "content",
                "ai",
                &job.model_id,
                &job.prompt_version,
                &job.input_hash,
                &meta,
            )
            .await?;
        }
        "factor_categorization" => {
            let v: serde_json::Value = serde_json::from_str(&job.output_json)
                .map_err(|e| AppError::Report(format!("Invalid job output JSON: {}", e)))?;
            let factors = v.get("factors").and_then(|x| x.as_array()).cloned().unwrap_or_default();
            for f in factors {
                let category = f.get("category").and_then(|x| x.as_str()).unwrap_or("Process").to_string();
                let description = f.get("description").and_then(|x| x.as_str()).unwrap_or("").to_string();
                let is_root = f.get("is_root").and_then(|x| x.as_bool()).unwrap_or(false);
                if description.trim().is_empty() {
                    continue;
                }
                let req = CreateContributingFactorRequest { incident_id: job.entity_id.clone(), category, description, is_root };
                req.validate()?;
                postmortems::create_contributing_factor(db, &format!("cf-{}", uuid::Uuid::new_v4()), &req).await?;
            }

            provenance::insert_field_provenance(
                db,
                "incident",
                &job.entity_id,
                "contributing_factors",
                if job.model_id.trim().is_empty() { "computed" } else { "ai" },
                &job.model_id,
                &job.prompt_version,
                &job.input_hash,
                &meta,
            )
            .await?;
        }
        _ => {
            return Err(AppError::Validation(format!("Unsupported accept for job_type '{}'", job.job_type)));
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_incident_enrichment(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<Option<incident_enrichments::IncidentEnrichment>, AppError> {
    incident_enrichments::get_incident_enrichment(&*db, &incident_id).await
}

fn hash_json(v: &serde_json::Value) -> Result<String, AppError> {
    let json = serde_json::to_vec(v)
        .map_err(|e| AppError::Internal(format!("Failed to serialize enrichment input hash: {}", e)))?;
    let mut hasher = Sha256::new();
    hasher.update(&json);
    let digest = hasher.finalize();
    Ok(base64::engine::general_purpose::STANDARD.encode(digest))
}

#[cfg(test)]
mod tests {
    use super::accept_job;
    use crate::db::migrations::run_migrations;
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
    use std::str::FromStr;
    use tempfile::tempdir;

    async fn setup_db() -> sqlx::SqlitePool {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("enrich-tests.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let options = SqliteConnectOptions::from_str(&db_url)
            .expect("sqlite url")
            .journal_mode(SqliteJournalMode::Wal)
            .pragma("foreign_keys", "ON")
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("connect");
        run_migrations(&pool).await.expect("migrations");
        pool
    }

    #[tokio::test]
    async fn accept_executive_summary_job_writes_enrichment_and_provenance() {
        let pool = setup_db().await;

        let service_id: String = sqlx::query_scalar("SELECT id FROM services LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("service");
        let inc_id = format!("inc-{}", uuid::Uuid::new_v4());
        sqlx::query(
            "INSERT INTO incidents (id, title, service_id, severity, impact, status, started_at, detected_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, (strftime('%Y-%m-%dT%H:%M:%SZ','now')), (strftime('%Y-%m-%dT%H:%M:%SZ','now')))",
        )
        .bind(&inc_id)
        .bind("Test Incident")
        .bind(&service_id)
        .bind("High")
        .bind("High")
        .bind("Active")
        .bind("2026-01-01T10:00:00Z")
        .bind("2026-01-01T10:05:00Z")
        .execute(&pool)
        .await
        .expect("insert incident");

        let job_id = format!("enj-{}", uuid::Uuid::new_v4());
        sqlx::query(
            "INSERT INTO enrichment_jobs (id, job_type, entity_type, entity_id, status, input_hash, output_json, model_id, prompt_version, created_at, completed_at)
             VALUES (?, 'incident_executive_summary', 'incident', ?, 'succeeded', 'hash', ?, 'qwen', 'v1', (strftime('%Y-%m-%dT%H:%M:%SZ','now')), (strftime('%Y-%m-%dT%H:%M:%SZ','now')))",
        )
        .bind(&job_id)
        .bind(&inc_id)
        .bind("{\"summary\":\"Executive summary text.\"}")
        .execute(&pool)
        .await
        .expect("insert job");

        accept_job(&pool, &job_id).await.expect("accept");

        let saved: Option<String> = sqlx::query_scalar("SELECT executive_summary FROM incident_enrichments WHERE incident_id = ?")
            .bind(&inc_id)
            .fetch_optional(&pool)
            .await
            .expect("select enrichment");
        assert_eq!(saved.as_deref(), Some("Executive summary text."));

        let prov_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM field_provenance WHERE entity_type = 'incident' AND entity_id = ? AND field_name = 'executive_summary' AND source_type = 'ai'",
        )
        .bind(&inc_id)
        .fetch_one(&pool)
        .await
        .expect("select provenance");
        assert_eq!(prov_count, 1);
    }
}
