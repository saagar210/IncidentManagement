pub mod migrations;
pub mod queries;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;

use crate::error::{AppError, AppResult};

pub type Db = SqlitePool;

pub async fn init_db(app_data_dir: PathBuf) -> AppResult<Db> {
    std::fs::create_dir_all(&app_data_dir).map_err(|e| {
        AppError::Database(format!("Failed to create app data dir: {}", e))
    })?;

    let db_path = app_data_dir.join("incidents.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .map_err(|e| AppError::Database(format!("Failed to connect to database: {}", e)))?;

    // Enable WAL mode for better concurrent read performance
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to set WAL mode: {}", e)))?;

    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to enable foreign keys: {}", e)))?;

    // Run migrations
    migrations::run_migrations(&pool).await?;

    Ok(pool)
}
