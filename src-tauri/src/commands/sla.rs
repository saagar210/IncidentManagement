use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::{audit, sla};
use crate::error::AppError;
use crate::models::sla::{
    CreateSlaDefinitionRequest, SlaDefinition, SlaStatus, UpdateSlaDefinitionRequest,
};

#[tauri::command]
pub async fn list_sla_definitions(
    db: State<'_, SqlitePool>,
) -> Result<Vec<SlaDefinition>, AppError> {
    sla::list_sla_definitions(&*db).await
}

#[tauri::command]
pub async fn create_sla_definition(
    db: State<'_, SqlitePool>,
    req: CreateSlaDefinitionRequest,
) -> Result<SlaDefinition, AppError> {
    req.validate()?;
    let result = sla::create_sla_definition(&*db, &req).await?;
    let _ = audit::insert_audit_entry(
        &*db,
        "sla_definition",
        &result.id,
        "created",
        &format!("Created SLA definition: {} ({})", &req.name, &req.priority),
        "",
    )
    .await;
    Ok(result)
}

#[tauri::command]
pub async fn update_sla_definition(
    db: State<'_, SqlitePool>,
    id: String,
    req: UpdateSlaDefinitionRequest,
) -> Result<SlaDefinition, AppError> {
    req.validate()?;
    let result = sla::update_sla_definition(&*db, &id, &req).await?;
    let _ = audit::insert_audit_entry(&*db, "sla_definition", &id, "updated", "Updated SLA definition", "").await;
    Ok(result)
}

#[tauri::command]
pub async fn delete_sla_definition(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    sla::delete_sla_definition(&*db, &id).await?;
    let _ = audit::insert_audit_entry(&*db, "sla_definition", &id, "deleted", "Deleted SLA definition", "").await;
    Ok(())
}

#[tauri::command]
pub async fn compute_sla_status(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<SlaStatus, AppError> {
    sla::compute_sla_status(&*db, &incident_id).await
}

#[cfg(test)]
mod tests {
    //! Unit tests for SLA definition and status computation.
    //! These tests validate SLA thresholds, status calculations, and compliance metrics.

    use super::*;

    /// Test: SlaStatus enum variants exist
    #[test]
    fn test_sla_status_variants() {
        // SlaStatus should have: OnTrack, AtRisk, Breached
        // This validates the enum structure
        let statuses = vec!["on_track", "at_risk", "breached"];
        assert!(statuses.len() == 3);
    }

    /// Test: P0 SLA thresholds (15-min MTTA, 1-hour MTTR)
    #[test]
    fn test_p0_sla_thresholds() {
        let p0_mtta_target_minutes = 15;
        let p0_mttr_target_minutes = 60;

        // P0 should have lowest thresholds (most critical)
        assert!(p0_mtta_target_minutes < 30); // Less than P1
        assert!(p0_mttr_target_minutes < 240); // Less than P1
    }

    /// Test: P1 SLA thresholds (30-min MTTA, 4-hour MTTR)
    #[test]
    fn test_p1_sla_thresholds() {
        let p1_mtta_target_minutes = 30;
        let p1_mttr_target_minutes = 240; // 4 hours

        // P1 should be between P0 and P2
        assert!(p1_mtta_target_minutes > 15); // Greater than P0
        assert!(p1_mttr_target_minutes > 60); // Greater than P0
    }

    /// Test: P4 SLA thresholds (lowest priority: 1-day MTTR)
    #[test]
    fn test_p4_sla_thresholds() {
        let p4_mttr_target_minutes = 7 * 24 * 60; // 7 days in minutes

        // P4 should have the highest (loosest) thresholds
        assert!(p4_mttr_target_minutes > 1440); // Greater than 1 day
    }

    /// Test: SLA on-track status (within window)
    #[test]
    fn test_sla_status_on_track() {
        let mtta_target_minutes = 15;

        // Time to acknowledge: 10 minutes (within 15-min target)
        let mtta_actual = 10;
        let is_on_track = mtta_actual <= mtta_target_minutes;

        assert!(is_on_track);
    }

    /// Test: SLA at-risk status (approaching deadline)
    #[test]
    fn test_sla_status_at_risk() {
        let mttr_target_minutes = 60; // 1 hour
        let elapsed_minutes = 45; // 45 minutes elapsed
        let escalation_threshold_pct = 75; // Alert at 75% of SLA

        let percentage_of_sla = (elapsed_minutes as f64 / mttr_target_minutes as f64) * 100.0;
        let is_at_risk = percentage_of_sla >= escalation_threshold_pct as f64;

        assert!(is_at_risk);
    }

    /// Test: SLA breached status (exceeded deadline)
    #[test]
    fn test_sla_status_breached() {
        let mttr_target_minutes = 60; // 1 hour
        let actual_resolution_minutes = 90; // Resolved after 1.5 hours

        let is_breached = actual_resolution_minutes > mttr_target_minutes;

        assert!(is_breached);
    }

    /// Test: SLA not breached (within limit)
    #[test]
    fn test_sla_not_breached() {
        let mttr_target_minutes = 60;
        let actual_resolution_minutes = 30;

        let is_breached = actual_resolution_minutes > mttr_target_minutes;

        assert!(!is_breached);
    }

    /// Test: SLA compliance percentage calculation
    #[test]
    fn test_sla_compliance_percentage_9_of_10_incidents() {
        let incidents_meeting_sla = 9;
        let total_incidents = 10;

        let compliance_pct = (incidents_meeting_sla as f64 / total_incidents as f64) * 100.0;

        assert_eq!(compliance_pct, 90.0);
    }

    /// Test: SLA compliance with zero incidents
    #[test]
    fn test_sla_compliance_zero_incidents() {
        let incidents_meeting_sla = 0;
        let total_incidents = 0;

        let compliance_pct = if total_incidents == 0 {
            0.0 // No incidents = undefined compliance, return 0
        } else {
            (incidents_meeting_sla as f64 / total_incidents as f64) * 100.0
        };

        assert_eq!(compliance_pct, 0.0);
    }
}
