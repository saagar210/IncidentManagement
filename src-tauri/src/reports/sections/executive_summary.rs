use docx_rs::*;

use crate::models::incident::Incident;
use crate::models::metrics::{format_minutes, format_percentage};

use super::{heading1, body_text, bullet_item, spacer};

pub fn build(
    docx: Docx,
    incidents: &[Incident],
    mttr: f64,
    mtta: f64,
    recurrence_rate: f64,
    total_incidents: i64,
    introduction: &str,
) -> Docx {
    let mut docx = docx.add_paragraph(heading1("Executive Summary"));

    // Custom introduction
    if !introduction.is_empty() {
        docx = docx.add_paragraph(body_text(introduction));
        docx = docx.add_paragraph(spacer());
    }

    // Summary paragraph
    let summary = format!(
        "This quarter saw {} total incident(s) with a Mean Time to Resolve (MTTR) of {} and a Mean Time to Acknowledge (MTTA) of {}. The recurrence rate was {}.",
        total_incidents,
        if total_incidents == 0 { "N/A".to_string() } else { format_minutes(mttr) },
        if total_incidents == 0 { "N/A".to_string() } else { format_minutes(mtta) },
        if total_incidents == 0 { "N/A".to_string() } else { format_percentage(recurrence_rate) },
    );
    docx = docx.add_paragraph(body_text(&summary));
    docx = docx.add_paragraph(spacer());

    // Key highlights
    docx = docx.add_paragraph(
        Paragraph::new()
            .add_run(Run::new().add_text("Key Highlights:").bold().size(11 * 2))
    );

    // Count by severity
    let critical_count = incidents.iter().filter(|i| i.severity == "Critical").count();
    let high_count = incidents.iter().filter(|i| i.severity == "High").count();
    let p0_count = incidents.iter().filter(|i| i.priority == "P0").count();
    let p1_count = incidents.iter().filter(|i| i.priority == "P1").count();
    let resolved_count = incidents.iter().filter(|i| i.status == "Resolved" || i.status == "Post-Mortem").count();

    docx = docx.add_paragraph(bullet_item(
        &format!("{} Critical and {} High severity incidents", critical_count, high_count)
    ));
    docx = docx.add_paragraph(bullet_item(
        &format!("{} P0 and {} P1 priority incidents", p0_count, p1_count)
    ));
    docx = docx.add_paragraph(bullet_item(
        &format!("{} of {} incidents resolved", resolved_count, total_incidents)
    ));

    let recurring_count = incidents.iter().filter(|i| i.is_recurring).count();
    if recurring_count > 0 {
        docx = docx.add_paragraph(bullet_item(
            &format!("{} recurring incident(s) detected", recurring_count)
        ));
    }

    // Unique services affected
    let mut services: Vec<&str> = incidents.iter().map(|i| i.service_name.as_str()).collect();
    services.sort();
    services.dedup();
    docx = docx.add_paragraph(bullet_item(
        &format!("{} service(s) affected", services.len())
    ));

    docx = docx.add_paragraph(spacer());

    docx
}
