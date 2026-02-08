use super::{client, OllamaState};
use crate::error::AppResult;

pub fn root_cause_system() -> &'static str {
    "You are a senior site reliability engineer specializing in root cause analysis. \
     Analyze incidents systematically using the 5 Whys methodology and fault tree analysis. \
     Be specific, actionable, and rank causes by likelihood."
}

pub fn root_cause_prompt(
    title: &str,
    severity: &str,
    service: &str,
    symptoms: &str,
    timeline: &str,
) -> String {
    format!(
        "Analyze this incident and suggest 3-5 ranked root causes with investigation steps:\n\n\
        Title: {}\n\
        Severity: {}\n\
        Service: {}\n\
        Symptoms: {}\n\
        Timeline: {}\n\n\
        For each root cause:\n\
        1. State the suspected root cause clearly\n\
        2. Explain why it's likely (evidence from symptoms/timeline)\n\
        3. List 2-3 concrete investigation steps to confirm or rule it out\n\
        4. Suggest immediate mitigation if this cause is confirmed\n\n\
        Rank from most likely to least likely. Be specific to the service and symptoms described.",
        title,
        severity,
        service,
        if symptoms.is_empty() { "Not provided" } else { symptoms },
        if timeline.is_empty() { "Not provided" } else { timeline },
    )
}

pub async fn suggest_root_causes(
    state: &OllamaState,
    title: &str,
    severity: &str,
    service: &str,
    symptoms: &str,
    timeline: &str,
) -> AppResult<String> {
    let prompt = root_cause_prompt(title, severity, service, symptoms, timeline);
    client::generate(
        state,
        &state.primary_model,
        &prompt,
        Some(root_cause_system()),
    )
    .await
}
