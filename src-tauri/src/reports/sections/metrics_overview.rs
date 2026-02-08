use std::collections::HashMap;
use docx_rs::*;

use crate::models::metrics::{format_minutes, format_percentage, format_decimal};
use crate::reports::charts::add_chart_image;

use super::{heading1, body_text, header_cell, text_cell, spacer};

pub fn build(
    docx: Docx,
    mttr: f64,
    mtta: f64,
    total_incidents: i64,
    recurrence_rate: f64,
    avg_tickets: f64,
    prev_mttr: Option<f64>,
    prev_mtta: Option<f64>,
    prev_total: Option<i64>,
    prev_recurrence: Option<f64>,
    prev_tickets: Option<f64>,
    chart_images: &HashMap<String, Vec<u8>>,
) -> Docx {
    let mut docx = docx.add_paragraph(heading1("Metrics Overview"));

    // Build metrics table
    let header_row = TableRow::new(vec![
        header_cell("Metric"),
        header_cell("Current Quarter"),
        header_cell("Previous Quarter"),
        header_cell("Change %"),
    ]);

    let rows = vec![
        build_metric_row("MTTR", &format_minutes(mttr), prev_mttr.map(|v| format_minutes(v)), mttr, prev_mttr),
        build_metric_row("MTTA", &format_minutes(mtta), prev_mtta.map(|v| format_minutes(v)), mtta, prev_mtta),
        build_metric_row("Total Incidents", &total_incidents.to_string(), prev_total.map(|v| v.to_string()), total_incidents as f64, prev_total.map(|v| v as f64)),
        build_metric_row("Recurrence Rate", &format_percentage(recurrence_rate), prev_recurrence.map(|v| format_percentage(v)), recurrence_rate, prev_recurrence),
        build_metric_row("Avg Tickets", &format_decimal(avg_tickets), prev_tickets.map(|v| format_decimal(v)), avg_tickets, prev_tickets),
    ];

    let mut all_rows = vec![header_row];
    all_rows.extend(rows);

    let table = Table::new(all_rows);
    docx = docx.add_table(table);
    docx = docx.add_paragraph(spacer());

    // Add chart images if provided (validate PNG magic bytes first)
    const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    for key in &["severity_chart", "service_chart", "trend_chart"] {
        if let Some(bytes) = chart_images.get(*key) {
            if bytes.len() >= 8 && bytes[..8] == *PNG_MAGIC {
                docx = docx.add_paragraph(body_text(&format!("Chart: {}", key.replace('_', " "))));
                // 5486400 EMU ~= 6 inches wide, 3200400 ~= 3.5 inches tall
                docx = add_chart_image(docx, bytes, 5486400, 3200400);
                docx = docx.add_paragraph(spacer());
            }
        }
    }

    docx
}

fn build_metric_row(
    name: &str,
    current_formatted: &str,
    prev_formatted: Option<String>,
    current: f64,
    previous: Option<f64>,
) -> TableRow {
    let change = match previous {
        Some(prev) if prev != 0.0 => {
            let pct = ((current - prev) / prev) * 100.0;
            format!("{:+.1}%", pct)
        }
        Some(_) if current > 0.0 => "N/A (prev was 0)".to_string(),
        Some(_) => "0.0%".to_string(),
        None => "\u{2014}".to_string(),
    };

    TableRow::new(vec![
        text_cell(name),
        text_cell(current_formatted),
        text_cell(&prev_formatted.unwrap_or_else(|| "\u{2014}".to_string())),
        text_cell(&change),
    ])
}
