use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::priority::{Impact, Severity, calculate_priority};
use crate::models::sla::*;

fn parse_sla_definition(row: &sqlx::sqlite::SqliteRow) -> SlaDefinition {
    SlaDefinition {
        id: row.get("id"),
        name: row.get("name"),
        priority: row.get("priority"),
        response_time_minutes: row.get("response_time_minutes"),
        resolve_time_minutes: row.get("resolve_time_minutes"),
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn compute_priority(severity: &str, impact: &str) -> String {
    let sev = Severity::from_str(severity).unwrap_or(Severity::Medium);
    let imp = Impact::from_str(impact).unwrap_or(Impact::Medium);
    calculate_priority(&sev, &imp).to_string()
}

fn parse_datetime(date_str: &str) -> Option<chrono::NaiveDateTime> {
    chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%SZ")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.fZ"))
        .ok()
}

pub async fn list_sla_definitions(pool: &SqlitePool) -> AppResult<Vec<SlaDefinition>> {
    let rows = sqlx::query("SELECT * FROM sla_definitions ORDER BY priority ASC")
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_sla_definition).collect())
}

pub async fn get_sla_definition(pool: &SqlitePool, id: &str) -> AppResult<SlaDefinition> {
    let row = sqlx::query("SELECT * FROM sla_definitions WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("SLA definition '{}' not found", id)))?;

    Ok(parse_sla_definition(&row))
}

pub async fn get_sla_for_priority(
    pool: &SqlitePool,
    priority: &str,
) -> AppResult<Option<SlaDefinition>> {
    let row = sqlx::query(
        "SELECT * FROM sla_definitions WHERE priority = ? AND is_active = 1",
    )
    .bind(priority)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(row.as_ref().map(parse_sla_definition))
}

pub async fn create_sla_definition(
    pool: &SqlitePool,
    req: &CreateSlaDefinitionRequest,
) -> AppResult<SlaDefinition> {
    let id = format!("sla-{}", uuid::Uuid::new_v4());

    sqlx::query(
        "INSERT INTO sla_definitions (id, name, priority, response_time_minutes, resolve_time_minutes) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.priority)
    .bind(req.response_time_minutes)
    .bind(req.resolve_time_minutes)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_sla_definition(pool, &id).await
}

pub async fn update_sla_definition(
    pool: &SqlitePool,
    id: &str,
    req: &UpdateSlaDefinitionRequest,
) -> AppResult<SlaDefinition> {
    // Verify it exists first
    let _existing = get_sla_definition(pool, id).await?;

    let mut set_clauses: Vec<String> = vec![];
    let mut binds: Vec<String> = vec![];

    if let Some(ref name) = req.name {
        set_clauses.push("name = ?".to_string());
        binds.push(name.clone());
    }
    if let Some(ref priority) = req.priority {
        set_clauses.push("priority = ?".to_string());
        binds.push(priority.clone());
    }
    if let Some(response) = req.response_time_minutes {
        set_clauses.push("response_time_minutes = ?".to_string());
        binds.push(response.to_string());
    }
    if let Some(resolve) = req.resolve_time_minutes {
        set_clauses.push("resolve_time_minutes = ?".to_string());
        binds.push(resolve.to_string());
    }
    if let Some(is_active) = req.is_active {
        set_clauses.push("is_active = ?".to_string());
        binds.push(if is_active { "1".to_string() } else { "0".to_string() });
    }

    // Always update updated_at
    set_clauses.push("updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')".to_string());

    let sql = format!(
        "UPDATE sla_definitions SET {} WHERE id = ?",
        set_clauses.join(", ")
    );

    let mut query = sqlx::query(&sql);
    for bind in &binds {
        query = query.bind(bind);
    }
    query = query.bind(id);

    query
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    get_sla_definition(pool, id).await
}

pub async fn delete_sla_definition(pool: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM sla_definitions WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "SLA definition '{}' not found",
            id
        )));
    }
    Ok(())
}

pub async fn compute_sla_status(
    pool: &SqlitePool,
    incident_id: &str,
) -> AppResult<SlaStatus> {
    // Fetch the incident
    let row = sqlx::query(
        "SELECT severity, impact, started_at, detected_at, responded_at, resolved_at FROM incidents WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(incident_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Incident '{}' not found", incident_id)))?;

    let severity: String = row.get("severity");
    let impact: String = row.get("impact");
    let started_at: String = row.get("started_at");
    let detected_at: String = row.get("detected_at");
    let responded_at: Option<String> = row.get("responded_at");
    let resolved_at: Option<String> = row.get("resolved_at");

    let priority = compute_priority(&severity, &impact);

    // Look up the active SLA for this priority
    let sla = get_sla_for_priority(pool, &priority).await?;

    let sla = match sla {
        Some(s) => s,
        None => {
            // No SLA defined for this priority
            return Ok(SlaStatus {
                priority,
                response_target_minutes: None,
                resolve_target_minutes: None,
                response_elapsed_minutes: None,
                resolve_elapsed_minutes: None,
                response_breached: false,
                resolve_breached: false,
            });
        }
    };

    let now = chrono::Utc::now().naive_utc();

    // Response elapsed: from detected_at to responded_at (or now)
    let response_elapsed = parse_datetime(&detected_at).map(|detected| {
        let end = responded_at
            .as_deref()
            .and_then(parse_datetime)
            .unwrap_or(now);
        let duration = end.signed_duration_since(detected);
        duration.num_minutes()
    });

    // Resolve elapsed: from started_at to resolved_at (or now)
    let resolve_elapsed = parse_datetime(&started_at).map(|started| {
        let end = resolved_at
            .as_deref()
            .and_then(parse_datetime)
            .unwrap_or(now);
        let duration = end.signed_duration_since(started);
        duration.num_minutes()
    });

    let response_breached = response_elapsed
        .map(|elapsed| elapsed > sla.response_time_minutes)
        .unwrap_or(false);

    let resolve_breached = resolve_elapsed
        .map(|elapsed| elapsed > sla.resolve_time_minutes)
        .unwrap_or(false);

    Ok(SlaStatus {
        priority,
        response_target_minutes: Some(sla.response_time_minutes),
        resolve_target_minutes: Some(sla.resolve_time_minutes),
        response_elapsed_minutes: response_elapsed,
        resolve_elapsed_minutes: resolve_elapsed,
        response_breached,
        resolve_breached,
    })
}
