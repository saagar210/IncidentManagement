pub mod executive_summary;
pub mod metrics_overview;
pub mod incident_timeline;
pub mod incident_breakdowns;
pub mod service_reliability;
pub mod qoq_comparison;
pub mod discussion_points;
pub mod action_items;

use docx_rs::*;

/// Helper: create a Heading 1 paragraph.
pub fn heading1(text: &str) -> Paragraph {
    Paragraph::new()
        .add_run(Run::new().add_text(text).bold().size(28 * 2)) // size is in half-points
        .style("Heading1")
}

/// Helper: create a Heading 2 paragraph.
pub fn heading2(text: &str) -> Paragraph {
    Paragraph::new()
        .add_run(Run::new().add_text(text).bold().size(24 * 2))
        .style("Heading2")
}

/// Helper: create a body paragraph.
pub fn body_text(text: &str) -> Paragraph {
    Paragraph::new()
        .add_run(Run::new().add_text(text).size(11 * 2))
}

/// Helper: create a bold label + value on one line.
pub fn label_value(label: &str, value: &str) -> Paragraph {
    Paragraph::new()
        .add_run(Run::new().add_text(label).bold().size(11 * 2))
        .add_run(Run::new().add_text(value).size(11 * 2))
}

/// Helper: create a bullet list item paragraph.
pub fn bullet_item(text: &str) -> Paragraph {
    Paragraph::new()
        .add_run(Run::new().add_text(format!("  \u{2022}  {}", text)).size(11 * 2))
}

/// Helper: create a table header cell (bold text, shaded background).
pub fn header_cell(text: &str) -> TableCell {
    TableCell::new()
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(text).bold().size(10 * 2))
        )
        .shading(Shading::new().fill("E0E0E0"))
}

/// Helper: create a regular table cell.
pub fn text_cell(text: &str) -> TableCell {
    TableCell::new()
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(text).size(10 * 2))
        )
}

/// Helper: add a blank spacer paragraph.
pub fn spacer() -> Paragraph {
    Paragraph::new()
        .add_run(Run::new().add_text(""))
}
