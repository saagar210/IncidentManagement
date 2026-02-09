use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::timeline_events;
use crate::error::AppError;

#[tauri::command]
pub async fn list_timeline_events_for_incident(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<Vec<timeline_events::TimelineEvent>, AppError> {
    timeline_events::list_timeline_events_for_incident(&*db, &incident_id).await
}

#[tauri::command]
pub async fn create_timeline_event(
    db: State<'_, SqlitePool>,
    req: timeline_events::CreateTimelineEventRequest,
) -> Result<timeline_events::TimelineEvent, AppError> {
    timeline_events::create_timeline_event(&*db, &req).await
}

#[tauri::command]
pub async fn delete_timeline_event(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    timeline_events::delete_timeline_event(&*db, &id).await?;
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimelineImportResult {
    pub created: i64,
    pub skipped: i64,
    pub errors: Vec<String>,
}

#[tauri::command]
pub async fn import_timeline_events_from_paste(
    db: State<'_, SqlitePool>,
    incident_id: String,
    paste_text: String,
    source: Option<String>,
) -> Result<TimelineImportResult, AppError> {
    let mut created: i64 = 0;
    let mut skipped: i64 = 0;
    let mut errors: Vec<String> = Vec::new();

    let src = source.unwrap_or_else(|| "paste".to_string());

    for (idx, raw_line) in paste_text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        // Expected: "<timestamp> <message>"
        // Timestamp formats supported by the DB layer:
        // - RFC3339 (no spaces): 2026-01-01T10:07:00Z
        // - Simple (contains a space): 2026-01-01 10:06
        let (ts, msg) = if line.len() >= 16
            && line.chars().nth(10) == Some(' ')
            && line.chars().nth(13) == Some(':')
        {
            (&line[..16], line[16..].trim_start())
        } else if let Some((a, b)) = line.split_once(' ') {
            (a, b.trim_start())
        } else {
            skipped += 1;
            errors.push(format!("Line {}: expected '<timestamp> <message>'", idx + 1));
            continue;
        };
        if msg.trim().is_empty() {
            skipped += 1;
            errors.push(format!("Line {}: message is required", idx + 1));
            continue;
        }

        let req = timeline_events::CreateTimelineEventRequest {
            incident_id: incident_id.clone(),
            occurred_at: ts.trim().to_string(),
            source: Some(src.clone()),
            message: msg.trim().to_string(),
            actor: None,
        };

        match timeline_events::create_timeline_event(&*db, &req).await {
            Ok(_) => created += 1,
            Err(e) => {
                skipped += 1;
                errors.push(format!("Line {}: {}", idx + 1, e));
            }
        }
    }

    Ok(TimelineImportResult {
        created,
        skipped,
        errors,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimelineJsonEvent {
    pub occurred_at: String,
    pub message: String,
    pub actor: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimelineJsonImport {
    pub events: Vec<TimelineJsonEvent>,
}

#[tauri::command]
pub async fn import_timeline_events_from_json(
    db: State<'_, SqlitePool>,
    incident_id: String,
    json_str: String,
    source: Option<String>,
) -> Result<TimelineImportResult, AppError> {
    let parsed: TimelineJsonImport = serde_json::from_str(&json_str)
        .map_err(|e| AppError::Validation(format!("Invalid JSON import schema: {}", e)))?;

    let mut created: i64 = 0;
    let mut skipped: i64 = 0;
    let mut errors: Vec<String> = Vec::new();

    let src = source.unwrap_or_else(|| "json".to_string());

    for (idx, ev) in parsed.events.iter().enumerate() {
        let req = timeline_events::CreateTimelineEventRequest {
            incident_id: incident_id.clone(),
            occurred_at: ev.occurred_at.clone(),
            source: Some(src.clone()),
            message: ev.message.clone(),
            actor: ev.actor.clone(),
        };
        match timeline_events::create_timeline_event(&*db, &req).await {
            Ok(_) => created += 1,
            Err(e) => {
                skipped += 1;
                errors.push(format!("Event {}: {}", idx + 1, e));
            }
        }
    }

    Ok(TimelineImportResult {
        created,
        skipped,
        errors,
    })
}
