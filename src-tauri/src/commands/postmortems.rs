use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::{audit, postmortems};
use crate::error::AppError;
use crate::models::postmortem::{
    ContributingFactor, CreateContributingFactorRequest, CreatePostmortemRequest,
    Postmortem, PostmortemTemplate, UpdatePostmortemRequest,
};

#[derive(serde::Serialize)]
pub struct PostmortemReadinessItem {
    pub code: String,
    pub label: String,
    pub destination: String,
}

#[derive(serde::Serialize)]
pub struct PostmortemReadiness {
    pub can_finalize: bool,
    pub missing: Vec<PostmortemReadinessItem>,
}

#[tauri::command]
pub async fn list_contributing_factors(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<Vec<ContributingFactor>, AppError> {
    postmortems::list_contributing_factors(&*db, &incident_id).await
}

#[tauri::command]
pub async fn create_contributing_factor(
    db: State<'_, SqlitePool>,
    req: CreateContributingFactorRequest,
) -> Result<ContributingFactor, AppError> {
    req.validate()?;
    let id = format!("cf-{}", uuid::Uuid::new_v4());
    let result = postmortems::create_contributing_factor(&*db, &id, &req).await?;
    if let Err(e) = audit::insert_audit_entry(
        &*db,
        "contributing_factor",
        &id,
        "created",
        &format!("Added contributing factor: {} ({})", &req.category, &req.description.chars().take(50).collect::<String>()),
        "",
    )
    .await
    {
        eprintln!(
            "Warning: failed to write audit entry for contributing factor create: {}",
            e
        );
    }
    Ok(result)
}

#[tauri::command]
pub async fn delete_contributing_factor(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    postmortems::delete_contributing_factor(&*db, &id).await?;
    if let Err(e) = audit::insert_audit_entry(&*db, "contributing_factor", &id, "deleted", "Deleted contributing factor", "").await {
        eprintln!("Warning: failed to write audit entry for contributing factor delete: {}", e);
    }
    Ok(())
}

#[tauri::command]
pub async fn list_postmortem_templates(
    db: State<'_, SqlitePool>,
) -> Result<Vec<PostmortemTemplate>, AppError> {
    postmortems::list_postmortem_templates(&*db).await
}

#[tauri::command]
pub async fn get_postmortem_by_incident(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<Option<Postmortem>, AppError> {
    postmortems::get_postmortem_by_incident(&*db, &incident_id).await
}

#[tauri::command]
pub async fn create_postmortem(
    db: State<'_, SqlitePool>,
    req: CreatePostmortemRequest,
) -> Result<Postmortem, AppError> {
    req.validate()?;
    let id = format!("pm-{}", uuid::Uuid::new_v4());
    let result = postmortems::create_postmortem(&*db, &id, &req).await?;
    if let Err(e) = audit::insert_audit_entry(
        &*db,
        "postmortem",
        &id,
        "created",
        &format!("Created post-mortem for incident {}", &req.incident_id),
        "",
    )
    .await
    {
        eprintln!("Warning: failed to write audit entry for postmortem create: {}", e);
    }
    Ok(result)
}

#[tauri::command]
pub async fn update_postmortem(
    db: State<'_, SqlitePool>,
    id: String,
    req: UpdatePostmortemRequest,
) -> Result<Postmortem, AppError> {
    req.validate()?;
    let result = postmortems::update_postmortem(&*db, &id, &req).await?;
    if let Err(e) = audit::insert_audit_entry(
        &*db,
        "postmortem",
        &id,
        "updated",
        "Updated post-mortem",
        "",
    )
    .await
    {
        eprintln!("Warning: failed to write audit entry for postmortem update: {}", e);
    }
    Ok(result)
}

#[tauri::command]
pub async fn delete_postmortem(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    postmortems::delete_postmortem(&*db, &id).await?;
    if let Err(e) = audit::insert_audit_entry(&*db, "postmortem", &id, "deleted", "Deleted post-mortem", "").await {
        eprintln!("Warning: failed to write audit entry for postmortem delete: {}", e);
    }
    Ok(())
}

#[tauri::command]
pub async fn list_postmortems(
    db: State<'_, SqlitePool>,
    status: Option<String>,
) -> Result<Vec<Postmortem>, AppError> {
    postmortems::list_postmortems(&*db, status.as_deref()).await
}

#[tauri::command]
pub async fn get_postmortem_readiness(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<PostmortemReadiness, AppError> {
    let pm = postmortems::get_postmortem_by_incident(&*db, &incident_id).await?;
    let Some(pm) = pm else {
        return Ok(PostmortemReadiness {
            can_finalize: false,
            missing: vec![PostmortemReadinessItem {
                code: "POSTMORTEM_NOT_CREATED".to_string(),
                label: "Post-mortem must be created".to_string(),
                destination: "postmortem".to_string(),
            }],
        });
    };

    // Reuse the same server-side requirements enforced by update_postmortem().
    let missing = postmortems::compute_readiness_missing_items(
        &*db,
        &pm.incident_id,
        &pm.content,
        pm.no_action_items_justified,
        &pm.no_action_items_justification,
    )
    .await?;

    Ok(PostmortemReadiness {
        can_finalize: missing.is_empty(),
        missing: missing
            .into_iter()
            .map(|m| PostmortemReadinessItem {
                code: m.code,
                label: m.label,
                destination: m.destination,
            })
            .collect(),
    })
}
