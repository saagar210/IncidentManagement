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
        (3, "Add tags", include_str!("sql/003_tags.sql")),
        (4, "Add custom fields", include_str!("sql/004_custom_fields.sql")),
        (5, "Add attachments", include_str!("sql/005_attachments.sql")),
        (6, "Add soft delete", include_str!("sql/006_soft_delete.sql")),
        (7, "Add report history", include_str!("sql/007_report_history.sql")),
        (8, "Add SLA definitions", include_str!("sql/008_sla_definitions.sql")),
        (9, "Add audit log", include_str!("sql/009_audit_log.sql")),
        (10, "Service catalog enhancement", include_str!("sql/010_service_catalog.sql")),
        (11, "Roles and checklists", include_str!("sql/011_roles_checklists.sql")),
        (12, "Expanded lifecycle states", include_str!("sql/012_lifecycle_states.sql")),
        (13, "Analytics and FTS", include_str!("sql/013_analytics_fts.sql")),
        (14, "Post-mortem and AI", include_str!("sql/014_postmortem_ai.sql")),
        (15, "UX features", include_str!("sql/015_ux_features.sql")),
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
