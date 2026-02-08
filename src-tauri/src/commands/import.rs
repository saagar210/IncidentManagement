use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sqlx::SqlitePool;
use tauri::State;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    pub mappings: HashMap<String, String>,
    pub default_values: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub incidents: Vec<serde_json::Value>,
    pub warnings: Vec<ImportWarning>,
    pub error_count: i64,
    pub ready_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportWarning {
    pub row: usize,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub created: i64,
    pub skipped: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportTemplate {
    pub id: String,
    pub name: String,
    pub column_mapping: String,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn parse_csv_headers(
    _file_path: String,
) -> Result<Vec<String>, AppError> {
    // Phase 4 implementation
    Err(AppError::Internal("CSV import not yet implemented".into()))
}

#[tauri::command]
pub async fn preview_csv_import(
    _file_path: String,
    _mapping: ColumnMapping,
) -> Result<ImportPreview, AppError> {
    Err(AppError::Internal("CSV import not yet implemented".into()))
}

#[tauri::command]
pub async fn execute_csv_import(
    _db: State<'_, SqlitePool>,
    _file_path: String,
    _mapping: ColumnMapping,
) -> Result<ImportResult, AppError> {
    Err(AppError::Internal("CSV import not yet implemented".into()))
}

#[tauri::command]
pub async fn list_import_templates(
    _db: State<'_, SqlitePool>,
) -> Result<Vec<ImportTemplate>, AppError> {
    Err(AppError::Internal("Import templates not yet implemented".into()))
}
