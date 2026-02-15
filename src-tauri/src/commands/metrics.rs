use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::{dashboard, metrics};
use crate::error::AppError;
use crate::models::metrics::{
    BacklogAgingBucket, DashboardData, DayCount, EscalationFunnelEntry, HourCount, MetricFilters,
    MetricDefinition, ServiceReliabilityScore, metric_glossary,
};

#[tauri::command]
pub async fn get_dashboard_data(
    db: State<'_, SqlitePool>,
    quarter_id: Option<String>,
    filters: MetricFilters,
) -> Result<DashboardData, AppError> {
    metrics::get_dashboard_data_for_quarter(&*db, quarter_id.as_deref(), &filters).await
}

#[tauri::command]
pub async fn get_incident_heatmap(
    db: State<'_, SqlitePool>,
    start_date: String,
    end_date: String,
) -> Result<Vec<DayCount>, AppError> {
    if start_date.is_empty() || end_date.is_empty() {
        return Err(AppError::Validation("Start and end dates are required".into()));
    }
    dashboard::get_incident_heatmap(&*db, &start_date, &end_date).await
}

#[tauri::command]
pub async fn get_incident_by_hour(
    db: State<'_, SqlitePool>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<HourCount>, AppError> {
    dashboard::get_incident_by_hour(
        &*db,
        start_date.as_deref(),
        end_date.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn get_backlog_aging(
    db: State<'_, SqlitePool>,
) -> Result<Vec<BacklogAgingBucket>, AppError> {
    metrics::get_backlog_aging(&*db).await
}

#[tauri::command]
pub async fn get_service_reliability(
    db: State<'_, SqlitePool>,
    start_date: String,
    end_date: String,
) -> Result<Vec<ServiceReliabilityScore>, AppError> {
    if start_date.is_empty() || end_date.is_empty() {
        return Err(AppError::Validation("Start and end dates are required".into()));
    }
    let range = metrics::DateRange {
        start: start_date,
        end: end_date,
    };
    metrics::get_service_reliability(&*db, &range).await
}

#[tauri::command]
pub async fn get_escalation_funnel(
    db: State<'_, SqlitePool>,
    start_date: String,
    end_date: String,
) -> Result<Vec<EscalationFunnelEntry>, AppError> {
    if start_date.is_empty() || end_date.is_empty() {
        return Err(AppError::Validation("Start and end dates are required".into()));
    }
    let range = metrics::DateRange {
        start: start_date,
        end: end_date,
    };
    metrics::get_escalation_funnel(&*db, &range).await
}

#[tauri::command]
pub async fn get_metric_glossary() -> Result<Vec<MetricDefinition>, AppError> {
    Ok(metric_glossary())
}
