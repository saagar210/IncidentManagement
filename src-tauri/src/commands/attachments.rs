use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tauri::{Manager, State};

use crate::error::{AppError, AppResult};

const MAX_ATTACHMENT_SIZE: u64 = 10 * 1024 * 1024; // 10MB

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub incident_id: String,
    pub filename: String,
    pub file_path: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: String,
}

#[tauri::command]
pub async fn upload_attachment(
    app: tauri::AppHandle,
    db: State<'_, SqlitePool>,
    incident_id: String,
    source_path: String,
    filename: String,
) -> Result<Attachment, AppError> {
    if filename.trim().is_empty() {
        return Err(AppError::Validation("Filename is required".into()));
    }
    if filename.len() > 255 {
        return Err(AppError::Validation("Filename too long".into()));
    }

    let metadata = tokio::fs::metadata(&source_path)
        .await
        .map_err(|e| AppError::Io(e))?;

    if metadata.len() > MAX_ATTACHMENT_SIZE {
        return Err(AppError::Validation(format!(
            "File too large ({:.1} MB). Maximum is 10 MB.",
            metadata.len() as f64 / 1024.0 / 1024.0
        )));
    }

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let attachments_dir = app_data_dir.join("attachments");
    tokio::fs::create_dir_all(&attachments_dir)
        .await
        .map_err(|e| AppError::Io(e))?;

    let id = format!("att-{}", uuid::Uuid::new_v4());
    let ext = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let stored_name = if ext.is_empty() {
        id.clone()
    } else {
        format!("{}.{}", id, ext)
    };
    let dest_path = attachments_dir.join(&stored_name);

    tokio::fs::copy(&source_path, &dest_path)
        .await
        .map_err(|e| AppError::Io(e))?;

    let mime_type = guess_mime(&filename);
    let dest_str = dest_path
        .to_str()
        .ok_or_else(|| AppError::Internal("Path conversion failed".into()))?;

    sqlx::query(
        "INSERT INTO attachments (id, incident_id, filename, file_path, mime_type, size_bytes) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&incident_id)
    .bind(&filename)
    .bind(dest_str)
    .bind(&mime_type)
    .bind(metadata.len() as i64)
    .execute(&*db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_attachment(&db, &id).await
}

#[tauri::command]
pub async fn list_attachments(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<Vec<Attachment>, AppError> {
    let rows =
        sqlx::query("SELECT * FROM attachments WHERE incident_id = ? ORDER BY created_at ASC")
            .bind(&incident_id)
            .fetch_all(&*db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_attachment).collect())
}

#[tauri::command]
pub async fn delete_attachment(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    let att = get_attachment(&db, &id).await?;

    // Delete physical file
    let _ = tokio::fs::remove_file(&att.file_path).await;

    sqlx::query("DELETE FROM attachments WHERE id = ?")
        .bind(&id)
        .execute(&*db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

async fn get_attachment(db: &SqlitePool, id: &str) -> AppResult<Attachment> {
    let row = sqlx::query("SELECT * FROM attachments WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Attachment '{}' not found", id)))?;

    Ok(parse_attachment(&row))
}

fn parse_attachment(row: &sqlx::sqlite::SqliteRow) -> Attachment {
    Attachment {
        id: row.get("id"),
        incident_id: row.get("incident_id"),
        filename: row.get("filename"),
        file_path: row.get("file_path"),
        mime_type: row.get::<Option<String>, _>("mime_type")
            .unwrap_or_else(|| "application/octet-stream".to_string()),
        size_bytes: row.get::<Option<i64>, _>("size_bytes").unwrap_or(0),
        created_at: row.get("created_at"),
    }
}

fn guess_mime(filename: &str) -> String {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "json" => "application/json",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
    .to_string()
}
