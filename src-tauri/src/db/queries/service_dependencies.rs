use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::service::ServiceDependency;

pub async fn insert_dependency(
    db: &SqlitePool,
    id: &str,
    service_id: &str,
    depends_on_service_id: &str,
    dependency_type: &str,
) -> AppResult<ServiceDependency> {
    // Verify both services exist
    let svc_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM services WHERE id = ?)")
        .bind(service_id)
        .fetch_one(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if !svc_exists {
        return Err(AppError::NotFound(format!("Service '{}' not found", service_id)));
    }

    let dep_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM services WHERE id = ?)")
        .bind(depends_on_service_id)
        .fetch_one(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if !dep_exists {
        return Err(AppError::NotFound(format!(
            "Dependency service '{}' not found",
            depends_on_service_id
        )));
    }

    // Cycle detection: check if depends_on_service_id already depends on service_id (directly or transitively)
    if would_create_cycle(db, service_id, depends_on_service_id).await? {
        return Err(AppError::Validation(
            "Adding this dependency would create a circular dependency".into(),
        ));
    }

    // Check for duplicate
    let dup: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM service_dependencies WHERE service_id = ? AND depends_on_service_id = ?)",
    )
    .bind(service_id)
    .bind(depends_on_service_id)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if dup {
        return Err(AppError::Conflict("This dependency already exists".into()));
    }

    sqlx::query(
        "INSERT INTO service_dependencies (id, service_id, depends_on_service_id, dependency_type) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(service_id)
    .bind(depends_on_service_id)
    .bind(dependency_type)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    get_dependency_by_id(db, id).await
}

pub async fn delete_dependency(db: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM service_dependencies WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Service dependency '{}' not found",
            id
        )));
    }
    Ok(())
}

pub async fn list_dependencies_for_service(
    db: &SqlitePool,
    service_id: &str,
) -> AppResult<Vec<ServiceDependency>> {
    let rows = sqlx::query(
        "SELECT sd.*, s.name as depends_on_service_name
         FROM service_dependencies sd
         JOIN services s ON s.id = sd.depends_on_service_id
         WHERE sd.service_id = ?
         ORDER BY s.name",
    )
    .bind(service_id)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_dependency_row).collect())
}

pub async fn list_dependents_of_service(
    db: &SqlitePool,
    service_id: &str,
) -> AppResult<Vec<ServiceDependency>> {
    let rows = sqlx::query(
        "SELECT sd.*, s.name as depends_on_service_name
         FROM service_dependencies sd
         JOIN services s ON s.id = sd.service_id
         WHERE sd.depends_on_service_id = ?
         ORDER BY s.name",
    )
    .bind(service_id)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // For dependents, the "name" we joined is actually the dependent service name
    Ok(rows
        .iter()
        .map(|row| ServiceDependency {
            id: row.get("id"),
            service_id: row.get("service_id"),
            depends_on_service_id: row.get("depends_on_service_id"),
            depends_on_service_name: row.get::<Option<String>, _>("depends_on_service_name"),
            dependency_type: row.get("dependency_type"),
            created_at: row.get("created_at"),
        })
        .collect())
}

async fn get_dependency_by_id(db: &SqlitePool, id: &str) -> AppResult<ServiceDependency> {
    let row = sqlx::query(
        "SELECT sd.*, s.name as depends_on_service_name
         FROM service_dependencies sd
         JOIN services s ON s.id = sd.depends_on_service_id
         WHERE sd.id = ?",
    )
    .bind(id)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Dependency '{}' not found", id)))?;

    Ok(parse_dependency_row(&row))
}

/// BFS cycle detection: checks if adding service_id → depends_on would create a cycle.
/// A cycle exists if depends_on_service_id can already reach service_id through existing deps.
async fn would_create_cycle(
    db: &SqlitePool,
    service_id: &str,
    depends_on_service_id: &str,
) -> AppResult<bool> {
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(depends_on_service_id.to_string());

    while let Some(current) = queue.pop_front() {
        if current == service_id {
            return Ok(true);
        }
        if !visited.insert(current.clone()) {
            continue;
        }

        let deps: Vec<String> = sqlx::query_scalar(
            "SELECT depends_on_service_id FROM service_dependencies WHERE service_id = ?",
        )
        .bind(&current)
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        for dep in deps {
            if !visited.contains(&dep) {
                queue.push_back(dep);
            }
        }
    }

    Ok(false)
}

fn parse_dependency_row(row: &sqlx::sqlite::SqliteRow) -> ServiceDependency {
    ServiceDependency {
        id: row.get("id"),
        service_id: row.get("service_id"),
        depends_on_service_id: row.get("depends_on_service_id"),
        depends_on_service_name: row.get::<Option<String>, _>("depends_on_service_name"),
        dependency_type: row.get("dependency_type"),
        created_at: row.get("created_at"),
    }
}
