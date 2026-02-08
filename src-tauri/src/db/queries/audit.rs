use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::audit::{AuditEntry, AuditFilters, NotificationSummary};
use crate::models::priority::{Impact, Severity, calculate_priority};

fn parse_audit_entry(row: &sqlx::sqlite::SqliteRow) -> AuditEntry {
    AuditEntry {
        id: row.get("id"),
        entity_type: row.get("entity_type"),
        entity_id: row.get("entity_id"),
        action: row.get("action"),
        summary: row.get("summary"),
        details: row.get("details"),
        created_at: row.get("created_at"),
    }
}

pub async fn insert_audit_entry(
    pool: &SqlitePool,
    entity_type: &str,
    entity_id: &str,
    action: &str,
    summary: &str,
    details: &str,
) -> AppResult<()> {
    let id = format!("aud-{}", uuid::Uuid::new_v4());
    sqlx::query(
        "INSERT INTO audit_entries (id, entity_type, entity_id, action, summary, details) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(entity_type)
    .bind(entity_id)
    .bind(action)
    .bind(summary)
    .bind(details)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

pub async fn list_audit_entries(
    pool: &SqlitePool,
    filters: &AuditFilters,
) -> AppResult<Vec<AuditEntry>> {
    let mut sql = String::from("SELECT * FROM audit_entries WHERE 1=1");
    let mut binds: Vec<String> = vec![];

    if let Some(ref entity_type) = filters.entity_type {
        sql.push_str(" AND entity_type = ?");
        binds.push(entity_type.clone());
    }
    if let Some(ref entity_id) = filters.entity_id {
        sql.push_str(" AND entity_id = ?");
        binds.push(entity_id.clone());
    }
    if let Some(ref action) = filters.action {
        sql.push_str(" AND action = ?");
        binds.push(action.clone());
    }

    sql.push_str(" ORDER BY created_at DESC");

    let limit = filters.limit.unwrap_or(100).min(500);
    sql.push_str(&format!(" LIMIT {}", limit));

    let mut query = sqlx::query(&sql);
    for bind in &binds {
        query = query.bind(bind);
    }

    let rows = query
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_audit_entry).collect())
}

pub async fn get_notification_summary(pool: &SqlitePool) -> AppResult<NotificationSummary> {
    // Active incidents
    let active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM incidents WHERE status = 'Active' AND deleted_at IS NULL",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Overdue action items
    let overdue: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM action_items ai
         JOIN incidents i ON ai.incident_id = i.id
         WHERE ai.status != 'Done'
         AND ai.due_date IS NOT NULL
         AND ai.due_date < strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         AND i.deleted_at IS NULL",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // SLA breaches: compute in Rust using the same priority matrix as everywhere else
    let sla_breaches = {
        let active_rows = sqlx::query(
            "SELECT i.severity, i.impact, i.started_at FROM incidents i
             WHERE i.status = 'Active' AND i.deleted_at IS NULL",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let sla_defs = sqlx::query(
            "SELECT priority, resolve_time_minutes FROM sla_definitions WHERE is_active = 1",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let sla_map: std::collections::HashMap<String, i64> = sla_defs
            .iter()
            .map(|r| {
                (
                    r.get::<String, _>("priority"),
                    r.get::<i64, _>("resolve_time_minutes"),
                )
            })
            .collect();

        let now = chrono::Utc::now().naive_utc();
        let mut breach_count: i64 = 0;

        for row in &active_rows {
            let severity: String = row.get("severity");
            let impact: String = row.get("impact");
            let started_at: String = row.get("started_at");

            let sev = Severity::from_str(&severity).unwrap_or(Severity::Medium);
            let imp = Impact::from_str(&impact).unwrap_or(Impact::Medium);
            let priority = calculate_priority(&sev, &imp).to_string();

            if let Some(&resolve_target) = sla_map.get(&priority) {
                if let Ok(started) =
                    chrono::NaiveDateTime::parse_from_str(&started_at, "%Y-%m-%dT%H:%M:%SZ")
                        .or_else(|_| chrono::NaiveDateTime::parse_from_str(&started_at, "%Y-%m-%dT%H:%M:%S%.fZ"))
                {
                    let elapsed_minutes = (now - started).num_minutes();
                    if elapsed_minutes > resolve_target {
                        breach_count += 1;
                    }
                }
            }
        }

        breach_count
    };

    // Recent audit entries (last 24 hours)
    let recent_audit: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_entries
         WHERE created_at > strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-1 day')",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(NotificationSummary {
        active_incidents: active,
        overdue_action_items: overdue,
        sla_breaches,
        recent_audit_count: recent_audit,
    })
}
