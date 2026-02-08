use sqlx::{Row, SqlitePool};
use crate::error::{AppError, AppResult};
use crate::models::postmortem::{
    ContributingFactor, CreateContributingFactorRequest, CreatePostmortemRequest,
    Postmortem, PostmortemTemplate, UpdatePostmortemRequest,
};

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

    let completed_at = if status == "final" && existing.status != "final" {
        Some(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string())
    } else {
        existing.completed_at.clone()
    };

    sqlx::query(
        "UPDATE postmortems SET content=?, status=?, reminder_at=?, completed_at=?, updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?"
    )
    .bind(content)
    .bind(status)
    .bind(reminder_at)
    .bind(&completed_at)
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
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
