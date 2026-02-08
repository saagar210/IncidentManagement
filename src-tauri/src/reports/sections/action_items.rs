use docx_rs::*;

use crate::models::incident::ActionItem;

use super::{heading1, body_text, header_cell, text_cell, spacer};

pub fn build(docx: Docx, action_items: &[ActionItem]) -> Docx {
    let mut docx = docx.add_paragraph(heading1("Action Items"));

    if action_items.is_empty() {
        docx = docx.add_paragraph(body_text("No action items recorded."));
        docx = docx.add_paragraph(spacer());
        return docx;
    }

    let header_row = TableRow::new(vec![
        header_cell("Title"),
        header_cell("Status"),
        header_cell("Owner"),
        header_cell("Due Date"),
        header_cell("Description"),
    ]);

    let mut rows = vec![header_row];

    for item in action_items {
        let due = item
            .due_date
            .as_deref()
            .unwrap_or("\u{2014}");

        let desc = if item.description.is_empty() {
            "\u{2014}"
        } else {
            &item.description
        };

        let owner = if item.owner.is_empty() {
            "Unassigned"
        } else {
            &item.owner
        };

        rows.push(TableRow::new(vec![
            text_cell(&item.title),
            text_cell(&item.status),
            text_cell(owner),
            text_cell(due),
            text_cell(desc),
        ]));
    }

    let table = Table::new(rows);
    docx = docx.add_table(table);
    docx = docx.add_paragraph(spacer());

    docx
}
