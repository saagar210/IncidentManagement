use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShiftHandoff {
    pub id: String,
    pub shift_end_time: Option<String>,
    pub content: String,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShiftHandoffRequest {
    pub shift_end_time: Option<String>,
    pub content: String,
    #[serde(default)]
    pub created_by: String,
}

impl CreateShiftHandoffRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.content.trim().is_empty() {
            return Err(AppError::Validation("Content is required".into()));
        }
        if self.content.len() > 100_000 {
            return Err(AppError::Validation("Content too long (max 100000 chars)".into()));
        }
        Ok(())
    }
}
