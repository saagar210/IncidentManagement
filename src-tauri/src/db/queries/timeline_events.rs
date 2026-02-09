use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimelineEvent {
    pub id: String,
    pub incident_id: String,
    pub occurred_at: String,
    pub source: String,
    pub message: String,
    pub actor: String,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateTimelineEventRequest {
    pub incident_id: String,
    pub occurred_at: String,
    pub source: Option<String>,
    pub message: String,
    pub actor: Option<String>,
}

impl CreateTimelineEventRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.incident_id.trim().is_empty() {
            return Err(AppError::Validation("Incident ID is required".into()));
        }
        if self.occurred_at.trim().is_empty() {
            return Err(AppError::Validation("occurred_at is required".into()));
        }
        if self.message.trim().is_empty() {
            return Err(AppError::Validation("Message is required".into()));
        }
        if self.message.len() > 20_000 {
            return Err(AppError::Validation("Message too long (max 20000 chars)".into()));
        }
        Ok(())
    }
}

fn parse_occurred_at(raw: &str) -> AppResult<String> {
    let s = raw.trim();
    if s.is_empty() {
        return Err(AppError::Validation("occurred_at is required".into()));
    }

    // Prefer RFC3339 / ISO-8601 strings with explicit timezone.
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc).to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    }

    // Accept a simple "YYYY-MM-DD HH:MM" (assumed UTC).
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M") {
        let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
        return Ok(dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    }

    Err(AppError::Validation(format!(
        "Invalid occurred_at '{}'. Expected RFC3339 or 'YYYY-MM-DD HH:MM'.",
        s
    )))
}

pub async fn list_timeline_events_for_incident(
    pool: &SqlitePool,
    incident_id: &str,
) -> AppResult<Vec<TimelineEvent>> {
    let rows = sqlx::query(
        "SELECT * FROM timeline_events WHERE incident_id = ? ORDER BY occurred_at ASC, created_at ASC",
    )
    .bind(incident_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_row).collect())
}

pub async fn list_timeline_events_for_incidents(
    pool: &SqlitePool,
    incident_ids: &[String],
) -> AppResult<HashMap<String, Vec<TimelineEvent>>> {
    let mut out: HashMap<String, Vec<TimelineEvent>> = HashMap::new();
    if incident_ids.is_empty() {
        return Ok(out);
    }

    // Build a safe IN (...) query with binds.
    let mut qb: sqlx::QueryBuilder<sqlx::Sqlite> = sqlx::QueryBuilder::new(
        "SELECT * FROM timeline_events WHERE incident_id IN (",
    );
    let mut separated = qb.separated(", ");
    for id in incident_ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(") ORDER BY incident_id ASC, occurred_at ASC, created_at ASC");

    let rows = qb
        .build()
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    for row in rows.iter() {
        let ev = parse_row(row);
        out.entry(ev.incident_id.clone()).or_default().push(ev);
    }
    Ok(out)
}

pub async fn create_timeline_event(
    pool: &SqlitePool,
    req: &CreateTimelineEventRequest,
) -> AppResult<TimelineEvent> {
    req.validate()?;

    let occurred_at = parse_occurred_at(&req.occurred_at)?;
    let source = req.source.clone().unwrap_or_else(|| "manual".to_string());
    let actor = req.actor.clone().unwrap_or_default();

    let id = format!("tme-{}", uuid::Uuid::new_v4());
    sqlx::query(
        "INSERT INTO timeline_events (id, incident_id, occurred_at, source, message, actor) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&req.incident_id)
    .bind(&occurred_at)
    .bind(&source)
    .bind(req.message.trim())
    .bind(&actor)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let row = sqlx::query("SELECT * FROM timeline_events WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(parse_row(&row))
}

pub async fn delete_timeline_event(pool: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM timeline_events WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Timeline event '{}' not found", id)));
    }
    Ok(())
}

fn parse_row(row: &sqlx::sqlite::SqliteRow) -> TimelineEvent {
    TimelineEvent {
        id: row.get("id"),
        incident_id: row.get("incident_id"),
        occurred_at: row.get("occurred_at"),
        source: row.get("source"),
        message: row.get("message"),
        actor: row.get("actor"),
        created_at: row.get("created_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::{create_timeline_event, list_timeline_events_for_incident, CreateTimelineEventRequest};
    use crate::db::migrations::run_migrations;
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
    use std::str::FromStr;
    use tempfile::tempdir;

    async fn setup_db() -> sqlx::SqlitePool {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("timeline-tests.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let options = SqliteConnectOptions::from_str(&db_url)
            .expect("sqlite url")
            .journal_mode(SqliteJournalMode::Wal)
            .pragma("foreign_keys", "ON")
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("connect");
        run_migrations(&pool).await.expect("migrations");
        pool
    }

    #[tokio::test]
    async fn timeline_event_accepts_simple_timestamp_and_orders() {
        let pool = setup_db().await;

        // Use an existing seeded incident by creating one minimal incident row via insert_incident helper.
        let service_id: String = sqlx::query_scalar("SELECT id FROM services LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("service");

        let inc_id = format!("inc-{}", uuid::Uuid::new_v4());
        sqlx::query(
            "INSERT INTO incidents (id, title, service_id, severity, impact, status, started_at, detected_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, (strftime('%Y-%m-%dT%H:%M:%SZ','now')), (strftime('%Y-%m-%dT%H:%M:%SZ','now')))",
        )
        .bind(&inc_id)
        .bind("Test Incident")
        .bind(&service_id)
        .bind("High")
        .bind("High")
        .bind("Active")
        .bind("2026-01-01T10:00:00Z")
        .bind("2026-01-01T10:05:00Z")
        .execute(&pool)
        .await
        .expect("insert incident");

        create_timeline_event(
            &pool,
            &CreateTimelineEventRequest {
                incident_id: inc_id.clone(),
                occurred_at: "2026-01-01 10:06".into(),
                source: None,
                message: "First event".into(),
                actor: None,
            },
        )
        .await
        .expect("create event");

        create_timeline_event(
            &pool,
            &CreateTimelineEventRequest {
                incident_id: inc_id.clone(),
                occurred_at: "2026-01-01T10:07:00Z".into(),
                source: None,
                message: "Second event".into(),
                actor: None,
            },
        )
        .await
        .expect("create event");

        let events = list_timeline_events_for_incident(&pool, &inc_id)
            .await
            .expect("list");
        assert_eq!(events.len(), 2);
        assert!(events[0].occurred_at < events[1].occurred_at);
    }
}
