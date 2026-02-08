use sqlx::{SqliteConnection, SqlitePool};

use crate::error::{AppError, AppResult};

pub async fn run_migrations(pool: &SqlitePool) -> AppResult<()> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| AppError::Database(format!("Failed to acquire DB connection: {}", e)))?;

    // Create migrations tracking table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        )"
    )
    .execute(&mut *conn)
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
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| AppError::Database(format!("Migration check failed: {}", e)))?;

        if !applied {
            if version == 12 {
                recover_lifecycle_migration_partial_state(&mut conn).await?;
            }

            // Execute each statement separately (SQLite doesn't support multiple statements in one query).
            // Keep CREATE TRIGGER ... END; blocks intact and ignore comment-only lines.
            for statement in split_migration_statements(sql) {
                sqlx::query(&statement)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Database(format!(
                        "Migration {} '{}' failed: {} (statement: {})",
                        version,
                        description,
                        e,
                        &statement[..statement.len().min(80)]
                    )))?;
            }

            sqlx::query("INSERT INTO _migrations (version, description) VALUES (?, ?)")
                .bind(version)
                .bind(description)
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Database(format!("Failed to record migration: {}", e)))?;
        }
    }

    Ok(())
}

async fn recover_lifecycle_migration_partial_state(conn: &mut SqliteConnection) -> AppResult<()> {
    let incidents_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='incidents')",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| AppError::Database(format!("Lifecycle recovery check failed: {}", e)))?;

    let incidents_new_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='incidents_new')",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| AppError::Database(format!("Lifecycle recovery check failed: {}", e)))?;

    // If both tables exist, a prior migration attempt likely failed before rename.
    // Keep canonical `incidents` and drop stale temp table so migration can rerun cleanly.
    if incidents_exists && incidents_new_exists {
        sqlx::query("DROP TABLE incidents_new")
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Database(format!(
                    "Failed to clean up partial lifecycle migration state: {}",
                    e
                ))
            })?;
    } else if !incidents_exists && incidents_new_exists {
        // If only incidents_new exists, a prior run likely dropped incidents but failed before rename.
        // Promote incidents_new back to incidents so migration 12 can rerun safely.
        sqlx::query("ALTER TABLE incidents_new RENAME TO incidents")
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Database(format!(
                    "Failed to recover incidents table from partial lifecycle migration state: {}",
                    e
                ))
            })?;
    }

    Ok(())
}

fn split_migration_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut in_trigger = false;

    for raw_line in sql.lines() {
        let without_comment = strip_inline_comment(raw_line);
        let trimmed = without_comment.trim();

        if trimmed.is_empty() {
            continue;
        }

        if !in_trigger && starts_trigger_statement(trimmed) {
            in_trigger = true;
        }

        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(trimmed);

        if in_trigger {
            if is_trigger_end(trimmed) {
                statements.push(current.trim().to_string());
                current.clear();
                in_trigger = false;
            }
        } else if contains_statement_terminator(trimmed) {
            statements.push(current.trim().to_string());
            current.clear();
        }
    }

    if !current.trim().is_empty() {
        statements.push(current.trim().to_string());
    }

    statements
}

fn starts_trigger_statement(line: &str) -> bool {
    line.to_ascii_uppercase().starts_with("CREATE TRIGGER")
}

fn is_trigger_end(line: &str) -> bool {
    let normalized: String = line.chars().filter(|c| !c.is_whitespace()).collect();
    normalized.eq_ignore_ascii_case("END;")
}

fn contains_statement_terminator(line: &str) -> bool {
    let mut chars = line.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;

    while let Some(ch) = chars.next() {
        if in_single {
            if ch == '\'' {
                if chars.peek() == Some(&'\'') {
                    chars.next();
                } else {
                    in_single = false;
                }
            }
            continue;
        }

        if in_double {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                } else {
                    in_double = false;
                }
            }
            continue;
        }

        if ch == '\'' {
            in_single = true;
            continue;
        }
        if ch == '"' {
            in_double = true;
            continue;
        }
        if ch == ';' {
            return true;
        }
    }

    false
}

fn strip_inline_comment(line: &str) -> String {
    let mut chars = line.chars().peekable();
    let mut out = String::new();
    let mut in_single = false;
    let mut in_double = false;

    while let Some(ch) = chars.next() {
        if in_single {
            out.push(ch);
            if ch == '\'' {
                if chars.peek() == Some(&'\'') {
                    out.push(chars.next().expect("peeked char exists"));
                } else {
                    in_single = false;
                }
            }
            continue;
        }
        if in_double {
            out.push(ch);
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    out.push(chars.next().expect("peeked char exists"));
                } else {
                    in_double = false;
                }
            }
            continue;
        }

        if ch == '\'' {
            in_single = true;
            out.push(ch);
            continue;
        }
        if ch == '"' {
            in_double = true;
            out.push(ch);
            continue;
        }

        if ch == '-' && chars.peek() == Some(&'-') {
            break;
        }

        out.push(ch);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::{recover_lifecycle_migration_partial_state, split_migration_statements};
    use sqlx::SqlitePool;

    #[test]
    fn keeps_trigger_body_as_single_statement() {
        let sql = r#"
            CREATE TABLE x (id INTEGER);
            CREATE TRIGGER x_ins
            AFTER INSERT ON x
            BEGIN
                UPDATE x SET id = NEW.id;
            END;
            INSERT INTO x (id) VALUES (1);
        "#;

        let statements = split_migration_statements(sql);
        assert_eq!(statements.len(), 3);
        assert!(statements[1].contains("CREATE TRIGGER"));
        assert!(statements[1].contains("END;"));
    }

    #[test]
    fn handles_trailing_inline_comment_after_terminator() {
        let sql = "INSERT INTO x (v) VALUES ('a;b'); -- trailing comment";
        let statements = split_migration_statements(sql);
        assert_eq!(statements, vec!["INSERT INTO x (v) VALUES ('a;b');"]);
    }

    #[test]
    fn handles_trigger_end_with_whitespace() {
        let sql = r#"
            CREATE TRIGGER x_ins
            AFTER INSERT ON x
            BEGIN
                UPDATE x SET id = NEW.id;
            END ;
            INSERT INTO x (id) VALUES (1);
        "#;

        let statements = split_migration_statements(sql);
        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("CREATE TRIGGER"));
        assert!(statements[0].contains("END ;"));
    }

    #[tokio::test]
    async fn recovers_partial_lifecycle_state_by_dropping_stale_temp_table() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite");
        sqlx::query("CREATE TABLE incidents (id TEXT PRIMARY KEY)")
            .execute(&pool)
            .await
            .expect("create incidents");
        sqlx::query("CREATE TABLE incidents_new (id TEXT PRIMARY KEY)")
            .execute(&pool)
            .await
            .expect("create incidents_new");

        let mut conn = pool.acquire().await.expect("acquire connection");
        recover_lifecycle_migration_partial_state(&mut conn)
            .await
            .expect("recover partial state");

        let incidents_new_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='incidents_new')",
        )
        .fetch_one(&pool)
        .await
        .expect("check incidents_new");
        assert!(!incidents_new_exists);
    }

    #[tokio::test]
    async fn recovers_partial_lifecycle_state_when_only_temp_table_exists() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite");
        sqlx::query("CREATE TABLE incidents_new (id TEXT PRIMARY KEY)")
            .execute(&pool)
            .await
            .expect("create incidents_new");

        let mut conn = pool.acquire().await.expect("acquire connection");
        recover_lifecycle_migration_partial_state(&mut conn)
            .await
            .expect("recover partial state");

        let incidents_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='incidents')",
        )
        .fetch_one(&pool)
        .await
        .expect("check incidents");
        let incidents_new_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='incidents_new')",
        )
        .fetch_one(&pool)
        .await
        .expect("check incidents_new");
        assert!(incidents_exists);
        assert!(!incidents_new_exists);
    }
}
