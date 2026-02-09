use serde::{Deserialize, Serialize};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributingFactor {
    pub id: String,
    pub incident_id: String,
    pub category: String,
    pub description: String,
    pub is_root: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContributingFactorRequest {
    pub incident_id: String,
    pub category: String,
    pub description: String,
    #[serde(default)]
    pub is_root: bool,
}

const VALID_CATEGORIES: &[&str] = &["Process", "Tooling", "Communication", "Human Factors", "External"];

impl CreateContributingFactorRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.incident_id.trim().is_empty() {
            return Err(AppError::Validation("Incident ID is required".into()));
        }
        if !VALID_CATEGORIES.contains(&self.category.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid category '{}'. Must be one of: {}", self.category, VALID_CATEGORIES.join(", ")
            )));
        }
        if self.description.trim().is_empty() {
            return Err(AppError::Validation("Description is required".into()));
        }
        if self.description.len() > 5000 {
            return Err(AppError::Validation("Description too long (max 5000 chars)".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmortemTemplate {
    pub id: String,
    pub name: String,
    pub incident_type: String,
    pub template_content: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Postmortem {
    pub id: String,
    pub incident_id: String,
    pub template_id: Option<String>,
    pub content: String,
    pub status: String,
    pub reminder_at: Option<String>,
    pub completed_at: Option<String>,
    #[serde(default)]
    pub no_action_items_justified: bool,
    #[serde(default)]
    pub no_action_items_justification: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePostmortemRequest {
    pub incident_id: String,
    pub template_id: Option<String>,
    #[serde(default = "default_pm_content")]
    pub content: String,
}

fn default_pm_content() -> String {
    "{}".to_string()
}

impl CreatePostmortemRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.incident_id.trim().is_empty() {
            return Err(AppError::Validation("Incident ID is required".into()));
        }
        if self.content.len() > 100_000 {
            return Err(AppError::Validation("Content too large".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePostmortemRequest {
    pub content: Option<String>,
    pub status: Option<String>,
    pub reminder_at: Option<String>,
    pub no_action_items_justified: Option<bool>,
    pub no_action_items_justification: Option<String>,
}

const VALID_PM_STATUSES: &[&str] = &["draft", "review", "final"];

impl UpdatePostmortemRequest {
    pub fn validate(&self) -> AppResult<()> {
        if let Some(ref status) = self.status {
            if !VALID_PM_STATUSES.contains(&status.as_str()) {
                return Err(AppError::Validation(format!(
                    "Invalid status '{}'. Must be one of: {}", status, VALID_PM_STATUSES.join(", ")
                )));
            }
        }
        if let Some(ref content) = self.content {
            if content.len() > 100_000 {
                return Err(AppError::Validation("Content too large".into()));
            }
        }
        if let Some(ref justification) = self.no_action_items_justification {
            if justification.len() > 10_000 {
                return Err(AppError::Validation("Justification too long (max 10000 chars)".into()));
            }
        }
        Ok(())
    }
}
