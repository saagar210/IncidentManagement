use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::incident::{
    ActionItem, CreateActionItemRequest, CreateIncidentRequest, Incident, IncidentFilters,
    UpdateActionItemRequest, UpdateIncidentRequest,
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
    sqlx::query(
        "INSERT INTO incidents (id, title, service_id, severity, impact, status, started_at, detected_at, responded_at, resolved_at, root_cause, resolution, tickets_submitted, affected_users, is_recurring, recurrence_of, lessons_learned, action_items, external_ref, notes) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(id)
    .bind(&req.title)
    .bind(&req.service_id)
    .bind(&req.severity)
    .bind(&req.impact)
    .bind(&req.status)
    .bind(&req.started_at)
    .bind(&req.detected_at)
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

    let title = req.title.as_ref().unwrap_or(&existing.title);
    let service_id = req.service_id.as_ref().unwrap_or(&existing.service_id);
    let severity = req.severity.as_ref().unwrap_or(&existing.severity);
    let impact = req.impact.as_ref().unwrap_or(&existing.impact);
    let status = req.status.as_ref().unwrap_or(&existing.status);
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

    // Handle optional fields -- use request value if Some, else keep existing
    let responded_at = if req.responded_at.is_some() {
        &req.responded_at
    } else {
        &existing.responded_at
    };
    let resolved_at = if req.resolved_at.is_some() {
        &req.resolved_at
    } else {
        &existing.resolved_at
    };
    let recurrence_of = if req.recurrence_of.is_some() {
        &req.recurrence_of
    } else {
        &existing.recurrence_of
    };

    sqlx::query(
        "UPDATE incidents SET title=?, service_id=?, severity=?, impact=?, status=?, started_at=?, detected_at=?, responded_at=?, resolved_at=?, root_cause=?, resolution=?, tickets_submitted=?, affected_users=?, is_recurring=?, recurrence_of=?, lessons_learned=?, action_items=?, external_ref=?, notes=?, updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?"
    )
    .bind(title)
    .bind(service_id)
    .bind(severity)
    .bind(impact)
    .bind(status)
    .bind(started_at)
    .bind(detected_at)
    .bind(responded_at)
    .bind(resolved_at)
    .bind(root_cause)
    .bind(resolution)
    .bind(tickets)
    .bind(affected)
    .bind(recurring)
    .bind(recurrence_of)
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
    let result = sqlx::query("DELETE FROM incidents WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Incident '{}' not found", id)));
    }
    Ok(())
}

pub async fn get_incident_by_id(db: &SqlitePool, id: &str) -> AppResult<Incident> {
    let row = sqlx::query(
        "SELECT i.*, s.name as service_name FROM incidents i LEFT JOIN services s ON i.service_id = s.id WHERE i.id = ?"
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
        "SELECT i.*, s.name as service_name FROM incidents i LEFT JOIN services s ON i.service_id = s.id WHERE 1=1",
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
    let pattern = format!("%{}%", query);
    let rows = sqlx::query(
        "SELECT i.*, s.name as service_name FROM incidents i LEFT JOIN services s ON i.service_id = s.id WHERE i.title LIKE ?1 OR i.root_cause LIKE ?1 OR i.resolution LIKE ?1 OR i.notes LIKE ?1 OR i.external_ref LIKE ?1 ORDER BY i.started_at DESC"
    )
    .bind(&pattern)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_incident).collect())
}

pub async fn bulk_update_status(db: &SqlitePool, ids: &[String], status: &str) -> AppResult<()> {
    for id in ids {
        sqlx::query(
            "UPDATE incidents SET status = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?"
        )
        .bind(status)
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }
    Ok(())
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
        sqlx::query("SELECT * FROM action_items WHERE incident_id = ? ORDER BY created_at ASC")
            .bind(iid)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        sqlx::query("SELECT * FROM action_items ORDER BY created_at ASC")
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
        service_name: row.get::<Option<String>, _>("service_name").unwrap_or_default(),
        severity,
        impact,
        priority,
        status: row.get("status"),
        started_at: row.get("started_at"),
        detected_at: row.get("detected_at"),
        responded_at: row.get("responded_at"),
        resolved_at: row.get("resolved_at"),
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
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
