use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::metrics::{DayCount, HourCount};

pub async fn get_incident_heatmap(
    db: &SqlitePool,
    start_date: &str,
    end_date: &str,
) -> AppResult<Vec<DayCount>> {
    let rows = sqlx::query(
        "SELECT date(detected_at) as day, COUNT(*) as count \
         FROM incidents \
         WHERE detected_at >= ? AND detected_at <= ? \
         GROUP BY day \
         ORDER BY day ASC"
    )
    .bind(start_date)
    .bind(end_date)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows
        .iter()
        .map(|r| DayCount {
            day: r.get("day"),
            count: r.get("count"),
        })
        .collect())
}

pub async fn get_incident_by_hour(
    db: &SqlitePool,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> AppResult<Vec<HourCount>> {
    let mut sql = String::from(
        "SELECT CAST(strftime('%H', detected_at) AS INTEGER) as hour, COUNT(*) as count \
         FROM incidents WHERE 1=1"
    );
    let mut binds: Vec<String> = vec![];

    if let Some(start) = start_date {
        sql.push_str(" AND detected_at >= ?");
        binds.push(start.to_string());
    }
    if let Some(end) = end_date {
        sql.push_str(" AND detected_at <= ?");
        binds.push(end.to_string());
    }

    sql.push_str(" GROUP BY hour ORDER BY hour ASC");

    let mut query = sqlx::query(&sql);
    for bind in &binds {
        query = query.bind(bind);
    }

    let rows = query
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows
        .iter()
        .map(|r| HourCount {
            hour: r.get("hour"),
            count: r.get("count"),
        })
        .collect())
}
