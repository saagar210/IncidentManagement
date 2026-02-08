use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: String,
}

#[tauri::command]
pub async fn create_backup(
    db_path: String,
    backup_dir: String,
) -> Result<String, AppError> {
    // Validate source database file exists
    let src_metadata = tokio::fs::metadata(&db_path)
        .await
        .map_err(|e| AppError::Io(e))?;

    if !src_metadata.is_file() {
        return Err(AppError::Validation(
            "Database path is not a file".into(),
        ));
    }

    // Create backup directory if it doesn't exist
    tokio::fs::create_dir_all(&backup_dir)
        .await
        .map_err(|e| AppError::Io(e))?;

    // Generate timestamped backup filename
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("backup_{}.db", timestamp);
    let backup_path = std::path::Path::new(&backup_dir).join(&backup_name);

    // Copy the SQLite file
    tokio::fs::copy(&db_path, &backup_path)
        .await
        .map_err(|e| AppError::Io(e))?;

    let path_str = backup_path
        .to_str()
        .ok_or_else(|| AppError::Internal("Invalid path encoding".into()))?
        .to_string();

    Ok(path_str)
}

#[tauri::command]
pub async fn list_backups(
    backup_dir: String,
) -> Result<Vec<BackupInfo>, AppError> {
    // If the directory doesn't exist, return empty
    let dir_exists = tokio::fs::metadata(&backup_dir).await.is_ok();
    if !dir_exists {
        return Ok(vec![]);
    }

    let mut entries = tokio::fs::read_dir(&backup_dir)
        .await
        .map_err(|e| AppError::Io(e))?;

    let mut backups: Vec<BackupInfo> = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| AppError::Io(e))?
    {
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Only include .db files that look like backups
        if !file_name.ends_with(".db") {
            continue;
        }

        let metadata = entry.metadata().await.map_err(|e| AppError::Io(e))?;
        if !metadata.is_file() {
            continue;
        }

        let path = entry.path();
        let path_str = path
            .to_str()
            .unwrap_or_default()
            .to_string();

        // Use file modified time for created_at
        let created_at = metadata
            .modified()
            .ok()
            .and_then(|t| {
                let duration = t
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?;
                let dt = chrono::DateTime::from_timestamp(
                    duration.as_secs() as i64,
                    duration.subsec_nanos(),
                )?;
                Some(dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            })
            .unwrap_or_default();

        backups.push(BackupInfo {
            name: file_name,
            path: path_str,
            size_bytes: metadata.len(),
            created_at,
        });
    }

    // Sort by created_at descending (most recent first)
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(backups)
}
