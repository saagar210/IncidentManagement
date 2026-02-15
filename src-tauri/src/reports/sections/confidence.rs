use docx_rs::*;

use crate::commands::quarter_review::QuarterReadinessReport;
use crate::db::queries::quarter_finalization::{QuarterFinalization, QuarterOverride};

use super::{heading1, heading2, body_text, bullet_item, header_cell, text_cell, spacer};

pub struct ConfidenceSectionInput<'a> {
    pub readiness: &'a QuarterReadinessReport,
    pub overrides: &'a [QuarterOverride],
    pub finalization: Option<&'a QuarterFinalization>,
    pub facts_changed_since_finalization: bool,
    pub inputs_hash: &'a str,
}

pub fn build(docx: Docx, input: ConfidenceSectionInput<'_>) -> Docx {
    let mut docx = docx.add_paragraph(heading1("Confidence and Readiness"));

    docx = docx.add_paragraph(body_text(&format!(
        "Readiness: {} ready, {} needs attention, {} total.",
        input.readiness.ready_incidents,
        input.readiness.needs_attention_incidents,
        input.readiness.total_incidents
    )));

    if let Some(fin) = input.finalization {
        docx = docx.add_paragraph(body_text(&format!(
            "Finalized: {} (by {}). Inputs hash: {}",
            fin.finalized_at, fin.finalized_by, input.inputs_hash
        )));
        if input.facts_changed_since_finalization {
            docx = docx.add_paragraph(body_text(
                "Warning: facts changed since finalization. Metrics may differ from the frozen snapshot.",
            ));
        } else {
            docx = docx.add_paragraph(body_text(
                "Snapshot is consistent with current facts (inputs hash matches).",
            ));
        }
    } else {
        docx = docx.add_paragraph(body_text(&format!(
            "Not finalized. Current inputs hash: {}",
            input.inputs_hash
        )));
    }

    docx = docx.add_paragraph(spacer());

    if input.readiness.findings.is_empty() {
        docx = docx.add_paragraph(body_text("No readiness findings for this quarter."));
    } else {
        docx = docx.add_paragraph(heading2("Readiness Checklist"));
        for f in &input.readiness.findings {
            docx = docx.add_paragraph(bullet_item(&format!(
                "[{}] {} ({} incident(s))",
                f.severity,
                f.message,
                f.incident_ids.len()
            )));
        }
    }

    if !input.overrides.is_empty() {
        docx = docx.add_paragraph(spacer());
        docx = docx.add_paragraph(heading2("Overrides and Known Gaps"));
        docx = docx.add_paragraph(body_text(
            "Overrides indicate accepted gaps for this quarter's leadership packet. These do not change metrics; they document assumptions and missing facts.",
        ));

        let header_row = TableRow::new(vec![
            header_cell("Rule"),
            header_cell("Incident"),
            header_cell("Reason"),
            header_cell("Approved By"),
            header_cell("Created At"),
        ]);
        let mut rows = vec![header_row];
        for o in input.overrides {
            rows.push(TableRow::new(vec![
                text_cell(&o.rule_key),
                text_cell(&o.incident_id),
                text_cell(&o.reason),
                text_cell(&o.approved_by),
                text_cell(&o.created_at),
            ]));
        }

        docx = docx.add_table(Table::new(rows));
    }

    docx = docx.add_paragraph(spacer());
    docx = docx.add_paragraph(heading2("Provenance Policy"));
    docx = docx.add_paragraph(body_text(
        "This packet distinguishes between facts, computed metrics, and optional AI enrichments:",
    ));
    docx = docx.add_paragraph(bullet_item(
        "Facts: user-entered or imported fields (timestamps, service, severity/impact, status).",
    ));
    docx = docx.add_paragraph(bullet_item(
        "Computed: metrics calculated deterministically from facts (MTTR, MTTA, trends).",
    ));
    docx = docx.add_paragraph(bullet_item(
        "AI Enrichments: optional drafts and summaries that can be accepted or ignored; metrics never depend on AI output.",
    ));

    docx = docx.add_paragraph(spacer());
    docx
}

