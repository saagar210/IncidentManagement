use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::report_history;
use crate::db::queries::{incidents, quarter_finalization as qf, settings};
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

        let bytes = base64::engine::general_purpose::STANDARD
            .decode(raw_b64)
            .map_err(|e| {
                AppError::Validation(format!("Invalid base64 for chart image '{}': {}", key, e))
            })?;
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

    // Compute report inputs hash for repeatability tracking (quarter membership is by detected_at).
    let inputs_hash = if let Some(ref qid) = quarter_id {
        compute_quarter_inputs_hash(&*db, qid).await?
    } else {
        "".to_string()
    };

    let quarter_finalized_at = if let Some(ref qid) = quarter_id {
        qf::get_finalization(&*db, qid).await?.map(|f| f.finalized_at)
    } else {
        None
    };

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
        &inputs_hash,
        1,
        quarter_finalized_at.as_deref(),
    )
    .await?;

    // Clean up temp file (best-effort, but don't swallow unexpected errors silently)
    if let Err(e) = tokio::fs::remove_file(&temp_path).await {
        if e.kind() != std::io::ErrorKind::NotFound {
            eprintln!("Warning: failed to remove temp report file '{}': {}", temp_path, e);
        }
    }

    Ok(history)
}

async fn compute_quarter_inputs_hash(db: &SqlitePool, quarter_id: &str) -> Result<String, AppError> {
    use base64::Engine;
    use sha2::{Digest, Sha256};
    use crate::models::incident::IncidentFilters;

    let q = settings::get_quarter_by_id(db, quarter_id).await?;
    let quarter_dates = Some((q.start_date.clone(), q.end_date.clone()));
    let filters = IncidentFilters { sort_order: Some("asc".to_string()), ..Default::default() };
    let incs = incidents::list_incidents(db, &filters, quarter_dates).await?;

    let mut rows: Vec<serde_json::Value> = incs
        .iter()
        .map(|i| {
            serde_json::json!({
                "id": i.id,
                "service_id": i.service_id,
                "severity": i.severity,
                "impact": i.impact,
                "status": i.status,
                "started_at": i.started_at,
                "detected_at": i.detected_at,
                "acknowledged_at": i.acknowledged_at,
                "responded_at": i.responded_at,
                "resolved_at": i.resolved_at,
                "external_ref": i.external_ref,
                "reopen_count": i.reopen_count
            })
        })
        .collect();
    rows.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));

    let json = serde_json::to_vec(&rows)
        .map_err(|e| AppError::Internal(format!("Failed to serialize report inputs hash: {}", e)))?;
    let mut hasher = Sha256::new();
    hasher.update(&json);
    let digest = hasher.finalize();
    Ok(base64::engine::general_purpose::STANDARD.encode(digest))
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
