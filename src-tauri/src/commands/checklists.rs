use sqlx::SqlitePool;
use tauri::State;

use crate::db::queries::{audit, checklists};
use crate::error::AppError;
use crate::models::checklist::{
    ChecklistItem, ChecklistTemplate, CreateChecklistTemplateRequest,
    CreateIncidentChecklistRequest, IncidentChecklist, ToggleChecklistItemRequest,
    UpdateChecklistTemplateRequest,
};

// ── Template Commands ─────────────────────────────────────────────

#[tauri::command]
pub async fn create_checklist_template(
    db: State<'_, SqlitePool>,
    req: CreateChecklistTemplateRequest,
) -> Result<ChecklistTemplate, AppError> {
    req.validate()?;
    let id = format!("ctpl-{}", uuid::Uuid::new_v4());
    let result = checklists::create_template(
        &*db,
        &id,
        &req.name,
        req.service_id.as_deref(),
        req.incident_type.as_deref(),
        &req.items,
    )
    .await?;
    if let Err(e) = audit::insert_audit_entry(
        &*db,
        "checklist_template",
        &id,
        "created",
        &format!("Created checklist template: {}", &req.name),
        "",
    )
    .await
    {
        eprintln!(
            "Warning: failed to write audit entry for checklist template create: {}",
            e
        );
    }
    Ok(result)
}

#[tauri::command]
pub async fn update_checklist_template(
    db: State<'_, SqlitePool>,
    id: String,
    req: UpdateChecklistTemplateRequest,
) -> Result<ChecklistTemplate, AppError> {
    let result = checklists::update_template(
        &*db,
        &id,
        req.name.as_deref(),
        None, // service_id changes not exposed in simple update
        None, // incident_type changes not exposed in simple update
        req.is_active,
        req.items.as_deref(),
    )
    .await?;
    if let Err(e) = audit::insert_audit_entry(
        &*db,
        "checklist_template",
        &id,
        "updated",
        "Updated checklist template",
        "",
    )
    .await
    {
        eprintln!(
            "Warning: failed to write audit entry for checklist template update: {}",
            e
        );
    }
    Ok(result)
}

#[tauri::command]
pub async fn delete_checklist_template(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    checklists::delete_template(&*db, &id).await?;
    if let Err(e) = audit::insert_audit_entry(
        &*db,
        "checklist_template",
        &id,
        "deleted",
        "Deleted checklist template",
        "",
    )
    .await
    {
        eprintln!(
            "Warning: failed to write audit entry for checklist template delete: {}",
            e
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn list_checklist_templates(
    db: State<'_, SqlitePool>,
) -> Result<Vec<ChecklistTemplate>, AppError> {
    checklists::list_templates(&*db).await
}

// ── Incident Checklist Commands ───────────────────────────────────

#[tauri::command]
pub async fn create_incident_checklist(
    db: State<'_, SqlitePool>,
    req: CreateIncidentChecklistRequest,
) -> Result<IncidentChecklist, AppError> {
    req.validate()?;
    let id = format!("icl-{}", uuid::Uuid::new_v4());

    let result = if let Some(template_id) = &req.template_id {
        checklists::create_checklist_from_template(&*db, &id, &req.incident_id, template_id).await?
    } else {
        let items = req.items.as_ref().ok_or_else(|| {
            AppError::Validation("items required when not using template".into())
        })?;
        let name = req
            .name
            .as_deref()
            .unwrap_or("Checklist");
        checklists::create_incident_checklist(&*db, &id, &req.incident_id, None, name, items)
            .await?
    };

    if let Err(e) = audit::insert_audit_entry(
        &*db,
        "incident",
        &req.incident_id,
        "checklist_created",
        &format!("Created checklist: {}", &result.name),
        "",
    )
    .await
    {
        eprintln!(
            "Warning: failed to write audit entry for incident checklist create: {}",
            e
        );
    }
    Ok(result)
}

#[tauri::command]
pub async fn list_incident_checklists(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<Vec<IncidentChecklist>, AppError> {
    checklists::list_incident_checklists(&*db, &incident_id).await
}

#[tauri::command]
pub async fn delete_incident_checklist(
    db: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    checklists::delete_incident_checklist(&*db, &id).await?;
    if let Err(e) = audit::insert_audit_entry(
        &*db,
        "incident_checklist",
        &id,
        "deleted",
        "Deleted incident checklist",
        "",
    )
    .await
    {
        eprintln!(
            "Warning: failed to write audit entry for incident checklist delete: {}",
            e
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_checklist_item(
    db: State<'_, SqlitePool>,
    item_id: String,
    req: ToggleChecklistItemRequest,
) -> Result<ChecklistItem, AppError> {
    checklists::toggle_checklist_item(&*db, &item_id, req.checked_by.as_deref()).await
}
