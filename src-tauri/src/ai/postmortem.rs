use super::{client, prompts, OllamaState};
use crate::error::AppResult;

pub async fn generate_postmortem_draft(
    state: &OllamaState,
    title: &str,
    severity: &str,
    service: &str,
    root_cause: &str,
    resolution: &str,
    lessons: &str,
    contributing_factors: &[String],
) -> AppResult<String> {
    let prompt = prompts::postmortem_prompt(
        title,
        severity,
        service,
        root_cause,
        resolution,
        lessons,
        contributing_factors,
    );
    client::generate(
        state,
        &state.primary_model,
        &prompt,
        Some(prompts::postmortem_system()),
    )
    .await
}
