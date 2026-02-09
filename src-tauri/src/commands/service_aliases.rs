use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::service_aliases;
use crate::error::AppError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateServiceAliasCmd {
    pub alias: String,
    pub service_id: String,
}

#[tauri::command]
pub async fn list_service_aliases(
    db: State<'_, SqlitePool>,
) -> Result<Vec<service_aliases::ServiceAlias>, AppError> {
    service_aliases::list_service_aliases(&*db).await
}

#[tauri::command]
pub async fn create_service_alias(
    db: State<'_, SqlitePool>,
    req: CreateServiceAliasCmd,
) -> Result<service_aliases::ServiceAlias, AppError> {
    service_aliases::create_service_alias(&*db, &req.alias, &req.service_id).await
}

#[tauri::command]
pub async fn delete_service_alias(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    service_aliases::delete_service_alias(&*db, &id).await
}

