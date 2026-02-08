use super::{client, prompts, OllamaState};
use crate::error::AppResult;

pub async fn generate_summary(
    state: &OllamaState,
    title: &str,
    severity: &str,
    status: &str,
    service: &str,
    root_cause: &str,
    resolution: &str,
    notes: &str,
) -> AppResult<String> {
    let prompt = prompts::summarize_prompt(
        title, severity, status, service, root_cause, resolution, notes,
    );
    client::generate(
        state,
        &state.primary_model,
        &prompt,
        Some(prompts::summarize_system()),
    )
    .await
}
