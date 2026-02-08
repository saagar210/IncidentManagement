use std::collections::HashMap;
use docx_rs::*;

use crate::models::incident::{ActionItem, Incident};
use crate::models::metrics::format_minutes;

use super::{heading1, bullet_item, spacer};

/// A generated discussion point for the quarterly review.
#[derive(Debug, Clone)]
pub struct DiscussionPoint {
    pub text: String,
    pub trigger: String,
    pub severity: String,
}

/// Generate discussion points based on the 10 rules.
pub fn generate(
    incidents: &[Incident],
    prev_incidents: &[Incident],
    mttr: f64,
    prev_mttr: Option<f64>,
    total_incidents: i64,
    prev_total: Option<i64>,
    action_items_all: &[ActionItem],
) -> Vec<DiscussionPoint> {
    let mut points: Vec<DiscussionPoint> = Vec::new();

    // Aggregate incidents per service
    let mut service_counts: HashMap<String, i64> = HashMap::new();
    let mut service_downtime: HashMap<String, i64> = HashMap::new();
    for inc in incidents {
        *service_counts.entry(inc.service_name.clone()).or_default() += 1;
        if let Some(d) = inc.duration_minutes {
            *service_downtime.entry(inc.service_name.clone()).or_default() += d;
        }
    }

    // Previous quarter service counts
    let mut prev_service_counts: HashMap<String, i64> = HashMap::new();
    for inc in prev_incidents {
        *prev_service_counts.entry(inc.service_name.clone()).or_default() += 1;
    }

    // Rule 1: Service with 3+ incidents -> systemic improvement question
    for (service, count) in &service_counts {
        if *count >= 3 {
            points.push(DiscussionPoint {
                text: format!(
                    "{} had {} incidents this quarter. Are there systemic improvements that should be prioritized?",
                    service, count
                ),
                trigger: "Rule 1: 3+ incidents on a service".to_string(),
                severity: "high".to_string(),
            });
        }
    }

    // Rule 2: Any recurring incident -> was original action item implemented?
    let recurring: Vec<&Incident> = incidents.iter().filter(|i| i.is_recurring).collect();
    for inc in &recurring {
        points.push(DiscussionPoint {
            text: format!(
                "'{}' is a recurring incident. Were the original remediation action items fully implemented?",
                inc.title
            ),
            trigger: "Rule 2: Recurring incident detected".to_string(),
            severity: "high".to_string(),
        });
    }

    // Rule 3: MTTR increased -> what contributed to slower resolution?
    if let Some(prev) = prev_mttr {
        if prev > 0.0 && mttr > prev * 1.05 {
            points.push(DiscussionPoint {
                text: format!(
                    "MTTR increased from {} to {}. What contributed to slower incident resolution?",
                    format_minutes(prev),
                    format_minutes(mttr)
                ),
                trigger: "Rule 3: MTTR increase".to_string(),
                severity: "medium".to_string(),
            });
        }
    }

    // Rule 4: MTTR decreased -> what practices should continue?
    if let Some(prev) = prev_mttr {
        if prev > 0.0 && mttr < prev * 0.95 {
            points.push(DiscussionPoint {
                text: format!(
                    "MTTR improved from {} to {}. Which response practices or tooling should we continue investing in?",
                    format_minutes(prev),
                    format_minutes(mttr)
                ),
                trigger: "Rule 4: MTTR decrease".to_string(),
                severity: "low".to_string(),
            });
        }
    }

    // Rule 5: Any P0 incident -> is incident response adequate?
    let p0_incidents: Vec<&Incident> = incidents.iter().filter(|i| i.priority == "P0").collect();
    if !p0_incidents.is_empty() {
        let titles: Vec<&str> = p0_incidents.iter().map(|i| i.title.as_str()).collect();
        points.push(DiscussionPoint {
            text: format!(
                "{} P0 incident(s) occurred ({}). Is our incident response process adequate for critical situations?",
                p0_incidents.len(),
                titles.join(", ")
            ),
            trigger: "Rule 5: P0 incident occurred".to_string(),
            severity: "critical".to_string(),
        });
    }

    // Rule 6: Total incidents up >25% -> trend or seasonal?
    if let Some(prev) = prev_total {
        if prev > 0 {
            let pct_increase = ((total_incidents - prev) as f64 / prev as f64) * 100.0;
            if pct_increase > 25.0 {
                points.push(DiscussionPoint {
                    text: format!(
                        "Total incidents increased by {:.0}% (from {} to {}). Is this a trend or seasonal variation?",
                        pct_increase, prev, total_incidents
                    ),
                    trigger: "Rule 6: >25% incident increase".to_string(),
                    severity: "medium".to_string(),
                });
            }
        }
    }

    // Rule 7: Service >60 min total downtime -> redundancy justified?
    for (service, downtime) in &service_downtime {
        if *downtime > 60 {
            points.push(DiscussionPoint {
                text: format!(
                    "{} had {} of total downtime. Is additional redundancy or failover investment justified?",
                    service,
                    format_minutes(*downtime as f64)
                ),
                trigger: "Rule 7: >60 min downtime on a service".to_string(),
                severity: "medium".to_string(),
            });
        }
    }

    // Rule 8: Previously-problematic service at zero -> what changed?
    for (service, prev_count) in &prev_service_counts {
        if *prev_count >= 2 && !service_counts.contains_key(service) {
            points.push(DiscussionPoint {
                text: format!(
                    "{} had {} incidents last quarter but zero this quarter. What changed?",
                    service, prev_count
                ),
                trigger: "Rule 8: Previously-problematic service now at zero".to_string(),
                severity: "low".to_string(),
            });
        }
    }

    // Rule 9: Action items from previous quarter -> completion status?
    let open_actions: Vec<&ActionItem> = action_items_all
        .iter()
        .filter(|a| a.status != "Done")
        .collect();
    if !open_actions.is_empty() {
        points.push(DiscussionPoint {
            text: format!(
                "{} action item(s) from previous incidents are still open. What is the status and expected completion?",
                open_actions.len()
            ),
            trigger: "Rule 9: Open action items".to_string(),
            severity: "medium".to_string(),
        });
    }

    // Rule 10: Avg tickets >10 -> improve proactive communication?
    if total_incidents > 0 {
        let avg_tickets: f64 = incidents.iter().map(|i| i.tickets_submitted as f64).sum::<f64>()
            / total_incidents as f64;
        if avg_tickets > 10.0 {
            points.push(DiscussionPoint {
                text: format!(
                    "Average tickets per incident was {:.1}. Should we improve proactive communication or self-service documentation?",
                    avg_tickets
                ),
                trigger: "Rule 10: Avg tickets >10".to_string(),
                severity: "medium".to_string(),
            });
        }
    }

    points
}

/// Write discussion points into the document.
pub fn build(docx: Docx, points: &[DiscussionPoint]) -> Docx {
    let mut docx = docx.add_paragraph(heading1("Discussion Points"));

    if points.is_empty() {
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text("No automatic discussion points generated for this quarter.")
                        .size(11 * 2),
                ),
        );
        docx = docx.add_paragraph(spacer());
        return docx;
    }

    for (i, point) in points.iter().enumerate() {
        let severity_label = match point.severity.as_str() {
            "critical" => "[CRITICAL]",
            "high" => "[HIGH]",
            "medium" => "[MEDIUM]",
            "low" => "[LOW]",
            _ => "",
        };

        docx = docx.add_paragraph(bullet_item(&format!(
            "{}. {} {}",
            i + 1,
            severity_label,
            point.text
        )));
    }

    docx = docx.add_paragraph(spacer());

    docx
}
