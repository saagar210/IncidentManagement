use docx_rs::*;

use crate::models::metrics::{format_minutes, format_percentage, format_decimal, QuarterlyTrends};

use super::{heading1, body_text, header_cell, text_cell, spacer};

pub fn build(docx: Docx, trends: &QuarterlyTrends) -> Docx {
    let mut docx = docx.add_paragraph(heading1("Quarter-over-Quarter Comparison"));

    if trends.quarters.is_empty() {
        docx = docx.add_paragraph(body_text(
            "No historical quarter data available for comparison."
        ));
        docx = docx.add_paragraph(spacer());
        return docx;
    }

    // Build header row dynamically from quarters
    let mut header_cells = vec![header_cell("Metric")];
    for quarter in &trends.quarters {
        header_cells.push(header_cell(quarter));
    }
    let header_row = TableRow::new(header_cells);

    // MTTR row
    let mttr_row = build_row("MTTR", &trends.mttr, |v| format_minutes(v));
    let mtta_row = build_row("MTTA", &trends.mtta, |v| format_minutes(v));
    let count_row = build_row("Total Incidents", &trends.incident_count.iter().map(|v| *v as f64).collect::<Vec<_>>(), |v| format!("{}", v as i64));
    let recurrence_row = build_row("Recurrence Rate", &trends.recurrence_rate, |v| format_percentage(v));
    let tickets_row = build_row("Avg Tickets", &trends.avg_tickets, |v| format_decimal(v));

    let table = Table::new(vec![
        header_row,
        mttr_row,
        mtta_row,
        count_row,
        recurrence_row,
        tickets_row,
    ]);

    docx = docx.add_table(table);
    docx = docx.add_paragraph(spacer());

    docx
}

fn build_row(metric_name: &str, values: &[f64], formatter: impl Fn(f64) -> String) -> TableRow {
    let mut cells = vec![text_cell(metric_name)];
    for val in values {
        cells.push(text_cell(&formatter(*val)));
    }
    TableRow::new(cells)
}
