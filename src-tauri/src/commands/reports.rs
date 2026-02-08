use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sqlx::SqlitePool;
use tauri::State;

use crate::error::AppError;
use crate::reports;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfigCmd {
    pub quarter_id: Option<String>,
    pub fiscal_year: Option<i32>,
    pub title: String,
    pub introduction: String,
    pub sections: ReportSectionsCmd,
    pub chart_images: HashMap<String, String>, // base64-encoded PNGs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSectionsCmd {
    pub executive_summary: bool,
    pub metrics_overview: bool,
    pub incident_timeline: bool,
    pub incident_breakdowns: bool,
    pub service_reliability: bool,
    pub qoq_comparison: bool,
    pub discussion_points: bool,
    pub action_items: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionPoint {
    pub text: String,
    pub trigger: String,
    pub severity: String,
}

#[tauri::command]
pub async fn generate_report(
    db: State<'_, SqlitePool>,
    config: ReportConfigCmd,
) -> Result<String, AppError> {
    use base64::Engine;

    // Decode chart images from base64 to raw bytes
    let mut chart_images: HashMap<String, Vec<u8>> = HashMap::new();
    for (key, b64_value) in &config.chart_images {
        // Strip data URL prefix if present (e.g., "data:image/png;base64,...")
        let raw_b64 = if let Some(pos) = b64_value.find(",") {
            &b64_value[pos + 1..]
        } else {
            b64_value.as_str()
        };

        match base64::engine::general_purpose::STANDARD.decode(raw_b64) {
            Ok(bytes) => {
                chart_images.insert(key.clone(), bytes);
            }
            Err(e) => {
                eprintln!("Warning: failed to decode chart image '{}': {}", key, e);
            }
        }
    }

    // Convert command config to internal report config
    let report_config = reports::ReportConfig {
        quarter_id: config.quarter_id,
        fiscal_year: config.fiscal_year,
        title: config.title,
        introduction: config.introduction,
        sections: reports::ReportSections {
            executive_summary: config.sections.executive_summary,
            metrics_overview: config.sections.metrics_overview,
            incident_timeline: config.sections.incident_timeline,
            incident_breakdowns: config.sections.incident_breakdowns,
            service_reliability: config.sections.service_reliability,
            qoq_comparison: config.sections.qoq_comparison,
            discussion_points: config.sections.discussion_points,
            action_items: config.sections.action_items,
        },
        chart_images,
    };

    // Generate the DOCX
    let docx_bytes = reports::generate_quarterly_report(&*db, &report_config).await?;

    // Write to a temp file
    let temp_dir = std::env::temp_dir();
    let filename = format!(
        "incident_report_{}.docx",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let temp_path = temp_dir.join(&filename);

    std::fs::write(&temp_path, &docx_bytes)
        .map_err(|e| AppError::Report(format!("Failed to write temp file: {}", e)))?;

    temp_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Report("Invalid temp path encoding".into()))
}

#[tauri::command]
pub async fn save_report(
    temp_path: String,
    save_path: String,
) -> Result<(), AppError> {
    std::fs::copy(&temp_path, &save_path)
        .map_err(|e| AppError::Report(format!("Failed to save report: {}", e)))?;

    // Clean up temp file (best-effort)
    let _ = std::fs::remove_file(&temp_path);

    Ok(())
}

#[tauri::command]
pub async fn generate_discussion_points(
    db: State<'_, SqlitePool>,
    quarter_id: String,
) -> Result<Vec<DiscussionPoint>, AppError> {
    let points = reports::generate_discussion_points_only(&*db, &quarter_id).await?;

    Ok(points
        .into_iter()
        .map(|p| DiscussionPoint {
            text: p.text,
            trigger: p.trigger,
            severity: p.severity,
        })
        .collect())
}
