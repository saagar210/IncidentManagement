use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::service::{CreateServiceRequest, Service, UpdateServiceRequest};

pub async fn insert_service(db: &SqlitePool, id: &str, req: &CreateServiceRequest) -> AppResult<Service> {
    sqlx::query(
        "INSERT INTO services (id, name, category, default_severity, default_impact, description) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(id)
    .bind(&req.name)
    .bind(&req.category)
    .bind(&req.default_severity)
    .bind(&req.default_impact)
    .bind(&req.description)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_service_by_id(db, id).await
}

pub async fn update_service(db: &SqlitePool, id: &str, req: &UpdateServiceRequest) -> AppResult<Service> {
    let existing = get_service_by_id(db, id).await?;

    let name = req.name.as_ref().unwrap_or(&existing.name);
    let category = req.category.as_ref().unwrap_or(&existing.category);
    let severity = req.default_severity.as_ref().unwrap_or(&existing.default_severity);
    let impact = req.default_impact.as_ref().unwrap_or(&existing.default_impact);
    let description = req.description.as_ref().unwrap_or(&existing.description);
    let is_active = req.is_active.unwrap_or(existing.is_active);

    sqlx::query(
        "UPDATE services SET name=?, category=?, default_severity=?, default_impact=?, description=?, is_active=?, updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?"
    )
    .bind(name)
    .bind(category)
    .bind(severity)
    .bind(impact)
    .bind(description)
    .bind(is_active)
    .bind(id)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_service_by_id(db, id).await
}

pub async fn delete_service(db: &SqlitePool, id: &str) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM incidents WHERE service_id = ?")
        .bind(id)
        .fetch_one(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if count > 0 {
        return Err(AppError::Conflict(format!(
            "Cannot delete service: {} incident(s) reference it", count
        )));
    }

    let result = sqlx::query("DELETE FROM services WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Service '{}' not found", id)));
    }
    Ok(())
}

pub async fn get_service_by_id(db: &SqlitePool, id: &str) -> AppResult<Service> {
    let row = sqlx::query("SELECT * FROM services WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Service '{}' not found", id)))?;

    Ok(parse_service_row(&row))
}

pub async fn list_all_services(db: &SqlitePool) -> AppResult<Vec<Service>> {
    let rows = sqlx::query("SELECT * FROM services ORDER BY name")
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_service_row).collect())
}

pub async fn list_active_services(db: &SqlitePool) -> AppResult<Vec<Service>> {
    let rows = sqlx::query("SELECT * FROM services WHERE is_active = 1 ORDER BY name")
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_service_row).collect())
}

fn parse_service_row(row: &sqlx::sqlite::SqliteRow) -> Service {
    Service {
        id: row.get("id"),
        name: row.get("name"),
        category: row.get("category"),
        default_severity: row.get("default_severity"),
        default_impact: row.get("default_impact"),
        description: row.get::<Option<String>, _>("description").unwrap_or_default(),
        is_active: row.get::<bool, _>("is_active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
