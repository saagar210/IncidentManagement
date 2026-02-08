use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedFilter {
    pub id: String,
    pub name: String,
    pub filters: String, // JSON string
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSavedFilterRequest {
    pub name: String,
    pub filters: String, // JSON string
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSavedFilterRequest {
    pub name: Option<String>,
    pub filters: Option<String>,
    pub is_default: Option<bool>,
}

const MAX_NAME_LEN: usize = 200;
const MAX_FILTERS_LEN: usize = 5_000;

impl CreateSavedFilterRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::Validation("Filter name is required".into()));
        }
        if self.name.len() > MAX_NAME_LEN {
            return Err(AppError::Validation("Filter name too long".into()));
        }
        if self.filters.len() > MAX_FILTERS_LEN {
            return Err(AppError::Validation("Filters JSON too large".into()));
        }
        // Basic JSON validation
        if serde_json::from_str::<serde_json::Value>(&self.filters).is_err() {
            return Err(AppError::Validation("Filters must be valid JSON".into()));
        }
        Ok(())
    }
}

impl UpdateSavedFilterRequest {
    pub fn validate(&self) -> AppResult<()> {
        if let Some(ref name) = self.name {
            if name.trim().is_empty() {
                return Err(AppError::Validation("Filter name cannot be empty".into()));
            }
            if name.len() > MAX_NAME_LEN {
                return Err(AppError::Validation("Filter name too long".into()));
            }
        }
        if let Some(ref filters) = self.filters {
            if filters.len() > MAX_FILTERS_LEN {
                return Err(AppError::Validation("Filters JSON too large".into()));
            }
            if serde_json::from_str::<serde_json::Value>(filters).is_err() {
                return Err(AppError::Validation("Filters must be valid JSON".into()));
            }
        }
        Ok(())
    }
}
