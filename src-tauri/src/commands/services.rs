use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::{audit, services};
use crate::error::AppError;
use crate::models::service::{CreateServiceRequest, Service, UpdateServiceRequest};

#[tauri::command]
pub async fn create_service(
    db: State<'_, SqlitePool>,
    service: CreateServiceRequest,
) -> Result<Service, AppError> {
    service.validate()?;
    let id = format!("svc-{}", uuid::Uuid::new_v4());
    let result = services::insert_service(&*db, &id, &service).await?;
    let _ = audit::insert_audit_entry(
        &*db,
        "service",
        &id,
        "created",
        &format!("Created service: {}", &service.name),
        "",
    )
    .await;
    Ok(result)
}

#[tauri::command]
pub async fn update_service(
    db: State<'_, SqlitePool>,
    id: String,
    service: UpdateServiceRequest,
) -> Result<Service, AppError> {
    service.validate()?;
    let result = services::update_service(&*db, &id, &service).await?;
    let _ = audit::insert_audit_entry(&*db, "service", &id, "updated", "Updated service", "").await;
    Ok(result)
}

#[tauri::command]
pub async fn delete_service(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    services::delete_service(&*db, &id).await?;
    let _ = audit::insert_audit_entry(&*db, "service", &id, "deleted", "Deleted service", "").await;
    Ok(())
}

#[tauri::command]
pub async fn list_services(
    db: State<'_, SqlitePool>,
) -> Result<Vec<Service>, AppError> {
    services::list_all_services(&*db).await
}

#[tauri::command]
pub async fn list_active_services(
    db: State<'_, SqlitePool>,
) -> Result<Vec<Service>, AppError> {
    services::list_active_services(&*db).await
}
