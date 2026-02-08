use sqlx::{Row, SqlitePool};
use tauri::State;

use crate::db::queries::audit;
use crate::error::AppError;
use crate::models::incident::IncidentFilters;
use crate::models::priority::{Impact, Severity, calculate_priority};

/// Sanitize a CSV field value to prevent CSV injection.
/// Prefixes with a single quote if the value starts with =, +, -, or @.
fn sanitize_csv_field(value: &str) -> String {
    if value.starts_with('=')
        || value.starts_with('+')
        || value.starts_with('-')
        || value.starts_with('@')
    {
        format!("'{}", value)
    } else {
        value.to_string()
    }
}

/// Build a filtered query for incidents based on IncidentFilters.
fn build_filtered_query(filters: &IncidentFilters) -> (String, Vec<String>) {
    let mut sql = String::from(
        "SELECT i.*, s.name as service_name FROM incidents i \
         LEFT JOIN services s ON i.service_id = s.id \
         WHERE i.deleted_at IS NULL",
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
    if let Some(ref date_from) = filters.date_from {
        sql.push_str(" AND i.started_at >= ?");
        binds.push(date_from.clone());
    }
    if let Some(ref date_to) = filters.date_to {
        sql.push_str(" AND i.started_at <= ?");
        binds.push(date_to.clone());
    }

    sql.push_str(" ORDER BY i.started_at DESC");

    (sql, binds)
}

#[tauri::command]
pub async fn export_incidents_csv(
    db: State<'_, SqlitePool>,
    filters_json: String,
) -> Result<String, AppError> {
    let filters: IncidentFilters =
        serde_json::from_str(&filters_json).unwrap_or_default();

    let (sql, binds) = build_filtered_query(&filters);

    let mut query = sqlx::query(&sql);
    for bind in &binds {
        query = query.bind(bind);
    }

    let rows = query
        .fetch_all(&*db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let temp_dir = std::env::temp_dir();
    let filename = format!("incidents_export_{}.csv", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    let path = temp_dir.join(&filename);

    let mut wtr = csv::Writer::from_path(&path)
        .map_err(|e| AppError::Csv(e.to_string()))?;

    // Write header row
    wtr.write_record([
        "ID",
        "Title",
        "Service ID",
        "Service Name",
        "Severity",
        "Impact",
        "Priority",
        "Status",
        "Started At",
        "Detected At",
        "Acknowledged At",
        "First Response At",
        "Mitigation Started At",
        "Responded At",
        "Resolved At",
        "Reopened At",
        "Reopen Count",
        "Duration (minutes)",
        "Root Cause",
        "Resolution",
        "Tickets Submitted",
        "Affected Users",
        "Is Recurring",
        "Recurrence Of",
        "Lessons Learned",
        "Action Items",
        "External Ref",
        "Notes",
        "Created At",
        "Updated At",
    ])
    .map_err(|e| AppError::Csv(e.to_string()))?;

    for row in &rows {
        let severity: String = row.get("severity");
        let impact: String = row.get("impact");
        let sev = Severity::from_str(&severity).unwrap_or(Severity::Medium);
        let imp = Impact::from_str(&impact).unwrap_or(Impact::Medium);
        let priority = calculate_priority(&sev, &imp).to_string();

        let id: String = row.get("id");
        let title: String = row.get("title");
        let service_id: String = row.get("service_id");
        let service_name: String = row
            .get::<Option<String>, _>("service_name")
            .unwrap_or_else(|| "Unknown".to_string());
        let status: String = row.get("status");
        let started_at: String = row.get("started_at");
        let detected_at: String = row.get("detected_at");
        let acknowledged_at: String = row
            .get::<Option<String>, _>("acknowledged_at")
            .unwrap_or_default();
        let first_response_at: String = row
            .get::<Option<String>, _>("first_response_at")
            .unwrap_or_default();
        let mitigation_started_at: String = row
            .get::<Option<String>, _>("mitigation_started_at")
            .unwrap_or_default();
        let responded_at: String = row
            .get::<Option<String>, _>("responded_at")
            .unwrap_or_default();
        let resolved_at: String = row
            .get::<Option<String>, _>("resolved_at")
            .unwrap_or_default();
        let reopened_at: String = row
            .get::<Option<String>, _>("reopened_at")
            .unwrap_or_default();
        let reopen_count: i64 = row.get::<Option<i64>, _>("reopen_count").unwrap_or(0);
        let duration_minutes: String = row
            .get::<Option<i64>, _>("duration_minutes")
            .map(|d| d.to_string())
            .unwrap_or_default();
        let root_cause: String = row
            .get::<Option<String>, _>("root_cause")
            .unwrap_or_default();
        let resolution: String = row
            .get::<Option<String>, _>("resolution")
            .unwrap_or_default();
        let tickets_submitted: i64 =
            row.get::<Option<i64>, _>("tickets_submitted").unwrap_or(0);
        let affected_users: i64 =
            row.get::<Option<i64>, _>("affected_users").unwrap_or(0);
        let is_recurring: bool = row.get::<bool, _>("is_recurring");
        let recurrence_of: String = row
            .get::<Option<String>, _>("recurrence_of")
            .unwrap_or_default();
        let lessons_learned: String = row
            .get::<Option<String>, _>("lessons_learned")
            .unwrap_or_default();
        let action_items: String = row
            .get::<Option<String>, _>("action_items")
            .unwrap_or_default();
        let external_ref: String = row
            .get::<Option<String>, _>("external_ref")
            .unwrap_or_default();
        let notes: String = row
            .get::<Option<String>, _>("notes")
            .unwrap_or_default();
        let created_at: String = row.get("created_at");
        let updated_at: String = row.get("updated_at");

        wtr.write_record([
            &sanitize_csv_field(&id),
            &sanitize_csv_field(&title),
            &sanitize_csv_field(&service_id),
            &sanitize_csv_field(&service_name),
            &sanitize_csv_field(&severity),
            &sanitize_csv_field(&impact),
            &sanitize_csv_field(&priority),
            &sanitize_csv_field(&status),
            &sanitize_csv_field(&started_at),
            &sanitize_csv_field(&detected_at),
            &sanitize_csv_field(&acknowledged_at),
            &sanitize_csv_field(&first_response_at),
            &sanitize_csv_field(&mitigation_started_at),
            &sanitize_csv_field(&responded_at),
            &sanitize_csv_field(&resolved_at),
            &sanitize_csv_field(&reopened_at),
            &reopen_count.to_string(),
            &duration_minutes,
            &sanitize_csv_field(&root_cause),
            &sanitize_csv_field(&resolution),
            &tickets_submitted.to_string(),
            &affected_users.to_string(),
            &is_recurring.to_string(),
            &sanitize_csv_field(&recurrence_of),
            &sanitize_csv_field(&lessons_learned),
            &sanitize_csv_field(&action_items),
            &sanitize_csv_field(&external_ref),
            &sanitize_csv_field(&notes),
            &sanitize_csv_field(&created_at),
            &sanitize_csv_field(&updated_at),
        ])
        .map_err(|e| AppError::Csv(e.to_string()))?;
    }

    wtr.flush().map_err(|e| AppError::Csv(e.to_string()))?;

    let path_str = path
        .to_str()
        .ok_or_else(|| AppError::Internal("Invalid path encoding".into()))?
        .to_string();

    let _ = audit::insert_audit_entry(
        &*db,
        "export",
        "csv",
        "created",
        &format!("Exported {} incidents to CSV", rows.len()),
        "",
    )
    .await;

    Ok(path_str)
}

#[tauri::command]
pub async fn export_incidents_json(
    db: State<'_, SqlitePool>,
    filters_json: String,
) -> Result<String, AppError> {
    let filters: IncidentFilters =
        serde_json::from_str(&filters_json).unwrap_or_default();

    let (sql, binds) = build_filtered_query(&filters);

    let mut query = sqlx::query(&sql);
    for bind in &binds {
        query = query.bind(bind);
    }

    let rows = query
        .fetch_all(&*db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Build JSON array from rows
    let mut incidents: Vec<serde_json::Value> = Vec::with_capacity(rows.len());
    for row in &rows {
        let severity: String = row.get("severity");
        let impact: String = row.get("impact");
        let sev = Severity::from_str(&severity).unwrap_or(Severity::Medium);
        let imp = Impact::from_str(&impact).unwrap_or(Impact::Medium);
        let priority = calculate_priority(&sev, &imp).to_string();

        let incident = serde_json::json!({
            "id": row.get::<String, _>("id"),
            "title": row.get::<String, _>("title"),
            "service_id": row.get::<String, _>("service_id"),
            "service_name": row.get::<Option<String>, _>("service_name").unwrap_or_else(|| "Unknown".to_string()),
            "severity": severity,
            "impact": impact,
            "priority": priority,
            "status": row.get::<String, _>("status"),
            "started_at": row.get::<String, _>("started_at"),
            "detected_at": row.get::<String, _>("detected_at"),
            "acknowledged_at": row.get::<Option<String>, _>("acknowledged_at"),
            "first_response_at": row.get::<Option<String>, _>("first_response_at"),
            "mitigation_started_at": row.get::<Option<String>, _>("mitigation_started_at"),
            "responded_at": row.get::<Option<String>, _>("responded_at"),
            "resolved_at": row.get::<Option<String>, _>("resolved_at"),
            "reopened_at": row.get::<Option<String>, _>("reopened_at"),
            "reopen_count": row.get::<Option<i64>, _>("reopen_count").unwrap_or(0),
            "duration_minutes": row.get::<Option<i64>, _>("duration_minutes"),
            "root_cause": row.get::<Option<String>, _>("root_cause").unwrap_or_default(),
            "resolution": row.get::<Option<String>, _>("resolution").unwrap_or_default(),
            "tickets_submitted": row.get::<Option<i64>, _>("tickets_submitted").unwrap_or(0),
            "affected_users": row.get::<Option<i64>, _>("affected_users").unwrap_or(0),
            "is_recurring": row.get::<bool, _>("is_recurring"),
            "recurrence_of": row.get::<Option<String>, _>("recurrence_of"),
            "lessons_learned": row.get::<Option<String>, _>("lessons_learned").unwrap_or_default(),
            "action_items": row.get::<Option<String>, _>("action_items").unwrap_or_default(),
            "external_ref": row.get::<Option<String>, _>("external_ref").unwrap_or_default(),
            "notes": row.get::<Option<String>, _>("notes").unwrap_or_default(),
            "created_at": row.get::<String, _>("created_at"),
            "updated_at": row.get::<String, _>("updated_at"),
        });
        incidents.push(incident);
    }

    let json_str = serde_json::to_string_pretty(&incidents)?;

    let temp_dir = std::env::temp_dir();
    let filename = format!(
        "incidents_export_{}.json",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let path = temp_dir.join(&filename);

    tokio::fs::write(&path, json_str.as_bytes())
        .await
        .map_err(|e| AppError::Io(e))?;

    let path_str = path
        .to_str()
        .ok_or_else(|| AppError::Internal("Invalid path encoding".into()))?
        .to_string();

    let _ = audit::insert_audit_entry(
        &*db,
        "export",
        "json",
        "created",
        &format!("Exported {} incidents to JSON", incidents.len()),
        "",
    )
    .await;

    Ok(path_str)
}
