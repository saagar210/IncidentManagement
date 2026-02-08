use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

pub const VALID_ROLES: &[&str] = &[
    "Incident Commander",
    "Communications Lead",
    "Technical Lead",
    "Scribe",
    "SME",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentRole {
    pub id: String,
    pub incident_id: String,
    pub role: String,
    pub assignee: String,
    pub is_primary: bool,
    pub assigned_at: String,
    pub unassigned_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignRoleRequest {
    pub incident_id: String,
    pub role: String,
    pub assignee: String,
    #[serde(default = "default_true")]
    pub is_primary: bool,
}

fn default_true() -> bool {
    true
}

impl AssignRoleRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.incident_id.trim().is_empty() {
            return Err(AppError::Validation("incident_id is required".into()));
        }
        if self.assignee.trim().is_empty() {
            return Err(AppError::Validation("assignee is required".into()));
        }
        if self.assignee.len() > 200 {
            return Err(AppError::Validation("assignee name too long".into()));
        }
        if !VALID_ROLES.contains(&self.role.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid role '{}'. Must be one of: {}",
                self.role,
                VALID_ROLES.join(", ")
            )));
        }
        Ok(())
    }
}
