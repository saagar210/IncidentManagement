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
    app_settings: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupImportResult {
    pub services: i64,
    pub incidents: i64,
    pub action_items: i64,
    pub quarter_configs: i64,
    pub settings: i64,
    pub errors: Vec<String>,
}

#[tauri::command]
pub async fn export_all_data(db: State<'_, SqlitePool>) -> Result<String, AppError> {
    // Export services
    let service_rows = sqlx::query("SELECT * FROM services ORDER BY name")
        .fetch_all(&*db)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
    let services: Vec<serde_json::Value> = service_rows
        .iter()
        .map(|r: &sqlx::sqlite::SqliteRow| {
            serde_json::json!({
                "id": r.get::<String, _>("id"),
                "name": r.get::<String, _>("name"),
                "category": r.get::<String, _>("category"),
                "default_severity": r.get::<String, _>("default_severity"),
                "default_impact": r.get::<String, _>("default_impact"),
                "description": r.get::<Option<String>, _>("description").unwrap_or_default(),
                "is_active": r.get::<bool, _>("is_active"),
                "created_at": r.get::<String, _>("created_at"),
                "updated_at": r.get::<String, _>("updated_at"),
            })
        })
        .collect();

    // Export incidents
    let incident_rows = sqlx::query("SELECT * FROM incidents ORDER BY started_at DESC")
        .fetch_all(&*db)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
    let incidents: Vec<serde_json::Value> = incident_rows
        .iter()
        .map(|r: &sqlx::sqlite::SqliteRow| {
            serde_json::json!({
                "id": r.get::<String, _>("id"),
                "title": r.get::<String, _>("title"),
                "service_id": r.get::<String, _>("service_id"),
                "severity": r.get::<String, _>("severity"),
                "impact": r.get::<String, _>("impact"),
                "status": r.get::<String, _>("status"),
                "started_at": r.get::<String, _>("started_at"),
                "detected_at": r.get::<String, _>("detected_at"),
                "responded_at": r.get::<Option<String>, _>("responded_at"),
                "resolved_at": r.get::<Option<String>, _>("resolved_at"),
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
        .collect();

    // Export action items
    let action_rows = sqlx::query("SELECT * FROM action_items ORDER BY created_at")
        .fetch_all(&*db)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
    let action_items: Vec<serde_json::Value> = action_rows
        .iter()
        .map(|r: &sqlx::sqlite::SqliteRow| {
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
        .collect();

    // Export quarter configs
    let quarter_rows =
        sqlx::query("SELECT * FROM quarter_config ORDER BY fiscal_year DESC, quarter_number DESC")
            .fetch_all(&*db)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
    let quarter_configs: Vec<serde_json::Value> = quarter_rows
        .iter()
        .map(|r: &sqlx::sqlite::SqliteRow| {
            serde_json::json!({
                "id": r.get::<String, _>("id"),
                "fiscal_year": r.get::<i64, _>("fiscal_year"),
                "quarter_number": r.get::<i64, _>("quarter_number"),
                "start_date": r.get::<String, _>("start_date"),
                "end_date": r.get::<String, _>("end_date"),
                "label": r.get::<String, _>("label"),
                "created_at": r.get::<String, _>("created_at"),
            })
        })
        .collect();

    // Export app settings
    let settings_rows = sqlx::query("SELECT * FROM app_settings")
        .fetch_all(&*db)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
    let mut settings_map = serde_json::Map::new();
    for r in &settings_rows {
        let key: String = Row::get(r, "key");
        let value: String = Row::get(r, "value");
        settings_map.insert(key, serde_json::Value::String(value));
    }

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let backup = BackupData {
        version: "1.0".to_string(),
        exported_at: now,
        services,
        incidents,
        action_items,
        quarter_configs,
        app_settings: serde_json::Value::Object(settings_map),
    };

    let json = serde_json::to_string_pretty(&backup)
        .map_err(|e| AppError::Internal(format!("Failed to serialize backup: {}", e)))?;

    // Write to temp file and return path
    let temp_dir = std::env::temp_dir();
    let file_name = format!("incident_backup_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    let file_path = temp_dir.join(file_name);

    std::fs::write(&file_path, &json)
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
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| AppError::Io(e))?;

    let backup: BackupData = serde_json::from_str(&content)
        .map_err(|e| AppError::Internal(format!("Invalid backup file: {}", e)))?;

    if backup.version != "1.0" {
        return Err(AppError::Validation(format!(
            "Unsupported backup version: {}",
            backup.version
        )));
    }

    let mut result = BackupImportResult {
        services: 0,
        incidents: 0,
        action_items: 0,
        quarter_configs: 0,
        settings: 0,
        errors: vec![],
    };

    // Import services first (incidents depend on them)
    for svc in &backup.services {
        match import_service(&db, svc).await {
            Ok(_) => result.services += 1,
            Err(e) => result.errors.push(format!("Service: {}", e)),
        }
    }

    // Import incidents
    for inc in &backup.incidents {
        match import_incident(&db, inc).await {
            Ok(_) => result.incidents += 1,
            Err(e) => result.errors.push(format!("Incident: {}", e)),
        }
    }

    // Import action items
    for ai in &backup.action_items {
        match import_action_item(&db, ai).await {
            Ok(_) => result.action_items += 1,
            Err(e) => result.errors.push(format!("Action item: {}", e)),
        }
    }

    // Import quarter configs
    for qc in &backup.quarter_configs {
        match import_quarter_config(&db, qc).await {
            Ok(_) => result.quarter_configs += 1,
            Err(e) => result.errors.push(format!("Quarter config: {}", e)),
        }
    }

    // Import app settings
    if let serde_json::Value::Object(map) = &backup.app_settings {
        for (key, value) in map {
            if let serde_json::Value::String(val) = value {
                match settings::set_setting(&db, key, val).await {
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
    let is_active = svc.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true);

    sqlx::query(
        "INSERT OR IGNORE INTO services (id, name, category, default_severity, default_impact, description, is_active) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&name)
    .bind(&category)
    .bind(&default_severity)
    .bind(&default_impact)
    .bind(description)
    .bind(is_active)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

async fn import_incident(db: &SqlitePool, inc: &serde_json::Value) -> Result<(), AppError> {
    let id = get_str(inc, "id")?;

    sqlx::query(
        "INSERT OR IGNORE INTO incidents (id, title, service_id, severity, impact, status, started_at, detected_at, responded_at, resolved_at, root_cause, resolution, tickets_submitted, affected_users, is_recurring, recurrence_of, lessons_learned, action_items, external_ref, notes) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(inc.get("title").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(inc.get("service_id").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(inc.get("severity").and_then(|v| v.as_str()).unwrap_or("Medium"))
    .bind(inc.get("impact").and_then(|v| v.as_str()).unwrap_or("Medium"))
    .bind(inc.get("status").and_then(|v| v.as_str()).unwrap_or("Resolved"))
    .bind(inc.get("started_at").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(inc.get("detected_at").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(inc.get("responded_at").and_then(|v| v.as_str()))
    .bind(inc.get("resolved_at").and_then(|v| v.as_str()))
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
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

async fn import_action_item(db: &SqlitePool, ai: &serde_json::Value) -> Result<(), AppError> {
    let id = get_str(ai, "id")?;

    sqlx::query(
        "INSERT OR IGNORE INTO action_items (id, incident_id, title, description, status, owner, due_date) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(ai.get("incident_id").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(ai.get("title").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(ai.get("description").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(ai.get("status").and_then(|v| v.as_str()).unwrap_or("Open"))
    .bind(ai.get("owner").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(ai.get("due_date").and_then(|v| v.as_str()))
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

async fn import_quarter_config(db: &SqlitePool, qc: &serde_json::Value) -> Result<(), AppError> {
    let id = get_str(qc, "id")?;

    sqlx::query(
        "INSERT OR IGNORE INTO quarter_config (id, fiscal_year, quarter_number, start_date, end_date, label) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(qc.get("fiscal_year").and_then(|v| v.as_i64()).unwrap_or(0))
    .bind(qc.get("quarter_number").and_then(|v| v.as_i64()).unwrap_or(1))
    .bind(qc.get("start_date").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(qc.get("end_date").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(qc.get("label").and_then(|v| v.as_str()).unwrap_or(""))
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

fn get_str(value: &serde_json::Value, field: &str) -> Result<String, AppError> {
    value
        .get(field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Validation(format!("Missing field '{}'", field)))
}
