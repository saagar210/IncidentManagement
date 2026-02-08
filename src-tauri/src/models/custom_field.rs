use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFieldDefinition {
    pub id: String,
    pub name: String,
    pub field_type: String,
    #[serde(default)]
    pub options: String,
    #[serde(default)]
    pub display_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCustomFieldRequest {
    pub name: String,
    pub field_type: String,
    #[serde(default)]
    pub options: String,
    #[serde(default)]
    pub display_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCustomFieldRequest {
    pub name: Option<String>,
    pub field_type: Option<String>,
    pub options: Option<String>,
    pub display_order: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFieldValue {
    pub incident_id: String,
    pub field_id: String,
    pub value: String,
}

const VALID_FIELD_TYPES: &[&str] = &["text", "number", "select"];
const MAX_NAME_LEN: usize = 200;
const MAX_OPTIONS_LEN: usize = 2000;

impl CreateCustomFieldRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::Validation("Field name is required".into()));
        }
        if self.name.len() > MAX_NAME_LEN {
            return Err(AppError::Validation("Field name too long".into()));
        }
        if !VALID_FIELD_TYPES.contains(&self.field_type.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid field type '{}'. Must be one of: {}",
                self.field_type,
                VALID_FIELD_TYPES.join(", ")
            )));
        }
        if self.options.len() > MAX_OPTIONS_LEN {
            return Err(AppError::Validation("Options text too long".into()));
        }
        Ok(())
    }
}

impl UpdateCustomFieldRequest {
    pub fn validate(&self) -> AppResult<()> {
        if let Some(ref name) = self.name {
            if name.trim().is_empty() {
                return Err(AppError::Validation("Field name cannot be empty".into()));
            }
            if name.len() > MAX_NAME_LEN {
                return Err(AppError::Validation("Field name too long".into()));
            }
        }
        if let Some(ref ft) = self.field_type {
            if !VALID_FIELD_TYPES.contains(&ft.as_str()) {
                return Err(AppError::Validation(format!(
                    "Invalid field type '{}'. Must be one of: {}",
                    ft,
                    VALID_FIELD_TYPES.join(", ")
                )));
            }
        }
        if let Some(ref opts) = self.options {
            if opts.len() > MAX_OPTIONS_LEN {
                return Err(AppError::Validation("Options text too long".into()));
            }
        }
        Ok(())
    }
}
