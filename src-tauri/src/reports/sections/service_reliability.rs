use std::collections::HashMap;
use docx_rs::*;

use crate::models::incident::Incident;
use crate::models::metrics::format_minutes;

use super::{heading1, body_text, header_cell, text_cell, spacer};

pub fn build(docx: Docx, incidents: &[Incident]) -> Docx {
    let mut docx = docx.add_paragraph(heading1("Service Reliability Summary"));

    if incidents.is_empty() {
        docx = docx.add_paragraph(body_text("No incidents recorded for this quarter."));
        docx = docx.add_paragraph(spacer());
        return docx;
    }

    // Aggregate by service
    let mut service_data: HashMap<String, ServiceStats> = HashMap::new();

    for incident in incidents {
        let entry = service_data
            .entry(incident.service_name.clone())
            .or_insert_with(|| ServiceStats {
                incident_count: 0,
                total_downtime_minutes: 0,
                total_mttr_minutes: 0.0,
                resolved_count: 0,
            });

        entry.incident_count += 1;
        if let Some(duration) = incident.duration_minutes {
            entry.total_downtime_minutes += duration;
            entry.total_mttr_minutes += duration as f64;
            entry.resolved_count += 1;
        }
    }

    let header_row = TableRow::new(vec![
        header_cell("Service"),
        header_cell("Incident Count"),
        header_cell("Total Downtime"),
        header_cell("Avg MTTR"),
    ]);

    let mut rows = vec![header_row];

    // Sort by incident count descending
    let mut service_list: Vec<_> = service_data.into_iter().collect();
    service_list.sort_by(|a, b| b.1.incident_count.cmp(&a.1.incident_count));

    for (name, stats) in &service_list {
        let avg_mttr = if stats.resolved_count > 0 {
            format_minutes(stats.total_mttr_minutes / stats.resolved_count as f64)
        } else {
            "\u{2014}".to_string()
        };

        rows.push(TableRow::new(vec![
            text_cell(name),
            text_cell(&stats.incident_count.to_string()),
            text_cell(&format_minutes(stats.total_downtime_minutes as f64)),
            text_cell(&avg_mttr),
        ]));
    }

    let table = Table::new(rows);
    docx = docx.add_table(table);
    docx = docx.add_paragraph(spacer());

    docx
}

struct ServiceStats {
    incident_count: i64,
    total_downtime_minutes: i64,
    total_mttr_minutes: f64,
    resolved_count: i64,
}
