use docx_rs::*;

use crate::models::incident::Incident;
use crate::models::metrics::format_minutes;
use crate::db::queries::timeline_events::TimelineEvent;

use crate::reports::markdown;
use super::{heading1, heading2, body_text, label_value, header_cell, text_cell, spacer};

fn critical_incidents<'a>(incidents: &'a [Incident]) -> Vec<&'a Incident> {
    incidents
        .iter()
        .filter(|i| i.priority == "P0" || i.priority == "P1")
        .collect()
}

fn add_details_table(mut docx: Docx, incident: &Incident) -> Docx {
    let duration = incident
        .duration_minutes
        .map(|d| format_minutes(d as f64))
        .unwrap_or_else(|| "Ongoing".to_string());

    let details_table = Table::new(vec![
        TableRow::new(vec![header_cell("Field"), header_cell("Value")]),
        TableRow::new(vec![text_cell("Severity"), text_cell(&incident.severity)]),
        TableRow::new(vec![text_cell("Impact"), text_cell(&incident.impact)]),
        TableRow::new(vec![text_cell("Priority"), text_cell(&incident.priority)]),
        TableRow::new(vec![text_cell("Status"), text_cell(&incident.status)]),
        TableRow::new(vec![text_cell("Duration"), text_cell(&duration)]),
        TableRow::new(vec![
            text_cell("Tickets"),
            text_cell(&incident.tickets_submitted.to_string()),
        ]),
        TableRow::new(vec![
            text_cell("Affected Users"),
            text_cell(&incident.affected_users.to_string()),
        ]),
    ]);
    docx = docx.add_table(details_table);
    docx = docx.add_paragraph(spacer());
    docx
}

fn add_timestamps(mut docx: Docx, incident: &Incident) -> Docx {
    docx = docx.add_paragraph(label_value("Started: ", &incident.started_at));
    docx = docx.add_paragraph(label_value("Detected: ", &incident.detected_at));
    if let Some(ref responded) = incident.responded_at {
        docx = docx.add_paragraph(label_value("Responded: ", responded));
    }
    if let Some(ref resolved) = incident.resolved_at {
        docx = docx.add_paragraph(label_value("Resolved: ", resolved));
    }
    docx = docx.add_paragraph(spacer());
    docx
}

fn add_timeline_events(
    mut docx: Docx,
    incident_id: &str,
    timeline_events: &std::collections::HashMap<String, Vec<TimelineEvent>>,
) -> Docx {
    let Some(events) = timeline_events.get(incident_id) else {
        return docx;
    };
    if events.is_empty() {
        return docx;
    }
    docx = docx.add_paragraph(
        Paragraph::new()
            .add_run(Run::new().add_text("Timeline Events:").bold().size(11 * 2))
    );
    for ev in events.iter().take(8) {
        let when = ev.occurred_at.get(..16).unwrap_or(&ev.occurred_at);
        let who = if ev.actor.trim().is_empty() {
            "".to_string()
        } else {
            format!(" ({})", ev.actor)
        };
        docx = docx.add_paragraph(body_text(&format!("  \u{2022}  {} - {}{}", when, ev.message, who)));
    }
    docx = docx.add_paragraph(spacer());
    docx
}

fn add_markdown_section(mut docx: Docx, title: &str, content: &str) -> Docx {
    if content.is_empty() {
        return docx;
    }
    docx = docx.add_paragraph(
        Paragraph::new()
            .add_run(Run::new().add_text(title).bold().size(11 * 2))
    );
    docx = markdown::append_markdown(docx, content);
    docx = docx.add_paragraph(spacer());
    docx
}

fn add_incident_breakdown(
    mut docx: Docx,
    incident: &Incident,
    timeline_events: &std::collections::HashMap<String, Vec<TimelineEvent>>,
) -> Docx {
    docx = docx.add_paragraph(heading2(&format!(
        "[{}] {} - {}",
        incident.priority, incident.title, incident.service_name
    )));

    docx = add_details_table(docx, incident);
    docx = add_timestamps(docx, incident);
    docx = add_timeline_events(docx, &incident.id, timeline_events);
    docx = add_markdown_section(docx, "Root Cause:", &incident.root_cause);
    docx = add_markdown_section(docx, "Resolution:", &incident.resolution);
    docx = add_markdown_section(docx, "Lessons Learned:", &incident.lessons_learned);

    if incident.is_recurring {
        docx = docx.add_paragraph(body_text(
            "This is a recurring incident. Review prior remediation actions."
        ));
        docx = docx.add_paragraph(spacer());
    }

    docx
}

pub fn build(
    docx: Docx,
    incidents: &[Incident],
    timeline_events: &std::collections::HashMap<String, Vec<TimelineEvent>>,
) -> Docx {
    let mut docx = docx.add_paragraph(heading1("Critical Incident Breakdowns"));

    let critical_incidents = critical_incidents(incidents);

    if critical_incidents.is_empty() {
        docx = docx.add_paragraph(body_text("No P0 or P1 incidents this quarter."));
        docx = docx.add_paragraph(spacer());
        return docx;
    }

    for incident in &critical_incidents {
        docx = add_incident_breakdown(docx, incident, timeline_events);
    }

    docx
}
