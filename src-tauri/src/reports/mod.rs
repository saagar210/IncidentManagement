pub mod charts;
pub mod markdown;
pub mod pdf;
pub mod sections;

use std::collections::HashMap;
use std::io::Cursor;

use docx_rs::*;
use sqlx::SqlitePool;
use base64::Engine;
use sha2::{Digest, Sha256};

use crate::commands::quarter_review::{compute_quarter_readiness, QuarterReadinessReport};
use crate::db::queries::{incidents, settings, metrics};
use crate::db::queries::quarter_finalization as qf;
use crate::db::queries::timeline_events as tme;
use crate::error::{AppError, AppResult};
use crate::models::incident::{ActionItem, Incident, IncidentFilters};
use crate::models::metrics::{MetricFilters, QuarterlyTrends};
use crate::models::quarter::QuarterConfig;
use crate::reports::sections::discussion_points::DiscussionPoint;

#[derive(Debug, Clone, serde::Deserialize)]
struct QuarterSnapshotPayloadV1 {
    readiness: QuarterReadinessReport,
    overrides: Vec<qf::QuarterOverride>,
    dashboard: crate::models::metrics::DashboardData,
    incident_ids: Vec<String>,
    notable_incident_ids: Vec<String>,
}

/// Report section configuration.
#[derive(Debug, Clone)]
pub struct ReportSections {
    pub executive_summary: bool,
    pub metrics_overview: bool,
    pub incident_timeline: bool,
    pub incident_breakdowns: bool,
    pub service_reliability: bool,
    pub qoq_comparison: bool,
    pub discussion_points: bool,
    pub action_items: bool,
}

/// Full report config used by the generation pipeline.
#[derive(Debug, Clone)]
pub struct ReportConfig {
    pub quarter_id: Option<String>,
    #[allow(dead_code)]
    pub fiscal_year: Option<i32>,
    pub title: String,
    pub introduction: String,
    pub sections: ReportSections,
    pub chart_images: HashMap<String, Vec<u8>>, // decoded PNG bytes
    pub format: ReportFormat,
}

/// Supported output formats for reports.
#[derive(Debug, Clone, PartialEq)]
pub enum ReportFormat {
    Docx,
    Pdf,
}

/// Collected data for report generation.
struct ReportData {
    incidents: Vec<Incident>,
    prev_incidents: Vec<Incident>,
    action_items_all: Vec<ActionItem>,
    quarter: Option<QuarterConfig>,
    #[allow(dead_code)]
    prev_quarter: Option<QuarterConfig>,
    readiness: Option<QuarterReadinessReport>,
    overrides: Vec<qf::QuarterOverride>,
    finalization: Option<qf::QuarterFinalization>,
    inputs_hash: String,
    facts_changed_since_finalization: bool,
    timeline_events: std::collections::HashMap<String, Vec<tme::TimelineEvent>>,
    mttr: f64,
    mtta: f64,
    total_incidents: i64,
    recurrence_rate: f64,
    avg_tickets: f64,
    prev_mttr: Option<f64>,
    prev_mtta: Option<f64>,
    prev_total: Option<i64>,
    prev_recurrence: Option<f64>,
    prev_tickets: Option<f64>,
    trends: QuarterlyTrends,
}

/// Main entry point: generate a quarterly report and return the bytes.
/// Supports both DOCX and PDF formats based on config.format.
pub async fn generate_quarterly_report(
    db: &SqlitePool,
    config: &ReportConfig,
) -> AppResult<Vec<u8>> {
    let data = fetch_report_data(db, config).await?;

    match config.format {
        ReportFormat::Pdf => {
            pdf::build_pdf(
                config,
                &data.incidents,
                &data.prev_incidents,
                data.quarter.as_ref(),
                data.readiness.as_ref(),
                &data.overrides,
                data.finalization.as_ref(),
                data.facts_changed_since_finalization,
                &data.inputs_hash,
                &data.timeline_events,
                data.mttr,
                data.mtta,
                data.total_incidents,
                data.recurrence_rate,
                data.avg_tickets,
                data.prev_mttr,
                data.prev_mtta,
                data.prev_total,
                data.prev_recurrence,
                data.prev_tickets,
                &data.action_items_all,
                &data.trends,
            )
        }
        ReportFormat::Docx => {
            let docx = build_document(config, &data);

            let mut buf: Vec<u8> = Vec::new();
            let cursor = Cursor::new(&mut buf);
            docx.build()
                .pack(cursor)
                .map_err(|e| AppError::Report(format!("Failed to build DOCX: {}", e)))?;

            Ok(buf)
        }
    }
}

/// Generate discussion points for preview (no DOCX build).
pub async fn generate_discussion_points_only(
    db: &SqlitePool,
    quarter_id: &str,
) -> AppResult<Vec<DiscussionPoint>> {
    let quarter = settings::get_quarter_by_id(db, quarter_id).await?;
    let prev_quarter = settings::get_previous_quarter(
        db,
        quarter.fiscal_year,
        quarter.quarter_number,
    )
    .await?;

    let filters = IncidentFilters {
        quarter_id: Some(quarter_id.to_string()),
        sort_order: Some("asc".to_string()),
        ..Default::default()
    };
    let quarter_dates = Some((quarter.start_date.clone(), quarter.end_date.clone()));
    let current_incidents = incidents::list_incidents(db, &filters, quarter_dates).await?;
    let total_incidents = current_incidents.len() as i64;

    // Calc MTTR for current quarter
    let mttr = calc_avg_duration(&current_incidents);

    // Previous quarter data
    let (prev_incidents, prev_mttr, prev_total) = if let Some(ref pq) = prev_quarter {
        let pf = IncidentFilters {
            sort_order: Some("asc".to_string()),
            ..Default::default()
        };
        let pd = Some((pq.start_date.clone(), pq.end_date.clone()));
        let pi = incidents::list_incidents(db, &pf, pd).await?;
        let pm = calc_avg_duration(&pi);
        let pt = pi.len() as i64;
        (pi, Some(pm), Some(pt))
    } else {
        (vec![], None, None)
    };

    let all_action_items = incidents::list_action_items(db, None).await?;

    Ok(sections::discussion_points::generate(
        &current_incidents,
        &prev_incidents,
        mttr,
        prev_mttr,
        total_incidents,
        prev_total,
        &all_action_items,
    ))
}

/// Fetch all data needed for the report.
async fn fetch_report_data(db: &SqlitePool, config: &ReportConfig) -> AppResult<ReportData> {
    // Get quarter info
    let quarter = if let Some(ref qid) = config.quarter_id {
        Some(settings::get_quarter_by_id(db, qid).await?)
    } else {
        None
    };

    let prev_quarter = if let Some(ref q) = quarter {
        settings::get_previous_quarter(db, q.fiscal_year, q.quarter_number).await?
    } else {
        None
    };

    let quarter_id = config.quarter_id.as_deref();

    // Readiness + overrides (best-effort; Phase 2 "killer workflow" expects them present for quarters).
    let readiness = if let Some(qid) = quarter_id {
        Some(compute_quarter_readiness(db, qid).await.map_err(|e| AppError::Report(format!("Readiness failed: {}", e)))?)
    } else {
        None
    };
    let overrides = if let Some(qid) = quarter_id {
        qf::list_overrides_for_quarter(db, qid).await?
    } else {
        vec![]
    };
    let finalization = if let Some(qid) = quarter_id {
        qf::get_finalization(db, qid).await?
    } else {
        None
    };

    // If finalized, prefer the frozen snapshot incident membership + metrics, but only if facts haven't changed.
    let snapshot_payload: Option<QuarterSnapshotPayloadV1> = if let Some(qid) = quarter_id {
        if finalization.is_some() {
            let snap = qf::get_snapshot_for_quarter(db, qid).await?;
            if let Some(snap) = snap {
                let payload: QuarterSnapshotPayloadV1 = serde_json::from_str(&snap.snapshot_json)
                    .map_err(|e| AppError::Report(format!("Invalid quarter snapshot JSON: {}", e)))?;
                Some(payload)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Fetch current quarter incidents
    let current_incidents = if let Some(ref payload) = snapshot_payload {
        incidents::list_incidents_by_ids(db, &payload.incident_ids).await?
    } else {
        let quarter_dates = quarter
            .as_ref()
            .map(|q| (q.start_date.clone(), q.end_date.clone()));
        let filters = IncidentFilters {
            sort_order: Some("asc".to_string()),
            ..Default::default()
        };
        incidents::list_incidents(db, &filters, quarter_dates).await?
    };

    let inputs_hash = compute_inputs_hash_from_incidents(&current_incidents)?;
    let facts_changed_since_finalization = finalization
        .as_ref()
        .map(|f| f.inputs_hash != inputs_hash)
        .unwrap_or(false);

    let total_incidents = current_incidents.len() as i64;

    // Previous quarter incidents
    let prev_dates = prev_quarter
        .as_ref()
        .map(|q| (q.start_date.clone(), q.end_date.clone()));
    let prev_incidents = if prev_dates.is_some() {
        let pf = IncidentFilters {
            sort_order: Some("asc".to_string()),
            ..Default::default()
        };
        incidents::list_incidents(db, &pf, prev_dates).await?
    } else {
        vec![]
    };

    // Compute metrics (prefer frozen snapshot values when finalized and facts unchanged).
    let (mttr, mtta, recurrence_rate, avg_tickets, total_incidents_metric) = if let Some(ref payload) = snapshot_payload {
        if !facts_changed_since_finalization {
            (
                payload.dashboard.mttr.value,
                payload.dashboard.mtta.value,
                payload.dashboard.recurrence_rate.value,
                payload.dashboard.avg_tickets.value,
                payload.dashboard.total_incidents,
            )
        } else {
            (
                calc_avg_duration(&current_incidents),
                calc_avg_mtta(&current_incidents),
                calc_recurrence_rate(&current_incidents),
                calc_avg_tickets(&current_incidents),
                total_incidents,
            )
        }
    } else {
        (
            calc_avg_duration(&current_incidents),
            calc_avg_mtta(&current_incidents),
            calc_recurrence_rate(&current_incidents),
            calc_avg_tickets(&current_incidents),
            total_incidents,
        )
    };

    let prev_mttr = if !prev_incidents.is_empty() {
        Some(calc_avg_duration(&prev_incidents))
    } else {
        None
    };
    let prev_mtta = if !prev_incidents.is_empty() {
        Some(calc_avg_mtta(&prev_incidents))
    } else {
        None
    };
    let prev_total = if !prev_incidents.is_empty() {
        Some(prev_incidents.len() as i64)
    } else {
        None
    };
    let prev_recurrence = if !prev_incidents.is_empty() {
        Some(calc_recurrence_rate(&prev_incidents))
    } else {
        None
    };
    let prev_tickets = if !prev_incidents.is_empty() {
        Some(calc_avg_tickets(&prev_incidents))
    } else {
        None
    };

    // Get all action items
    let action_items_all = incidents::list_action_items(db, None).await?;

    // Get quarterly trends via dashboard metrics (or frozen snapshot when available and consistent).
    let metric_filters = MetricFilters::default();
    let dashboard = metrics::get_dashboard_data_for_quarter(db, quarter_id, &metric_filters).await?;
    let trends = if let Some(ref payload) = snapshot_payload {
        if !facts_changed_since_finalization {
            payload.dashboard.trends.clone()
        } else {
            dashboard.trends.clone()
        }
    } else {
        dashboard.trends.clone()
    };

    let notable_incident_ids = if let Some(ref payload) = snapshot_payload {
        payload.notable_incident_ids.clone()
    } else {
        top_notable_incidents(&current_incidents, 5)
    };

    let mut timeline_ids: Vec<String> = Vec::new();
    for inc in &current_incidents {
        if inc.priority == "P0" || inc.priority == "P1" {
            timeline_ids.push(inc.id.clone());
        }
    }
    for id in &notable_incident_ids {
        if !timeline_ids.iter().any(|x| x == id) {
            timeline_ids.push(id.clone());
        }
    }
    let timeline_events = tme::list_timeline_events_for_incidents(db, &timeline_ids).await?;

    let readiness_for_report = if let Some(ref payload) = snapshot_payload {
        if !facts_changed_since_finalization {
            Some(payload.readiness.clone())
        } else {
            readiness.clone()
        }
    } else {
        readiness.clone()
    };

    let overrides_for_report = if let Some(ref payload) = snapshot_payload {
        if !facts_changed_since_finalization {
            payload.overrides.clone()
        } else {
            overrides.clone()
        }
    } else {
        overrides.clone()
    };

    Ok(ReportData {
        incidents: current_incidents,
        prev_incidents,
        action_items_all,
        quarter,
        prev_quarter,
        readiness: readiness_for_report,
        overrides: overrides_for_report,
        finalization,
        inputs_hash,
        facts_changed_since_finalization,
        timeline_events,
        mttr,
        mtta,
        total_incidents: total_incidents_metric,
        recurrence_rate,
        avg_tickets,
        prev_mttr,
        prev_mtta,
        prev_total,
        prev_recurrence,
        prev_tickets,
        trends,
    })
}

/// Build the DOCX document from collected data.
fn build_document(config: &ReportConfig, data: &ReportData) -> Docx {
    let mut docx = Docx::new();

    // Title page
    docx = docx.add_paragraph(
        Paragraph::new()
            .add_run(Run::new().add_text(&config.title).bold().size(36 * 2))
            .style("Heading1")
    );

    if let Some(ref q) = data.quarter {
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(&q.label).size(16 * 2))
        );
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(format!(
                    "Period: {} to {}",
                    q.start_date, q.end_date
                )).size(12 * 2))
        );
    }

    docx = docx.add_paragraph(sections::spacer());

    // Phase 2: Confidence checklist + provenance policy (quarter-centric, drives packet trust).
    if let Some(ref readiness) = data.readiness {
        docx = sections::confidence::build(
            docx,
            sections::confidence::ConfidenceSectionInput {
                readiness,
                overrides: &data.overrides,
                finalization: data.finalization.as_ref(),
                facts_changed_since_finalization: data.facts_changed_since_finalization,
                inputs_hash: &data.inputs_hash,
            },
        );
    }

    // Add enabled sections
    if config.sections.executive_summary {
        docx = sections::executive_summary::build(
            docx,
            &data.incidents,
            data.mttr,
            data.mtta,
            data.recurrence_rate,
            data.total_incidents,
            &config.introduction,
        );
    }

    if config.sections.metrics_overview {
        docx = sections::metrics_overview::build(
            docx,
            data.mttr,
            data.mtta,
            data.total_incidents,
            data.recurrence_rate,
            data.avg_tickets,
            data.prev_mttr,
            data.prev_mtta,
            data.prev_total,
            data.prev_recurrence,
            data.prev_tickets,
            &config.chart_images,
        );
    }

    if config.sections.incident_timeline {
        docx = sections::incident_timeline::build(docx, &data.incidents);
    }

    if config.sections.incident_breakdowns {
        docx = sections::incident_breakdowns::build(docx, &data.incidents, &data.timeline_events);
    }

    if config.sections.service_reliability {
        docx = sections::service_reliability::build(docx, &data.incidents);
    }

    if config.sections.qoq_comparison {
        docx = sections::qoq_comparison::build(docx, &data.trends);
    }

    if config.sections.discussion_points {
        let points = sections::discussion_points::generate(
            &data.incidents,
            &data.prev_incidents,
            data.mttr,
            data.prev_mttr,
            data.total_incidents,
            data.prev_total,
            &data.action_items_all,
        );
        docx = sections::discussion_points::build(docx, &points);
    }

    if config.sections.action_items {
        docx = sections::action_items::build(docx, &data.action_items_all);
    }

    docx
}

// -- In-memory metric helpers (avoid extra DB queries) --

fn calc_avg_duration(incidents: &[Incident]) -> f64 {
    let resolved: Vec<&Incident> = incidents
        .iter()
        .filter(|i| i.duration_minutes.is_some())
        .collect();
    if resolved.is_empty() {
        return 0.0;
    }
    let total: f64 = resolved
        .iter()
        .map(|i| i.duration_minutes.unwrap_or(0) as f64)
        .sum();
    total / resolved.len() as f64
}

fn calc_avg_mtta(incidents: &[Incident]) -> f64 {
    // MTTA = responded_at - detected_at in minutes
    // Only count incidents where both timestamps parse successfully and duration is non-negative
    let mtta_values: Vec<f64> = incidents
        .iter()
        .filter_map(|i| {
            let detected = chrono::NaiveDateTime::parse_from_str(&i.detected_at, "%Y-%m-%dT%H:%M:%SZ").ok()?;
            let responded = chrono::NaiveDateTime::parse_from_str(i.responded_at.as_ref()?, "%Y-%m-%dT%H:%M:%SZ").ok()?;
            let diff = responded.signed_duration_since(detected);
            let minutes = diff.num_minutes() as f64;
            // Filter out negative durations (bad data: responded before detected)
            if minutes < 0.0 { None } else { Some(minutes) }
        })
        .collect();
    if mtta_values.is_empty() {
        return 0.0;
    }
    let total: f64 = mtta_values.iter().sum();
    total / mtta_values.len() as f64
}

fn calc_recurrence_rate(incidents: &[Incident]) -> f64 {
    if incidents.is_empty() {
        return 0.0;
    }
    let recurring = incidents.iter().filter(|i| i.is_recurring).count();
    (recurring as f64 / incidents.len() as f64) * 100.0
}

fn calc_avg_tickets(incidents: &[Incident]) -> f64 {
    if incidents.is_empty() {
        return 0.0;
    }
    let total: f64 = incidents.iter().map(|i| i.tickets_submitted as f64).sum();
    total / incidents.len() as f64
}

fn compute_inputs_hash_from_incidents(incs: &[Incident]) -> AppResult<String> {
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

fn top_notable_incidents(incs: &[Incident], n: usize) -> Vec<String> {
    let mut v: Vec<&Incident> = incs.iter().collect();
    v.sort_by(|a, b| {
        let ad = a.duration_minutes.unwrap_or(0);
        let bd = b.duration_minutes.unwrap_or(0);
        bd.cmp(&ad).then_with(|| a.id.cmp(&b.id))
    });
    v.into_iter().take(n).map(|i| i.id.clone()).collect()
}
