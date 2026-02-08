use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use tauri::State;

use crate::error::AppError;
use crate::import::column_mapper::{self, ColumnMapping, MappedIncident};
use crate::import::csv_parser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub incidents: Vec<PreviewRow>,
    pub warnings: Vec<ImportWarning>,
    pub error_count: i64,
    pub ready_count: i64,
    pub warning_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewRow {
    pub title: String,
    pub service_name: String,
    pub severity: String,
    pub impact: String,
    pub status: String,
    pub started_at: String,
    pub detected_at: String,
    pub row_status: String, // "ready", "warning", "error"
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportWarning {
    pub row: usize,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub created: i64,
    pub skipped: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportTemplate {
    pub id: String,
    pub name: String,
    pub column_mapping: String,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn parse_csv_headers(file_path: String) -> Result<Vec<String>, AppError> {
    let headers = csv_parser::parse_csv_headers(&file_path)?;
    Ok(headers)
}

#[tauri::command]
pub async fn preview_csv_import(
    db: State<'_, SqlitePool>,
    file_path: String,
    mapping: ColumnMapping,
) -> Result<ImportPreview, AppError> {
    let rows = csv_parser::parse_csv_rows(&file_path)?;

    if rows.is_empty() {
        return Ok(ImportPreview {
            incidents: vec![],
            warnings: vec![],
            error_count: 0,
            ready_count: 0,
            warning_count: 0,
        });
    }

    let mapped = column_mapper::apply_mapping(&rows, &mapping);

    // Load services for name matching
    let services = load_service_names(&db).await?;

    let mut preview_rows = Vec::new();
    let mut all_warnings = Vec::new();
    let mut error_count: i64 = 0;
    let mut ready_count: i64 = 0;
    let mut warning_count: i64 = 0;

    for (idx, incident) in mapped.iter().enumerate() {
        let mut messages: Vec<String> = Vec::new();
        let mut row_status = "ready".to_string();

        // Check service exists
        if !incident.service_name.is_empty()
            && !services.contains_key(&incident.service_name.to_lowercase())
        {
            messages.push(format!(
                "Service '{}' not found - will need to be created or mapped",
                incident.service_name
            ));
            if row_status != "error" {
                row_status = "warning".to_string();
            }
        }

        // Collect errors from mapping
        for err in &incident.errors {
            messages.push(err.clone());
            row_status = "error".to_string();
        }

        // Collect warnings from mapping
        for warn in &incident.warnings {
            messages.push(warn.clone());
            all_warnings.push(ImportWarning {
                row: idx + 1,
                field: String::new(),
                message: warn.clone(),
            });
            if row_status == "ready" {
                row_status = "warning".to_string();
            }
        }

        match row_status.as_str() {
            "error" => error_count += 1,
            "warning" => warning_count += 1,
            _ => ready_count += 1,
        }

        preview_rows.push(PreviewRow {
            title: incident.title.clone(),
            service_name: incident.service_name.clone(),
            severity: incident.severity.clone(),
            impact: incident.impact.clone(),
            status: incident.status.clone(),
            started_at: incident.started_at.clone(),
            detected_at: incident.detected_at.clone(),
            row_status,
            messages,
        });
    }

    Ok(ImportPreview {
        incidents: preview_rows,
        warnings: all_warnings,
        error_count,
        ready_count,
        warning_count,
    })
}

#[tauri::command]
pub async fn execute_csv_import(
    db: State<'_, SqlitePool>,
    file_path: String,
    mapping: ColumnMapping,
) -> Result<ImportResult, AppError> {
    let rows = csv_parser::parse_csv_rows(&file_path)?;

    if rows.is_empty() {
        return Ok(ImportResult {
            created: 0,
            skipped: 0,
            errors: vec![],
        });
    }

    let mapped = column_mapper::apply_mapping(&rows, &mapping);
    let services = load_service_names(&db).await?;

    let mut created: i64 = 0;
    let mut skipped: i64 = 0;
    let mut errors: Vec<String> = Vec::new();

    for (idx, incident) in mapped.iter().enumerate() {
        // Skip rows with errors
        if !incident.errors.is_empty() {
            skipped += 1;
            errors.push(format!("Row {}: Skipped due to errors: {}", idx + 1, incident.errors.join("; ")));
            continue;
        }

        // Resolve service_id from name
        let service_id = match resolve_service_id(&services, &incident.service_name) {
            Some(id) => id,
            None => {
                skipped += 1;
                errors.push(format!(
                    "Row {}: Service '{}' not found",
                    idx + 1,
                    incident.service_name
                ));
                continue;
            }
        };

        // Insert the incident
        match insert_imported_incident(&db, &service_id, incident).await {
            Ok(_) => created += 1,
            Err(e) => {
                skipped += 1;
                errors.push(format!("Row {}: {}", idx + 1, e));
            }
        }
    }

    Ok(ImportResult {
        created,
        skipped,
        errors,
    })
}

#[tauri::command]
pub async fn save_import_template(
    db: State<'_, SqlitePool>,
    name: String,
    column_mapping: String,
) -> Result<ImportTemplate, AppError> {
    let id = format!("tpl-{}", uuid::Uuid::new_v4());

    sqlx::query(
        "INSERT INTO import_templates (id, name, column_mapping) VALUES (?, ?, ?)"
    )
    .bind(&id)
    .bind(&name)
    .bind(&column_mapping)
    .execute(&*db)
    .await
    .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

    let row = sqlx::query("SELECT * FROM import_templates WHERE id = ?")
        .bind(&id)
        .fetch_one(&*db)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

    Ok(parse_template_row(&row))
}

#[tauri::command]
pub async fn list_import_templates(
    db: State<'_, SqlitePool>,
) -> Result<Vec<ImportTemplate>, AppError> {
    let rows = sqlx::query("SELECT * FROM import_templates ORDER BY name")
        .fetch_all(&*db)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_template_row).collect())
}

#[tauri::command]
pub async fn delete_import_template(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM import_templates WHERE id = ?")
        .bind(&id)
        .execute(&*db)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Import template '{}' not found",
            id
        )));
    }

    Ok(())
}

// ---- Helper Functions ----

/// Load all services as a map of lowercase_name -> (id, name)
async fn load_service_names(
    db: &SqlitePool,
) -> Result<HashMap<String, (String, String)>, AppError> {
    let rows = sqlx::query("SELECT id, name FROM services")
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut map = HashMap::new();
    for row in rows {
        let id: String = row.get("id");
        let name: String = row.get("name");
        map.insert(name.to_lowercase(), (id, name));
    }
    Ok(map)
}

/// Match a service name to its ID (case-insensitive).
fn resolve_service_id(
    services: &HashMap<String, (String, String)>,
    name: &str,
) -> Option<String> {
    services
        .get(&name.to_lowercase())
        .map(|(id, _)| id.clone())
}

/// Insert a single incident from import data.
async fn insert_imported_incident(
    db: &SqlitePool,
    service_id: &str,
    incident: &MappedIncident,
) -> Result<(), AppError> {
    let id = format!("inc-{}", uuid::Uuid::new_v4());

    sqlx::query(
        "INSERT INTO incidents (id, title, service_id, severity, impact, status, started_at, detected_at, responded_at, resolved_at, root_cause, resolution, tickets_submitted, affected_users, is_recurring, lessons_learned, external_ref, notes) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&incident.title)
    .bind(service_id)
    .bind(&incident.severity)
    .bind(&incident.impact)
    .bind(&incident.status)
    .bind(&incident.started_at)
    .bind(&incident.detected_at)
    .bind(&incident.responded_at)
    .bind(&incident.resolved_at)
    .bind(&incident.root_cause)
    .bind(&incident.resolution)
    .bind(incident.tickets_submitted)
    .bind(incident.affected_users)
    .bind(incident.is_recurring)
    .bind(&incident.lessons_learned)
    .bind(&incident.external_ref)
    .bind(&incident.notes)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

fn parse_template_row(row: &sqlx::sqlite::SqliteRow) -> ImportTemplate {
    ImportTemplate {
        id: row.get("id"),
        name: row.get("name"),
        column_mapping: row.get("column_mapping"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
