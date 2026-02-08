use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReportHistory {
    pub id: String,
    pub title: String,
    pub quarter_id: Option<String>,
    pub format: String,
    pub generated_at: String,
    pub file_path: String,
    pub config_json: String,
    pub file_size_bytes: Option<i64>,
}
