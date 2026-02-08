use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarterConfig {
    pub id: String,
    pub fiscal_year: i64,
    pub quarter_number: i64,
    pub start_date: String,
    pub end_date: String,
    pub label: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertQuarterRequest {
    pub id: Option<String>,
    pub fiscal_year: i64,
    pub quarter_number: i64,
    pub start_date: String,
    pub end_date: String,
    pub label: String,
}

impl UpsertQuarterRequest {
    pub fn validate(&self) -> AppResult<()> {
        if !(1..=4).contains(&self.quarter_number) {
            return Err(AppError::Validation(
                "Quarter number must be between 1 and 4".into(),
            ));
        }
        if self.start_date.trim().is_empty() {
            return Err(AppError::Validation("Start date is required".into()));
        }
        if self.end_date.trim().is_empty() {
            return Err(AppError::Validation("End date is required".into()));
        }
        if self.end_date <= self.start_date {
            return Err(AppError::Validation(
                "End date must be after start date".into(),
            ));
        }
        if self.label.trim().is_empty() {
            return Err(AppError::Validation("Label is required".into()));
        }
        Ok(())
    }
}
