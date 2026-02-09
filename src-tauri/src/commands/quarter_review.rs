use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::{incidents, settings};
use crate::error::AppError;
use crate::models::incident::{Incident, IncidentFilters};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReadinessFinding {
    pub rule_key: String,
    pub severity: String, // "critical" | "warning"
    pub message: String,
    pub incident_ids: Vec<String>,
    pub remediation: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuarterReadinessReport {
    pub quarter_id: String,
    pub quarter_label: String,
    pub total_incidents: i64,
    pub ready_incidents: i64,
    pub needs_attention_incidents: i64,
    pub findings: Vec<ReadinessFinding>,
}

fn is_empty(s: &str) -> bool {
    s.trim().is_empty()
}

fn incident_missing_required_fields(inc: &Incident) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if is_empty(&inc.title) {
        missing.push("title");
    }
    if is_empty(&inc.service_id) {
        missing.push("service_id");
    }
    if is_empty(&inc.severity) {
        missing.push("severity");
    }
    if is_empty(&inc.impact) {
        missing.push("impact");
    }
    if is_empty(&inc.status) {
        missing.push("status");
    }
    if is_empty(&inc.started_at) {
        missing.push("started_at");
    }
    if is_empty(&inc.detected_at) {
        missing.push("detected_at");
    }
    missing
}

fn incident_has_timestamp_ordering_issue(inc: &Incident) -> bool {
    // These comparisons assume ISO-8601 strings, which is the app contract.
    if !is_empty(&inc.detected_at) && !is_empty(&inc.started_at) && inc.detected_at < inc.started_at {
        return true;
    }
    if let Some(ref responded) = inc.responded_at {
        if responded < &inc.detected_at {
            return true;
        }
    }
    if let Some(ref acknowledged) = inc.acknowledged_at {
        if acknowledged < &inc.started_at {
            return true;
        }
    }
    if let Some(ref resolved) = inc.resolved_at {
        if resolved < &inc.started_at {
            return true;
        }
    }
    false
}

fn incident_status_requires_resolved_at(inc: &Incident) -> bool {
    inc.status == "Resolved" && inc.resolved_at.as_deref().unwrap_or("").trim().is_empty()
}

fn incident_is_carried_over(inc: &Incident, quarter_end: &str) -> bool {
    // Included in-quarter by detected_at, but unresolved by quarter end.
    // Treat resolved_at after quarter end as "carried over" for leadership discussion.
    match inc.resolved_at.as_deref() {
        None => true,
        Some(resolved) => resolved > quarter_end,
    }
}

#[tauri::command]
pub async fn get_quarter_readiness(
    db: State<'_, SqlitePool>,
    quarter_id: String,
) -> Result<QuarterReadinessReport, AppError> {
    compute_quarter_readiness(&*db, &quarter_id).await
}

pub async fn compute_quarter_readiness(
    db: &SqlitePool,
    quarter_id: &str,
) -> Result<QuarterReadinessReport, AppError> {
    let quarter = settings::get_quarter_by_id(db, quarter_id).await?;
    let quarter_dates = Some((quarter.start_date.clone(), quarter.end_date.clone()));

    // No additional filters: readiness is quarter-scoped, and incidents are included by detected_at.
    let filters = IncidentFilters::default();
    let incs = incidents::list_incidents(db, &filters, quarter_dates).await?;

    let mut missing_required: Vec<String> = Vec::new();
    let mut bad_ordering: Vec<String> = Vec::new();
    let mut resolved_missing_ts: Vec<String> = Vec::new();
    let mut carried_over: Vec<String> = Vec::new();

    let mut ready = 0_i64;
    for inc in &incs {
        let mut ok = true;

        if !incident_missing_required_fields(inc).is_empty() {
            missing_required.push(inc.id.clone());
            ok = false;
        }
        if incident_has_timestamp_ordering_issue(inc) {
            bad_ordering.push(inc.id.clone());
            ok = false;
        }
        if incident_status_requires_resolved_at(inc) {
            resolved_missing_ts.push(inc.id.clone());
            ok = false;
        }
        if inc.status != "Resolved" && incident_is_carried_over(inc, &quarter.end_date) {
            carried_over.push(inc.id.clone());
            // carried-over is a warning, not a hard failure for readiness.
        }

        if ok {
            ready += 1;
        }
    }

    let total = incs.len() as i64;
    let needs_attention = total - ready;

    let mut findings: Vec<ReadinessFinding> = Vec::new();
    if !missing_required.is_empty() {
        findings.push(ReadinessFinding {
            rule_key: "missing_required_fields".into(),
            severity: "critical".into(),
            message: "Some incidents are missing required fields for quarterly reporting.".into(),
            incident_ids: missing_required,
            remediation: "Open each incident and fill in the missing required fields (title, service, severity/impact, status, started_at, detected_at).".into(),
        });
    }
    if !bad_ordering.is_empty() {
        findings.push(ReadinessFinding {
            rule_key: "timestamp_ordering".into(),
            severity: "critical".into(),
            message: "Some incidents have inconsistent timestamp ordering.".into(),
            incident_ids: bad_ordering,
            remediation: "Fix timestamps so detected_at >= started_at, and other timestamps do not precede detected/started.".into(),
        });
    }
    if !resolved_missing_ts.is_empty() {
        findings.push(ReadinessFinding {
            rule_key: "resolved_requires_resolved_at".into(),
            severity: "critical".into(),
            message: "Some incidents are marked Resolved but have no resolved_at timestamp.".into(),
            incident_ids: resolved_missing_ts,
            remediation: "Set resolved_at for resolved incidents (or change status if not resolved).".into(),
        });
    }
    if !carried_over.is_empty() {
        findings.push(ReadinessFinding {
            rule_key: "carried_over".into(),
            severity: "warning".into(),
            message: "Some incidents detected this quarter were not resolved by quarter end (carried over).".into(),
            incident_ids: carried_over,
            remediation: "Confirm these are correct and ensure the quarterly packet includes a carried-over section with current status/context.".into(),
        });
    }

    Ok(QuarterReadinessReport {
        quarter_id: quarter_id.to_string(),
        quarter_label: quarter.label,
        total_incidents: total,
        ready_incidents: ready,
        needs_attention_incidents: needs_attention,
        findings,
    })
}
