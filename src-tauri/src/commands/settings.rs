use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tauri::State;

use crate::db::queries::settings;
use crate::error::AppError;
use crate::models::quarter::{QuarterConfig, UpsertQuarterRequest};

#[tauri::command]
pub async fn get_quarter_configs(
    db: State<'_, SqlitePool>,
) -> Result<Vec<QuarterConfig>, AppError> {
    settings::get_quarter_configs(&*db).await
}

#[tauri::command]
pub async fn upsert_quarter_config(
    db: State<'_, SqlitePool>,
    config: UpsertQuarterRequest,
) -> Result<QuarterConfig, AppError> {
    config.validate()?;
    settings::upsert_quarter(&*db, &config).await
}

#[tauri::command]
pub async fn delete_quarter_config(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    settings::delete_quarter(&*db, &id).await
}

#[tauri::command]
pub async fn get_setting(
    db: State<'_, SqlitePool>,
    key: String,
) -> Result<Option<String>, AppError> {
    settings::get_setting(&*db, &key).await
}

#[tauri::command]
pub async fn set_setting(
    db: State<'_, SqlitePool>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    settings::set_setting(&*db, &key, &value).await
}

// ===================== Data Export / Import =====================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupData {
    version: String,
    exported_at: String,
    services: Vec<serde_json::Value>,
    incidents: Vec<serde_json::Value>,
    action_items: Vec<serde_json::Value>,
    quarter_configs: Vec<serde_json::Value>,
    #[serde(default)]
    custom_field_definitions: Vec<serde_json::Value>,
    #[serde(default)]
    custom_field_values: Vec<serde_json::Value>,
    app_settings: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupImportResult {
    pub services: i64,
    pub incidents: i64,
    pub action_items: i64,
    pub quarter_configs: i64,
    pub custom_field_definitions: i64,
    pub custom_field_values: i64,
    pub settings: i64,
    pub errors: Vec<String>,
}

#[tauri::command]
pub async fn export_all_data(db: State<'_, SqlitePool>) -> Result<String, AppError> {
    let backup = build_backup_data(&db).await?;
    let json = serde_json::to_string_pretty(&backup)
        .map_err(|e| AppError::Internal(format!("Failed to serialize backup: {}", e)))?;
    write_backup_to_temp_file(&json).await
}

async fn build_backup_data(db: &SqlitePool) -> Result<BackupData, AppError> {
    let services = fetch_backup_services(db).await?;
    let incidents = fetch_backup_incidents(db).await?;
    let action_items = fetch_backup_action_items(db).await?;
    let quarter_configs = fetch_backup_quarter_configs(db).await?;
    let custom_field_definitions = fetch_backup_custom_field_definitions(db).await?;
    let custom_field_values = fetch_backup_custom_field_values(db).await?;
    let app_settings = fetch_backup_app_settings(db).await?;

    Ok(BackupData {
        version: "1.0".to_string(),
        exported_at: now_utc_string(),
        services,
        incidents,
        action_items,
        quarter_configs,
        custom_field_definitions,
        custom_field_values,
        app_settings,
    })
}

async fn fetch_backup_services(db: &SqlitePool) -> Result<Vec<serde_json::Value>, AppError> {
    fetch_json_rows(db, "SELECT * FROM services ORDER BY name", |r| {
        serde_json::json!({
            "id": r.get::<String, _>("id"),
            "name": r.get::<String, _>("name"),
            "category": r.get::<String, _>("category"),
            "default_severity": r.get::<String, _>("default_severity"),
            "default_impact": r.get::<String, _>("default_impact"),
            "description": r.get::<Option<String>, _>("description").unwrap_or_default(),
            "owner": r.get::<Option<String>, _>("owner").unwrap_or_default(),
            "tier": r.get::<Option<String>, _>("tier").unwrap_or_else(|| "T3".to_string()),
            "runbook": r.get::<Option<String>, _>("runbook").unwrap_or_default(),
            "is_active": r.get::<bool, _>("is_active"),
            "created_at": r.get::<String, _>("created_at"),
            "updated_at": r.get::<String, _>("updated_at"),
        })
    })
    .await
}

async fn fetch_backup_incidents(db: &SqlitePool) -> Result<Vec<serde_json::Value>, AppError> {
    fetch_json_rows(db, "SELECT * FROM incidents ORDER BY started_at DESC", |r| {
        serde_json::json!({
            "id": r.get::<String, _>("id"),
            "title": r.get::<String, _>("title"),
            "service_id": r.get::<String, _>("service_id"),
            "severity": r.get::<String, _>("severity"),
            "impact": r.get::<String, _>("impact"),
            "status": r.get::<String, _>("status"),
            "started_at": r.get::<String, _>("started_at"),
            "detected_at": r.get::<String, _>("detected_at"),
            "acknowledged_at": r.get::<Option<String>, _>("acknowledged_at"),
            "first_response_at": r.get::<Option<String>, _>("first_response_at"),
            "mitigation_started_at": r.get::<Option<String>, _>("mitigation_started_at"),
            "responded_at": r.get::<Option<String>, _>("responded_at"),
            "resolved_at": r.get::<Option<String>, _>("resolved_at"),
            "reopened_at": r.get::<Option<String>, _>("reopened_at"),
            "reopen_count": r.get::<i64, _>("reopen_count"),
            "root_cause": r.get::<Option<String>, _>("root_cause").unwrap_or_default(),
            "resolution": r.get::<Option<String>, _>("resolution").unwrap_or_default(),
            "tickets_submitted": r.get::<Option<i64>, _>("tickets_submitted").unwrap_or(0),
            "affected_users": r.get::<Option<i64>, _>("affected_users").unwrap_or(0),
            "is_recurring": r.get::<bool, _>("is_recurring"),
            "recurrence_of": r.get::<Option<String>, _>("recurrence_of"),
            "lessons_learned": r.get::<Option<String>, _>("lessons_learned").unwrap_or_default(),
            "action_items": r.get::<Option<String>, _>("action_items").unwrap_or_default(),
            "external_ref": r.get::<Option<String>, _>("external_ref").unwrap_or_default(),
            "notes": r.get::<Option<String>, _>("notes").unwrap_or_default(),
            "created_at": r.get::<String, _>("created_at"),
            "updated_at": r.get::<String, _>("updated_at"),
        })
    })
    .await
}

async fn fetch_backup_action_items(db: &SqlitePool) -> Result<Vec<serde_json::Value>, AppError> {
    fetch_json_rows(db, "SELECT * FROM action_items ORDER BY created_at", |r| {
        serde_json::json!({
            "id": r.get::<String, _>("id"),
            "incident_id": r.get::<String, _>("incident_id"),
            "title": r.get::<String, _>("title"),
            "description": r.get::<Option<String>, _>("description").unwrap_or_default(),
            "status": r.get::<Option<String>, _>("status").unwrap_or_else(|| "Open".to_string()),
            "owner": r.get::<Option<String>, _>("owner").unwrap_or_default(),
            "due_date": r.get::<Option<String>, _>("due_date"),
            "created_at": r.get::<String, _>("created_at"),
            "updated_at": r.get::<String, _>("updated_at"),
        })
    })
    .await
}

async fn fetch_backup_quarter_configs(db: &SqlitePool) -> Result<Vec<serde_json::Value>, AppError> {
    fetch_json_rows(
        db,
        "SELECT * FROM quarter_config ORDER BY fiscal_year DESC, quarter_number DESC",
        |r| {
            serde_json::json!({
                "id": r.get::<String, _>("id"),
                "fiscal_year": r.get::<i64, _>("fiscal_year"),
                "quarter_number": r.get::<i64, _>("quarter_number"),
                "start_date": r.get::<String, _>("start_date"),
                "end_date": r.get::<String, _>("end_date"),
                "label": r.get::<String, _>("label"),
                "created_at": r.get::<String, _>("created_at"),
            })
        },
    )
    .await
}

async fn fetch_backup_custom_field_definitions(
    db: &SqlitePool,
) -> Result<Vec<serde_json::Value>, AppError> {
    fetch_json_rows(
        db,
        "SELECT * FROM custom_field_definitions ORDER BY display_order ASC, name ASC",
        |r| {
            serde_json::json!({
                "id": r.get::<String, _>("id"),
                "name": r.get::<String, _>("name"),
                "field_type": r.get::<String, _>("field_type"),
                "options": r.get::<Option<String>, _>("options").unwrap_or_default(),
                "display_order": r.get::<i64, _>("display_order"),
                "created_at": r.get::<String, _>("created_at"),
                "updated_at": r.get::<String, _>("updated_at"),
            })
        },
    )
    .await
}

async fn fetch_backup_custom_field_values(
    db: &SqlitePool,
) -> Result<Vec<serde_json::Value>, AppError> {
    fetch_json_rows(
        db,
        "SELECT * FROM custom_field_values ORDER BY incident_id, field_id",
        |r| {
            serde_json::json!({
                "incident_id": r.get::<String, _>("incident_id"),
                "field_id": r.get::<String, _>("field_id"),
                "value": r.get::<String, _>("value"),
            })
        },
    )
    .await
}

async fn fetch_backup_app_settings(db: &SqlitePool) -> Result<serde_json::Value, AppError> {
    let rows = sqlx::query("SELECT * FROM app_settings")
        .fetch_all(db)
        .await
        .map_err(map_db_error)?;

    let mut settings_map = serde_json::Map::new();
    for row in &rows {
        let key: String = Row::get(row, "key");
        let value: String = Row::get(row, "value");
        settings_map.insert(key, serde_json::Value::String(value));
    }

    Ok(serde_json::Value::Object(settings_map))
}

fn map_db_error(e: sqlx::Error) -> AppError {
    AppError::Database(e.to_string())
}

async fn fetch_json_rows<F>(
    db: &SqlitePool,
    sql: &str,
    mapper: F,
) -> Result<Vec<serde_json::Value>, AppError>
where
    F: Fn(&sqlx::sqlite::SqliteRow) -> serde_json::Value,
{
    let rows = sqlx::query(sql).fetch_all(db).await.map_err(map_db_error)?;
    Ok(rows.iter().map(mapper).collect())
}

async fn write_backup_to_temp_file(json: &str) -> Result<String, AppError> {
    let temp_dir = std::env::temp_dir();
    let file_name = format!(
        "incident_backup_{}.json",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let file_path = temp_dir.join(file_name);

    tokio::fs::write(&file_path, json)
        .await
        .map_err(|e| AppError::Io(e))?;

    file_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Internal("Failed to convert path to string".into()))
}

#[tauri::command]
pub async fn import_backup(
    db: State<'_, SqlitePool>,
    file_path: String,
) -> Result<BackupImportResult, AppError> {
    // Validate file size (max 50MB to prevent OOM)
    let metadata = tokio::fs::metadata(&file_path)
        .await
        .map_err(|e| AppError::Io(e))?;
    if metadata.len() > 50 * 1024 * 1024 {
        return Err(AppError::Validation(
            "Backup file too large (max 50MB)".into(),
        ));
    }

    let content = tokio::fs::read_to_string(&file_path)
        .await
        .map_err(|e| AppError::Io(e))?;

    let backup: BackupData = serde_json::from_str(&content)
        .map_err(|e| AppError::Internal(format!("Invalid backup file: {}", e)))?;

    if backup.version != "1.0" {
        return Err(AppError::Validation(format!(
            "Unsupported backup version: {}",
            backup.version
        )));
    }

    import_backup_data(&db, &backup).await
}

async fn import_backup_data(
    db: &SqlitePool,
    backup: &BackupData,
) -> Result<BackupImportResult, AppError> {
    let mut result = BackupImportResult {
        services: 0,
        incidents: 0,
        action_items: 0,
        quarter_configs: 0,
        custom_field_definitions: 0,
        custom_field_values: 0,
        settings: 0,
        errors: vec![],
    };

    // Import services first (incidents depend on them)
    for svc in &backup.services {
        match import_service(db, svc).await {
            Ok(_) => result.services += 1,
            Err(e) => result.errors.push(format!("Service: {}", e)),
        }
    }

    // Import custom field definitions before values
    for field in &backup.custom_field_definitions {
        match import_custom_field_definition(db, field).await {
            Ok(_) => result.custom_field_definitions += 1,
            Err(e) => result
                .errors
                .push(format!("Custom field definition: {}", e)),
        }
    }

    // Import incidents
    for inc in &backup.incidents {
        match import_incident(db, inc).await {
            Ok(_) => result.incidents += 1,
            Err(e) => result.errors.push(format!("Incident: {}", e)),
        }
    }

    // Import custom field values after incidents + definitions
    for value in &backup.custom_field_values {
        match import_custom_field_value(db, value).await {
            Ok(_) => result.custom_field_values += 1,
            Err(e) => result.errors.push(format!("Custom field value: {}", e)),
        }
    }

    // Import action items
    for ai in &backup.action_items {
        match import_action_item(db, ai).await {
            Ok(_) => result.action_items += 1,
            Err(e) => result.errors.push(format!("Action item: {}", e)),
        }
    }

    // Import quarter configs
    for qc in &backup.quarter_configs {
        match import_quarter_config(db, qc).await {
            Ok(_) => result.quarter_configs += 1,
            Err(e) => result.errors.push(format!("Quarter config: {}", e)),
        }
    }

    // Import app settings
    if let serde_json::Value::Object(map) = &backup.app_settings {
        for (key, value) in map {
            if let serde_json::Value::String(val) = value {
                match settings::set_setting(db, key, val).await {
                    Ok(_) => result.settings += 1,
                    Err(e) => result.errors.push(format!("Setting '{}': {}", key, e)),
                }
            }
        }
    }

    Ok(result)
}

// ---- Import helpers ----

async fn import_service(db: &SqlitePool, svc: &serde_json::Value) -> Result<(), AppError> {
    let id = get_str(svc, "id")?;
    let name = get_str(svc, "name")?;
    let category = get_str(svc, "category")?;
    let default_severity = get_str(svc, "default_severity")?;
    let default_impact = get_str(svc, "default_impact")?;
    let description = svc.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let owner = svc.get("owner").and_then(|v| v.as_str()).unwrap_or("");
    let tier = svc.get("tier").and_then(|v| v.as_str()).unwrap_or("T3");
    let runbook = svc.get("runbook").and_then(|v| v.as_str()).unwrap_or("");
    let is_active = svc.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true);
    let created_at = get_optional_str(svc, "created_at")
        .map(ToString::to_string)
        .unwrap_or_else(now_utc_string);
    let updated_at = get_optional_str(svc, "updated_at")
        .map(ToString::to_string)
        .unwrap_or_else(now_utc_string);

    sqlx::query(
        "INSERT OR IGNORE INTO services (id, name, category, default_severity, default_impact, description, owner, tier, runbook, is_active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&name)
    .bind(&category)
    .bind(&default_severity)
    .bind(&default_impact)
    .bind(description)
    .bind(owner)
    .bind(tier)
    .bind(runbook)
    .bind(is_active)
    .bind(created_at)
    .bind(updated_at)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

async fn import_incident(db: &SqlitePool, inc: &serde_json::Value) -> Result<(), AppError> {
    let id = get_str(inc, "id")?;
    let created_at = get_optional_str(inc, "created_at")
        .map(ToString::to_string)
        .unwrap_or_else(now_utc_string);
    let updated_at = get_optional_str(inc, "updated_at")
        .map(ToString::to_string)
        .unwrap_or_else(now_utc_string);

    sqlx::query(
        "INSERT OR IGNORE INTO incidents (id, title, service_id, severity, impact, status, started_at, detected_at, acknowledged_at, first_response_at, mitigation_started_at, responded_at, resolved_at, reopened_at, reopen_count, root_cause, resolution, tickets_submitted, affected_users, is_recurring, recurrence_of, lessons_learned, action_items, external_ref, notes, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(inc.get("title").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(inc.get("service_id").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(inc.get("severity").and_then(|v| v.as_str()).unwrap_or("Medium"))
    .bind(inc.get("impact").and_then(|v| v.as_str()).unwrap_or("Medium"))
    .bind(inc.get("status").and_then(|v| v.as_str()).unwrap_or("Resolved"))
    .bind(inc.get("started_at").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(inc.get("detected_at").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(inc.get("acknowledged_at").and_then(|v| v.as_str()))
    .bind(inc.get("first_response_at").and_then(|v| v.as_str()))
    .bind(inc.get("mitigation_started_at").and_then(|v| v.as_str()))
    .bind(inc.get("responded_at").and_then(|v| v.as_str()))
    .bind(inc.get("resolved_at").and_then(|v| v.as_str()))
    .bind(inc.get("reopened_at").and_then(|v| v.as_str()))
    .bind(inc.get("reopen_count").and_then(|v| v.as_i64()).unwrap_or(0))
    .bind(inc.get("root_cause").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(inc.get("resolution").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(inc.get("tickets_submitted").and_then(|v| v.as_i64()).unwrap_or(0))
    .bind(inc.get("affected_users").and_then(|v| v.as_i64()).unwrap_or(0))
    .bind(inc.get("is_recurring").and_then(|v| v.as_bool()).unwrap_or(false))
    .bind(inc.get("recurrence_of").and_then(|v| v.as_str()))
    .bind(inc.get("lessons_learned").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(inc.get("action_items").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(inc.get("external_ref").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(inc.get("notes").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(created_at)
    .bind(updated_at)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

async fn import_action_item(db: &SqlitePool, ai: &serde_json::Value) -> Result<(), AppError> {
    let id = get_str(ai, "id")?;
    let created_at = get_optional_str(ai, "created_at")
        .map(ToString::to_string)
        .unwrap_or_else(now_utc_string);
    let updated_at = get_optional_str(ai, "updated_at")
        .map(ToString::to_string)
        .unwrap_or_else(now_utc_string);

    sqlx::query(
        "INSERT OR IGNORE INTO action_items (id, incident_id, title, description, status, owner, due_date, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(ai.get("incident_id").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(ai.get("title").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(ai.get("description").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(ai.get("status").and_then(|v| v.as_str()).unwrap_or("Open"))
    .bind(ai.get("owner").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(ai.get("due_date").and_then(|v| v.as_str()))
    .bind(created_at)
    .bind(updated_at)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

async fn import_quarter_config(db: &SqlitePool, qc: &serde_json::Value) -> Result<(), AppError> {
    let id = get_str(qc, "id")?;
    let created_at = get_optional_str(qc, "created_at")
        .map(ToString::to_string)
        .unwrap_or_else(now_utc_string);

    sqlx::query(
        "INSERT OR IGNORE INTO quarter_config (id, fiscal_year, quarter_number, start_date, end_date, label, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(qc.get("fiscal_year").and_then(|v| v.as_i64()).unwrap_or(0))
    .bind(qc.get("quarter_number").and_then(|v| v.as_i64()).unwrap_or(1))
    .bind(qc.get("start_date").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(qc.get("end_date").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(qc.get("label").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(created_at)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

async fn import_custom_field_definition(
    db: &SqlitePool,
    field: &serde_json::Value,
) -> Result<(), AppError> {
    let id = get_str(field, "id")?;
    let name = get_str(field, "name")?;
    let field_type = get_str(field, "field_type")?;
    let options = field.get("options").and_then(|v| v.as_str()).unwrap_or("");
    let display_order = field
        .get("display_order")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let created_at = get_optional_str(field, "created_at")
        .map(ToString::to_string)
        .unwrap_or_else(now_utc_string);
    let updated_at = get_optional_str(field, "updated_at")
        .map(ToString::to_string)
        .unwrap_or_else(now_utc_string);

    sqlx::query(
        "INSERT OR IGNORE INTO custom_field_definitions (id, name, field_type, options, display_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(id)
    .bind(name)
    .bind(field_type)
    .bind(options)
    .bind(display_order)
    .bind(created_at)
    .bind(updated_at)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

async fn import_custom_field_value(
    db: &SqlitePool,
    value: &serde_json::Value,
) -> Result<(), AppError> {
    let incident_id = get_str(value, "incident_id")?;
    let field_id = get_str(value, "field_id")?;
    let field_value = value.get("value").and_then(|v| v.as_str()).unwrap_or("");

    sqlx::query(
        "INSERT OR IGNORE INTO custom_field_values (incident_id, field_id, value) VALUES (?, ?, ?)"
    )
    .bind(incident_id)
    .bind(field_id)
    .bind(field_value)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

fn get_optional_str<'a>(value: &'a serde_json::Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(|v| v.as_str())
}

fn get_str(value: &serde_json::Value, field: &str) -> Result<String, AppError> {
    value
        .get(field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Validation(format!("Missing field '{}'", field)))
}

fn now_utc_string() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        build_backup_data, import_action_item, import_backup_data, import_custom_field_definition,
        import_custom_field_value, import_incident, import_service,
    };
    use crate::db::migrations::run_migrations;
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
    use std::str::FromStr;
    use tempfile::tempdir;

    async fn setup_db() -> (tempfile::TempDir, sqlx::SqlitePool) {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("settings-tests.db");
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

    #[tokio::test]
    async fn build_backup_data_includes_custom_fields() {
        let (_dir, pool) = setup_db().await;

        sqlx::query(
            "INSERT INTO incidents (id, title, service_id, severity, impact, status, started_at, detected_at) VALUES (?, ?, (SELECT id FROM services LIMIT 1), 'High', 'High', 'Active', '2026-01-01T10:00:00Z', '2026-01-01T10:01:00Z')",
        )
        .bind("inc-bk-1")
        .bind("Backup Test Incident")
        .execute(&pool)
        .await
        .expect("insert incident");

        sqlx::query(
            "INSERT INTO custom_field_definitions (id, name, field_type, options, display_order) VALUES ('cf-1', 'Region', 'text', '', 0)"
        )
        .execute(&pool)
        .await
        .expect("insert definition");
        sqlx::query(
            "INSERT INTO custom_field_values (incident_id, field_id, value) VALUES ('inc-bk-1', 'cf-1', 'us-east-1')"
        )
        .execute(&pool)
        .await
        .expect("insert value");

        let backup = build_backup_data(&pool).await.expect("build backup");
        assert!(!backup.custom_field_definitions.is_empty());
        assert!(!backup.custom_field_values.is_empty());
    }

    #[tokio::test]
    async fn import_helpers_preserve_timestamps_and_metadata() {
        let (_dir, pool) = setup_db().await;

        let service = serde_json::json!({
            "id": "svc-import-1",
            "name": "Imported Service",
            "category": "Infrastructure",
            "default_severity": "High",
            "default_impact": "High",
            "description": "desc",
            "owner": "SRE",
            "tier": "T1",
            "runbook": "runbook",
            "is_active": true,
            "created_at": "2025-01-01T00:00:00Z",
            "updated_at": "2025-01-02T00:00:00Z"
        });
        import_service(&pool, &service).await.expect("import service");

        let incident = serde_json::json!({
            "id": "inc-import-1",
            "title": "Imported Incident",
            "service_id": "svc-import-1",
            "severity": "High",
            "impact": "High",
            "status": "Resolved",
            "started_at": "2025-01-01T10:00:00Z",
            "detected_at": "2025-01-01T10:01:00Z",
            "responded_at": "2025-01-01T10:05:00Z",
            "resolved_at": "2025-01-01T11:00:00Z",
            "root_cause": "rc",
            "resolution": "fix",
            "tickets_submitted": 2,
            "affected_users": 20,
            "is_recurring": false,
            "recurrence_of": null,
            "lessons_learned": "",
            "action_items": "",
            "external_ref": "",
            "notes": "",
            "created_at": "2025-01-01T12:00:00Z",
            "updated_at": "2025-01-01T13:00:00Z"
        });
        import_incident(&pool, &incident).await.expect("import incident");

        let action_item = serde_json::json!({
            "id": "ai-import-1",
            "incident_id": "inc-import-1",
            "title": "Follow up",
            "description": "",
            "status": "Open",
            "owner": "",
            "due_date": null,
            "created_at": "2025-01-01T14:00:00Z",
            "updated_at": "2025-01-01T15:00:00Z"
        });
        import_action_item(&pool, &action_item)
            .await
            .expect("import action item");

        let field_def = serde_json::json!({
            "id": "cf-import-1",
            "name": "Team",
            "field_type": "text",
            "options": "",
            "display_order": 1,
            "created_at": "2025-01-01T16:00:00Z",
            "updated_at": "2025-01-01T17:00:00Z"
        });
        import_custom_field_definition(&pool, &field_def)
            .await
            .expect("import custom field definition");
        let field_value = serde_json::json!({
            "incident_id": "inc-import-1",
            "field_id": "cf-import-1",
            "value": "Platform"
        });
        import_custom_field_value(&pool, &field_value)
            .await
            .expect("import custom field value");

        let service_owner: String = sqlx::query_scalar("SELECT owner FROM services WHERE id = 'svc-import-1'")
            .fetch_one(&pool)
            .await
            .expect("service owner");
        let service_created_at: String = sqlx::query_scalar("SELECT created_at FROM services WHERE id = 'svc-import-1'")
            .fetch_one(&pool)
            .await
            .expect("service created_at");
        assert_eq!(service_owner, "SRE");
        assert_eq!(service_created_at, "2025-01-01T00:00:00Z");

        let incident_created_at: String = sqlx::query_scalar("SELECT created_at FROM incidents WHERE id = 'inc-import-1'")
            .fetch_one(&pool)
            .await
            .expect("incident created_at");
        let action_item_updated_at: String = sqlx::query_scalar("SELECT updated_at FROM action_items WHERE id = 'ai-import-1'")
            .fetch_one(&pool)
            .await
            .expect("action item updated_at");
        let cf_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM custom_field_values WHERE incident_id = 'inc-import-1' AND field_id = 'cf-import-1'"
        )
        .fetch_one(&pool)
        .await
        .expect("custom field count");
        assert_eq!(incident_created_at, "2025-01-01T12:00:00Z");
        assert_eq!(action_item_updated_at, "2025-01-01T15:00:00Z");
        assert_eq!(cf_count, 1);
    }

    #[tokio::test]
    async fn backup_round_trip_restores_custom_data() {
        let (_src_dir, src_pool) = setup_db().await;

        sqlx::query(
            "INSERT INTO services (id, name, category, default_severity, default_impact, description, owner, tier, runbook, is_active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("svc-rt-1")
        .bind("Round Trip Service")
        .bind("Infrastructure")
        .bind("High")
        .bind("High")
        .bind("rt-desc")
        .bind("Platform")
        .bind("T1")
        .bind("rt-runbook")
        .bind(true)
        .bind("2025-03-01T00:00:00Z")
        .bind("2025-03-02T00:00:00Z")
        .execute(&src_pool)
        .await
        .expect("insert service");

        sqlx::query(
            "INSERT INTO incidents (id, title, service_id, severity, impact, status, started_at, detected_at, resolved_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("inc-rt-1")
        .bind("Round Trip Incident")
        .bind("svc-rt-1")
        .bind("High")
        .bind("High")
        .bind("Resolved")
        .bind("2025-03-01T10:00:00Z")
        .bind("2025-03-01T10:01:00Z")
        .bind("2025-03-01T11:00:00Z")
        .bind("2025-03-01T12:00:00Z")
        .bind("2025-03-01T13:00:00Z")
        .execute(&src_pool)
        .await
        .expect("insert incident");

        sqlx::query(
            "INSERT INTO action_items (id, incident_id, title, description, status, owner, due_date, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("ai-rt-1")
        .bind("inc-rt-1")
        .bind("Round Trip Action")
        .bind("")
        .bind("Open")
        .bind("")
        .bind(None::<String>)
        .bind("2025-03-01T14:00:00Z")
        .bind("2025-03-01T15:00:00Z")
        .execute(&src_pool)
        .await
        .expect("insert action item");

        sqlx::query(
            "INSERT INTO custom_field_definitions (id, name, field_type, options, display_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("cf-rt-1")
        .bind("Region")
        .bind("text")
        .bind("")
        .bind(0_i64)
        .bind("2025-03-01T16:00:00Z")
        .bind("2025-03-01T17:00:00Z")
        .execute(&src_pool)
        .await
        .expect("insert cf def");

        sqlx::query(
            "INSERT INTO custom_field_values (incident_id, field_id, value) VALUES (?, ?, ?)"
        )
        .bind("inc-rt-1")
        .bind("cf-rt-1")
        .bind("us-west-2")
        .execute(&src_pool)
        .await
        .expect("insert cf value");

        let backup = build_backup_data(&src_pool).await.expect("build backup");

        let (_dst_dir, dst_pool) = setup_db().await;
        let import_result = import_backup_data(&dst_pool, &backup)
            .await
            .expect("import backup data");
        assert!(import_result.errors.is_empty());
        assert!(import_result.services >= 1);
        assert!(import_result.incidents >= 1);
        assert!(import_result.action_items >= 1);
        assert!(import_result.custom_field_definitions >= 1);
        assert!(import_result.custom_field_values >= 1);

        let restored_owner: String =
            sqlx::query_scalar("SELECT owner FROM services WHERE id = 'svc-rt-1'")
                .fetch_one(&dst_pool)
                .await
                .expect("restored service");
        let restored_incident_updated_at: String =
            sqlx::query_scalar("SELECT updated_at FROM incidents WHERE id = 'inc-rt-1'")
                .fetch_one(&dst_pool)
                .await
                .expect("restored incident");
        let restored_cf_value: String = sqlx::query_scalar(
            "SELECT value FROM custom_field_values WHERE incident_id = 'inc-rt-1' AND field_id = 'cf-rt-1'",
        )
        .fetch_one(&dst_pool)
        .await
        .expect("restored custom field value");

        assert_eq!(restored_owner, "Platform");
        assert_eq!(restored_incident_updated_at, "2025-03-01T13:00:00Z");
        assert_eq!(restored_cf_value, "us-west-2");
    }
}
