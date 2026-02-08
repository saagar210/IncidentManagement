pub mod migrations;
pub mod queries;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::str::FromStr;

use crate::error::{AppError, AppResult};

pub type Db = SqlitePool;

pub async fn init_db(app_data_dir: PathBuf) -> AppResult<Db> {
    std::fs::create_dir_all(&app_data_dir).map_err(|e| {
        AppError::Database(format!("Failed to create app data dir: {}", e))
    })?;

    let db_path = app_data_dir.join("incidents.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    // Use connect options so PRAGMA settings apply to EVERY connection in the pool
    let options = SqliteConnectOptions::from_str(&db_url)
        .map_err(|e| AppError::Database(format!("Invalid database URL: {}", e)))?
        .journal_mode(SqliteJournalMode::Wal)
        .pragma("foreign_keys", "ON")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(|e| AppError::Database(format!("Failed to connect to database: {}", e)))?;

    // Run migrations
    migrations::run_migrations(&pool).await?;

    Ok(pool)
}
