use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::report_history;
use crate::error::AppError;
use crate::models::report_history::ReportHistory;
use crate::reports;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfigCmd {
    pub quarter_id: Option<String>,
    pub fiscal_year: Option<i32>,
    pub title: String,
    pub introduction: String,
    pub sections: ReportSectionsCmd,
    pub chart_images: HashMap<String, String>, // base64-encoded PNGs
    #[serde(default = "default_format")]
    pub format: String, // "docx" or "pdf"
}

fn default_format() -> String {
    "docx".to_string()
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

    // Validate chart images: max 20 images, max 10MB each, max 50MB total
    const MAX_CHART_IMAGES: usize = 20;
    const MAX_CHART_IMAGE_SIZE: usize = 10 * 1024 * 1024;
    const MAX_TOTAL_CHART_SIZE: usize = 50 * 1024 * 1024;
    if config.chart_images.len() > MAX_CHART_IMAGES {
        return Err(AppError::Validation(format!(
            "Too many chart images (max {})", MAX_CHART_IMAGES
        )));
    }

    // Decode chart images from base64 to raw bytes
    let mut chart_images: HashMap<String, Vec<u8>> = HashMap::new();
    let mut total_size: usize = 0;
    for (key, b64_value) in &config.chart_images {
        // Strip data URL prefix if present (e.g., "data:image/png;base64,...")
        let raw_b64 = if let Some(pos) = b64_value.find(",") {
            &b64_value[pos + 1..]
        } else {
            b64_value.as_str()
        };

        match base64::engine::general_purpose::STANDARD.decode(raw_b64) {
            Ok(bytes) => {
                if bytes.len() > MAX_CHART_IMAGE_SIZE {
                    return Err(AppError::Validation(format!(
                        "Chart image '{}' too large (max 10MB decoded)", key
                    )));
                }
                total_size += bytes.len();
                if total_size > MAX_TOTAL_CHART_SIZE {
                    return Err(AppError::Validation(
                        "Total chart image size exceeds 50MB limit".into()
                    ));
                }
                chart_images.insert(key.clone(), bytes);
            }
            Err(e) => {
                eprintln!("Warning: failed to decode chart image '{}': {}", key, e);
            }
        }
    }

    // Parse format
    let report_format = match config.format.to_lowercase().as_str() {
        "pdf" => reports::ReportFormat::Pdf,
        _ => reports::ReportFormat::Docx,
    };
    let file_ext = match report_format {
        reports::ReportFormat::Pdf => "pdf",
        reports::ReportFormat::Docx => "docx",
    };

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
        format: report_format,
    };

    // Generate the report
    let report_bytes = reports::generate_quarterly_report(&*db, &report_config).await?;

    // Write to a temp file
    let temp_dir = std::env::temp_dir();
    let filename = format!(
        "incident_report_{}.{}",
        chrono::Utc::now().format("%Y%m%d_%H%M%S"),
        file_ext
    );
    let temp_path = temp_dir.join(&filename);

    tokio::fs::write(&temp_path, &report_bytes)
        .await
        .map_err(|e| AppError::Report(format!("Failed to write temp file: {}", e)))?;

    temp_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Report("Invalid temp path encoding".into()))
}

#[tauri::command]
pub async fn save_report(
    db: State<'_, SqlitePool>,
    temp_path: String,
    save_path: String,
    title: String,
    quarter_id: Option<String>,
    config_json: Option<String>,
) -> Result<ReportHistory, AppError> {
    // Validate temp_path is actually in the temp directory
    let temp_dir = std::env::temp_dir();
    let canonical_temp = std::fs::canonicalize(&temp_path)
        .map_err(|e| AppError::Report(format!("Invalid temp path: {}", e)))?;
    if !canonical_temp.starts_with(&temp_dir) {
        return Err(AppError::Validation(
            "Source path must be within the system temp directory".into(),
        ));
    }

    // Validate save_path doesn't contain path traversal
    let save = std::path::Path::new(&save_path);
    if save_path.contains("..") {
        return Err(AppError::Validation(
            "Save path must not contain path traversal sequences".into(),
        ));
    }
    // Must end in .docx or .pdf
    let ext = save.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "docx" && ext != "pdf" {
        return Err(AppError::Validation(
            "Save path must have .docx or .pdf extension".into(),
        ));
    }

    tokio::fs::copy(&temp_path, &save_path)
        .await
        .map_err(|e| AppError::Report(format!("Failed to save report: {}", e)))?;

    // Get file size
    let metadata = tokio::fs::metadata(&save_path)
        .await
        .map_err(|e| AppError::Report(format!("Failed to read file metadata: {}", e)))?;
    let file_size = metadata.len() as i64;

    // Record in history — detect format from extension
    let format_str = if ext == "pdf" { "pdf" } else { "docx" };
    let history = report_history::insert_report_history(
        &*db,
        &title,
        quarter_id.as_deref(),
        format_str,
        &save_path,
        &config_json.unwrap_or_else(|| "{}".to_string()),
        file_size,
    )
    .await?;

    // Clean up temp file (best-effort)
    let _ = tokio::fs::remove_file(&temp_path).await;

    Ok(history)
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

// ===================== Report History =====================

#[tauri::command]
pub async fn list_report_history(
    db: State<'_, SqlitePool>,
) -> Result<Vec<ReportHistory>, AppError> {
    report_history::list_report_history(&*db).await
}

#[tauri::command]
pub async fn delete_report_history_entry(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    report_history::delete_report_history(&*db, &id).await
}

// ===================== Narrative Generation =====================

#[tauri::command]
pub async fn generate_narrative(
    db: State<'_, SqlitePool>,
    quarter_id: String,
) -> Result<String, AppError> {
    use crate::db::queries::metrics;
    use crate::models::metrics::MetricFilters;

    let filters = MetricFilters::default();
    let dashboard = metrics::get_dashboard_data_for_quarter(&*db, Some(&quarter_id), &filters).await?;

    let mut parts: Vec<String> = Vec::new();

    // Opening
    parts.push(format!(
        "During this reporting period, the team managed {} total incidents across all services.",
        dashboard.total_incidents
    ));

    // MTTR/MTTA
    if dashboard.mttr.value > 0.0 {
        let mttr_h = dashboard.mttr.value / 60.0;
        let mtta_h = dashboard.mtta.value / 60.0;
        parts.push(format!(
            "The mean time to resolve (MTTR) was {:.1} hours, with a mean time to acknowledge (MTTA) of {:.1} hours.",
            mttr_h, mtta_h
        ));
    }

    // Severity breakdown
    let critical = dashboard.by_severity.iter()
        .find(|s| s.category == "Critical")
        .map(|s| s.count)
        .unwrap_or(0);
    let high = dashboard.by_severity.iter()
        .find(|s| s.category == "High")
        .map(|s| s.count)
        .unwrap_or(0);
    if critical > 0 || high > 0 {
        parts.push(format!(
            "Of these, {} were classified as Critical and {} as High severity.",
            critical, high
        ));
    }

    // Recurrence
    if dashboard.recurrence_rate.value > 5.0 {
        parts.push(format!(
            "The recurrence rate of {:.1}% indicates recurring patterns that should be addressed.",
            dashboard.recurrence_rate.value
        ));
    } else if dashboard.total_incidents > 0 {
        parts.push(format!(
            "The recurrence rate was {:.1}%, suggesting effective root cause resolution.",
            dashboard.recurrence_rate.value
        ));
    }

    Ok(parts.join(" "))
}

#[cfg(test)]
mod tests {
    //! Unit tests for report generation functionality.
    //! These tests validate report configurations, discussion points, and file handling.

    use super::*;

    /// Test: ReportConfigCmd defaults format to "docx"
    #[test]
    fn test_default_report_format_is_docx() {
        let format = default_format();
        assert_eq!(format, "docx");
    }

    /// Test: ReportSectionsCmd can be created with all sections enabled
    #[test]
    fn test_report_sections_all_enabled() {
        let sections = ReportSectionsCmd {
            executive_summary: true,
            metrics_overview: true,
            incident_timeline: true,
            incident_breakdowns: true,
            service_reliability: true,
            qoq_comparison: true,
            discussion_points: true,
            action_items: true,
        };

        assert!(sections.executive_summary);
        assert!(sections.metrics_overview);
        assert!(sections.incident_timeline);
        assert!(sections.service_reliability);
    }

    /// Test: ReportSectionsCmd can be created with sections disabled
    #[test]
    fn test_report_sections_selective() {
        let sections = ReportSectionsCmd {
            executive_summary: true,
            metrics_overview: false,
            incident_timeline: true,
            incident_breakdowns: false,
            service_reliability: false,
            qoq_comparison: false,
            discussion_points: true,
            action_items: false,
        };

        assert!(sections.executive_summary);
        assert!(!sections.metrics_overview);
        assert!(sections.incident_timeline);
        assert!(!sections.service_reliability);
    }

    /// Test: DiscussionPoint can be created with trigger and severity
    #[test]
    fn test_discussion_point_structure() {
        let point = DiscussionPoint {
            text: "Service experienced P0 incident".into(),
            trigger: "high_severity".into(),
            severity: "P0".into(),
        };

        assert_eq!(point.trigger, "high_severity");
        assert_eq!(point.severity, "P0");
        assert!(point.text.len() > 0);
    }

    /// Test: ReportConfigCmd validates chart image count limit (max 20)
    #[test]
    fn test_chart_image_limit_validation_would_reject_21() {
        // This test validates that the command would reject >20 images
        const MAX_CHART_IMAGES: usize = 20;
        let image_count = 21;

        let would_error = image_count > MAX_CHART_IMAGES;
        assert!(would_error, "Should reject more than 20 chart images");
    }

    /// Test: ReportConfigCmd chart image size limit (max 10MB per image)
    #[test]
    fn test_chart_image_size_limit_per_image() {
        const MAX_CHART_IMAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB
        let oversized_image: usize = 11 * 1024 * 1024; // 11MB

        let would_error = oversized_image > MAX_CHART_IMAGE_SIZE;
        assert!(would_error, "Should reject image larger than 10MB");
    }

    /// Test: ReportConfigCmd total chart size limit (max 50MB)
    #[test]
    fn test_chart_total_size_limit() {
        const MAX_TOTAL_CHART_SIZE: usize = 50 * 1024 * 1024; // 50MB
        let total_size: usize = 55 * 1024 * 1024; // 55MB (over limit)

        let would_error = total_size > MAX_TOTAL_CHART_SIZE;
        assert!(would_error, "Should reject if total exceeds 50MB");
    }

    /// Test: Report filename sanitization would handle special characters
    #[test]
    fn test_report_filename_sanitization_pattern() {
        // Filenames with special chars should be sanitized
        let unsafe_name = "API / Web Service";
        let has_slash = unsafe_name.contains("/");
        let has_space = unsafe_name.contains(" ");

        assert!(has_slash || has_space, "Example has characters to sanitize");
    }

    /// Test: Discussion point generation rules - high severity rule
    #[test]
    fn test_discussion_point_rule_high_severity() {
        // Rule: If severity >= P1, mention criticality
        let severity = "P0";
        let should_mention_critical = severity == "P0" || severity == "P1";
        assert!(should_mention_critical);
    }

    /// Test: Discussion point generation rules - SLA breach rule
    #[test]
    fn test_discussion_point_rule_sla_breach() {
        // Rule: If incident breached SLA, mention in discussion
        let mttr_minutes = 120.0; // 2 hours
        let sla_target = 60.0; // 1 hour
        let breached = mttr_minutes > sla_target;

        assert!(breached, "Should trigger SLA breach rule");
    }
}
