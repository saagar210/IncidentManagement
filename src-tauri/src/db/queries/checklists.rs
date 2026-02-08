use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::checklist::{
    ChecklistItem, ChecklistTemplate, ChecklistTemplateItem, IncidentChecklist,
};

// ── Template CRUD ─────────────────────────────────────────────────

pub async fn create_template(
    db: &SqlitePool,
    id: &str,
    name: &str,
    service_id: Option<&str>,
    incident_type: Option<&str>,
    items: &[String],
) -> AppResult<ChecklistTemplate> {
    sqlx::query(
        "INSERT INTO checklist_templates (id, name, service_id, incident_type) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(name)
    .bind(service_id)
    .bind(incident_type)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    for (i, label) in items.iter().enumerate() {
        let item_id = format!("cti-{}", uuid::Uuid::new_v4());
        sqlx::query(
            "INSERT INTO checklist_template_items (id, template_id, label, sort_order) VALUES (?, ?, ?, ?)",
        )
        .bind(&item_id)
        .bind(id)
        .bind(label)
        .bind(i as i32)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }

    get_template_by_id(db, id).await
}

pub async fn update_template(
    db: &SqlitePool,
    id: &str,
    name: Option<&str>,
    service_id: Option<Option<&str>>,
    incident_type: Option<Option<&str>>,
    is_active: Option<bool>,
    items: Option<&[String]>,
) -> AppResult<ChecklistTemplate> {
    let existing = get_template_by_id(db, id).await?;

    let name = name.unwrap_or(&existing.name);
    let is_active = is_active.unwrap_or(existing.is_active);

    // Handle service_id: if Some(Some("x")) -> set to "x", Some(None) -> set to null, None -> keep existing
    let svc_id = match service_id {
        Some(v) => v,
        None => existing.service_id.as_deref(),
    };
    let inc_type = match incident_type {
        Some(v) => v,
        None => existing.incident_type.as_deref(),
    };

    sqlx::query(
        "UPDATE checklist_templates SET name=?, service_id=?, incident_type=?, is_active=?, updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?",
    )
    .bind(name)
    .bind(svc_id)
    .bind(inc_type)
    .bind(is_active)
    .bind(id)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Replace items if provided
    if let Some(new_items) = items {
        sqlx::query("DELETE FROM checklist_template_items WHERE template_id = ?")
            .bind(id)
            .execute(db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        for (i, label) in new_items.iter().enumerate() {
            let item_id = format!("cti-{}", uuid::Uuid::new_v4());
            sqlx::query(
                "INSERT INTO checklist_template_items (id, template_id, label, sort_order) VALUES (?, ?, ?, ?)",
            )
            .bind(&item_id)
            .bind(id)
            .bind(label)
            .bind(i as i32)
            .execute(db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        }
    }

    get_template_by_id(db, id).await
}

pub async fn delete_template(db: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM checklist_templates WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Checklist template '{}' not found",
            id
        )));
    }
    Ok(())
}

pub async fn list_templates(db: &SqlitePool) -> AppResult<Vec<ChecklistTemplate>> {
    let rows = sqlx::query("SELECT * FROM checklist_templates ORDER BY name")
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut templates = Vec::new();
    for row in &rows {
        let id: String = row.get("id");
        let items = list_template_items(db, &id).await?;
        templates.push(ChecklistTemplate {
            id,
            name: row.get("name"),
            service_id: row.get("service_id"),
            incident_type: row.get("incident_type"),
            is_active: row.get::<bool, _>("is_active"),
            items,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        });
    }
    Ok(templates)
}

async fn get_template_by_id(db: &SqlitePool, id: &str) -> AppResult<ChecklistTemplate> {
    let row = sqlx::query("SELECT * FROM checklist_templates WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Checklist template '{}' not found", id)))?;

    let items = list_template_items(db, id).await?;

    Ok(ChecklistTemplate {
        id: row.get("id"),
        name: row.get("name"),
        service_id: row.get("service_id"),
        incident_type: row.get("incident_type"),
        is_active: row.get::<bool, _>("is_active"),
        items,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

async fn list_template_items(
    db: &SqlitePool,
    template_id: &str,
) -> AppResult<Vec<ChecklistTemplateItem>> {
    let rows = sqlx::query(
        "SELECT * FROM checklist_template_items WHERE template_id = ? ORDER BY sort_order",
    )
    .bind(template_id)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows
        .iter()
        .map(|row| ChecklistTemplateItem {
            id: row.get("id"),
            template_id: row.get("template_id"),
            label: row.get("label"),
            sort_order: row.get::<i32, _>("sort_order"),
        })
        .collect())
}

// ── Incident Checklist CRUD ───────────────────────────────────────

pub async fn create_incident_checklist(
    db: &SqlitePool,
    id: &str,
    incident_id: &str,
    template_id: Option<&str>,
    name: &str,
    items: &[String],
) -> AppResult<IncidentChecklist> {
    sqlx::query(
        "INSERT INTO incident_checklists (id, incident_id, template_id, name) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(incident_id)
    .bind(template_id)
    .bind(name)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    for (i, label) in items.iter().enumerate() {
        let item_id = format!("cli-{}", uuid::Uuid::new_v4());
        sqlx::query(
            "INSERT INTO checklist_items (id, checklist_id, label, sort_order) VALUES (?, ?, ?, ?)",
        )
        .bind(&item_id)
        .bind(id)
        .bind(label)
        .bind(i as i32)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }

    get_incident_checklist_by_id(db, id).await
}

pub async fn create_checklist_from_template(
    db: &SqlitePool,
    id: &str,
    incident_id: &str,
    template_id: &str,
) -> AppResult<IncidentChecklist> {
    let template = get_template_by_id(db, template_id).await?;

    sqlx::query(
        "INSERT INTO incident_checklists (id, incident_id, template_id, name) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(incident_id)
    .bind(template_id)
    .bind(&template.name)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    for item in &template.items {
        let item_id = format!("cli-{}", uuid::Uuid::new_v4());
        sqlx::query(
            "INSERT INTO checklist_items (id, checklist_id, template_item_id, label, sort_order) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&item_id)
        .bind(id)
        .bind(&item.id)
        .bind(&item.label)
        .bind(item.sort_order)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }

    get_incident_checklist_by_id(db, id).await
}

pub async fn list_incident_checklists(
    db: &SqlitePool,
    incident_id: &str,
) -> AppResult<Vec<IncidentChecklist>> {
    let rows = sqlx::query(
        "SELECT * FROM incident_checklists WHERE incident_id = ? ORDER BY created_at",
    )
    .bind(incident_id)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let mut checklists = Vec::new();
    for row in &rows {
        let cl_id: String = row.get("id");
        let items = list_checklist_items(db, &cl_id).await?;
        checklists.push(IncidentChecklist {
            id: cl_id,
            incident_id: row.get("incident_id"),
            template_id: row.get("template_id"),
            name: row.get("name"),
            items,
            created_at: row.get("created_at"),
        });
    }
    Ok(checklists)
}

pub async fn delete_incident_checklist(db: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM incident_checklists WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Incident checklist '{}' not found",
            id
        )));
    }
    Ok(())
}

pub async fn toggle_checklist_item(
    db: &SqlitePool,
    item_id: &str,
    checked_by: Option<&str>,
) -> AppResult<ChecklistItem> {
    let row = sqlx::query("SELECT * FROM checklist_items WHERE id = ?")
        .bind(item_id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Checklist item '{}' not found", item_id)))?;

    let is_checked: bool = row.get::<bool, _>("is_checked");
    let new_checked = !is_checked;

    if new_checked {
        sqlx::query(
            "UPDATE checklist_items SET is_checked=1, checked_at=strftime('%Y-%m-%dT%H:%M:%SZ','now'), checked_by=? WHERE id=?",
        )
        .bind(checked_by)
        .bind(item_id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    } else {
        sqlx::query(
            "UPDATE checklist_items SET is_checked=0, checked_at=NULL, checked_by=NULL WHERE id=?",
        )
        .bind(item_id)
        .execute(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }

    let updated_row = sqlx::query("SELECT * FROM checklist_items WHERE id = ?")
        .bind(item_id)
        .fetch_one(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(parse_checklist_item(&updated_row))
}

async fn get_incident_checklist_by_id(
    db: &SqlitePool,
    id: &str,
) -> AppResult<IncidentChecklist> {
    let row = sqlx::query("SELECT * FROM incident_checklists WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Incident checklist '{}' not found", id)))?;

    let items = list_checklist_items(db, id).await?;

    Ok(IncidentChecklist {
        id: row.get("id"),
        incident_id: row.get("incident_id"),
        template_id: row.get("template_id"),
        name: row.get("name"),
        items,
        created_at: row.get("created_at"),
    })
}

async fn list_checklist_items(
    db: &SqlitePool,
    checklist_id: &str,
) -> AppResult<Vec<ChecklistItem>> {
    let rows =
        sqlx::query("SELECT * FROM checklist_items WHERE checklist_id = ? ORDER BY sort_order")
            .bind(checklist_id)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(parse_checklist_item).collect())
}

fn parse_checklist_item(row: &sqlx::sqlite::SqliteRow) -> ChecklistItem {
    ChecklistItem {
        id: row.get("id"),
        checklist_id: row.get("checklist_id"),
        template_item_id: row.get("template_item_id"),
        label: row.get("label"),
        is_checked: row.get::<bool, _>("is_checked"),
        checked_at: row.get("checked_at"),
        checked_by: row.get("checked_by"),
        sort_order: row.get::<i32, _>("sort_order"),
    }
}
