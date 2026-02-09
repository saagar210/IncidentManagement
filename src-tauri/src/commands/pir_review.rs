use sqlx::{Row, SqlitePool};
use std::io::{Cursor, Write};
use tauri::State;

use crate::db::queries::{incidents, postmortems, tags};
use crate::error::AppError;
use crate::models::incident::{ActionItem, Incident};
use crate::models::postmortem::{ContributingFactor, Postmortem};

#[derive(Debug, Clone, serde::Serialize)]
pub struct PirBrief {
    pub markdown: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PirInsightCount {
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PirReviewInsights {
    pub top_factor_categories: Vec<PirInsightCount>,
    pub top_factor_descriptions: Vec<PirInsightCount>,
    pub external_root_no_action_items_justified: i64,
}

fn extract_markdown(content: &str) -> String {
    if content.trim().is_empty() || content.trim() == "{}" {
        return String::new();
    }
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(md) = v.get("markdown").and_then(|m| m.as_str()) {
            return md.to_string();
        }
    }
    content.to_string()
}

fn md_escape_inline(text: &str) -> String {
    text.replace('\n', " ").trim().to_string()
}

fn append_summary_section(out: &mut String, inc: &Incident, pm: Option<&Postmortem>) {
    out.push_str("## Summary\n\n");

    let summary_md = pm
        .map(|p| extract_markdown(&p.content))
        .unwrap_or_default();
    if !summary_md.trim().is_empty() {
        out.push_str(summary_md.trim());
        out.push_str("\n\n");
        return;
    }

    let fallback = [
        (!inc.root_cause.trim().is_empty()).then_some(("Root Cause", inc.root_cause.as_str())),
        (!inc.resolution.trim().is_empty()).then_some(("Resolution", inc.resolution.as_str())),
        (!inc.notes.trim().is_empty()).then_some(("Notes", inc.notes.as_str())),
    ]
    .into_iter()
    .flatten()
    .map(|(h, v)| format!("**{}:** {}", h, md_escape_inline(v)))
    .collect::<Vec<_>>()
    .join("\n\n");

    if fallback.is_empty() {
        out.push_str("_No summary content recorded._\n\n");
    } else {
        out.push_str(&fallback);
        out.push_str("\n\n");
    }
}

fn append_impact_section(out: &mut String, inc: &Incident) {
    out.push_str("## Impact\n\n");
    out.push_str(&format!(
        "- Service: {}\n- Severity: {}\n- Impact: {}\n- Priority: {}\n- Status: {}\n",
        md_escape_inline(&inc.service_name),
        md_escape_inline(&inc.severity),
        md_escape_inline(&inc.impact),
        md_escape_inline(&inc.priority),
        md_escape_inline(&inc.status)
    ));
    out.push_str(&format!(
        "- Tickets submitted: {}\n- Affected users: {}\n\n",
        inc.tickets_submitted, inc.affected_users
    ));
}

fn append_timeline_section(out: &mut String, inc: &Incident) {
    out.push_str("## Timeline\n\n");
    out.push_str(&format!("- Started: {}\n", inc.started_at));
    out.push_str(&format!("- Detected: {}\n", inc.detected_at));
    if let Some(v) = inc.acknowledged_at.as_deref() {
        out.push_str(&format!("- Acknowledged: {}\n", v));
    }
    if let Some(v) = inc.first_response_at.as_deref() {
        out.push_str(&format!("- First response: {}\n", v));
    }
    if let Some(v) = inc.mitigation_started_at.as_deref() {
        out.push_str(&format!("- Mitigation started: {}\n", v));
    }
    if let Some(v) = inc.responded_at.as_deref() {
        out.push_str(&format!("- Responded: {}\n", v));
    }
    if let Some(v) = inc.resolved_at.as_deref() {
        out.push_str(&format!("- Resolved: {}\n", v));
    }
    out.push_str("\n");
}

fn append_contributing_factors_section(
    out: &mut String,
    factors: &[ContributingFactor],
) {
    out.push_str("## Contributing Factors\n\n");
    if factors.is_empty() {
        out.push_str("_No contributing factors recorded._\n\n");
        return;
    }

    for cf in factors {
        let root = if cf.is_root { " (root)" } else { "" };
        out.push_str(&format!(
            "- **{}**{}: {}\n",
            md_escape_inline(&cf.category),
            root,
            md_escape_inline(&cf.description)
        ));
    }
    out.push_str("\n");
}

fn append_action_items_section(
    out: &mut String,
    action_items: &[ActionItem],
    pm: Option<&Postmortem>,
) {
    out.push_str("## Action Items\n\n");
    if !action_items.is_empty() {
        for ai in action_items {
            let due = ai.due_date.as_deref().unwrap_or("N/A");
            let owner = if ai.owner.trim().is_empty() { "Unassigned" } else { ai.owner.as_str() };
            let completed = ai.completed_at.as_deref().unwrap_or("");
            let validated = if ai.validated_at.is_some() { "Validated" } else { "" };
            out.push_str(&format!(
                "- **{}** [{}] (Owner: {}, Due: {})\n",
                md_escape_inline(&ai.title),
                md_escape_inline(&ai.status),
                md_escape_inline(owner),
                md_escape_inline(due)
            ));
            if !completed.is_empty() {
                out.push_str(&format!("  - Completed: {}\n", completed));
            }
            if !validated.is_empty() {
                out.push_str("  - Validated: yes\n");
            }
            if !ai.outcome_notes.trim().is_empty() {
                out.push_str(&format!("  - Outcome: {}\n", md_escape_inline(&ai.outcome_notes)));
            }
        }
        out.push_str("\n");
        return;
    }

    if let Some(pm) = pm {
        if pm.no_action_items_justified {
            out.push_str("No action items were justified for this incident.\n\n");
            if !pm.no_action_items_justification.trim().is_empty() {
                out.push_str(&format!(
                    "**Justification:** {}\n\n",
                    md_escape_inline(&pm.no_action_items_justification)
                ));
            }
        } else {
            out.push_str("_No action items recorded._\n\n");
        }
    } else {
        out.push_str("_No action items recorded._\n\n");
    }
}

fn append_lessons_section(out: &mut String, inc: &Incident) {
    out.push_str("## Lessons Learned\n\n");
    if inc.lessons_learned.trim().is_empty() {
        out.push_str("_No lessons learned recorded._\n\n");
    } else {
        out.push_str(inc.lessons_learned.trim());
        out.push_str("\n\n");
    }
}

fn append_references_section(
    out: &mut String,
    inc: &Incident,
    tags: &[String],
) {
    out.push_str("## References\n\n");
    if !inc.external_ref.trim().is_empty() {
        out.push_str(&format!("- External ref: {}\n", md_escape_inline(&inc.external_ref)));
    }
    if !tags.is_empty() {
        out.push_str(&format!("- Tags: {}\n", tags.join(", ")));
    }
    out.push_str(&format!("- Incident ID: {}\n", inc.id));
}

#[tauri::command]
pub async fn generate_pir_brief_markdown(
    db: State<'_, SqlitePool>,
    incident_id: String,
) -> Result<PirBrief, AppError> {
    let inc = incidents::get_incident_by_id(&*db, &incident_id).await?;
    let pm = postmortems::get_postmortem_by_incident(&*db, &incident_id).await?;
    let factors = postmortems::list_contributing_factors(&*db, &incident_id).await?;
    let action_items = incidents::list_action_items(&*db, Some(&incident_id)).await?;
    let tag_list = tags::get_incident_tags(&*db, &incident_id).await?;

    let mut out = String::new();
    out.push_str(&format!("# PIR Brief: {}\n\n", md_escape_inline(&inc.title)));

    append_summary_section(&mut out, &inc, pm.as_ref());
    append_impact_section(&mut out, &inc);
    append_timeline_section(&mut out, &inc);
    append_contributing_factors_section(&mut out, &factors);
    append_action_items_section(&mut out, &action_items, pm.as_ref());
    append_lessons_section(&mut out, &inc);
    append_references_section(&mut out, &inc, &tag_list);

    Ok(PirBrief { markdown: out })
}

#[tauri::command]
pub async fn generate_pir_brief_file(
    db: State<'_, SqlitePool>,
    incident_id: String,
    format: String, // "docx" or "pdf"
) -> Result<String, AppError> {
    let brief = generate_pir_brief_markdown(db, incident_id).await?;
    let md = brief.markdown;

    let file_ext = if format.to_lowercase() == "pdf" { "pdf" } else { "docx" };
    let suffix = format!(".{}", file_ext);
    let mut tmp = tempfile::Builder::new()
        .prefix("pir_brief_")
        .suffix(&suffix)
        .tempfile()
        .map_err(|e| AppError::Report(format!("Failed to create temp file: {}", e)))?;

    let bytes = if file_ext == "pdf" {
        build_pdf_from_markdown(&md)?
    } else {
        build_docx_from_markdown(&md)?
    };

    tmp.write_all(&bytes)
        .map_err(|e| AppError::Report(format!("Failed to write temp file: {}", e)))?;

    let (_file, path) = tmp
        .keep()
        .map_err(|e| AppError::Report(format!("Failed to persist temp file: {}", e)))?;
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Report("Invalid temp path encoding".into()))
}

fn build_docx_from_markdown(md: &str) -> Result<Vec<u8>, AppError> {
    use docx_rs::Docx;

    let docx = crate::reports::markdown::append_markdown(Docx::new(), md);
    let mut buf: Vec<u8> = Vec::new();
    let cursor = Cursor::new(&mut buf);
    docx.build()
        .pack(cursor)
        .map_err(|e| AppError::Report(format!("Failed to build DOCX: {}", e)))?;
    Ok(buf)
}

fn load_pdf_font_family() -> Result<genpdf::fonts::FontFamily<genpdf::fonts::FontData>, AppError> {
    use genpdf::fonts;
    fonts::from_files("", "LiberationSans", None)
        .or_else(|_| fonts::from_files("/Library/Fonts", "Arial", None))
        .or_else(|_| fonts::from_files("/System/Library/Fonts/Supplemental", "Arial", None))
        .map_err(|e| {
            AppError::Report(format!(
                "Failed to load PDF fonts: {}. Install Liberation Sans or Arial.",
                e
            ))
        })
}

fn markdown_to_paragraphs(md: &str) -> Vec<String> {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    let parser = Parser::new_ext(md.trim(), Options::empty());
    let mut out: Vec<String> = Vec::new();
    let mut text_buf = String::new();

    let flush = |buf: &mut String, out: &mut Vec<String>| {
        if !buf.trim().is_empty() {
            out.push(buf.trim().to_string());
        }
        buf.clear();
    };

    for event in parser {
        match event {
            Event::Start(Tag::Heading { .. }) => flush(&mut text_buf, &mut out),
            Event::Text(t) => text_buf.push_str(&t),
            Event::Code(c) => {
                text_buf.push('`');
                text_buf.push_str(&c);
                text_buf.push('`');
            }
            Event::SoftBreak | Event::HardBreak => text_buf.push(' '),
            Event::Start(Tag::Item) => {
                flush(&mut text_buf, &mut out);
                text_buf.push_str("\u{2022}  ");
            }
            Event::End(TagEnd::Paragraph) | Event::End(TagEnd::Item) | Event::End(TagEnd::Heading(_)) => {
                flush(&mut text_buf, &mut out);
            }
            _ => {}
        }
    }
    flush(&mut text_buf, &mut out);
    out
}

fn build_pdf_from_markdown(md: &str) -> Result<Vec<u8>, AppError> {
    use genpdf::elements::{Break, Paragraph};
    use genpdf::{Document, SimplePageDecorator};
    let font_family = load_pdf_font_family()?;

    let mut doc = Document::new(font_family);
    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(20);
    doc.set_page_decorator(decorator);

    // Basic markdown -> plain-ish paragraphs (enough for sharing).
    for p in markdown_to_paragraphs(md) {
        doc.push(Paragraph::new(p));
        doc.push(Break::new(0.2));
    }

    let mut buf: Vec<u8> = Vec::new();
    doc.render(&mut buf)
        .map_err(|e| AppError::Report(format!("Failed to render PDF: {}", e)))?;
    Ok(buf)
}

#[tauri::command]
pub async fn get_pir_review_insights(
    db: State<'_, SqlitePool>,
) -> Result<PirReviewInsights, AppError> {
    let top_factor_categories = sqlx::query(
        "SELECT category, COUNT(*) as c FROM contributing_factors GROUP BY category ORDER BY c DESC LIMIT 5",
    )
    .fetch_all(&*db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .into_iter()
    .map(|r| PirInsightCount {
        label: r.get::<String, _>("category"),
        count: r.get::<i64, _>("c"),
    })
    .collect();

    let top_factor_descriptions = sqlx::query(
        "SELECT description, COUNT(*) as c \
         FROM contributing_factors \
         WHERE TRIM(description) != '' \
         GROUP BY description \
         ORDER BY c DESC \
         LIMIT 5",
    )
    .fetch_all(&*db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .into_iter()
    .map(|r| PirInsightCount {
        label: r.get::<String, _>("description"),
        count: r.get::<i64, _>("c"),
    })
    .collect();

    let external_root_no_action_items_justified: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT i.id) \
         FROM incidents i \
         JOIN contributing_factors cf ON cf.incident_id = i.id \
         JOIN postmortems pm ON pm.incident_id = i.id \
         WHERE cf.category = 'External' AND cf.is_root = 1 AND pm.no_action_items_justified = 1 AND i.deleted_at IS NULL",
    )
    .fetch_one(&*db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(PirReviewInsights {
        top_factor_categories,
        top_factor_descriptions,
        external_root_no_action_items_justified,
    })
}
