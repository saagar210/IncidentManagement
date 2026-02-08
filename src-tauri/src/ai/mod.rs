pub mod client;
pub mod dedup;
pub mod postmortem;
pub mod prompts;
pub mod root_cause;
pub mod similar;
pub mod stakeholder;
pub mod summarize;
pub mod trends;

use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct OllamaState {
    pub available: Arc<RwLock<bool>>,
    pub base_url: String,
    pub primary_model: String,
    pub fast_model: String,
}

impl Default for OllamaState {
    fn default() -> Self {
        Self {
            available: Arc::new(RwLock::new(false)),
            base_url: "http://localhost:11434".to_string(),
            primary_model: "qwen3:30b-a3b".to_string(),
            fast_model: "qwen3:4b".to_string(),
        }
    }
}
