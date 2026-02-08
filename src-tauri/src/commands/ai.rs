use sqlx::SqlitePool;
use tauri::State;

use crate::ai::{self, OllamaState, similar, trends};
use crate::error::AppError;

#[derive(serde::Serialize)]
pub struct AiStatus {
    pub available: bool,
    pub base_url: String,
    pub primary_model: String,
    pub fast_model: String,
}

#[tauri::command]
pub async fn get_ai_status(
    ollama: State<'_, OllamaState>,
) -> Result<AiStatus, AppError> {
    let available = *ollama.available.read().await;
    Ok(AiStatus {
        available,
        base_url: ollama.base_url.clone(),
        primary_model: ollama.primary_model.clone(),
        fast_model: ollama.fast_model.clone(),
    })
}

#[tauri::command]
pub async fn check_ai_health(
    ollama: State<'_, OllamaState>,
) -> Result<bool, AppError> {
    ai::client::update_health(&*ollama).await;
    Ok(*ollama.available.read().await)
}

#[tauri::command]
pub async fn ai_summarize_incident(
    ollama: State<'_, OllamaState>,
    title: String,
    severity: String,
    status: String,
    service: String,
    root_cause: String,
    resolution: String,
    notes: String,
) -> Result<String, AppError> {
    ai::summarize::generate_summary(&*ollama, &title, &severity, &status, &service, &root_cause, &resolution, &notes).await
}

#[tauri::command]
pub async fn ai_stakeholder_update(
    ollama: State<'_, OllamaState>,
    title: String,
    severity: String,
    status: String,
    service: String,
    impact: String,
    notes: String,
) -> Result<String, AppError> {
    ai::stakeholder::generate_stakeholder_update(&*ollama, &title, &severity, &status, &service, &impact, &notes).await
}

#[tauri::command]
pub async fn ai_postmortem_draft(
    ollama: State<'_, OllamaState>,
    title: String,
    severity: String,
    service: String,
    root_cause: String,
    resolution: String,
    lessons: String,
    contributing_factors: Vec<String>,
) -> Result<String, AppError> {
    ai::postmortem::generate_postmortem_draft(&*ollama, &title, &severity, &service, &root_cause, &resolution, &lessons, &contributing_factors).await
}

#[tauri::command]
pub async fn find_similar_incidents(
    db: State<'_, SqlitePool>,
    query: String,
    exclude_id: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<similar::SimilarIncident>, AppError> {
    similar::find_similar(&*db, &query, exclude_id.as_deref(), limit.unwrap_or(5)).await
}

#[tauri::command]
pub async fn ai_suggest_root_causes(
    ollama: State<'_, OllamaState>,
    title: String,
    severity: String,
    service: String,
    symptoms: String,
    timeline: String,
) -> Result<String, AppError> {
    ai::root_cause::suggest_root_causes(
        &*ollama, &title, &severity, &service, &symptoms, &timeline,
    )
    .await
}

#[tauri::command]
pub async fn check_duplicate_incidents(
    db: State<'_, SqlitePool>,
    title: String,
    service_id: String,
) -> Result<Vec<similar::SimilarIncident>, AppError> {
    ai::dedup::check_duplicates(&*db, &title, &service_id).await
}

#[tauri::command]
pub async fn detect_service_trends(
    db: State<'_, SqlitePool>,
) -> Result<Vec<trends::ServiceTrend>, AppError> {
    ai::trends::detect_service_trends(&*db).await
}
