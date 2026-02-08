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
    pub owner: String,
    pub tier: String,
    pub runbook: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    pub id: String,
    pub service_id: String,
    pub depends_on_service_id: String,
    pub depends_on_service_name: Option<String>,
    pub dependency_type: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceRequest {
    pub name: String,
    pub category: String,
    pub default_severity: String,
    pub default_impact: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default = "default_tier")]
    pub tier: String,
    #[serde(default)]
    pub runbook: String,
}

fn default_tier() -> String {
    "T3".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServiceRequest {
    pub name: Option<String>,
    pub category: Option<String>,
    pub default_severity: Option<String>,
    pub default_impact: Option<String>,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub tier: Option<String>,
    pub runbook: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceDependencyRequest {
    pub service_id: String,
    pub depends_on_service_id: String,
    #[serde(default = "default_dependency_type")]
    pub dependency_type: String,
}

fn default_dependency_type() -> String {
    "runtime".to_string()
}

pub const VALID_TIERS: &[&str] = &["T1", "T2", "T3", "T4"];
pub const VALID_DEPENDENCY_TYPES: &[&str] = &["runtime", "build", "data", "optional"];

const VALID_CATEGORIES: &[&str] = &[
    "Communication",
    "Infrastructure",
    "Development",
    "Productivity",
    "Security",
    "Other",
];

const VALID_LEVELS: &[&str] = &["Critical", "High", "Medium", "Low"];

const MAX_SERVICE_NAME_LEN: usize = 200;
const MAX_SERVICE_DESC_LEN: usize = 2_000;
const MAX_OWNER_LEN: usize = 200;
const MAX_RUNBOOK_LEN: usize = 50_000;

impl CreateServiceRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::Validation("Service name is required".into()));
        }
        if self.name.len() > MAX_SERVICE_NAME_LEN {
            return Err(AppError::Validation(format!(
                "Service name too long (max {} characters)", MAX_SERVICE_NAME_LEN
            )));
        }
        if self.description.len() > MAX_SERVICE_DESC_LEN {
            return Err(AppError::Validation("Service description too long".into()));
        }
        if self.owner.len() > MAX_OWNER_LEN {
            return Err(AppError::Validation("Service owner too long".into()));
        }
        if self.runbook.len() > MAX_RUNBOOK_LEN {
            return Err(AppError::Validation("Runbook too long".into()));
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
        if !VALID_TIERS.contains(&self.tier.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid tier '{}'. Must be one of: {}",
                self.tier,
                VALID_TIERS.join(", ")
            )));
        }
        Ok(())
    }
}

impl CreateServiceDependencyRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.service_id.trim().is_empty() {
            return Err(AppError::Validation("service_id is required".into()));
        }
        if self.depends_on_service_id.trim().is_empty() {
            return Err(AppError::Validation("depends_on_service_id is required".into()));
        }
        if self.service_id == self.depends_on_service_id {
            return Err(AppError::Validation("A service cannot depend on itself".into()));
        }
        if !VALID_DEPENDENCY_TYPES.contains(&self.dependency_type.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid dependency type '{}'. Must be one of: {}",
                self.dependency_type,
                VALID_DEPENDENCY_TYPES.join(", ")
            )));
        }
        Ok(())
    }
}

impl UpdateServiceRequest {
    pub fn validate(&self) -> AppResult<()> {
        if let Some(ref name) = self.name {
            if name.trim().is_empty() {
                return Err(AppError::Validation("Service name cannot be empty".into()));
            }
            if name.len() > MAX_SERVICE_NAME_LEN {
                return Err(AppError::Validation(format!(
                    "Service name too long (max {} characters)", MAX_SERVICE_NAME_LEN
                )));
            }
        }
        if let Some(ref description) = self.description {
            if description.len() > MAX_SERVICE_DESC_LEN {
                return Err(AppError::Validation("Service description too long".into()));
            }
        }
        if let Some(ref owner) = self.owner {
            if owner.len() > MAX_OWNER_LEN {
                return Err(AppError::Validation("Service owner too long".into()));
            }
        }
        if let Some(ref runbook) = self.runbook {
            if runbook.len() > MAX_RUNBOOK_LEN {
                return Err(AppError::Validation("Runbook too long".into()));
            }
        }
        if let Some(ref category) = self.category {
            if !VALID_CATEGORIES.contains(&category.as_str()) {
                return Err(AppError::Validation(format!(
                    "Invalid category '{}'. Must be one of: {}",
                    category, VALID_CATEGORIES.join(", ")
                )));
            }
        }
        if let Some(ref severity) = self.default_severity {
            if !VALID_LEVELS.contains(&severity.as_str()) {
                return Err(AppError::Validation(format!(
                    "Invalid severity '{}'. Must be one of: {}",
                    severity, VALID_LEVELS.join(", ")
                )));
            }
        }
        if let Some(ref impact) = self.default_impact {
            if !VALID_LEVELS.contains(&impact.as_str()) {
                return Err(AppError::Validation(format!(
                    "Invalid impact '{}'. Must be one of: {}",
                    impact, VALID_LEVELS.join(", ")
                )));
            }
        }
        if let Some(ref tier) = self.tier {
            if !VALID_TIERS.contains(&tier.as_str()) {
                return Err(AppError::Validation(format!(
                    "Invalid tier '{}'. Must be one of: {}",
                    tier, VALID_TIERS.join(", ")
                )));
            }
        }
        Ok(())
    }
}
