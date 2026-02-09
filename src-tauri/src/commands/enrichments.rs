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

fn incident_input_json(inc: &crate::models::incident::Incident) -> serde_json::Value {
    serde_json::json!({
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
    })
}

fn enrichment_model_and_prompt(
    ollama: &OllamaState,
    job_type: &str,
) -> (String, String) {
    match job_type {
        "factor_categorization" => ("".to_string(), "computed-v1".to_string()),
        _ => (ollama.primary_model.clone(), "v1".to_string()),
    }
}

async fn output_incident_executive_summary(
    ollama: &OllamaState,
    inc: &crate::models::incident::Incident,
    ai_available: bool,
) -> Result<serde_json::Value, AppError> {
    if !ai_available {
        return Err(AppError::Validation("AI unavailable".into()));
    }
    let summary = ai::summarize::generate_summary(
        ollama,
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

async fn output_stakeholder_update(
    ollama: &OllamaState,
    inc: &crate::models::incident::Incident,
    ai_available: bool,
) -> Result<serde_json::Value, AppError> {
    if !ai_available {
        return Err(AppError::Validation("AI unavailable".into()));
    }
    let content = ai::stakeholder::generate_stakeholder_update(
        ollama,
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

async fn output_postmortem_draft(
    db: &SqlitePool,
    ollama: &OllamaState,
    inc: &crate::models::incident::Incident,
    ai_available: bool,
) -> Result<serde_json::Value, AppError> {
    if !ai_available {
        return Err(AppError::Validation("AI unavailable".into()));
    }
    let factors = postmortems::list_contributing_factors(db, &inc.id).await?;
    let factor_lines: Vec<String> = factors
        .iter()
        .map(|f| format!("[{}] {}", f.category, f.description))
        .collect();
    let markdown = ai::postmortem::generate_postmortem_draft(
        ollama,
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

fn output_factor_categorization(
    inc: &crate::models::incident::Incident,
) -> Result<serde_json::Value, AppError> {
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

async fn compute_enrichment_output(
    db: &SqlitePool,
    ollama: &OllamaState,
    inc: &crate::models::incident::Incident,
    job_type: &str,
    ai_available: bool,
) -> Result<serde_json::Value, AppError> {
    match job_type {
        "incident_executive_summary" => output_incident_executive_summary(ollama, inc, ai_available).await,
        "stakeholder_update" => output_stakeholder_update(ollama, inc, ai_available).await,
        "postmortem_draft" => output_postmortem_draft(db, ollama, inc, ai_available).await,
        "factor_categorization" => output_factor_categorization(inc),
        _ => Err(AppError::Validation(format!("Unknown job_type '{}'", job_type))),
    }
}

async fn complete_job_from_output(
    db: &SqlitePool,
    job_id: &str,
    output: Result<serde_json::Value, AppError>,
) -> Result<(), AppError> {
    match output {
        Ok(val) => {
            let out_str = serde_json::to_string(&val)
                .map_err(|e| AppError::Report(format!("Failed to serialize enrichment output: {}", e)))?;
            enrichment_jobs::complete_job_success(db, job_id, &out_str).await?;
        }
        Err(e) => {
            enrichment_jobs::complete_job_failure(db, job_id, &format!("{}", e)).await?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn run_incident_enrichment(
    db: State<'_, SqlitePool>,
    ollama: State<'_, OllamaState>,
    req: RunEnrichmentCmd,
) -> Result<enrichment_jobs::EnrichmentJob, AppError> {
    let inc = incidents::get_incident_by_id(&*db, &req.incident_id).await?;

    let input_obj = incident_input_json(&inc);
    let input_hash = hash_json(&input_obj)?;

    let (model_id, prompt_version) = enrichment_model_and_prompt(&ollama, &req.job_type);

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

    let output = compute_enrichment_output(&*db, &*ollama, &inc, &req.job_type, ai_available).await;
    complete_job_from_output(&*db, &job.id, output).await?;

    job = enrichment_jobs::get_job(&*db, &job.id).await?.ok_or_else(|| AppError::Database("Job disappeared".into()))?;
    Ok(job)
}

async fn accept_executive_summary(
    db: &SqlitePool,
    job: &enrichment_jobs::EnrichmentJob,
    meta: &str,
) -> Result<(), AppError> {
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
        &provenance::FieldProvenanceInsert {
            entity_type: "incident",
            entity_id: &job.entity_id,
            field_name: "executive_summary",
            source_type: "ai",
            source_ref: &job.model_id,
            source_version: &job.prompt_version,
            input_hash: &job.input_hash,
            meta_json: meta,
        },
    )
    .await?;
    Ok(())
}

async fn accept_stakeholder(
    db: &SqlitePool,
    job: &enrichment_jobs::EnrichmentJob,
    meta: &str,
) -> Result<(), AppError> {
    let v: serde_json::Value = serde_json::from_str(&job.output_json)
        .map_err(|e| AppError::Report(format!("Invalid job output JSON: {}", e)))?;
    let content = v.get("content").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let update_type = v
        .get("update_type")
        .and_then(|x| x.as_str())
        .unwrap_or("status")
        .to_string();
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
        &provenance::FieldProvenanceInsert {
            entity_type: "stakeholder_update",
            entity_id: &created.id,
            field_name: "content",
            source_type: "ai",
            source_ref: &job.model_id,
            source_version: &job.prompt_version,
            input_hash: &job.input_hash,
            meta_json: meta,
        },
    )
    .await?;
    Ok(())
}

async fn ensure_postmortem_exists(
    db: &SqlitePool,
    incident_id: &str,
) -> Result<crate::models::postmortem::Postmortem, AppError> {
    let existing = postmortems::get_postmortem_by_incident(db, incident_id).await?;
    if existing.is_none() {
        let create = CreatePostmortemRequest {
            incident_id: incident_id.to_string(),
            template_id: None,
            content: "{}".into(),
        };
        create.validate()?;
        let _pm = postmortems::create_postmortem(
            db,
            &format!("pm-{}", uuid::Uuid::new_v4()),
            &create,
        )
        .await?;
    }
    postmortems::get_postmortem_by_incident(db, incident_id)
        .await?
        .ok_or_else(|| AppError::Database("Postmortem missing after create".into()))
}

async fn accept_postmortem(
    db: &SqlitePool,
    job: &enrichment_jobs::EnrichmentJob,
    meta: &str,
) -> Result<(), AppError> {
    let v: serde_json::Value = serde_json::from_str(&job.output_json)
        .map_err(|e| AppError::Report(format!("Invalid job output JSON: {}", e)))?;
    let markdown = v.get("markdown").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let pm = ensure_postmortem_exists(db, &job.entity_id).await?;
    let update = UpdatePostmortemRequest {
        content: Some(serde_json::json!({ "markdown": markdown }).to_string()),
        status: None,
        reminder_at: None,
        no_action_items_justified: None,
        no_action_items_justification: None,
    };
    update.validate()?;
    postmortems::update_postmortem(db, &pm.id, &update).await?;
    provenance::insert_field_provenance(
        db,
        &provenance::FieldProvenanceInsert {
            entity_type: "postmortem",
            entity_id: &pm.id,
            field_name: "content",
            source_type: "ai",
            source_ref: &job.model_id,
            source_version: &job.prompt_version,
            input_hash: &job.input_hash,
            meta_json: meta,
        },
    )
    .await?;
    Ok(())
}

fn parse_factor(v: &serde_json::Value, incident_id: &str) -> Option<CreateContributingFactorRequest> {
    let description = v.get("description").and_then(|x| x.as_str()).unwrap_or("").to_string();
    if description.trim().is_empty() {
        return None;
    }
    let category = v.get("category").and_then(|x| x.as_str()).unwrap_or("Process").to_string();
    let is_root = v.get("is_root").and_then(|x| x.as_bool()).unwrap_or(false);
    let req = CreateContributingFactorRequest {
        incident_id: incident_id.to_string(),
        category,
        description,
        is_root,
    };
    Some(req)
}

async fn accept_factors(
    db: &SqlitePool,
    job: &enrichment_jobs::EnrichmentJob,
    meta: &str,
) -> Result<(), AppError> {
    let v: serde_json::Value = serde_json::from_str(&job.output_json)
        .map_err(|e| AppError::Report(format!("Invalid job output JSON: {}", e)))?;
    let factors = v
        .get("factors")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    for f in factors {
        let Some(req) = parse_factor(&f, &job.entity_id) else {
            continue;
        };
        req.validate()?;
        postmortems::create_contributing_factor(db, &format!("cf-{}", uuid::Uuid::new_v4()), &req)
            .await?;
    }

    let source_type = if job.model_id.trim().is_empty() { "computed" } else { "ai" };
    provenance::insert_field_provenance(
        db,
        &provenance::FieldProvenanceInsert {
            entity_type: "incident",
            entity_id: &job.entity_id,
            field_name: "contributing_factors",
            source_type,
            source_ref: &job.model_id,
            source_version: &job.prompt_version,
            input_hash: &job.input_hash,
            meta_json: meta,
        },
    )
    .await?;
    Ok(())
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
        "incident_executive_summary" => accept_executive_summary(db, &job, &meta).await?,
        "stakeholder_update" => accept_stakeholder(db, &job, &meta).await?,
        "postmortem_draft" => accept_postmortem(db, &job, &meta).await?,
        "factor_categorization" => accept_factors(db, &job, &meta).await?,
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

#[tauri::command]
pub async fn accept_enrichment_job(
    db: State<'_, SqlitePool>,
    req: AcceptEnrichmentCmd,
) -> Result<(), AppError> {
    accept_job(&*db, &req.job_id).await?;
    Ok(())
}
