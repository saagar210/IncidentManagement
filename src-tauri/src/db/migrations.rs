use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};

pub async fn run_migrations(pool: &SqlitePool) -> AppResult<()> {
    // Create migrations tracking table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        )"
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to create migrations table: {}", e)))?;

    let migrations: Vec<(i64, &str, &str)> = vec![
        (1, "Create core schema", include_str!("sql/001_core_schema.sql")),
        (2, "Seed default data", include_str!("sql/002_seed_data.sql")),
    ];

    for (version, description, sql) in migrations {
        let applied: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM _migrations WHERE version = ?)"
        )
        .bind(version)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Migration check failed: {}", e)))?;

        if !applied {
            // Execute each statement separately (SQLite doesn't support multiple statements in one query)
            for statement in sql.split(';') {
                let trimmed = statement.trim();
                if !trimmed.is_empty() {
                    sqlx::query(trimmed)
                        .execute(pool)
                        .await
                        .map_err(|e| AppError::Database(format!(
                            "Migration {} '{}' failed: {} (statement: {})",
                            version, description, e, &trimmed[..trimmed.len().min(80)]
                        )))?;
                }
            }

            sqlx::query("INSERT INTO _migrations (version, description) VALUES (?, ?)")
                .bind(version)
                .bind(description)
                .execute(pool)
                .await
                .map_err(|e| AppError::Database(format!("Failed to record migration: {}", e)))?;
        }
    }

    Ok(())
}
