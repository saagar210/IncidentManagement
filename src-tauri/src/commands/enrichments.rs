use sqlx::SqlitePool;
use tauri::State;

use crate::ai::OllamaState;
use crate::db::queries::{enrichment_jobs, incident_enrichments};
use crate::error::AppError;

#[path = "enrichments_accept.rs"]
mod enrichments_accept;
#[path = "enrichments_run.rs"]
mod enrichments_run;

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
    enrichments_run::run_incident_enrichment(&*db, &*ollama, &req.job_type, &req.incident_id).await
}

#[tauri::command]
pub async fn accept_enrichment_job(
    db: State<'_, SqlitePool>,
    req: AcceptEnrichmentCmd,
) -> Result<(), AppError> {
    enrichments_accept::accept_job_by_id(&*db, &req.job_id).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_incident_enrichment(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<Option<incident_enrichments::IncidentEnrichment>, AppError> {
    let v = incident_enrichments::get_incident_enrichment(&*db, &incident_id).await?;
    Ok(v)
}
