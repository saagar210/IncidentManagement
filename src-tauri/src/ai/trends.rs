use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceTrend {
    pub service_id: String,
    pub service_name: String,
    pub trend_type: String,
    pub message: String,
    pub incident_count_current: i64,
    pub incident_count_previous: i64,
}

/// Detect trending services by comparing incident counts between
/// the last 7 days and the previous 7 days.
///
/// Flags:
/// - "degrading": current count > previous count * 1.5 (50%+ increase)
/// - "high_volume": 3+ incidents in the last 7 days
pub async fn detect_service_trends(db: &SqlitePool) -> AppResult<Vec<ServiceTrend>> {
    let rows = sqlx::query(
        "SELECT
            s.id as service_id,
            s.name as service_name,
            COALESCE(SUM(CASE
                WHEN i.created_at >= datetime('now', '-7 days') THEN 1
                ELSE 0
            END), 0) as current_count,
            COALESCE(SUM(CASE
                WHEN i.created_at >= datetime('now', '-14 days')
                 AND i.created_at < datetime('now', '-7 days') THEN 1
                ELSE 0
            END), 0) as previous_count
         FROM services s
         LEFT JOIN incidents i ON i.service_id = s.id AND i.deleted_at IS NULL
         WHERE s.deleted_at IS NULL
         GROUP BY s.id, s.name
         HAVING current_count > 0 OR previous_count > 0",
    )
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Database(format!("Failed to query service trends: {}", e)))?;

    let mut trends: Vec<ServiceTrend> = Vec::new();

    for row in &rows {
        let service_id: String = row.get("service_id");
        let service_name: String = row.get("service_name");
        let current: i64 = row.get("current_count");
        let previous: i64 = row.get("previous_count");

        // Check for degrading trend: current > previous * 1.5
        if previous > 0 && current as f64 > previous as f64 * 1.5 {
            trends.push(ServiceTrend {
                service_id: service_id.clone(),
                service_name: service_name.clone(),
                trend_type: "degrading".to_string(),
                message: format!(
                    "{} has {} incidents in the last 7 days vs {} in the previous 7 days ({}% increase)",
                    service_name,
                    current,
                    previous,
                    ((current as f64 - previous as f64) / previous as f64 * 100.0) as i64,
                ),
                incident_count_current: current,
                incident_count_previous: previous,
            });
        } else if previous == 0 && current > 0 {
            // New incidents where there were none before — also degrading
            trends.push(ServiceTrend {
                service_id: service_id.clone(),
                service_name: service_name.clone(),
                trend_type: "degrading".to_string(),
                message: format!(
                    "{} has {} new incidents in the last 7 days with none in the previous period",
                    service_name, current,
                ),
                incident_count_current: current,
                incident_count_previous: previous,
            });
        }

        // Check for high volume: 3+ in the last 7 days
        if current >= 3 {
            // Avoid duplicate if already flagged as degrading with the same service
            let already_flagged = trends.iter().any(|t| {
                t.service_id == service_id && t.trend_type == "high_volume"
            });
            if !already_flagged {
                trends.push(ServiceTrend {
                    service_id: service_id.clone(),
                    service_name: service_name.clone(),
                    trend_type: "high_volume".to_string(),
                    message: format!(
                        "{} has {} incidents in the last 7 days",
                        service_name, current,
                    ),
                    incident_count_current: current,
                    incident_count_previous: previous,
                });
            }
        }
    }

    Ok(trends)
}
