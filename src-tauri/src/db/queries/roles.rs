use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::role::IncidentRole;

pub async fn assign_role(
    db: &SqlitePool,
    id: &str,
    incident_id: &str,
    role: &str,
    assignee: &str,
    is_primary: bool,
) -> AppResult<IncidentRole> {
    sqlx::query(
        "INSERT INTO incident_roles (id, incident_id, role, assignee, is_primary) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(id)
    .bind(incident_id)
    .bind(role)
    .bind(assignee)
    .bind(is_primary)
    .execute(db)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AppError::Conflict(format!(
                "{} is already assigned as {} for this incident",
                assignee, role
            ))
        } else {
            AppError::Database(e.to_string())
        }
    })?;

    get_role_by_id(db, id).await
}

pub async fn unassign_role(db: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE incident_roles SET unassigned_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ? AND unassigned_at IS NULL"
    )
    .bind(id)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Role assignment '{}' not found or already unassigned",
            id
        )));
    }
    Ok(())
}

pub async fn list_roles_for_incident(
    db: &SqlitePool,
    incident_id: &str,
) -> AppResult<Vec<IncidentRole>> {
    let rows = sqlx::query(
        "SELECT * FROM incident_roles WHERE incident_id = ? AND unassigned_at IS NULL ORDER BY role, assignee"
    )
    .bind(incident_id)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_role_row).collect())
}

async fn get_role_by_id(db: &SqlitePool, id: &str) -> AppResult<IncidentRole> {
    let row = sqlx::query("SELECT * FROM incident_roles WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Role assignment '{}' not found", id)))?;

    Ok(parse_role_row(&row))
}

fn parse_role_row(row: &sqlx::sqlite::SqliteRow) -> IncidentRole {
    IncidentRole {
        id: row.get("id"),
        incident_id: row.get("incident_id"),
        role: row.get("role"),
        assignee: row.get("assignee"),
        is_primary: row.get::<bool, _>("is_primary"),
        assigned_at: row.get("assigned_at"),
        unassigned_at: row.get("unassigned_at"),
    }
}
