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
        (16, "PIR readiness", include_str!("sql/016_pir_readiness.sql")),
        (17, "Action item follow-through", include_str!("sql/017_action_item_followthrough.sql")),
        (18, "Index detected_at", include_str!("sql/018_detected_at_index.sql")),
        (19, "Quarter finalization", include_str!("sql/019_quarter_finalization.sql")),
        (20, "Service aliases and import templates", include_str!("sql/020_service_aliases_and_import_templates.sql")),
        (21, "Timeline events", include_str!("sql/021_timeline_events.sql")),
        (22, "Field provenance", include_str!("sql/022_field_provenance.sql")),
        (23, "Enrichment jobs", include_str!("sql/023_enrichment_jobs.sql")),
        (24, "Report history inputs hash", include_str!("sql/024_report_history_inputs_hash.sql")),
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

    match (incidents_exists, incidents_new_exists) {
        // If both tables exist, a prior migration attempt likely failed before rename.
        // Keep canonical `incidents` and drop stale temp table so migration can rerun cleanly.
        (true, true) => {
            sqlx::query("DROP TABLE incidents_new")
                .execute(&mut *conn)
                .await
                .map_err(|e| {
                    AppError::Database(format!(
                        "Failed to clean up partial lifecycle migration state: {}",
                        e
                    ))
                })?;
        }
        // If only incidents_new exists, a prior run likely dropped incidents but failed before rename.
        // Promote incidents_new back to incidents so migration 12 can rerun safely.
        (false, true) => {
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
        _ => {}
    }

    Ok(())
}

fn split_migration_statements(sql: &str) -> Vec<String> {
    let mut parser = MigrationStatementParser::default();

    for raw_line in sql.lines() {
        if let Some(line) = normalize_migration_line(raw_line) {
            parser.push_line(&line);
        }
    }

    parser.finish()
}

fn starts_trigger_statement(line: &str) -> bool {
    line.to_ascii_uppercase().starts_with("CREATE TRIGGER")
}

fn is_trigger_end(line: &str) -> bool {
    let normalized: String = line.chars().filter(|c| !c.is_whitespace()).collect();
    normalized.eq_ignore_ascii_case("END;")
}

fn contains_statement_terminator(line: &str) -> bool {
    let mut found = false;
    walk_unquoted_chars(line, |_, ch, _| {
        if ch == ';' {
            found = true;
            return true;
        }
        false
    });
    found
}

fn strip_inline_comment(line: &str) -> String {
    let mut comment_start = None;
    walk_unquoted_chars(line, |idx, ch, next| {
        if ch == '-' && next == Some('-') {
            comment_start = Some(idx);
            return true;
        }
        false
    });

    match comment_start {
        Some(idx) => line[..idx].trim_end().to_string(),
        None => line.to_string(),
    }
}

#[derive(Default)]
struct MigrationStatementParser {
    statements: Vec<String>,
    current: String,
    in_trigger: bool,
}

impl MigrationStatementParser {
    fn push_line(&mut self, line: &str) {
        if !self.in_trigger && starts_trigger_statement(line) {
            self.in_trigger = true;
        }

        self.append_line(line);

        if self.should_flush(line) {
            self.flush_current();
        }
    }

    fn finish(mut self) -> Vec<String> {
        if !self.current.trim().is_empty() {
            self.flush_current();
        }
        self.statements
    }

    fn append_line(&mut self, line: &str) {
        if !self.current.is_empty() {
            self.current.push('\n');
        }
        self.current.push_str(line);
    }

    fn should_flush(&self, line: &str) -> bool {
        if self.in_trigger {
            return is_trigger_end(line);
        }
        contains_statement_terminator(line)
    }

    fn flush_current(&mut self) {
        let statement = self.current.trim().to_string();
        if !statement.is_empty() {
            self.statements.push(statement);
        }
        self.current.clear();
        self.in_trigger = false;
    }
}

fn normalize_migration_line(raw_line: &str) -> Option<String> {
    let without_comment = strip_inline_comment(raw_line);
    let trimmed = without_comment.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

#[derive(Clone, Copy)]
enum QuoteMode {
    Single,
    Double,
}

fn walk_unquoted_chars(
    line: &str,
    mut on_unquoted: impl FnMut(usize, char, Option<char>) -> bool,
) {
    let mut chars = line.char_indices().peekable();
    let mut quote_mode: Option<QuoteMode> = None;

    while let Some((idx, ch)) = chars.next() {
        if let Some(mode) = quote_mode {
            if is_quote_terminator(mode, ch, &mut chars) {
                quote_mode = None;
            }
            continue;
        }

        if ch == '\'' {
            quote_mode = Some(QuoteMode::Single);
            continue;
        }
        if ch == '"' {
            quote_mode = Some(QuoteMode::Double);
            continue;
        }

        let next = chars.peek().map(|(_, c)| *c);
        if on_unquoted(idx, ch, next) {
            break;
        }
    }
}

fn is_quote_terminator(
    mode: QuoteMode,
    ch: char,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
) -> bool {
    let quote = match mode {
        QuoteMode::Single => '\'',
        QuoteMode::Double => '"',
    };

    if ch != quote {
        return false;
    }

    if chars.peek().map(|(_, next)| *next) == Some(quote) {
        chars.next();
        return false;
    }

    true
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
