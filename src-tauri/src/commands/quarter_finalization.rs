use sqlx::SqlitePool;
use tauri::State;

use crate::commands::quarter_review::{compute_quarter_readiness, QuarterReadinessReport};
use crate::db::queries::{incidents, metrics, quarter_finalization};
use crate::db::queries::settings;
use crate::error::AppError;
use crate::models::incident::IncidentFilters;
use crate::models::metrics::MetricFilters;

use base64::Engine;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, serde::Serialize)]
pub struct QuarterFinalizationStatus {
    pub quarter_id: String,
    pub finalized: bool,
    pub finalization: Option<quarter_finalization::QuarterFinalization>,
    pub readiness: QuarterReadinessReport,
    pub overrides: Vec<quarter_finalization::QuarterOverride>,
    pub snapshot_inputs_hash: Option<String>,
    pub current_inputs_hash: String,
    pub facts_changed_since_finalization: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpsertOverrideCmd {
    pub quarter_id: String,
    pub rule_key: String,
    pub incident_id: String,
    pub reason: String,
    #[serde(default)]
    pub approved_by: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FinalizeQuarterCmd {
    pub quarter_id: String,
    #[serde(default)]
    pub finalized_by: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FinalizeQuarterResult {
    pub finalization: quarter_finalization::QuarterFinalization,
    pub snapshot: quarter_finalization::QuarterSnapshot,
}

fn require_overrides_for_critical_findings(
    readiness: &QuarterReadinessReport,
    overrides: &[quarter_finalization::QuarterOverride],
) -> Result<(), AppError> {
    // Gate on critical findings: every incident_id in a critical finding must have an override recorded.
    let mut missing: Vec<String> = Vec::new();
    for finding in readiness.findings.iter().filter(|f| f.severity == "critical") {
        for incident_id in &finding.incident_ids {
            let has = overrides
                .iter()
                .any(|o| o.rule_key == finding.rule_key && o.incident_id == *incident_id);
            if !has {
                missing.push(format!("{}:{}", finding.rule_key, incident_id));
            }
        }
    }
    if !missing.is_empty() {
        return Err(AppError::Validation(format!(
            "Cannot finalize: missing overrides for critical findings: {}",
            missing.join(", ")
        )));
    }
    Ok(())
}

fn carried_over_incident_ids(
    incs: &[crate::models::incident::Incident],
    quarter_end: &str,
) -> Vec<String> {
    incs.iter()
        .filter(|i| match i.resolved_at.as_deref() {
            None => true,
            Some(resolved) => resolved > quarter_end,
        })
        .map(|i| i.id.clone())
        .collect()
}

fn build_snapshot_json(
    quarter: &crate::models::quarter::QuarterConfig,
    readiness: &QuarterReadinessReport,
    overrides: &[quarter_finalization::QuarterOverride],
    dashboard: &crate::models::metrics::DashboardData,
    incident_ids: &[String],
    notable_incident_ids: &[String],
    carried_over_incident_ids: &[String],
    inputs_hash: &str,
) -> Result<String, AppError> {
    let snapshot_obj = serde_json::json!({
        "schema_version": 1,
        "quarter": {
            "id": quarter.id,
            "label": quarter.label,
            "start_date": quarter.start_date,
            "end_date": quarter.end_date
        },
        "readiness": readiness,
        "overrides": overrides,
        "dashboard": dashboard,
        "incident_ids": incident_ids,
        "notable_incident_ids": notable_incident_ids,
        "carried_over_incident_ids": carried_over_incident_ids,
        "generated_at": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "inputs_hash": inputs_hash
    });
    serde_json::to_string(&snapshot_obj)
        .map_err(|e| AppError::Report(format!("Failed to serialize quarter snapshot: {}", e)))
}

#[tauri::command]
pub async fn get_quarter_finalization_status(
    db: State<'_, SqlitePool>,
    quarter_id: String,
) -> Result<QuarterFinalizationStatus, AppError> {
    let readiness = compute_quarter_readiness(&*db, &quarter_id).await?;
    let overrides = quarter_finalization::list_overrides_for_quarter(&*db, &quarter_id).await?;
    let finalization = quarter_finalization::get_finalization(&*db, &quarter_id).await?;

    let current_inputs_hash = compute_quarter_inputs_hash(&*db, &quarter_id).await?;
    let snapshot_inputs_hash = finalization.as_ref().map(|f| f.inputs_hash.clone());
    let facts_changed = match snapshot_inputs_hash.as_ref() {
        None => false,
        Some(h) => h != &current_inputs_hash,
    };

    Ok(QuarterFinalizationStatus {
        quarter_id,
        finalized: finalization.is_some(),
        finalization,
        readiness,
        overrides,
        snapshot_inputs_hash,
        current_inputs_hash,
        facts_changed_since_finalization: facts_changed,
    })
}

#[tauri::command]
pub async fn upsert_quarter_override(
    db: State<'_, SqlitePool>,
    req: UpsertOverrideCmd,
) -> Result<quarter_finalization::QuarterOverride, AppError> {
    quarter_finalization::upsert_override(
        &*db,
        &req.quarter_id,
        &req.rule_key,
        &req.incident_id,
        &req.reason,
        &req.approved_by,
    )
    .await
}

#[tauri::command]
pub async fn finalize_quarter(
    db: State<'_, SqlitePool>,
    req: FinalizeQuarterCmd,
) -> Result<FinalizeQuarterResult, AppError> {
    let quarter = settings::get_quarter_by_id(&*db, &req.quarter_id).await?;

    let readiness = compute_quarter_readiness(&*db, &req.quarter_id).await?;
    let overrides = quarter_finalization::list_overrides_for_quarter(&*db, &req.quarter_id).await?;
    require_overrides_for_critical_findings(&readiness, &overrides)?;

    let metric_filters = MetricFilters::default();
    let dashboard = metrics::get_dashboard_data_for_quarter(&*db, Some(&req.quarter_id), &metric_filters).await?;

    let quarter_dates = Some((quarter.start_date.clone(), quarter.end_date.clone()));
    let filters = IncidentFilters { sort_order: Some("asc".to_string()), ..Default::default() };
    let incs = incidents::list_incidents(&*db, &filters, quarter_dates).await?;

    let inputs_hash = compute_inputs_hash_from_incidents(&incs)?;

    let notable_ids = top_notable_incidents(&incs, 5);
    let incident_ids: Vec<String> = incs.iter().map(|i| i.id.clone()).collect();
    let carried_over_ids = carried_over_incident_ids(&incs, quarter.end_date.as_str());
    let snapshot_json = build_snapshot_json(
        &quarter,
        &readiness,
        &overrides,
        &dashboard,
        &incident_ids,
        &notable_ids,
        &carried_over_ids,
        &inputs_hash,
    )?;

    let snapshot = quarter_finalization::upsert_snapshot(&*db, &req.quarter_id, &inputs_hash, &snapshot_json).await?;

    let finalized_by = if req.finalized_by.trim().is_empty() { "self".to_string() } else { req.finalized_by.clone() };
    let finalization = quarter_finalization::finalize_quarter(
        &*db,
        &req.quarter_id,
        &finalized_by,
        &snapshot.id,
        &inputs_hash,
        &req.notes,
    )
    .await?;

    Ok(FinalizeQuarterResult { finalization, snapshot })
}

#[tauri::command]
pub async fn unfinalize_quarter(
    db: State<'_, SqlitePool>,
    quarter_id: String,
) -> Result<(), AppError> {
    quarter_finalization::unfinalize_quarter(&*db, &quarter_id).await
}

async fn compute_quarter_inputs_hash(pool: &SqlitePool, quarter_id: &str) -> Result<String, AppError> {
    let quarter = settings::get_quarter_by_id(pool, quarter_id).await?;
    let quarter_dates = Some((quarter.start_date.clone(), quarter.end_date.clone()));
    let filters = IncidentFilters { sort_order: Some("asc".to_string()), ..Default::default() };
    let incs = incidents::list_incidents(pool, &filters, quarter_dates).await?;
    compute_inputs_hash_from_incidents(&incs)
}

fn compute_inputs_hash_from_incidents(incs: &[crate::models::incident::Incident]) -> Result<String, AppError> {
    // Stable hash: sort by id; include only "facts" that drive metrics + reporting trust.
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
        .map_err(|e| AppError::Internal(format!("Failed to serialize quarter inputs hash: {}", e)))?;
    let mut hasher = Sha256::new();
    hasher.update(&json);
    let digest = hasher.finalize();
    Ok(base64::engine::general_purpose::STANDARD.encode(digest))
}

fn top_notable_incidents(incs: &[crate::models::incident::Incident], n: usize) -> Vec<String> {
    let mut v: Vec<&crate::models::incident::Incident> = incs.iter().collect();
    v.sort_by(|a, b| {
        let ad = a.duration_minutes.unwrap_or(0);
        let bd = b.duration_minutes.unwrap_or(0);
        bd.cmp(&ad).then_with(|| a.id.cmp(&b.id))
    });
    v.into_iter().take(n).map(|i| i.id.clone()).collect()
}

#[tauri::command]
pub async fn delete_quarter_override(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    quarter_finalization::delete_override(&*db, &id).await?;
    Ok(())
}
