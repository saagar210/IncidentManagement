use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub title: String,
    pub service_id: String,
    #[serde(default)]
    pub service_name: String,
    pub severity: String,
    pub impact: String,
    pub priority: String,
    pub status: String,
    pub started_at: String,
    pub detected_at: String,
    pub responded_at: Option<String>,
    pub resolved_at: Option<String>,
    pub duration_minutes: Option<i64>,
    #[serde(default)]
    pub root_cause: String,
    #[serde(default)]
    pub resolution: String,
    #[serde(default)]
    pub tickets_submitted: i64,
    #[serde(default)]
    pub affected_users: i64,
    #[serde(default)]
    pub is_recurring: bool,
    pub recurrence_of: Option<String>,
    #[serde(default)]
    pub lessons_learned: String,
    #[serde(default)]
    pub action_items: String,
    #[serde(default)]
    pub external_ref: String,
    #[serde(default)]
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIncidentRequest {
    pub title: String,
    pub service_id: String,
    pub severity: String,
    pub impact: String,
    pub status: String,
    pub started_at: String,
    pub detected_at: String,
    pub responded_at: Option<String>,
    pub resolved_at: Option<String>,
    #[serde(default)]
    pub root_cause: String,
    #[serde(default)]
    pub resolution: String,
    #[serde(default)]
    pub tickets_submitted: i64,
    #[serde(default)]
    pub affected_users: i64,
    #[serde(default)]
    pub is_recurring: bool,
    pub recurrence_of: Option<String>,
    #[serde(default)]
    pub lessons_learned: String,
    #[serde(default)]
    pub action_items: String,
    #[serde(default)]
    pub external_ref: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIncidentRequest {
    pub title: Option<String>,
    pub service_id: Option<String>,
    pub severity: Option<String>,
    pub impact: Option<String>,
    pub status: Option<String>,
    pub started_at: Option<String>,
    pub detected_at: Option<String>,
    pub responded_at: Option<String>,
    pub resolved_at: Option<String>,
    pub root_cause: Option<String>,
    pub resolution: Option<String>,
    pub tickets_submitted: Option<i64>,
    pub affected_users: Option<i64>,
    pub is_recurring: Option<bool>,
    pub recurrence_of: Option<String>,
    pub lessons_learned: Option<String>,
    pub action_items: Option<String>,
    pub external_ref: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IncidentFilters {
    pub service_id: Option<String>,
    pub severity: Option<String>,
    pub impact: Option<String>,
    pub status: Option<String>,
    pub quarter_id: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub id: String,
    pub incident_id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub status: String,
    #[serde(default)]
    pub owner: String,
    pub due_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateActionItemRequest {
    pub incident_id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_action_status")]
    pub status: String,
    #[serde(default)]
    pub owner: String,
    pub due_date: Option<String>,
}

fn default_action_status() -> String {
    "Open".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateActionItemRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub owner: Option<String>,
    pub due_date: Option<String>,
}

const VALID_SEVERITIES: &[&str] = &["Critical", "High", "Medium", "Low"];
const VALID_IMPACTS: &[&str] = &["Critical", "High", "Medium", "Low"];
const VALID_STATUSES: &[&str] = &["Active", "Monitoring", "Resolved", "Post-Mortem"];
const VALID_ACTION_STATUSES: &[&str] = &["Open", "In-Progress", "Done"];

impl CreateIncidentRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.title.trim().is_empty() {
            return Err(AppError::Validation("Title is required".into()));
        }
        if self.service_id.trim().is_empty() {
            return Err(AppError::Validation("Service is required".into()));
        }
        if !VALID_SEVERITIES.contains(&self.severity.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid severity '{}'. Must be one of: {}",
                self.severity,
                VALID_SEVERITIES.join(", ")
            )));
        }
        if !VALID_IMPACTS.contains(&self.impact.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid impact '{}'. Must be one of: {}",
                self.impact,
                VALID_IMPACTS.join(", ")
            )));
        }
        if !VALID_STATUSES.contains(&self.status.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid status '{}'. Must be one of: {}",
                self.status,
                VALID_STATUSES.join(", ")
            )));
        }
        if self.started_at.trim().is_empty() {
            return Err(AppError::Validation("Started at is required".into()));
        }
        if self.detected_at.trim().is_empty() {
            return Err(AppError::Validation("Detected at is required".into()));
        }
        // Date ordering validation
        if self.detected_at < self.started_at {
            return Err(AppError::Validation(
                "Detected at must be on or after started at".into(),
            ));
        }
        if let Some(ref responded) = self.responded_at {
            if responded < &self.detected_at {
                return Err(AppError::Validation(
                    "Responded at must be on or after detected at".into(),
                ));
            }
        }
        if let Some(ref resolved) = self.resolved_at {
            if resolved < &self.started_at {
                return Err(AppError::Validation(
                    "Resolved at must be on or after started at".into(),
                ));
            }
        }
        Ok(())
    }
}

impl CreateActionItemRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.title.trim().is_empty() {
            return Err(AppError::Validation("Action item title is required".into()));
        }
        if self.incident_id.trim().is_empty() {
            return Err(AppError::Validation("Incident ID is required".into()));
        }
        if !VALID_ACTION_STATUSES.contains(&self.status.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid action item status '{}'. Must be one of: {}",
                self.status,
                VALID_ACTION_STATUSES.join(", ")
            )));
        }
        Ok(())
    }
}
