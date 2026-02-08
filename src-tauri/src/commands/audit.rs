use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::audit;
use crate::error::AppError;
use crate::models::audit::{AuditEntry, AuditFilters, NotificationSummary};

#[tauri::command]
pub async fn list_audit_entries(
    db: State<'_, SqlitePool>,
    filters: AuditFilters,
) -> Result<Vec<AuditEntry>, AppError> {
    audit::list_audit_entries(&*db, &filters).await
}

#[tauri::command]
pub async fn get_notification_summary(
    db: State<'_, SqlitePool>,
) -> Result<NotificationSummary, AppError> {
    audit::get_notification_summary(&*db).await
}
