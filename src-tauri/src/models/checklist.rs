use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistTemplate {
    pub id: String,
    pub name: String,
    pub service_id: Option<String>,
    pub incident_type: Option<String>,
    pub is_active: bool,
    pub items: Vec<ChecklistTemplateItem>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistTemplateItem {
    pub id: String,
    pub template_id: String,
    pub label: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChecklistTemplateRequest {
    pub name: String,
    pub service_id: Option<String>,
    pub incident_type: Option<String>,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChecklistTemplateRequest {
    pub name: Option<String>,
    pub service_id: Option<String>,
    pub incident_type: Option<String>,
    pub is_active: Option<bool>,
    pub items: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentChecklist {
    pub id: String,
    pub incident_id: String,
    pub template_id: Option<String>,
    pub name: String,
    pub items: Vec<ChecklistItem>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: String,
    pub checklist_id: String,
    pub template_item_id: Option<String>,
    pub label: String,
    pub is_checked: bool,
    pub checked_at: Option<String>,
    pub checked_by: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIncidentChecklistRequest {
    pub incident_id: String,
    pub template_id: Option<String>,
    pub name: Option<String>,
    pub items: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleChecklistItemRequest {
    pub checked_by: Option<String>,
}

const MAX_NAME_LEN: usize = 200;
const MAX_ITEM_LABEL_LEN: usize = 500;
const MAX_ITEMS: usize = 50;

impl CreateChecklistTemplateRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::Validation("Template name is required".into()));
        }
        if self.name.len() > MAX_NAME_LEN {
            return Err(AppError::Validation("Template name too long".into()));
        }
        if self.items.is_empty() {
            return Err(AppError::Validation(
                "At least one checklist item is required".into(),
            ));
        }
        if self.items.len() > MAX_ITEMS {
            return Err(AppError::Validation(format!(
                "Too many items (max {})",
                MAX_ITEMS
            )));
        }
        for item in &self.items {
            if item.trim().is_empty() {
                return Err(AppError::Validation(
                    "Checklist item label cannot be empty".into(),
                ));
            }
            if item.len() > MAX_ITEM_LABEL_LEN {
                return Err(AppError::Validation("Checklist item label too long".into()));
            }
        }
        Ok(())
    }
}

impl CreateIncidentChecklistRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.incident_id.trim().is_empty() {
            return Err(AppError::Validation("incident_id is required".into()));
        }
        if self.template_id.is_none() && self.items.is_none() {
            return Err(AppError::Validation(
                "Either template_id or items must be provided".into(),
            ));
        }
        if let Some(ref items) = self.items {
            if items.is_empty() {
                return Err(AppError::Validation(
                    "At least one checklist item is required".into(),
                ));
            }
            if items.len() > MAX_ITEMS {
                return Err(AppError::Validation(format!(
                    "Too many items (max {})",
                    MAX_ITEMS
                )));
            }
            for item in items {
                if item.trim().is_empty() {
                    return Err(AppError::Validation(
                        "Checklist item label cannot be empty".into(),
                    ));
                }
            }
        }
        if let Some(ref name) = self.name {
            if name.len() > MAX_NAME_LEN {
                return Err(AppError::Validation("Checklist name too long".into()));
            }
        }
        Ok(())
    }
}
