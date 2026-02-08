use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub category: String,
    pub default_severity: String,
    pub default_impact: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceRequest {
    pub name: String,
    pub category: String,
    pub default_severity: String,
    pub default_impact: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServiceRequest {
    pub name: Option<String>,
    pub category: Option<String>,
    pub default_severity: Option<String>,
    pub default_impact: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

const VALID_CATEGORIES: &[&str] = &[
    "Communication",
    "Infrastructure",
    "Development",
    "Productivity",
    "Security",
    "Other",
];

const VALID_LEVELS: &[&str] = &["Critical", "High", "Medium", "Low"];

impl CreateServiceRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::Validation("Service name is required".into()));
        }
        if !VALID_CATEGORIES.contains(&self.category.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid category '{}'. Must be one of: {}",
                self.category,
                VALID_CATEGORIES.join(", ")
            )));
        }
        if !VALID_LEVELS.contains(&self.default_severity.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid severity '{}'. Must be one of: {}",
                self.default_severity,
                VALID_LEVELS.join(", ")
            )));
        }
        if !VALID_LEVELS.contains(&self.default_impact.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid impact '{}'. Must be one of: {}",
                self.default_impact,
                VALID_LEVELS.join(", ")
            )));
        }
        Ok(())
    }
}
