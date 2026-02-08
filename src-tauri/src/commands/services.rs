use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::{audit, service_dependencies, services};
use crate::error::AppError;
use crate::models::service::{
    CreateServiceDependencyRequest, CreateServiceRequest, Service, ServiceDependency,
    UpdateServiceRequest,
};

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
pub async fn delete_service(db: State<'_, SqlitePool>, id: String) -> Result<(), AppError> {
    services::delete_service(&*db, &id).await?;
    let _ =
        audit::insert_audit_entry(&*db, "service", &id, "deleted", "Deleted service", "").await;
    Ok(())
}

#[tauri::command]
pub async fn get_service(db: State<'_, SqlitePool>, id: String) -> Result<Service, AppError> {
    services::get_service_by_id(&*db, &id).await
}

#[tauri::command]
pub async fn list_services(db: State<'_, SqlitePool>) -> Result<Vec<Service>, AppError> {
    services::list_all_services(&*db).await
}

#[tauri::command]
pub async fn list_active_services(db: State<'_, SqlitePool>) -> Result<Vec<Service>, AppError> {
    services::list_active_services(&*db).await
}

// Service dependency commands

#[tauri::command]
pub async fn add_service_dependency(
    db: State<'_, SqlitePool>,
    req: CreateServiceDependencyRequest,
) -> Result<ServiceDependency, AppError> {
    req.validate()?;
    let id = format!("sdep-{}", uuid::Uuid::new_v4());
    let result = service_dependencies::insert_dependency(
        &*db,
        &id,
        &req.service_id,
        &req.depends_on_service_id,
        &req.dependency_type,
    )
    .await?;
    let _ = audit::insert_audit_entry(
        &*db,
        "service",
        &req.service_id,
        "dependency_added",
        &format!("Added dependency on service {}", &req.depends_on_service_id),
        "",
    )
    .await;
    Ok(result)
}

#[tauri::command]
pub async fn remove_service_dependency(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    service_dependencies::delete_dependency(&*db, &id).await?;
    let _ = audit::insert_audit_entry(
        &*db,
        "service_dependency",
        &id,
        "deleted",
        "Removed service dependency",
        "",
    )
    .await;
    Ok(())
}

#[tauri::command]
pub async fn list_service_dependencies(
    db: State<'_, SqlitePool>,
    service_id: String,
) -> Result<Vec<ServiceDependency>, AppError> {
    service_dependencies::list_dependencies_for_service(&*db, &service_id).await
}

#[tauri::command]
pub async fn list_service_dependents(
    db: State<'_, SqlitePool>,
    service_id: String,
) -> Result<Vec<ServiceDependency>, AppError> {
    service_dependencies::list_dependents_of_service(&*db, &service_id).await
}
