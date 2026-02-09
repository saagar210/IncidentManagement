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
    #[serde(default)]
    pub inputs_hash: String,
    #[serde(default)]
    pub report_version: i64,
    pub quarter_finalized_at: Option<String>,
}
