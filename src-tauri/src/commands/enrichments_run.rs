use base64::Engine;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use crate::ai::{self, OllamaState};
use crate::db::queries::{enrichment_jobs, incidents, postmortems};
use crate::error::AppError;

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

fn enrichment_model_and_prompt(ollama: &OllamaState, job_type: &str) -> (String, String) {
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

fn output_factor_categorization(inc: &crate::models::incident::Incident) -> serde_json::Value {
    // Deterministic fallback: map root_cause into a Process factor if present.
    if inc.root_cause.trim().is_empty() {
        serde_json::json!({ "factors": [] })
    } else {
        serde_json::json!({
            "factors": [
                { "category": "Process", "description": inc.root_cause, "is_root": true }
            ]
        })
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
        "factor_categorization" => Ok(output_factor_categorization(inc)),
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
            let out_str = serde_json::to_string(&val).map_err(|e| {
                AppError::Report(format!("Failed to serialize enrichment output: {}", e))
            })?;
            enrichment_jobs::complete_job_success(db, job_id, &out_str).await?;
        }
        Err(e) => {
            enrichment_jobs::complete_job_failure(db, job_id, &format!("{}", e)).await?;
        }
    }
    Ok(())
}

fn hash_json(v: &serde_json::Value) -> Result<String, AppError> {
    let json = serde_json::to_vec(v)
        .map_err(|e| AppError::Internal(format!("Failed to serialize enrichment input hash: {}", e)))?;
    let mut hasher = Sha256::new();
    hasher.update(&json);
    let digest = hasher.finalize();
    Ok(base64::engine::general_purpose::STANDARD.encode(digest))
}

pub(crate) async fn run_incident_enrichment(
    db: &SqlitePool,
    ollama: &OllamaState,
    job_type: &str,
    incident_id: &str,
) -> Result<enrichment_jobs::EnrichmentJob, AppError> {
    let inc = incidents::get_incident_by_id(db, incident_id).await?;
    let input_obj = incident_input_json(&inc);
    let input_hash = hash_json(&input_obj)?;

    let (model_id, prompt_version) = enrichment_model_and_prompt(ollama, job_type);
    let mut job = enrichment_jobs::create_job_running(
        db,
        job_type,
        "incident",
        incident_id,
        &input_hash,
        &model_id,
        &prompt_version,
    )
    .await?;

    // If AI isn't available, produce deterministic fallback output for some jobs.
    let ai_available = *ollama.available.read().await;
    let output = compute_enrichment_output(db, ollama, &inc, job_type, ai_available).await;
    complete_job_from_output(db, &job.id, output).await?;

    job = enrichment_jobs::get_job(db, &job.id)
        .await?
        .ok_or_else(|| AppError::Database("Job disappeared".into()))?;
    Ok(job)
}

