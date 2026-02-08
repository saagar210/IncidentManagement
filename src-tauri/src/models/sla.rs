use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const VALID_PRIORITIES: &[&str] = &["P0", "P1", "P2", "P3", "P4"];
const MAX_NAME_LEN: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaDefinition {
    pub id: String,
    pub name: String,
    pub priority: String,
    pub response_time_minutes: i64,
    pub resolve_time_minutes: i64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSlaDefinitionRequest {
    pub name: String,
    pub priority: String,
    pub response_time_minutes: i64,
    pub resolve_time_minutes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSlaDefinitionRequest {
    pub name: Option<String>,
    pub priority: Option<String>,
    pub response_time_minutes: Option<i64>,
    pub resolve_time_minutes: Option<i64>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaStatus {
    pub priority: String,
    pub response_target_minutes: Option<i64>,
    pub resolve_target_minutes: Option<i64>,
    pub response_elapsed_minutes: Option<i64>,
    pub resolve_elapsed_minutes: Option<i64>,
    pub response_breached: bool,
    pub resolve_breached: bool,
}

impl CreateSlaDefinitionRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::Validation("SLA name is required".into()));
        }
        if self.name.len() > MAX_NAME_LEN {
            return Err(AppError::Validation("SLA name too long".into()));
        }
        if !VALID_PRIORITIES.contains(&self.priority.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid priority '{}'. Must be one of: {}",
                self.priority,
                VALID_PRIORITIES.join(", ")
            )));
        }
        if self.response_time_minutes <= 0 {
            return Err(AppError::Validation(
                "Response time must be greater than 0".into(),
            ));
        }
        if self.resolve_time_minutes <= 0 {
            return Err(AppError::Validation(
                "Resolve time must be greater than 0".into(),
            ));
        }
        if self.resolve_time_minutes < self.response_time_minutes {
            return Err(AppError::Validation(
                "Resolve time must be greater than or equal to response time".into(),
            ));
        }
        Ok(())
    }
}

impl UpdateSlaDefinitionRequest {
    pub fn validate(&self) -> AppResult<()> {
        if let Some(ref name) = self.name {
            if name.trim().is_empty() {
                return Err(AppError::Validation("SLA name cannot be empty".into()));
            }
            if name.len() > MAX_NAME_LEN {
                return Err(AppError::Validation("SLA name too long".into()));
            }
        }
        if let Some(ref priority) = self.priority {
            if !VALID_PRIORITIES.contains(&priority.as_str()) {
                return Err(AppError::Validation(format!(
                    "Invalid priority '{}'. Must be one of: {}",
                    priority,
                    VALID_PRIORITIES.join(", ")
                )));
            }
        }
        if let Some(response) = self.response_time_minutes {
            if response <= 0 {
                return Err(AppError::Validation(
                    "Response time must be greater than 0".into(),
                ));
            }
        }
        if let Some(resolve) = self.resolve_time_minutes {
            if resolve <= 0 {
                return Err(AppError::Validation(
                    "Resolve time must be greater than 0".into(),
                ));
            }
        }
        if let (Some(response), Some(resolve)) =
            (self.response_time_minutes, self.resolve_time_minutes)
        {
            if resolve < response {
                return Err(AppError::Validation(
                    "Resolve time must be greater than or equal to response time".into(),
                ));
            }
        }
        Ok(())
    }
}
