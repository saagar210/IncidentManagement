use sqlx::{Row, SqlitePool};
use crate::error::{AppError, AppResult};
use crate::db::queries::incidents;
use crate::models::postmortem::{
    ContributingFactor, CreateContributingFactorRequest, CreatePostmortemRequest,
    Postmortem, PostmortemTemplate, UpdatePostmortemRequest,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessMissingItem {
    pub code: String,
    pub label: String,
    pub destination: String,
}

impl ReadinessMissingItem {
    fn new(code: &str, label: &str, destination: &str) -> Self {
        Self {
            code: code.to_string(),
            label: label.to_string(),
            destination: destination.to_string(),
        }
    }
}

fn extract_markdown(content: &str) -> String {
    // Content is stored as either raw markdown or a JSON object: {"markdown": "..."}.
    // For readiness checks we only care whether the user has provided meaningful text.
    if content.trim().is_empty() || content.trim() == "{}" {
        return String::new();
    }
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(md) = v.get("markdown").and_then(|m| m.as_str()) {
            return md.to_string();
        }
    }
    content.to_string()
}

pub async fn compute_readiness_missing_items(
    db: &SqlitePool,
    incident_id: &str,
    content: &str,
    no_action_items_justified: bool,
    no_action_items_justification: &str,
) -> AppResult<Vec<ReadinessMissingItem>> {
    let mut missing: Vec<ReadinessMissingItem> = Vec::new();

    if extract_markdown(content).trim().is_empty() {
        missing.push(ReadinessMissingItem::new(
            "POSTMORTEM_MARKDOWN",
            "Post-mortem content (markdown)",
            "postmortem",
        ));
    }

    let factors = list_contributing_factors(db, incident_id).await?;
    if factors.is_empty() {
        missing.push(ReadinessMissingItem::new(
            "CONTRIBUTING_FACTORS",
            "At least one contributing factor",
            "postmortem",
        ));
    }

    missing.extend(
        compute_action_items_missing(
            db,
            incident_id,
            no_action_items_justified,
            no_action_items_justification,
        )
        .await?,
    );

    Ok(missing)
}

async fn compute_action_items_missing(
    db: &SqlitePool,
    incident_id: &str,
    no_action_items_justified: bool,
    no_action_items_justification: &str,
) -> AppResult<Vec<ReadinessMissingItem>> {
    // Action items can exist in the normalized action_items table and/or the legacy
    // incident.action_items field. Either is acceptable for readiness.
    let action_items = incidents::list_action_items(db, Some(incident_id)).await?;
    let incident = incidents::get_incident_by_id(db, incident_id).await?;
    let legacy_action_items = incident.action_items.trim();

    let has_any_action_items = !action_items.is_empty() || !legacy_action_items.is_empty();
    if has_any_action_items {
        return Ok(vec![]);
    }

    if !no_action_items_justified {
        return Ok(vec![ReadinessMissingItem::new(
            "ACTION_ITEMS",
            "At least one action item (or mark as no action items justified)",
            "actions",
        )]);
    }

    if no_action_items_justification.trim().is_empty() {
        return Ok(vec![ReadinessMissingItem::new(
            "ACTION_ITEMS_JUSTIFICATION",
            "No action items justification text",
            "postmortem",
        )]);
    }

    Ok(vec![])
}

// --- Contributing Factors ---

pub async fn list_contributing_factors(db: &SqlitePool, incident_id: &str) -> AppResult<Vec<ContributingFactor>> {
    let rows = sqlx::query("SELECT * FROM contributing_factors WHERE incident_id = ? ORDER BY is_root DESC, created_at ASC")
        .bind(incident_id)
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_contributing_factor).collect())
}

pub async fn create_contributing_factor(db: &SqlitePool, id: &str, req: &CreateContributingFactorRequest) -> AppResult<ContributingFactor> {
    sqlx::query("INSERT INTO contributing_factors (id, incident_id, category, description, is_root) VALUES (?, ?, ?, ?, ?)")
        .bind(id)
        .bind(&req.incident_id)
        .bind(&req.category)
        .bind(&req.description)
        .bind(req.is_root)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let row = sqlx::query("SELECT * FROM contributing_factors WHERE id = ?")
        .bind(id)
        .fetch_one(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(parse_contributing_factor(&row))
}

pub async fn delete_contributing_factor(db: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM contributing_factors WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Contributing factor '{}' not found", id)));
    }
    Ok(())
}

fn parse_contributing_factor(row: &sqlx::sqlite::SqliteRow) -> ContributingFactor {
    ContributingFactor {
        id: row.get("id"),
        incident_id: row.get("incident_id"),
        category: row.get("category"),
        description: row.get("description"),
        is_root: row.get::<bool, _>("is_root"),
        created_at: row.get("created_at"),
    }
}

// --- Post-mortem Templates ---

pub async fn list_postmortem_templates(db: &SqlitePool) -> AppResult<Vec<PostmortemTemplate>> {
    let rows = sqlx::query("SELECT * FROM postmortem_templates ORDER BY is_default DESC, name ASC")
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_template).collect())
}

fn parse_template(row: &sqlx::sqlite::SqliteRow) -> PostmortemTemplate {
    PostmortemTemplate {
        id: row.get("id"),
        name: row.get("name"),
        incident_type: row.get("incident_type"),
        template_content: row.get("template_content"),
        is_default: row.get::<bool, _>("is_default"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

// --- Postmortems ---

pub async fn get_postmortem_by_incident(db: &SqlitePool, incident_id: &str) -> AppResult<Option<Postmortem>> {
    let row = sqlx::query("SELECT * FROM postmortems WHERE incident_id = ?")
        .bind(incident_id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(row.as_ref().map(parse_postmortem))
}

pub async fn get_postmortem(db: &SqlitePool, id: &str) -> AppResult<Postmortem> {
    let row = sqlx::query("SELECT * FROM postmortems WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Post-mortem '{}' not found", id)))?;

    Ok(parse_postmortem(&row))
}

pub async fn create_postmortem(db: &SqlitePool, id: &str, req: &CreatePostmortemRequest) -> AppResult<Postmortem> {
    sqlx::query("INSERT INTO postmortems (id, incident_id, template_id, content) VALUES (?, ?, ?, ?)")
        .bind(id)
        .bind(&req.incident_id)
        .bind(&req.template_id)
        .bind(&req.content)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    get_postmortem(db, id).await
}

pub async fn update_postmortem(db: &SqlitePool, id: &str, req: &UpdatePostmortemRequest) -> AppResult<Postmortem> {
    let existing = get_postmortem(db, id).await?;

    let content = req.content.as_ref().unwrap_or(&existing.content);
    let status = req.status.as_ref().unwrap_or(&existing.status);
    let reminder_at = req.reminder_at.as_ref().or(existing.reminder_at.as_ref());
    let no_action_items_justified = req.no_action_items_justified.unwrap_or(existing.no_action_items_justified);
    let no_action_items_justification = req
        .no_action_items_justification
        .as_deref()
        .unwrap_or(&existing.no_action_items_justification);

    if status == "final" && existing.status != "final" {
        let missing = compute_readiness_missing_items(
            db,
            &existing.incident_id,
            content,
            no_action_items_justified,
            no_action_items_justification,
        )
        .await?;
        if !missing.is_empty() {
            return Err(AppError::Validation(format!(
                "Cannot finalize post-mortem: missing {}",
                missing
                    .iter()
                    .map(|m| m.label.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ")
            )));
        }
    }

    let completed_at = if status == "final" && existing.status != "final" {
        Some(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string())
    } else {
        existing.completed_at.clone()
    };

    sqlx::query(
        "UPDATE postmortems
         SET content=?,
             status=?,
             reminder_at=?,
             completed_at=?,
             no_action_items_justified=?,
             no_action_items_justification=?,
             updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now')
         WHERE id=?"
    )
    .bind(content)
    .bind(status)
    .bind(reminder_at)
    .bind(&completed_at)
    .bind(no_action_items_justified)
    .bind(no_action_items_justification)
    .bind(id)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_postmortem(db, id).await
}

pub async fn delete_postmortem(db: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM postmortems WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Post-mortem '{}' not found", id)));
    }
    Ok(())
}

pub async fn list_postmortems(db: &SqlitePool, status: Option<&str>) -> AppResult<Vec<Postmortem>> {
    let rows = if let Some(s) = status {
        sqlx::query("SELECT * FROM postmortems WHERE status = ? ORDER BY updated_at DESC")
            .bind(s)
            .fetch_all(db)
            .await
    } else {
        sqlx::query("SELECT * FROM postmortems ORDER BY updated_at DESC")
            .fetch_all(db)
            .await
    }
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_postmortem).collect())
}

fn parse_postmortem(row: &sqlx::sqlite::SqliteRow) -> Postmortem {
    Postmortem {
        id: row.get("id"),
        incident_id: row.get("incident_id"),
        template_id: row.get("template_id"),
        content: row.get("content"),
        status: row.get("status"),
        reminder_at: row.get("reminder_at"),
        completed_at: row.get("completed_at"),
        no_action_items_justified: row
            .try_get::<bool, _>("no_action_items_justified")
            .unwrap_or(false),
        no_action_items_justification: row
            .try_get::<String, _>("no_action_items_justification")
            .unwrap_or_default(),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::{create_postmortem, update_postmortem, create_contributing_factor};
    use crate::db::migrations::run_migrations;
    use crate::db::queries::incidents;
    use crate::error::AppError;
    use crate::models::incident::{CreateActionItemRequest, CreateIncidentRequest};
    use crate::models::postmortem::{CreateContributingFactorRequest, CreatePostmortemRequest, UpdatePostmortemRequest};
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
    use std::str::FromStr;
    use tempfile::tempdir;

    async fn setup_db() -> sqlx::SqlitePool {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("postmortems-query-tests.db");
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
        pool
    }

    fn seed_incident() -> CreateIncidentRequest {
        CreateIncidentRequest {
            title: "Slack outage".to_string(),
            service_id: "svc-slack".to_string(),
            severity: "High".to_string(),
            impact: "High".to_string(),
            status: "Post-Mortem".to_string(),
            started_at: "2026-02-01T00:00:00Z".to_string(),
            detected_at: "2026-02-01T00:00:00Z".to_string(),
            acknowledged_at: None,
            first_response_at: None,
            mitigation_started_at: None,
            responded_at: None,
            resolved_at: None,
            root_cause: "".to_string(),
            resolution: "".to_string(),
            tickets_submitted: 0,
            affected_users: 0,
            is_recurring: false,
            recurrence_of: None,
            lessons_learned: "".to_string(),
            action_items: "".to_string(),
            external_ref: "".to_string(),
            notes: "".to_string(),
        }
    }

    async fn add_action_item(db: &sqlx::SqlitePool, incident_id: &str) {
        incidents::insert_action_item(
            db,
            "ai-1",
            &CreateActionItemRequest {
                incident_id: incident_id.to_string(),
                title: "Write vendor outage playbook".to_string(),
                description: "".to_string(),
                status: "Open".to_string(),
                owner: "IT".to_string(),
                due_date: None,
            },
        )
        .await
        .expect("insert action item");
    }

    async fn add_contributing_factor(db: &sqlx::SqlitePool, incident_id: &str) {
        create_contributing_factor(
            db,
            "cf-1",
            &CreateContributingFactorRequest {
                incident_id: incident_id.to_string(),
                category: "External".to_string(),
                description: "Slack had a global service disruption".to_string(),
                is_root: true,
            },
        )
        .await
        .expect("insert contributing factor");
    }

    async fn create_blank_postmortem(db: &sqlx::SqlitePool, incident_id: &str, pm_id: &str) -> super::Postmortem {
        create_postmortem(
            db,
            pm_id,
            &CreatePostmortemRequest {
                incident_id: incident_id.to_string(),
                template_id: None,
                content: "{}".to_string(),
            },
        )
        .await
        .expect("create postmortem")
    }

    #[tokio::test]
    async fn cannot_finalize_without_minimum_review_content() {
        let db = setup_db().await;

        let incident_id = "inc-1";
        incidents::insert_incident(&db, incident_id, &seed_incident())
            .await
            .expect("insert incident");

        let pm = create_blank_postmortem(&db, incident_id, "pm-1").await;

        let err = update_postmortem(
            &db,
            &pm.id,
            &UpdatePostmortemRequest {
                content: Some("{\"markdown\":\"\"}".to_string()),
                status: Some("final".to_string()),
                reminder_at: None,
                no_action_items_justified: None,
                no_action_items_justification: None,
            },
        )
        .await
        .unwrap_err();

        match err {
            AppError::Validation(msg) => {
                assert!(msg.contains("Cannot finalize post-mortem"));
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn can_finalize_when_minimum_review_content_is_present() {
        let db = setup_db().await;

        let incident_id = "inc-2";
        incidents::insert_incident(&db, incident_id, &seed_incident())
            .await
            .expect("insert incident");

        add_action_item(&db, incident_id).await;
        add_contributing_factor(&db, incident_id).await;
        let pm = create_blank_postmortem(&db, incident_id, "pm-2").await;

        let updated = update_postmortem(
            &db,
            &pm.id,
            &UpdatePostmortemRequest {
                content: Some("{\"markdown\":\"# Summary\\n\\nImpact was material.\"}".to_string()),
                status: Some("final".to_string()),
                reminder_at: None,
                no_action_items_justified: None,
                no_action_items_justification: None,
            },
        )
        .await
        .expect("finalize");

        assert_eq!(updated.status, "final");
        assert!(updated.completed_at.is_some());
    }

    #[tokio::test]
    async fn can_finalize_with_no_action_items_when_justified_and_explained() {
        let db = setup_db().await;

        let incident_id = "inc-3";
        incidents::insert_incident(&db, incident_id, &seed_incident())
            .await
            .expect("insert incident");

        add_contributing_factor(&db, incident_id).await;
        let pm = create_blank_postmortem(&db, incident_id, "pm-3").await;

        let updated = update_postmortem(
            &db,
            &pm.id,
            &UpdatePostmortemRequest {
                content: Some("{\"markdown\":\"# Summary\\n\\nVendor outage.\"}".to_string()),
                status: Some("final".to_string()),
                reminder_at: None,
                no_action_items_justified: Some(true),
                no_action_items_justification: Some(
                    "External vendor outage; no internal process or system changes identified."
                        .to_string(),
                ),
            },
        )
        .await
        .expect("finalize");

        assert_eq!(updated.status, "final");
        assert!(updated.completed_at.is_some());
    }

    #[tokio::test]
    async fn cannot_finalize_with_no_action_items_when_justified_but_missing_justification() {
        let db = setup_db().await;

        let incident_id = "inc-4";
        incidents::insert_incident(&db, incident_id, &seed_incident())
            .await
            .expect("insert incident");

        add_contributing_factor(&db, incident_id).await;
        let pm = create_blank_postmortem(&db, incident_id, "pm-4").await;

        let err = update_postmortem(
            &db,
            &pm.id,
            &UpdatePostmortemRequest {
                content: Some("{\"markdown\":\"# Summary\\n\\nVendor outage.\"}".to_string()),
                status: Some("final".to_string()),
                reminder_at: None,
                no_action_items_justified: Some(true),
                no_action_items_justification: Some("   ".to_string()),
            },
        )
        .await
        .unwrap_err();

        match err {
            AppError::Validation(msg) => {
                assert!(msg.contains("Cannot finalize post-mortem"));
                assert!(msg.contains("No action items justification"));
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }
}
