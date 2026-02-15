use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricFilters {
    pub service_ids: Option<Vec<String>>,
    pub min_severity: Option<String>,
    pub min_impact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricResult {
    pub value: f64,
    pub previous_value: Option<f64>,
    pub trend: String,
    pub formatted_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: i64,
    pub previous_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDowntime {
    pub service_id: String,
    pub service_name: String,
    pub total_minutes: i64,
    pub formatted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarterlyTrends {
    pub quarters: Vec<String>,
    pub mttr: Vec<f64>,
    pub mtta: Vec<f64>,
    pub incident_count: Vec<i64>,
    pub recurrence_rate: Vec<f64>,
    pub avg_tickets: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub mttr: MetricResult,
    pub mtta: MetricResult,
    pub recurrence_rate: MetricResult,
    pub avg_tickets: MetricResult,
    pub by_severity: Vec<CategoryCount>,
    pub by_impact: Vec<CategoryCount>,
    pub by_service: Vec<CategoryCount>,
    pub downtime_by_service: Vec<ServiceDowntime>,
    pub trends: QuarterlyTrends,
    pub total_incidents: i64,
    pub period_label: String,
}

impl MetricResult {
    pub fn no_data() -> Self {
        Self {
            value: 0.0,
            previous_value: None,
            trend: "NoData".to_string(),
            formatted_value: "—".to_string(),
        }
    }
}

pub fn calculate_trend(current: f64, previous: Option<f64>) -> String {
    if current.is_nan() || current.is_infinite() {
        return "NoData".to_string();
    }
    match previous {
        None => "NoData".to_string(),
        Some(prev) => {
            if prev.is_nan() || prev.is_infinite() {
                return "NoData".to_string();
            }
            if prev == 0.0 && current == 0.0 {
                "Flat".to_string()
            } else if prev == 0.0 {
                "Up".to_string()
            } else {
                let pct_change = ((current - prev) / prev).abs();
                if pct_change <= 0.01 {
                    "Flat".to_string()
                } else if current > prev {
                    "Up".to_string()
                } else {
                    "Down".to_string()
                }
            }
        }
    }
}

pub fn format_minutes(minutes: f64) -> String {
    if minutes.is_nan() || minutes.is_infinite() {
        return "—".to_string();
    }
    if minutes < 1.0 {
        "< 1 min".to_string()
    } else if minutes < 60.0 {
        format!("{:.0} min", minutes)
    } else {
        let hours = (minutes / 60.0).floor() as i64;
        let mins = (minutes % 60.0).round() as i64;
        if mins == 0 {
            format!("{}h", hours)
        } else {
            format!("{}h {}m", hours, mins)
        }
    }
}

pub fn format_percentage(value: f64) -> String {
    format!("{:.1}%", value)
}

pub fn format_decimal(value: f64) -> String {
    format!("{:.1}", value)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayCount {
    pub day: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourCount {
    pub hour: i32,
    pub count: i64,
}

/// Backlog aging: how many open incidents in each age bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklogAgingBucket {
    pub label: String,
    pub count: i64,
}

/// Leadership-facing metric glossary entry.
/// This is the single source of truth for report appendix + UI help text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    /// Stable identifier used by UI and reports (e.g. "mttr").
    pub key: String,
    pub name: String,
    pub definition: String,
    pub calculation: String,
    pub inclusion: String,
}

pub fn metric_glossary() -> Vec<MetricDefinition> {
    vec![
        MetricDefinition {
            key: "mttr".into(),
            name: "MTTR (Mean Time To Resolve)".into(),
            definition: "Average time to resolve incidents in the quarter.".into(),
            calculation: "Average of (resolved_at - started_at) in minutes, for incidents with resolved_at.".into(),
            inclusion: "Incidents are included in-quarter by detected_at.".into(),
        },
        MetricDefinition {
            key: "mtta".into(),
            name: "MTTA (Mean Time To Acknowledge)".into(),
            definition: "Average time to acknowledge incidents in the quarter.".into(),
            calculation: "Average of (acknowledged_at - detected_at) in minutes; if acknowledged_at is missing, falls back to responded_at.".into(),
            inclusion: "Incidents are included in-quarter by detected_at.".into(),
        },
        MetricDefinition {
            key: "total_incidents".into(),
            name: "Total Incidents".into(),
            definition: "Count of incidents detected during the quarter.".into(),
            calculation: "COUNT(incidents) where deleted_at IS NULL and detected_at is within the quarter range.".into(),
            inclusion: "Included in-quarter by detected_at.".into(),
        },
        MetricDefinition {
            key: "recurrence_rate".into(),
            name: "Recurrence Rate".into(),
            definition: "Percent of incidents marked recurring during the quarter.".into(),
            calculation: "100 * (recurring incidents / total incidents).".into(),
            inclusion: "Included in-quarter by detected_at.".into(),
        },
        MetricDefinition {
            key: "avg_tickets".into(),
            name: "Average Tickets".into(),
            definition: "Average number of tickets submitted per incident during the quarter.".into(),
            calculation: "Average of tickets_submitted across in-quarter incidents.".into(),
            inclusion: "Included in-quarter by detected_at.".into(),
        },
    ]
}

/// Service reliability scorecard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceReliabilityScore {
    pub service_id: String,
    pub service_name: String,
    pub incident_count: i64,
    pub mttr_minutes: f64,
    pub mttr_formatted: String,
    pub sla_compliance_pct: f64,
}

/// Escalation funnel: severity distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationFunnelEntry {
    pub severity: String,
    pub count: i64,
    pub percentage: f64,
}
