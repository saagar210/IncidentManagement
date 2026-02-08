use docx_rs::*;

use crate::models::incident::Incident;
use crate::models::metrics::format_minutes;

use super::{heading1, heading2, body_text, label_value, header_cell, text_cell, spacer};

pub fn build(docx: Docx, incidents: &[Incident]) -> Docx {
    let mut docx = docx.add_paragraph(heading1("Critical Incident Breakdowns"));

    // Filter to P0 and P1 incidents
    let critical_incidents: Vec<&Incident> = incidents
        .iter()
        .filter(|i| i.priority == "P0" || i.priority == "P1")
        .collect();

    if critical_incidents.is_empty() {
        docx = docx.add_paragraph(body_text("No P0 or P1 incidents this quarter."));
        docx = docx.add_paragraph(spacer());
        return docx;
    }

    for incident in &critical_incidents {
        // Incident heading
        docx = docx.add_paragraph(heading2(&format!(
            "[{}] {} - {}",
            incident.priority, incident.title, incident.service_name
        )));

        // Details table
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
            TableRow::new(vec![text_cell("Tickets"), text_cell(&incident.tickets_submitted.to_string())]),
            TableRow::new(vec![text_cell("Affected Users"), text_cell(&incident.affected_users.to_string())]),
        ]);
        docx = docx.add_table(details_table);
        docx = docx.add_paragraph(spacer());

        // Timeline
        docx = docx.add_paragraph(label_value("Started: ", &incident.started_at));
        docx = docx.add_paragraph(label_value("Detected: ", &incident.detected_at));
        if let Some(ref responded) = incident.responded_at {
            docx = docx.add_paragraph(label_value("Responded: ", responded));
        }
        if let Some(ref resolved) = incident.resolved_at {
            docx = docx.add_paragraph(label_value("Resolved: ", resolved));
        }
        docx = docx.add_paragraph(spacer());

        // Root cause
        if !incident.root_cause.is_empty() {
            docx = docx.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text("Root Cause:").bold().size(11 * 2))
            );
            docx = docx.add_paragraph(body_text(&incident.root_cause));
            docx = docx.add_paragraph(spacer());
        }

        // Resolution
        if !incident.resolution.is_empty() {
            docx = docx.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text("Resolution:").bold().size(11 * 2))
            );
            docx = docx.add_paragraph(body_text(&incident.resolution));
            docx = docx.add_paragraph(spacer());
        }

        // Lessons learned
        if !incident.lessons_learned.is_empty() {
            docx = docx.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text("Lessons Learned:").bold().size(11 * 2))
            );
            docx = docx.add_paragraph(body_text(&incident.lessons_learned));
            docx = docx.add_paragraph(spacer());
        }

        // Recurring flag
        if incident.is_recurring {
            docx = docx.add_paragraph(body_text(
                "This is a recurring incident. Review prior remediation actions."
            ));
            docx = docx.add_paragraph(spacer());
        }
    }

    docx
}
