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

    let mut tx = db
        .begin()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    for id in ids {
        let existing = sqlx::query(
            "SELECT status, acknowledged_at, resolved_at, reopened_at, reopen_count FROM incidents WHERE id = ? AND deleted_at IS NULL"
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Incident '{}' not found", id)))?;

        let existing_status: String = existing.get("status");
        let existing_acknowledged_at: Option<String> = existing.get("acknowledged_at");
        let existing_resolved_at: Option<String> = existing.get("resolved_at");
        let existing_reopened_at: Option<String> = existing.get("reopened_at");
        let existing_reopen_count: i64 = existing.get("reopen_count");

        let status_changed = status != existing_status;
        if status_changed {
            let allowed = allowed_transitions(&existing_status);
            if !allowed.contains(&status) {
                return Err(AppError::Validation(format!(
                    "Cannot transition from '{}' to '{}'. Allowed: {}",
                    existing_status,
                    status,
                    allowed.join(", ")
                )));
            }
        }

        let reopen_count = if status_changed && is_reopen(&existing_status, status) {
            existing_reopen_count + 1
        } else {
            existing_reopen_count
        };
        let reopened_at = if status_changed && is_reopen(&existing_status, status) {
            Some(now.clone())
        } else {
            existing_reopened_at
        };
        let acknowledged_at =
            if status_changed && status == "Acknowledged" && existing_acknowledged_at.is_none() {
                Some(now.clone())
            } else {
                existing_acknowledged_at
            };
        let resolved_at = if status_changed && status == "Resolved" && existing_resolved_at.is_none() {
            Some(now.clone())
        } else {
            existing_resolved_at
        };

        sqlx::query(
            "UPDATE incidents SET status = ?, acknowledged_at = ?, resolved_at = ?, reopened_at = ?, reopen_count = ?, updated_at = ? WHERE id = ?"
        )
        .bind(status)
        .bind(acknowledged_at)
        .bind(resolved_at)
        .bind(reopened_at)
        .bind(reopen_count)
        .bind(&now)
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
        "INSERT INTO action_items (id, incident_id, title, description, status, owner, due_date, outcome_notes) VALUES (?, ?, ?, ?, ?, ?, ?, '')"
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
    let outcome_notes = req.outcome_notes.as_ref().unwrap_or(&existing.outcome_notes);
    let due_date = if req.due_date.is_some() {
        &req.due_date
    } else {
        &existing.due_date
    };

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let completed_at = if status == "Done" && existing.status != "Done" {
        Some(now.clone())
    } else if status != "Done" && existing.status == "Done" {
        None
    } else {
        existing.completed_at.clone()
    };

    let validated_at = match req.validated {
        Some(true) => Some(now.clone()),
        Some(false) => None,
        None => existing.validated_at.clone(),
    };

    sqlx::query(
        "UPDATE action_items
         SET title=?,
             description=?,
             status=?,
             owner=?,
             due_date=?,
             completed_at=?,
             outcome_notes=?,
             validated_at=?,
             updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now')
         WHERE id=?"
    )
    .bind(title)
    .bind(description)
    .bind(status)
    .bind(owner)
    .bind(due_date)
    .bind(&completed_at)
    .bind(outcome_notes)
    .bind(&validated_at)
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
    // Keep this shape consistent with list_action_items(), which always provides
    // an incident_title column (NULL when not joined).
    let row = sqlx::query("SELECT a.*, NULL as incident_title FROM action_items a WHERE a.id = ?")
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
        completed_at: row.get("completed_at"),
        outcome_notes: row.get::<Option<String>, _>("outcome_notes").unwrap_or_default(),
        validated_at: row.get("validated_at"),
        incident_title: row.get::<Option<String>, _>("incident_title"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::{bulk_update_status, get_incident_by_id, insert_incident, insert_action_item, update_action_item};
    use crate::db::migrations::run_migrations;
    use crate::models::incident::{CreateActionItemRequest, CreateIncidentRequest, UpdateActionItemRequest};
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
    use std::str::FromStr;
    use tempfile::tempdir;

    async fn setup_db() -> (tempfile::TempDir, sqlx::SqlitePool, String) {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("incidents-query-tests.db");
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
        let service_id: String = sqlx::query_scalar("SELECT id FROM services LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("seeded service");
        (dir, pool, service_id)
    }

    fn make_create_request(service_id: &str, status: &str) -> CreateIncidentRequest {
        CreateIncidentRequest {
            title: "Bulk Update Test".into(),
            service_id: service_id.to_string(),
            severity: "High".into(),
            impact: "High".into(),
            status: status.to_string(),
            started_at: "2026-01-01T10:00:00Z".into(),
            detected_at: "2026-01-01T10:01:00Z".into(),
            acknowledged_at: None,
            first_response_at: None,
            mitigation_started_at: None,
            responded_at: None,
            resolved_at: if status == "Resolved" {
                Some("2026-01-01T11:00:00Z".into())
            } else {
                None
            },
            root_cause: String::new(),
            resolution: String::new(),
            tickets_submitted: 0,
            affected_users: 0,
            is_recurring: false,
            recurrence_of: None,
            lessons_learned: String::new(),
            action_items: String::new(),
            external_ref: String::new(),
            notes: String::new(),
        }
    }

    async fn seed_incident_with_action_item(
        pool: &sqlx::SqlitePool,
        service_id: &str,
        incident_id: &str,
        action_item_id: &str,
    ) {
        let request = make_create_request(service_id, "Active");
        insert_incident(pool, incident_id, &request)
            .await
            .expect("insert incident");

        insert_action_item(
            pool,
            action_item_id,
            &CreateActionItemRequest {
                incident_id: incident_id.to_string(),
                title: "Write incident review playbook".to_string(),
                description: "".to_string(),
                status: "Open".to_string(),
                owner: "".to_string(),
                due_date: None,
            },
        )
        .await
        .expect("insert action item");
    }

    #[tokio::test]
    async fn bulk_update_status_rejects_invalid_transition() {
        let (_dir, pool, service_id) = setup_db().await;
        let request = make_create_request(&service_id, "Active");
        insert_incident(&pool, "inc-test-1", &request)
            .await
            .expect("insert incident");

        let err = bulk_update_status(&pool, &["inc-test-1".to_string()], "Post-Mortem")
            .await
            .expect_err("invalid transition should fail");

        assert!(format!("{}", err).contains("Cannot transition"));
    }

    #[tokio::test]
    async fn bulk_update_status_sets_reopen_metadata() {
        let (_dir, pool, service_id) = setup_db().await;
        let request = make_create_request(&service_id, "Resolved");
        insert_incident(&pool, "inc-test-2", &request)
            .await
            .expect("insert incident");

        bulk_update_status(&pool, &["inc-test-2".to_string()], "Active")
            .await
            .expect("bulk update");

        let updated = get_incident_by_id(&pool, "inc-test-2")
            .await
            .expect("get incident");
        assert_eq!(updated.status, "Active");
        assert_eq!(updated.reopen_count, 1);
        assert!(updated.reopened_at.is_some());
    }

    #[tokio::test]
    async fn action_item_completed_at_sets_and_clears() {
        let (_dir, pool, service_id) = setup_db().await;
        seed_incident_with_action_item(&pool, &service_id, "inc-ai-1", "ai-test-1").await;

        let done = update_action_item(
            &pool,
            "ai-test-1",
            &UpdateActionItemRequest {
                title: None,
                description: None,
                status: Some("Done".to_string()),
                owner: None,
                due_date: None,
                outcome_notes: Some("Updated internal docs to clarify escalation paths.".to_string()),
                validated: None,
            },
        )
        .await
        .expect("mark done");
        assert_eq!(done.status, "Done");
        assert!(done.completed_at.is_some());
        assert_eq!(
            done.outcome_notes,
            "Updated internal docs to clarify escalation paths."
        );

        let reopened = update_action_item(
            &pool,
            "ai-test-1",
            &UpdateActionItemRequest {
                title: None,
                description: None,
                status: Some("Open".to_string()),
                owner: None,
                due_date: None,
                outcome_notes: None,
                validated: Some(false),
            },
        )
        .await
        .expect("reopen");
        assert_eq!(reopened.status, "Open");
        assert!(reopened.completed_at.is_none());
    }

    #[tokio::test]
    async fn action_item_validation_toggle_sets_and_clears() {
        let (_dir, pool, service_id) = setup_db().await;
        seed_incident_with_action_item(&pool, &service_id, "inc-ai-2", "ai-test-2").await;

        let done = update_action_item(
            &pool,
            "ai-test-2",
            &UpdateActionItemRequest {
                title: None,
                description: None,
                status: Some("Done".to_string()),
                owner: None,
                due_date: None,
                outcome_notes: None,
                validated: None,
            },
        )
        .await
        .expect("mark done");
        assert!(done.completed_at.is_some());

        let validated = update_action_item(
            &pool,
            "ai-test-2",
            &UpdateActionItemRequest {
                title: None,
                description: None,
                status: None,
                owner: None,
                due_date: None,
                outcome_notes: None,
                validated: Some(true),
            },
        )
        .await
        .expect("validate");
        assert!(validated.validated_at.is_some());

        let cleared = update_action_item(
            &pool,
            "ai-test-2",
            &UpdateActionItemRequest {
                title: None,
                description: None,
                status: None,
                owner: None,
                due_date: None,
                outcome_notes: None,
                validated: Some(false),
            },
        )
        .await
        .expect("clear validation");
        assert!(cleared.validated_at.is_none());
    }
}
