use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use tauri::State;

use crate::db::queries::incidents;
use crate::db::queries::provenance;
use crate::error::AppError;
use crate::import::column_mapper::{self, ColumnMapping, MappedIncident};
use crate::import::csv_parser;
use crate::models::incident::CreateIncidentRequest;

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
    pub updated: i64,
    pub skipped: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportTemplate {
    pub id: String,
    pub name: String,
    pub column_mapping: String,
    pub source: String,
    pub schema_version: i64,
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
            updated: 0,
            skipped: 0,
            errors: vec![],
        });
    }

    let mapped = column_mapper::apply_mapping(&rows, &mapping);
    let services = load_service_names(&db).await?;

    let mut created: i64 = 0;
    let mut updated: i64 = 0;
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
        match upsert_imported_incident(&db, &service_id, incident, &file_path, idx + 1).await {
            Ok(UpsertOutcome::Created) => created += 1,
            Ok(UpsertOutcome::Updated) => updated += 1,
            Ok(UpsertOutcome::NoChange) => skipped += 1,
            Err(e) => {
                skipped += 1;
                errors.push(format!("Row {}: {}", idx + 1, e));
            }
        }
    }

    Ok(ImportResult {
        created,
        updated,
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

    // Add alias names that map to canonical services.
    let alias_rows = sqlx::query(
        r#"
        SELECT sa.alias AS alias, sa.service_id AS service_id, s.name AS service_name
        FROM service_aliases sa
        JOIN services s ON s.id = sa.service_id
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    for row in alias_rows {
        let alias: String = row.get("alias");
        let service_id: String = row.get("service_id");
        let service_name: String = row.get("service_name");
        map.entry(alias.to_lowercase())
            .or_insert((service_id, service_name));
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

    let req = CreateIncidentRequest {
        title: incident.title.clone(),
        service_id: service_id.to_string(),
        severity: incident.severity.clone(),
        impact: incident.impact.clone(),
        status: incident.status.clone(),
        started_at: incident.started_at.clone(),
        detected_at: incident.detected_at.clone(),
        acknowledged_at: None,
        first_response_at: None,
        mitigation_started_at: None,
        responded_at: incident.responded_at.clone(),
        resolved_at: incident.resolved_at.clone(),
        root_cause: incident.root_cause.clone(),
        resolution: incident.resolution.clone(),
        tickets_submitted: incident.tickets_submitted,
        affected_users: incident.affected_users,
        is_recurring: incident.is_recurring,
        recurrence_of: None,
        lessons_learned: incident.lessons_learned.clone(),
        action_items: String::new(),
        external_ref: incident.external_ref.clone(),
        notes: incident.notes.clone(),
    };
    req.validate()?;
    incidents::insert_incident(db, &id, &req).await?;

    async fn record_import_fact(
        db: &SqlitePool,
        incident_id: &str,
        field_name: &str,
        meta_json: &str,
    ) -> Result<(), AppError> {
        provenance::insert_field_provenance(
            db,
            &provenance::FieldProvenanceInsert {
                entity_type: "incident",
                entity_id: incident_id,
                field_name,
                source_type: "import",
                source_ref: "csv",
                source_version: "",
                input_hash: "",
                meta_json,
            },
        )
        .await?;
        Ok(())
    }

    // Record provenance for key imported facts.
    let meta = serde_json::json!({
        "source": "csv",
    })
    .to_string();
    record_import_fact(db, &id, "service_id", &meta).await?;
    record_import_fact(db, &id, "severity", &meta).await?;
    record_import_fact(db, &id, "impact", &meta).await?;
    record_import_fact(db, &id, "status", &meta).await?;
    record_import_fact(db, &id, "started_at", &meta).await?;
    record_import_fact(db, &id, "detected_at", &meta).await?;
    if let Some(ref resolved_at) = incident.resolved_at {
        if !resolved_at.trim().is_empty() {
            record_import_fact(db, &id, "resolved_at", &meta).await?;
        }
    }
    if !incident.external_ref.trim().is_empty() {
        record_import_fact(db, &id, "external_ref", &meta).await?;
    }

    Ok(())
}

enum UpsertOutcome {
    Created,
    Updated,
    NoChange,
}

async fn upsert_imported_incident(
    db: &SqlitePool,
    service_id: &str,
    incident: &MappedIncident,
    file_path: &str,
    row_number: usize,
) -> Result<UpsertOutcome, AppError> {
    let ext_ref = incident.external_ref.trim();
    if !ext_ref.is_empty() {
        let existing_id: Option<String> = sqlx::query_scalar(
            "SELECT id FROM incidents WHERE external_ref = ? AND deleted_at IS NULL LIMIT 1",
        )
        .bind(ext_ref)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(id) = existing_id {
            return update_existing_from_import(db, &id, service_id, incident, file_path, row_number).await;
        }
    }

    insert_imported_incident(db, service_id, incident).await?;
    Ok(UpsertOutcome::Created)
}

async fn update_existing_from_import(
    db: &SqlitePool,
    id: &str,
    service_id: &str,
    incident: &MappedIncident,
    file_path: &str,
    row_number: usize,
) -> Result<UpsertOutcome, AppError> {
    use crate::models::incident::UpdateIncidentRequest;

    let existing = incidents::get_incident_by_id(db, id).await?;

    // Conservative merge strategy:
    // - never overwrite non-empty text fields
    // - only fill missing facts (timestamps/service/severity/impact/status) if absent
    let mut req = UpdateIncidentRequest {
        title: None,
        service_id: None,
        severity: None,
        impact: None,
        status: None,
        started_at: None,
        detected_at: None,
        acknowledged_at: None,
        first_response_at: None,
        mitigation_started_at: None,
        responded_at: None,
        resolved_at: None,
        root_cause: None,
        resolution: None,
        tickets_submitted: None,
        affected_users: None,
        is_recurring: None,
        recurrence_of: None,
        lessons_learned: None,
        action_items: None,
        external_ref: None,
        notes: None,
    };

    let mut changed_fields: Vec<&'static str> = Vec::new();

    if existing.service_id.trim().is_empty() {
        req.service_id = Some(service_id.to_string());
        changed_fields.push("service_id");
    }
    if existing.severity.trim().is_empty() {
        req.severity = Some(incident.severity.clone());
        changed_fields.push("severity");
    }
    if existing.impact.trim().is_empty() {
        req.impact = Some(incident.impact.clone());
        changed_fields.push("impact");
    }
    if existing.status.trim().is_empty() {
        req.status = Some(incident.status.clone());
        changed_fields.push("status");
    }
    if existing.started_at.trim().is_empty() {
        req.started_at = Some(incident.started_at.clone());
        changed_fields.push("started_at");
    }
    if existing.detected_at.trim().is_empty() {
        req.detected_at = Some(incident.detected_at.clone());
        changed_fields.push("detected_at");
    }
    if existing.responded_at.is_none() {
        if let Some(ref r) = incident.responded_at {
            req.responded_at = Some(r.clone());
            changed_fields.push("responded_at");
        }
    }
    if existing.resolved_at.is_none() {
        if let Some(ref r) = incident.resolved_at {
            req.resolved_at = Some(r.clone());
            changed_fields.push("resolved_at");
        }
    }
    if existing.external_ref.trim().is_empty() && !incident.external_ref.trim().is_empty() {
        req.external_ref = Some(incident.external_ref.clone());
        changed_fields.push("external_ref");
    }

    if changed_fields.is_empty() {
        return Ok(UpsertOutcome::NoChange);
    }

    req.validate()?;
    incidents::update_incident(db, id, &req).await?;

    // Record provenance for any filled-in facts.
    let meta = serde_json::json!({
        "source": "csv",
        "file_path": file_path,
        "row": row_number
    })
    .to_string();
    for f in changed_fields {
        provenance::insert_field_provenance(
            db,
            &provenance::FieldProvenanceInsert {
                entity_type: "incident",
                entity_id: id,
                field_name: f,
                source_type: "import",
                source_ref: "csv",
                source_version: "",
                input_hash: "",
                meta_json: &meta,
            },
        )
        .await?;
    }

    Ok(UpsertOutcome::Updated)
}

fn parse_template_row(row: &sqlx::sqlite::SqliteRow) -> ImportTemplate {
    ImportTemplate {
        id: row.get("id"),
        name: row.get("name"),
        column_mapping: row.get("column_mapping"),
        source: row.get("source"),
        schema_version: row.get("schema_version"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::insert_imported_incident;
    use crate::db::migrations::run_migrations;
    use crate::import::column_mapper::MappedIncident;
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
    use std::str::FromStr;
    use tempfile::tempdir;

    async fn setup_db() -> (tempfile::TempDir, sqlx::SqlitePool) {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("import-tests.db");
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
        (dir, pool)
    }

    fn mapped_incident(overrides: impl FnOnce(&mut MappedIncident)) -> MappedIncident {
        let mut incident = MappedIncident {
            title: "CSV Imported Incident".into(),
            service_name: "PagerDuty".into(),
            severity: "High".into(),
            impact: "High".into(),
            status: "Active".into(),
            started_at: "2026-01-01T10:00:00Z".into(),
            detected_at: "2026-01-01T10:05:00Z".into(),
            responded_at: None,
            resolved_at: None,
            root_cause: String::new(),
            resolution: String::new(),
            tickets_submitted: 1,
            affected_users: 10,
            is_recurring: false,
            lessons_learned: String::new(),
            external_ref: String::new(),
            notes: String::new(),
            warnings: vec![],
            errors: vec![],
        };
        overrides(&mut incident);
        incident
    }

    #[tokio::test]
    async fn insert_imported_incident_rejects_invalid_date_order() {
        let (_dir, pool) = setup_db().await;
        let service_id: String = sqlx::query_scalar("SELECT id FROM services LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("seeded service");

        let incident = mapped_incident(|inc| {
            inc.detected_at = "2026-01-01T09:59:00Z".into();
        });

        let err = insert_imported_incident(&pool, &service_id, &incident)
            .await
            .expect_err("expected validation error");
        assert!(format!("{}", err).contains("Detected at must be on or after started at"));
    }

    #[tokio::test]
    async fn insert_imported_incident_inserts_valid_row() {
        let (_dir, pool) = setup_db().await;
        let service_id: String = sqlx::query_scalar("SELECT id FROM services LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("seeded service");

        let incident = mapped_incident(|inc| {
            inc.status = "Resolved".into();
            inc.resolved_at = Some("2026-01-01T11:00:00Z".into());
        });

        insert_imported_incident(&pool, &service_id, &incident)
            .await
            .expect("insert succeeds");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM incidents")
            .fetch_one(&pool)
            .await
            .expect("count incidents");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn insert_imported_incident_records_field_provenance() {
        let (_dir, pool) = setup_db().await;
        let service_id: String = sqlx::query_scalar("SELECT id FROM services LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("seeded service");

        let incident = mapped_incident(|inc| {
            inc.external_ref = "JIRA-123".into();
        });

        insert_imported_incident(&pool, &service_id, &incident)
            .await
            .expect("insert succeeds");

        let inc_id: String = sqlx::query_scalar("SELECT id FROM incidents LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("incident id");

        let prov_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM field_provenance WHERE entity_type = 'incident' AND entity_id = ? AND source_type = 'import'",
        )
        .bind(&inc_id)
        .fetch_one(&pool)
        .await
        .expect("provenance count");

        assert!(prov_count >= 5, "expected provenance records, got {}", prov_count);
    }
}
