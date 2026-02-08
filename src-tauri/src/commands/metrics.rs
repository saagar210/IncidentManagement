use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::metrics;
use crate::error::AppError;
use crate::models::metrics::{DashboardData, MetricFilters};

#[tauri::command]
pub async fn get_dashboard_data(
    db: State<'_, SqlitePool>,
    quarter_id: Option<String>,
    filters: MetricFilters,
) -> Result<DashboardData, AppError> {
    metrics::get_dashboard_data_for_quarter(&*db, quarter_id.as_deref(), &filters).await
}
