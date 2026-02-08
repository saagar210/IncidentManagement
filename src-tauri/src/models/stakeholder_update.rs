use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeholderUpdate {
    pub id: String,
    pub incident_id: String,
    pub content: String,
    pub update_type: String,
    pub generated_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStakeholderUpdateRequest {
    pub incident_id: String,
    pub content: String,
    #[serde(default = "default_update_type")]
    pub update_type: String,
    #[serde(default = "default_generated_by")]
    pub generated_by: String,
}

fn default_update_type() -> String {
    "status".to_string()
}

fn default_generated_by() -> String {
    "manual".to_string()
}

const VALID_UPDATE_TYPES: &[&str] = &["status", "initial", "final", "custom"];
const VALID_GENERATED_BY: &[&str] = &["manual", "template", "ai"];

impl CreateStakeholderUpdateRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.incident_id.trim().is_empty() {
            return Err(AppError::Validation("Incident ID is required".into()));
        }
        if self.content.trim().is_empty() {
            return Err(AppError::Validation("Content is required".into()));
        }
        if self.content.len() > 50_000 {
            return Err(AppError::Validation("Content too long (max 50000 chars)".into()));
        }
        if !VALID_UPDATE_TYPES.contains(&self.update_type.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid update_type '{}'. Must be one of: {}",
                self.update_type,
                VALID_UPDATE_TYPES.join(", ")
            )));
        }
        if !VALID_GENERATED_BY.contains(&self.generated_by.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid generated_by '{}'. Must be one of: {}",
                self.generated_by,
                VALID_GENERATED_BY.join(", ")
            )));
        }
        Ok(())
    }
}
