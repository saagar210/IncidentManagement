use tauri::State;
use sqlx::SqlitePool;

use crate::error::AppError;

/// Delete a temp file created by backend commands (reports/exports).
/// This is intentionally strict to avoid arbitrary file deletion.
#[tauri::command]
pub async fn delete_temp_file(
    _db: State<'_, SqlitePool>,
    temp_path: String,
) -> Result<(), AppError> {
    let temp_dir = std::env::temp_dir();

    let canonical = match std::fs::canonicalize(&temp_path) {
        Ok(p) => p,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(AppError::Validation(format!("Invalid temp path: {}", e))),
    };

    if !canonical.starts_with(&temp_dir) {
        return Err(AppError::Validation(
            "Temp path must be within the system temp directory".into(),
        ));
    }

    // Only allow deleting temp files that this app creates.
    let file_name = canonical
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let allowed = file_name.starts_with("incident_report_")
        || file_name.starts_with("incidents_export_")
        || file_name.starts_with("incident_backup_");
    if !allowed {
        return Err(AppError::Validation(
            "Unsupported temp file name".into(),
        ));
    }

    match tokio::fs::remove_file(&canonical).await {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(AppError::Io(e)),
    }
}
