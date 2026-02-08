use super::{client, prompts, OllamaState};
use crate::error::AppResult;

pub async fn generate_stakeholder_update(
    state: &OllamaState,
    title: &str,
    severity: &str,
    status: &str,
    service: &str,
    impact: &str,
    notes: &str,
) -> AppResult<String> {
    let prompt =
        prompts::stakeholder_prompt(title, severity, status, service, impact, notes);
    client::generate(
        state,
        &state.primary_model,
        &prompt,
        Some(prompts::stakeholder_system()),
    )
    .await
}
