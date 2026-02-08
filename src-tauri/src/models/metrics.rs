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
    match previous {
        None => "NoData".to_string(),
        Some(prev) => {
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
