use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

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
    db: State<'_, SqlitePool>,
    backup_dir: String,
) -> Result<String, AppError> {
    let db_path = resolve_main_db_path(&db).await?;
    create_backup_from_path(&db_path, &backup_dir).await
}

async fn create_backup_from_path(
    db_path: &str,
    backup_dir: &str,
) -> Result<String, AppError> {
    // Validate source database file exists
    let src_metadata = tokio::fs::metadata(db_path)
        .await
        .map_err(|e| AppError::Io(e))?;

    if !src_metadata.is_file() {
        return Err(AppError::Validation(
            "Database path is not a file".into(),
        ));
    }

    // Create backup directory if it doesn't exist
    tokio::fs::create_dir_all(backup_dir)
        .await
        .map_err(|e| AppError::Io(e))?;

    // Generate timestamped backup filename
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("backup_{}.db", timestamp);
    let backup_path = std::path::Path::new(backup_dir).join(&backup_name);

    // Copy the SQLite file
    tokio::fs::copy(db_path, &backup_path)
        .await
        .map_err(|e| AppError::Io(e))?;

    let path_str = backup_path
        .to_str()
        .ok_or_else(|| AppError::Internal("Invalid path encoding".into()))?
        .to_string();

    Ok(path_str)
}

async fn resolve_main_db_path(db: &SqlitePool) -> Result<String, AppError> {
    let path: Option<String> =
        sqlx::query_scalar("SELECT file FROM pragma_database_list WHERE name = 'main'")
            .fetch_optional(db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

    let resolved = path.unwrap_or_default();
    if resolved.trim().is_empty() {
        return Err(AppError::Validation(
            "Unable to resolve database path for backup".into(),
        ));
    }

    Ok(resolved)
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

#[cfg(test)]
mod tests {
    use super::{create_backup_from_path, resolve_main_db_path};
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
    use std::str::FromStr;
    use tempfile::tempdir;

    async fn setup_file_db() -> (tempfile::TempDir, sqlx::SqlitePool, String) {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("incidents.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let options = SqliteConnectOptions::from_str(&db_url)
            .expect("valid sqlite url")
            .journal_mode(SqliteJournalMode::Wal)
            .pragma("foreign_keys", "ON")
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("connect");

        sqlx::query("CREATE TABLE IF NOT EXISTS smoke (id TEXT PRIMARY KEY)")
            .execute(&pool)
            .await
            .expect("create table");

        (dir, pool, db_path.to_string_lossy().to_string())
    }

    #[tokio::test]
    async fn resolve_main_db_path_returns_file_path() {
        let (_dir, pool, expected_path) = setup_file_db().await;
        let resolved = resolve_main_db_path(&pool).await.expect("resolve path");
        let resolved_canonical = std::fs::canonicalize(&resolved)
            .expect("canonicalize resolved path");
        let expected_canonical = std::fs::canonicalize(&expected_path)
            .expect("canonicalize expected path");
        assert_eq!(resolved_canonical, expected_canonical);
    }

    #[tokio::test]
    async fn create_backup_from_path_copies_database_file() {
        let (dir, _pool, db_path) = setup_file_db().await;
        let backup_dir = dir.path().join("backups");
        let backup_dir_str = backup_dir.to_string_lossy().to_string();
        let backup_path = create_backup_from_path(&db_path, &backup_dir_str)
            .await
            .expect("create backup");
        assert!(std::path::Path::new(&backup_path).exists());
        assert!(backup_path.ends_with(".db"));
    }
}
