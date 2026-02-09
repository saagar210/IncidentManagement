use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceAlias {
    pub id: String,
    pub alias: String,
    pub service_id: String,
    pub service_name: String,
    pub created_at: String,
}

pub async fn list_service_aliases(pool: &SqlitePool) -> AppResult<Vec<ServiceAlias>> {
    let rows = sqlx::query(
        r#"
        SELECT sa.id, sa.alias, sa.service_id, s.name AS service_name, sa.created_at
        FROM service_aliases sa
        JOIN services s ON s.id = sa.service_id
        ORDER BY lower(sa.alias) ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows
        .iter()
        .map(|row| ServiceAlias {
            id: row.get("id"),
            alias: row.get("alias"),
            service_id: row.get("service_id"),
            service_name: row.get("service_name"),
            created_at: row.get("created_at"),
        })
        .collect())
}

pub async fn create_service_alias(
    pool: &SqlitePool,
    alias: &str,
    service_id: &str,
) -> AppResult<ServiceAlias> {
    if alias.trim().is_empty() {
        return Err(AppError::Validation("Alias is required".into()));
    }
    if service_id.trim().is_empty() {
        return Err(AppError::Validation("Service ID is required".into()));
    }

    // Ensure the service exists.
    let service_name: Option<String> = sqlx::query_scalar("SELECT name FROM services WHERE id = ?")
        .bind(service_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let Some(service_name) = service_name else {
        return Err(AppError::NotFound(format!("Service '{}' not found", service_id)));
    };

    let id = format!("sal-{}", uuid::Uuid::new_v4());
    sqlx::query("INSERT INTO service_aliases (id, alias, service_id) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(alias.trim())
        .bind(service_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let created_at: String = sqlx::query_scalar("SELECT created_at FROM service_aliases WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(ServiceAlias {
        id,
        alias: alias.trim().to_string(),
        service_id: service_id.to_string(),
        service_name,
        created_at,
    })
}

pub async fn delete_service_alias(pool: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM service_aliases WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Service alias '{}' not found", id)));
    }
    Ok(())
}

/// Resolve a service ID from an import name using either canonical service name or aliases (case-insensitive).
pub async fn resolve_service_id_from_name(pool: &SqlitePool, name: &str) -> AppResult<Option<String>> {
    let n = name.trim();
    if n.is_empty() {
        return Ok(None);
    }

    // 1) Direct service name match.
    let direct: Option<String> = sqlx::query_scalar("SELECT id FROM services WHERE name = ? COLLATE NOCASE")
        .bind(n)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    if direct.is_some() {
        return Ok(direct);
    }

    // 2) Alias match.
    let alias: Option<String> = sqlx::query_scalar("SELECT service_id FROM service_aliases WHERE alias = ? COLLATE NOCASE")
        .bind(n)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(alias)
}

