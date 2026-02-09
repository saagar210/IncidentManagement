use super::OllamaState;
use crate::error::{AppError, AppResult};

/// Check if Ollama is reachable
pub async fn check_health(state: &OllamaState) -> bool {
    let url = format!("{}/api/tags", state.base_url);
    match reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

/// Generate text from Ollama
pub async fn generate(
    state: &OllamaState,
    model: &str,
    prompt: &str,
    system: Option<&str>,
) -> AppResult<String> {
    let is_available = *state.available.read().await;
    if !is_available {
        return Err(AppError::Ai(
            "Ollama is not available. Please install and start Ollama.".into(),
        ));
    }

    let url = format!("{}/api/generate", state.base_url);
    let mut body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": {
            "temperature": 0.7,
            "num_predict": 2048,
        }
    });

    if let Some(sys) = system {
        body["system"] = serde_json::Value::String(sys.to_string());
    }

    let resp = reqwest::Client::new()
        .post(&url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| AppError::Ai(format!("Ollama request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = match resp.text().await {
            Ok(t) => t,
            Err(e) => format!("<failed to read response body: {}>", e),
        };
        return Err(AppError::Ai(format!(
            "Ollama returned {}: {}",
            status, text
        )));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::Ai(format!("Failed to parse Ollama response: {}", e)))?;

    json["response"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Ai("No response field in Ollama output".into()))
}

/// Update availability status
pub async fn update_health(state: &OllamaState) {
    let healthy = check_health(state).await;
    let mut available = state.available.write().await;
    *available = healthy;
}
