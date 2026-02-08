use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::incident::{
    ActionItem, CreateActionItemRequest, CreateIncidentRequest, Incident, IncidentFilters,
    UpdateActionItemRequest, UpdateIncidentRequest, allowed_transitions, is_reopen,
};
use crate::models::priority::{Impact, Severity, calculate_priority};

fn compute_priority(severity: &str, impact: &str) -> String {
    let sev = Severity::from_str(severity).unwrap_or(Severity::Medium);
    let imp = Impact::from_str(impact).unwrap_or(Impact::Medium);
    calculate_priority(&sev, &imp).to_string()
}

pub async fn insert_incident(
    db: &SqlitePool,
    id: &str,
    req: &CreateIncidentRequest,
) -> AppResult<Incident> {
    // Validate recurrence_of references an existing incident
    if let Some(ref rec_id) = req.recurrence_of {
        if !rec_id.is_empty() {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM incidents WHERE id = ? AND deleted_at IS NULL"
            )
            .bind(rec_id)
            .fetch_one(db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            if exists == 0 {
                return Err(AppError::Validation(format!(
                    "Referenced incident '{}' not found", rec_id
                )));
            }
        }
    }

    sqlx::query(
        "INSERT INTO incidents (id, title, service_id, severity, impact, status, started_at, detected_at, acknowledged_at, first_response_at, mitigation_started_at, responded_at, resolved_at, root_cause, resolution, tickets_submitted, affected_users, is_recurring, recurrence_of, lessons_learned, action_items, external_ref, notes) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(id)
    .bind(&req.title)
    .bind(&req.service_id)
    .bind(&req.severity)
    .bind(&req.impact)
    .bind(&req.status)
    .bind(&req.started_at)
    .bind(&req.detected_at)
    .bind(&req.acknowledged_at)
    .bind(&req.first_response_at)
    .bind(&req.mitigation_started_at)
    .bind(&req.responded_at)
    .bind(&req.resolved_at)
    .bind(&req.root_cause)
    .bind(&req.resolution)
    .bind(req.tickets_submitted)
    .bind(req.affected_users)
    .bind(req.is_recurring)
    .bind(&req.recurrence_of)
    .bind(&req.lessons_learned)
    .bind(&req.action_items)
    .bind(&req.external_ref)
    .bind(&req.notes)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_incident_by_id(db, id).await
}

pub async fn update_incident(
    db: &SqlitePool,
    id: &str,
    req: &UpdateIncidentRequest,
) -> AppResult<Incident> {
    let existing = get_incident_by_id(db, id).await?;

    // Validate recurrence_of references an existing incident
    if let Some(ref rec_id) = req.recurrence_of {
        if !rec_id.is_empty() && rec_id != id {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM incidents WHERE id = ? AND deleted_at IS NULL"
            )
            .bind(rec_id)
            .fetch_one(db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            if exists == 0 {
                return Err(AppError::Validation(format!(
                    "Referenced incident '{}' not found", rec_id
                )));
            }
        }
    }

    let title = req.title.as_ref().unwrap_or(&existing.title);
    let service_id = req.service_id.as_ref().unwrap_or(&existing.service_id);
    let severity = req.severity.as_ref().unwrap_or(&existing.severity);
    let impact = req.impact.as_ref().unwrap_or(&existing.impact);
    let new_status = req.status.as_ref().unwrap_or(&existing.status);
    let started_at = req.started_at.as_ref().unwrap_or(&existing.started_at);
    let detected_at = req.detected_at.as_ref().unwrap_or(&existing.detected_at);
    let root_cause = req.root_cause.as_ref().unwrap_or(&existing.root_cause);
    let resolution = req.resolution.as_ref().unwrap_or(&existing.resolution);
    let tickets = req.tickets_submitted.unwrap_or(existing.tickets_submitted);
    let affected = req.affected_users.unwrap_or(existing.affected_users);
    let recurring = req.is_recurring.unwrap_or(existing.is_recurring);
    let lessons = req.lessons_learned.as_ref().unwrap_or(&existing.lessons_learned);
    let actions = req.action_items.as_ref().unwrap_or(&existing.action_items);
    let ext_ref = req.external_ref.as_ref().unwrap_or(&existing.external_ref);
    let notes = req.notes.as_ref().unwrap_or(&existing.notes);

    // State transition validation
    let status_changed = new_status != &existing.status;
    if status_changed {
        let allowed = allowed_transitions(&existing.status);
        if !allowed.contains(&new_status.as_str()) {
            return Err(AppError::Validation(format!(
                "Cannot transition from '{}' to '{}'. Allowed: {}",
                existing.status, new_status, allowed.join(", ")
            )));
        }
    }

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // Handle reopen: increment reopen_count and set reopened_at
    let reopen_count = if status_changed && is_reopen(&existing.status, new_status) {
        existing.reopen_count + 1
    } else {
        existing.reopen_count
    };
    let reopened_at = if status_changed && is_reopen(&existing.status, new_status) {
        Some(now.clone())
    } else {
        existing.reopened_at.clone()
    };

    // Auto-set acknowledged_at when transitioning to Acknowledged
    let acknowledged_at = if req.acknowledged_at.is_some() {
        req.acknowledged_at.clone()
    } else if status_changed && new_status == "Acknowledged" && existing.acknowledged_at.is_none() {
        Some(now.clone())
    } else {
        existing.acknowledged_at.clone()
    };

    // Handle optional timestamp fields
    let first_response_at = if req.first_response_at.is_some() {
        req.first_response_at.clone()
    } else {
        existing.first_response_at.clone()
    };

    let mitigation_started_at = if req.mitigation_started_at.is_some() {
        req.mitigation_started_at.clone()
    } else {
        existing.mitigation_started_at.clone()
    };

    let responded_at = if req.responded_at.is_some() {
        req.responded_at.clone()
    } else {
        existing.responded_at.clone()
    };

    // Auto-set resolved_at when status changes to "Resolved" and it's not already set
    let resolved_at = if req.resolved_at.is_some() {
        req.resolved_at.clone()
    } else if status_changed && new_status == "Resolved" && existing.resolved_at.is_none() {
        Some(now.clone())
    } else {
        existing.resolved_at.clone()
    };

    let recurrence_of = if req.recurrence_of.is_some() {
        req.recurrence_of.clone()
    } else {
        existing.recurrence_of.clone()
    };

    // Validate date ordering using the merged (final) values
    if detected_at < started_at {
        return Err(AppError::Validation(
            "Detected at must be on or after started at".into(),
        ));
    }
    if let Some(ref resp) = responded_at {
        if resp.as_str() < detected_at.as_str() {
            return Err(AppError::Validation(
                "Responded at must be on or after detected at".into(),
            ));
        }
    }
    if let Some(ref res) = resolved_at {
        if res.as_str() < started_at.as_str() {
            return Err(AppError::Validation(
                "Resolved at must be on or after started at".into(),
            ));
        }
    }

    sqlx::query(
        "UPDATE incidents SET title=?, service_id=?, severity=?, impact=?, status=?, started_at=?, detected_at=?, acknowledged_at=?, first_response_at=?, mitigation_started_at=?, responded_at=?, resolved_at=?, reopened_at=?, reopen_count=?, root_cause=?, resolution=?, tickets_submitted=?, affected_users=?, is_recurring=?, recurrence_of=?, lessons_learned=?, action_items=?, external_ref=?, notes=?, updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?"
    )
    .bind(title)
    .bind(service_id)
    .bind(severity)
    .bind(impact)
    .bind(new_status)
    .bind(started_at)
    .bind(detected_at)
    .bind(&acknowledged_at)
    .bind(&first_response_at)
    .bind(&mitigation_started_at)
    .bind(&responded_at)
    .bind(&resolved_at)
    .bind(&reopened_at)
    .bind(reopen_count)
    .bind(root_cause)
    .bind(resolution)
    .bind(tickets)
    .bind(affected)
    .bind(recurring)
    .bind(&recurrence_of)
    .bind(lessons)
    .bind(actions)
    .bind(ext_ref)
    .bind(notes)
    .bind(id)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_incident_by_id(db, id).await
}

pub async fn delete_incident(db: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE incidents SET deleted_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ? AND deleted_at IS NULL"
    )
    .bind(id)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Incident '{}' not found", id)));
    }
    Ok(())
}

pub async fn list_deleted_incidents(db: &SqlitePool) -> AppResult<Vec<Incident>> {
    let rows = sqlx::query(
        "SELECT i.*, s.name as service_name FROM incidents i LEFT JOIN services s ON i.service_id = s.id WHERE i.deleted_at IS NOT NULL ORDER BY i.deleted_at DESC"
    )
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_incident).collect())
}

pub async fn restore_incident(db: &SqlitePool, id: &str) -> AppResult<Incident> {
    let result = sqlx::query(
        "UPDATE incidents SET deleted_at = NULL, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ? AND deleted_at IS NOT NULL"
    )
    .bind(id)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Deleted incident '{}' not found", id)));
    }
    get_incident_by_id(db, id).await
}

pub async fn permanent_delete_incident(db: &SqlitePool, id: &str) -> AppResult<()> {
    // Verify it's in trash first
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM incidents WHERE id = ? AND deleted_at IS NOT NULL"
    )
    .bind(id)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if exists == 0 {
        return Err(AppError::NotFound(format!("Deleted incident '{}' not found", id)));
    }

    // Use a transaction to clean up related data
    let mut tx = db.begin()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Delete action items for this incident
    sqlx::query("DELETE FROM action_items WHERE incident_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Delete audit entries for this incident
    sqlx::query("DELETE FROM audit_entries WHERE entity_type = 'incident' AND entity_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Delete tags for this incident
    sqlx::query("DELETE FROM incident_tags WHERE incident_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Delete custom field values for this incident
    sqlx::query("DELETE FROM custom_field_values WHERE incident_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Delete attachments for this incident
    sqlx::query("DELETE FROM attachments WHERE incident_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Delete incident roles
    sqlx::query("DELETE FROM incident_roles WHERE incident_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Delete incident checklists (items cascade via FK)
    sqlx::query("DELETE FROM incident_checklists WHERE incident_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Finally delete the incident itself
    sqlx::query("DELETE FROM incidents WHERE id = ? AND deleted_at IS NOT NULL")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

#[allow(dead_code)]
pub async fn purge_old_deleted(db: &SqlitePool, days: i64) -> AppResult<i64> {
    let result = sqlx::query(
        "DELETE FROM incidents WHERE deleted_at IS NOT NULL AND julianday('now') - julianday(deleted_at) > ?"
    )
    .bind(days)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(result.rows_affected() as i64)
}

pub async fn count_deleted_incidents(db: &SqlitePool) -> AppResult<i64> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM incidents WHERE deleted_at IS NOT NULL"
    )
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(count)
}

pub async fn get_incident_by_id(db: &SqlitePool, id: &str) -> AppResult<Incident> {
    let row = sqlx::query(
        "SELECT i.*, s.name as service_name FROM incidents i LEFT JOIN services s ON i.service_id = s.id WHERE i.id = ? AND i.deleted_at IS NULL"
    )
    .bind(id)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Incident '{}' not found", id)))?;

    Ok(parse_incident(&row))
}

pub async fn list_incidents(
    db: &SqlitePool,
    filters: &IncidentFilters,
    quarter_dates: Option<(String, String)>,
) -> AppResult<Vec<Incident>> {
    let mut sql = String::from(
        "SELECT i.*, s.name as service_name FROM incidents i LEFT JOIN services s ON i.service_id = s.id WHERE i.deleted_at IS NULL",
    );
    let mut binds: Vec<String> = vec![];

    if let Some(ref service_id) = filters.service_id {
        sql.push_str(" AND i.service_id = ?");
        binds.push(service_id.clone());
    }
    if let Some(ref severity) = filters.severity {
        sql.push_str(" AND i.severity = ?");
        binds.push(severity.clone());
    }
    if let Some(ref impact) = filters.impact {
        sql.push_str(" AND i.impact = ?");
        binds.push(impact.clone());
    }
    if let Some(ref status) = filters.status {
        sql.push_str(" AND i.status = ?");
        binds.push(status.clone());
    }

    // Date range from quarter or explicit dates
    if let Some((start, end)) = quarter_dates {
        sql.push_str(" AND i.started_at >= ?");
        binds.push(start);
        sql.push_str(" AND i.started_at <= ?");
        binds.push(end);
    } else {
        if let Some(ref date_from) = filters.date_from {
            sql.push_str(" AND i.started_at >= ?");
            binds.push(date_from.clone());
        }
        if let Some(ref date_to) = filters.date_to {
            sql.push_str(" AND i.started_at <= ?");
            binds.push(date_to.clone());
        }
    }

    // Sorting
    let sort_col = match filters.sort_by.as_deref() {
        Some("title") => "i.title",
        Some("severity") => "i.severity",
        Some("impact") => "i.impact",
        Some("status") => "i.status",
        Some("service") => "s.name",
        Some("duration") => "i.duration_minutes",
        _ => "i.started_at",
    };
    let sort_dir = match filters.sort_order.as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    };
    sql.push_str(&format!(" ORDER BY {} {}", sort_col, sort_dir));

    let mut query = sqlx::query(&sql);
    for bind in &binds {
        query = query.bind(bind);
    }

    let rows = query
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_incident).collect())
}

pub async fn search_incidents(db: &SqlitePool, query: &str) -> AppResult<Vec<Incident>> {
    // Use FTS5 for full-text search when available, fall back to LIKE
    // Escape FTS5 special characters and build a prefix query
    let fts_query = query
        .replace('"', "\"\"")
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|w| format!("\"{}\"*", w))
        .collect::<Vec<_>>()
        .join(" ");

    if fts_query.is_empty() {
        return Ok(vec![]);
    }

    // Try FTS5 search first
    let fts_result = sqlx::query(
        "SELECT i.*, s.name as service_name FROM incidents i LEFT JOIN services s ON i.service_id = s.id WHERE i.deleted_at IS NULL AND i.rowid IN (SELECT rowid FROM incidents_fts WHERE incidents_fts MATCH ?1) ORDER BY i.started_at DESC"
    )
    .bind(&fts_query)
    .fetch_all(db)
    .await;

    match fts_result {
        Ok(rows) => Ok(rows.iter().map(parse_incident).collect()),
        Err(_) => {
            // Fallback to LIKE search if FTS5 table doesn't exist yet
            let escaped = query
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            let pattern = format!("%{}%", escaped);
            let rows = sqlx::query(
                "SELECT i.*, s.name as service_name FROM incidents i LEFT JOIN services s ON i.service_id = s.id WHERE i.deleted_at IS NULL AND (i.title LIKE ?1 ESCAPE '\\' OR i.root_cause LIKE ?1 ESCAPE '\\' OR i.resolution LIKE ?1 ESCAPE '\\' OR i.notes LIKE ?1 ESCAPE '\\' OR i.external_ref LIKE ?1 ESCAPE '\\') ORDER BY i.started_at DESC"
            )
            .bind(&pattern)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            Ok(rows.iter().map(parse_incident).collect())
        }
    }
}

pub async fn bulk_update_status(db: &SqlitePool, ids: &[String], status: &str) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    // Validate status before beginning transaction
    const VALID_STATUSES: &[&str] = &["Active", "Acknowledged", "Monitoring", "Resolved", "Post-Mortem"];
    if !VALID_STATUSES.contains(&status) {
        return Err(AppError::Validation(format!(
            "Invalid status '{}'. Must be one of: {}",
            status,
            VALID_STATUSES.join(", ")
        )));
    }

    let mut tx = db.begin()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    for id in ids {
        sqlx::query(
            "UPDATE incidents SET status = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?"
        )
        .bind(status)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub async fn bulk_delete_incidents(db: &SqlitePool, ids: &[String]) -> AppResult<i64> {
    if ids.is_empty() {
        return Ok(0);
    }
    if ids.len() > 100 {
        return Err(AppError::Validation("Cannot bulk delete more than 100 incidents at once".into()));
    }

    let mut tx = db.begin()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut count: i64 = 0;
    for id in ids {
        let result = sqlx::query(
            "UPDATE incidents SET deleted_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ? AND deleted_at IS NULL"
        )
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        count += result.rows_affected() as i64;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(count)
}

// Action items

pub async fn insert_action_item(
    db: &SqlitePool,
    id: &str,
    req: &CreateActionItemRequest,
) -> AppResult<ActionItem> {
    sqlx::query(
        "INSERT INTO action_items (id, incident_id, title, description, status, owner, due_date) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(id)
    .bind(&req.incident_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.status)
    .bind(&req.owner)
    .bind(&req.due_date)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_action_item_by_id(db, id).await
}

pub async fn update_action_item(
    db: &SqlitePool,
    id: &str,
    req: &UpdateActionItemRequest,
) -> AppResult<ActionItem> {
    let existing = get_action_item_by_id(db, id).await?;

    let title = req.title.as_ref().unwrap_or(&existing.title);
    let description = req.description.as_ref().unwrap_or(&existing.description);
    let status = req.status.as_ref().unwrap_or(&existing.status);
    let owner = req.owner.as_ref().unwrap_or(&existing.owner);
    let due_date = if req.due_date.is_some() {
        &req.due_date
    } else {
        &existing.due_date
    };

    sqlx::query(
        "UPDATE action_items SET title=?, description=?, status=?, owner=?, due_date=?, updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?"
    )
    .bind(title)
    .bind(description)
    .bind(status)
    .bind(owner)
    .bind(due_date)
    .bind(id)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_action_item_by_id(db, id).await
}

pub async fn delete_action_item(db: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM action_items WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Action item '{}' not found", id)));
    }
    Ok(())
}

pub async fn get_action_item_by_id(db: &SqlitePool, id: &str) -> AppResult<ActionItem> {
    let row = sqlx::query("SELECT * FROM action_items WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Action item '{}' not found", id)))?;

    Ok(parse_action_item(&row))
}

pub async fn list_action_items(
    db: &SqlitePool,
    incident_id: Option<&str>,
) -> AppResult<Vec<ActionItem>> {
    let rows = if let Some(iid) = incident_id {
        sqlx::query("SELECT a.*, NULL as incident_title FROM action_items a WHERE a.incident_id = ? ORDER BY a.created_at ASC")
            .bind(iid)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        sqlx::query("SELECT a.*, i.title as incident_title FROM action_items a JOIN incidents i ON a.incident_id = i.id WHERE i.deleted_at IS NULL ORDER BY CASE WHEN a.due_date IS NOT NULL AND a.due_date < strftime('%Y-%m-%dT%H:%M:%SZ', 'now') AND a.status != 'Done' THEN 0 ELSE 1 END, a.due_date ASC, a.created_at ASC")
            .fetch_all(db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    };

    Ok(rows.iter().map(parse_action_item).collect())
}

fn parse_incident(row: &sqlx::sqlite::SqliteRow) -> Incident {
    let severity: String = row.get("severity");
    let impact: String = row.get("impact");
    let priority = compute_priority(&severity, &impact);

    Incident {
        id: row.get("id"),
        title: row.get("title"),
        service_id: row.get("service_id"),
        service_name: row.get::<Option<String>, _>("service_name").unwrap_or_else(|| "Unknown Service".to_string()),
        severity,
        impact,
        priority,
        status: row.get("status"),
        started_at: row.get("started_at"),
        detected_at: row.get("detected_at"),
        acknowledged_at: row.get("acknowledged_at"),
        first_response_at: row.get("first_response_at"),
        mitigation_started_at: row.get("mitigation_started_at"),
        responded_at: row.get("responded_at"),
        resolved_at: row.get("resolved_at"),
        reopened_at: row.get("reopened_at"),
        reopen_count: row.get::<Option<i64>, _>("reopen_count").unwrap_or(0),
        duration_minutes: row.get("duration_minutes"),
        root_cause: row.get::<Option<String>, _>("root_cause").unwrap_or_default(),
        resolution: row.get::<Option<String>, _>("resolution").unwrap_or_default(),
        tickets_submitted: row.get::<Option<i64>, _>("tickets_submitted").unwrap_or(0),
        affected_users: row.get::<Option<i64>, _>("affected_users").unwrap_or(0),
        is_recurring: row.get::<bool, _>("is_recurring"),
        recurrence_of: row.get("recurrence_of"),
        lessons_learned: row.get::<Option<String>, _>("lessons_learned").unwrap_or_default(),
        action_items: row.get::<Option<String>, _>("action_items").unwrap_or_default(),
        external_ref: row.get::<Option<String>, _>("external_ref").unwrap_or_default(),
        notes: row.get::<Option<String>, _>("notes").unwrap_or_default(),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn parse_action_item(row: &sqlx::sqlite::SqliteRow) -> ActionItem {
    ActionItem {
        id: row.get("id"),
        incident_id: row.get("incident_id"),
        title: row.get("title"),
        description: row.get::<Option<String>, _>("description").unwrap_or_default(),
        status: row.get::<Option<String>, _>("status").unwrap_or_else(|| "Open".to_string()),
        owner: row.get::<Option<String>, _>("owner").unwrap_or_default(),
        due_date: row.get("due_date"),
        incident_title: row.get::<Option<String>, _>("incident_title"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
