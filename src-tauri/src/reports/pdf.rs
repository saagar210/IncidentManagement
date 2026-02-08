//! PDF report generation using genpdf.
//!
//! Parallel PDF builder that mirrors the DOCX report structure.

use genpdf::elements::{Break, Paragraph};
use genpdf::fonts;
use genpdf::style::Style;
use genpdf::{Document, Element, SimplePageDecorator};

use crate::error::{AppError, AppResult};
use crate::models::incident::{ActionItem, Incident};
use crate::models::metrics::{format_minutes, format_percentage, QuarterlyTrends};
use crate::models::quarter::QuarterConfig;
use crate::reports::ReportConfig;

/// Build a PDF document and return the bytes.
pub fn build_pdf(
    config: &ReportConfig,
    incidents: &[Incident],
    _prev_incidents: &[Incident],
    quarter: Option<&QuarterConfig>,
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
    action_items: &[ActionItem],
    _trends: &QuarterlyTrends,
) -> AppResult<Vec<u8>> {
    // Use built-in Liberation Sans font family (bundled with genpdf)
    let font_family = fonts::from_files("", "LiberationSans", None)
        .or_else(|_| {
            // Fallback: try system fonts on macOS
            fonts::from_files("/Library/Fonts", "Arial", None)
        })
        .or_else(|_| {
            fonts::from_files("/System/Library/Fonts/Supplemental", "Arial", None)
        })
        .map_err(|e| AppError::Report(format!("Failed to load PDF fonts: {}. Install Liberation Sans or Arial.", e)))?;

    let mut doc = Document::new(font_family);
    doc.set_title(&config.title);

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(20);
    doc.set_page_decorator(decorator);

    // Title
    doc.push(
        Paragraph::new(&config.title)
            .styled(Style::new().bold().with_font_size(24)),
    );

    if let Some(q) = quarter {
        doc.push(
            Paragraph::new(&q.label)
                .styled(Style::new().with_font_size(14)),
        );
        doc.push(
            Paragraph::new(format!("Period: {} to {}", q.start_date, q.end_date))
                .styled(Style::new().with_font_size(10)),
        );
    }

    doc.push(Break::new(1));

    // Executive Summary
    if config.sections.executive_summary {
        push_heading(&mut doc, "Executive Summary");

        if !config.introduction.is_empty() {
            doc.push(Paragraph::new(&config.introduction));
            doc.push(Break::new(0.5));
        }

        let summary = format!(
            "This quarter saw {} total incident(s) with a Mean Time to Resolve (MTTR) of {} and a Mean Time to Acknowledge (MTTA) of {}. The recurrence rate was {}.",
            total_incidents,
            if total_incidents == 0 { "N/A".into() } else { format_minutes(mttr) },
            if total_incidents == 0 { "N/A".into() } else { format_minutes(mtta) },
            if total_incidents == 0 { "N/A".into() } else { format_percentage(recurrence_rate) },
        );
        doc.push(Paragraph::new(summary));
        doc.push(Break::new(0.5));

        let critical_count = incidents.iter().filter(|i| i.severity == "Critical").count();
        let high_count = incidents.iter().filter(|i| i.severity == "High").count();
        let p0_count = incidents.iter().filter(|i| i.priority == "P0").count();
        let p1_count = incidents.iter().filter(|i| i.priority == "P1").count();
        let resolved_count = incidents.iter().filter(|i| i.status == "Resolved" || i.status == "Post-Mortem").count();

        doc.push(bullet(&format!("{} Critical and {} High severity incidents", critical_count, high_count)));
        doc.push(bullet(&format!("{} P0 and {} P1 priority incidents", p0_count, p1_count)));
        doc.push(bullet(&format!("{} of {} incidents resolved", resolved_count, total_incidents)));

        let recurring_count = incidents.iter().filter(|i| i.is_recurring).count();
        if recurring_count > 0 {
            doc.push(bullet(&format!("{} recurring incident(s) detected", recurring_count)));
        }

        doc.push(Break::new(1));
    }

    // Metrics Overview
    if config.sections.metrics_overview {
        push_heading(&mut doc, "Metrics Overview");

        push_metric_row(&mut doc, "Total Incidents", &total_incidents.to_string(), prev_total.map(|v| v.to_string()).as_deref());
        push_metric_row(&mut doc, "MTTR", &format_minutes(mttr), prev_mttr.map(|v| format_minutes(v)).as_deref());
        push_metric_row(&mut doc, "MTTA", &format_minutes(mtta), prev_mtta.map(|v| format_minutes(v)).as_deref());
        push_metric_row(&mut doc, "Recurrence Rate", &format_percentage(recurrence_rate), prev_recurrence.map(|v| format_percentage(v)).as_deref());
        push_metric_row(&mut doc, "Avg Tickets/Incident", &format!("{:.1}", avg_tickets), prev_tickets.map(|v| format!("{:.1}", v)).as_deref());

        doc.push(Break::new(1));
    }

    // Incident Timeline
    if config.sections.incident_timeline {
        push_heading(&mut doc, "Incident Timeline");

        if incidents.is_empty() {
            doc.push(Paragraph::new("No incidents recorded this quarter."));
        } else {
            for incident in incidents {
                let duration = incident.duration_minutes
                    .map(|d| format_minutes(d as f64))
                    .unwrap_or_else(|| "Ongoing".to_string());

                doc.push(
                    Paragraph::new(format!(
                        "[{}] {} - {} ({})",
                        incident.priority, incident.title, incident.service_name, duration
                    )).styled(Style::new().bold().with_font_size(10)),
                );
            }
        }

        doc.push(Break::new(1));
    }

    // Critical Incident Breakdowns
    if config.sections.incident_breakdowns {
        push_heading(&mut doc, "Critical Incident Breakdowns");

        let critical: Vec<&Incident> = incidents.iter()
            .filter(|i| i.priority == "P0" || i.priority == "P1")
            .collect();

        if critical.is_empty() {
            doc.push(Paragraph::new("No P0 or P1 incidents this quarter."));
        } else {
            for incident in &critical {
                doc.push(
                    Paragraph::new(format!(
                        "[{}] {} - {}",
                        incident.priority, incident.title, incident.service_name
                    )).styled(Style::new().bold().with_font_size(12)),
                );

                let duration = incident.duration_minutes
                    .map(|d| format_minutes(d as f64))
                    .unwrap_or_else(|| "Ongoing".to_string());

                doc.push(Paragraph::new(format!("Severity: {} | Impact: {} | Duration: {}", incident.severity, incident.impact, duration)));

                if !incident.root_cause.is_empty() {
                    doc.push(Paragraph::new("Root Cause:").styled(Style::new().bold()));
                    push_markdown_text(&mut doc, &incident.root_cause);
                }
                if !incident.resolution.is_empty() {
                    doc.push(Paragraph::new("Resolution:").styled(Style::new().bold()));
                    push_markdown_text(&mut doc, &incident.resolution);
                }
                if !incident.lessons_learned.is_empty() {
                    doc.push(Paragraph::new("Lessons Learned:").styled(Style::new().bold()));
                    push_markdown_text(&mut doc, &incident.lessons_learned);
                }

                doc.push(Break::new(0.5));
            }
        }

        doc.push(Break::new(1));
    }

    // Service Reliability
    if config.sections.service_reliability {
        push_heading(&mut doc, "Service Reliability");

        let mut service_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for incident in incidents {
            *service_counts.entry(&incident.service_name).or_default() += 1;
        }

        let mut sorted: Vec<_> = service_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        for (service, count) in &sorted {
            doc.push(Paragraph::new(format!("{}: {} incident(s)", service, count)));
        }

        doc.push(Break::new(1));
    }

    // Action Items
    if config.sections.action_items {
        push_heading(&mut doc, "Action Items");

        if action_items.is_empty() {
            doc.push(Paragraph::new("No action items."));
        } else {
            for item in action_items {
                let status_icon = if item.status == "done" { "[x]" } else { "[ ]" };
                doc.push(Paragraph::new(format!(
                    "{} {} (Owner: {}, Due: {})",
                    status_icon,
                    item.title,
                    if item.owner.is_empty() { "Unassigned" } else { &item.owner },
                    item.due_date.as_deref().unwrap_or("N/A"),
                )));
            }
        }

        doc.push(Break::new(1));
    }

    // Render to bytes
    let mut buf: Vec<u8> = Vec::new();
    doc.render(&mut buf)
        .map_err(|e| AppError::Report(format!("Failed to render PDF: {}", e)))?;

    Ok(buf)
}

fn push_heading(doc: &mut Document, text: &str) {
    doc.push(
        Paragraph::new(text)
            .styled(Style::new().bold().with_font_size(16)),
    );
    doc.push(Break::new(0.3));
}

fn bullet(text: &str) -> Paragraph {
    Paragraph::new(format!("\u{2022}  {}", text))
}

fn push_metric_row(doc: &mut Document, label: &str, current: &str, previous: Option<&str>) {
    let line = if let Some(prev) = previous {
        format!("{}: {} (prev: {})", label, current, prev)
    } else {
        format!("{}: {}", label, current)
    };
    doc.push(Paragraph::new(line));
}

/// Simple markdown text renderer for PDF — strips markdown syntax and renders as plain text.
/// Full markdown rendering in PDF is limited by genpdf capabilities.
fn push_markdown_text(doc: &mut Document, md: &str) {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    let parser = Parser::new_ext(md.trim(), Options::empty());
    let mut text_buf = String::new();
    for event in parser {
        match event {
            Event::Text(t) => text_buf.push_str(&t),
            Event::Code(c) => {
                text_buf.push('`');
                text_buf.push_str(&c);
                text_buf.push('`');
            }
            Event::SoftBreak | Event::HardBreak => text_buf.push(' '),
            Event::Start(Tag::Item) => {
                if !text_buf.trim().is_empty() {
                    doc.push(Paragraph::new(text_buf.trim()));
                    text_buf.clear();
                }
                text_buf.push_str("\u{2022}  ");
            }
            Event::End(TagEnd::Paragraph) | Event::End(TagEnd::Item) => {
                if !text_buf.trim().is_empty() {
                    doc.push(Paragraph::new(text_buf.trim()));
                }
                text_buf.clear();
            }
            _ => {}
        }
    }
    if !text_buf.trim().is_empty() {
        doc.push(Paragraph::new(text_buf.trim()));
    }
}
