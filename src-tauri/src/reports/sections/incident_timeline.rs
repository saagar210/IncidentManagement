use docx_rs::*;

use crate::models::incident::Incident;
use crate::models::metrics::format_minutes;

use super::{heading1, header_cell, text_cell, spacer};

pub fn build(docx: Docx, incidents: &[Incident]) -> Docx {
    let mut docx = docx.add_paragraph(heading1("Incident Timeline"));

    if incidents.is_empty() {
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("No incidents recorded for this quarter.").size(11 * 2))
        );
        docx = docx.add_paragraph(spacer());
        return docx;
    }

    // Sort chronologically (incidents come pre-sorted but we ensure it)
    let mut sorted: Vec<&Incident> = incidents.iter().collect();
    sorted.sort_by(|a, b| a.started_at.cmp(&b.started_at));

    let header_row = TableRow::new(vec![
        header_cell("Date"),
        header_cell("Title"),
        header_cell("Service"),
        header_cell("Severity"),
        header_cell("Impact"),
        header_cell("Priority"),
        header_cell("Duration"),
        header_cell("Status"),
    ]);

    let mut rows = vec![header_row];

    for incident in &sorted {
        let date = incident.started_at.get(..10).unwrap_or(&incident.started_at);
        let duration = incident
            .duration_minutes
            .map(|d| format_minutes(d as f64))
            .unwrap_or_else(|| "Ongoing".to_string());

        rows.push(TableRow::new(vec![
            text_cell(date),
            text_cell(&incident.title),
            text_cell(&incident.service_name),
            text_cell(&incident.severity),
            text_cell(&incident.impact),
            text_cell(&incident.priority),
            text_cell(&duration),
            text_cell(&incident.status),
        ]));
    }

    let table = Table::new(rows);
    docx = docx.add_table(table);
    docx = docx.add_paragraph(spacer());

    docx
}
