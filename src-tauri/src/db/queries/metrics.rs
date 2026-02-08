use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::metrics::{
    CategoryCount, DashboardData, MetricFilters, MetricResult, QuarterlyTrends, ServiceDowntime,
    calculate_trend, format_decimal, format_minutes, format_percentage,
};

pub(crate) struct DateRange {
    start: String,
    end: String,
}

/// Build a WHERE clause and a vec of bind values for dynamic metric queries.
/// Returns (where_clause_string, bind_values) where bind_values are applied
/// in order using `?` placeholders.
fn build_where_clause(range: &DateRange, filters: &MetricFilters) -> (String, Vec<String>) {
    let mut conditions = vec![
        "i.deleted_at IS NULL".to_string(),
        "i.started_at >= ?".to_string(),
        "i.started_at <= ?".to_string(),
    ];
    let mut params: Vec<String> = vec![
        range.start.clone(),
        range.end.clone(),
    ];

    if let Some(ref sids) = filters.service_ids {
        if !sids.is_empty() {
            let placeholders: Vec<&str> = sids.iter().map(|_| "?").collect();
            conditions.push(format!("i.service_id IN ({})", placeholders.join(",")));
            for sid in sids {
                params.push(sid.clone());
            }
        }
    }

    let where_clause = conditions.join(" AND ");
    (where_clause, params)
}

/// Helper to execute a dynamic SQL query that returns a single optional f64 value.
async fn query_scalar_f64(db: &SqlitePool, sql: &str, params: &[String]) -> AppResult<f64> {
    let mut query = sqlx::query(sql);
    for p in params {
        query = query.bind(p);
    }
    let row = query
        .fetch_one(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // The first column holds the aggregate result
    Ok(row.get::<Option<f64>, _>(0).unwrap_or(0.0))
}

/// Helper to execute a dynamic SQL query that returns a single i64 value.
async fn query_scalar_i64(db: &SqlitePool, sql: &str, params: &[String]) -> AppResult<i64> {
    let mut query = sqlx::query(sql);
    for p in params {
        query = query.bind(p);
    }
    let row = query
        .fetch_one(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(row.get::<i64, _>(0))
}

async fn calc_mttr(db: &SqlitePool, range: &DateRange, filters: &MetricFilters) -> AppResult<f64> {
    let (wc, params) = build_where_clause(range, filters);
    let sql = format!(
        "SELECT AVG(duration_minutes) FROM incidents i WHERE {} AND i.resolved_at IS NOT NULL",
        wc
    );
    query_scalar_f64(db, &sql, &params).await
}

async fn calc_mtta(db: &SqlitePool, range: &DateRange, filters: &MetricFilters) -> AppResult<f64> {
    let (wc, params) = build_where_clause(range, filters);
    let sql = format!(
        "SELECT AVG(CAST((julianday(i.responded_at) - julianday(i.detected_at)) * 1440 AS REAL)) FROM incidents i WHERE {} AND i.responded_at IS NOT NULL",
        wc
    );
    query_scalar_f64(db, &sql, &params).await
}

async fn count_incidents(db: &SqlitePool, range: &DateRange, filters: &MetricFilters) -> AppResult<i64> {
    let (wc, params) = build_where_clause(range, filters);
    let sql = format!("SELECT COUNT(*) FROM incidents i WHERE {}", wc);
    query_scalar_i64(db, &sql, &params).await
}

async fn calc_recurrence_rate(db: &SqlitePool, range: &DateRange, filters: &MetricFilters) -> AppResult<f64> {
    let total = count_incidents(db, range, filters).await?;
    if total == 0 {
        return Ok(0.0);
    }
    let (wc, params) = build_where_clause(range, filters);
    let sql = format!("SELECT COUNT(*) FROM incidents i WHERE {} AND i.is_recurring = 1", wc);
    let recurring = query_scalar_i64(db, &sql, &params).await?;
    Ok((recurring as f64 / total as f64) * 100.0)
}

async fn calc_avg_tickets(db: &SqlitePool, range: &DateRange, filters: &MetricFilters) -> AppResult<f64> {
    let (wc, params) = build_where_clause(range, filters);
    let sql = format!("SELECT AVG(CAST(i.tickets_submitted AS REAL)) FROM incidents i WHERE {}", wc);
    query_scalar_f64(db, &sql, &params).await
}

async fn incidents_by_category(db: &SqlitePool, range: &DateRange, filters: &MetricFilters, column: &str) -> AppResult<Vec<CategoryCount>> {
    // Whitelist column names to prevent SQL injection
    let safe_column = match column {
        "severity" | "impact" | "status" => column,
        _ => return Err(AppError::Validation(format!("Invalid grouping column: {}", column))),
    };
    let (wc, params) = build_where_clause(range, filters);
    let sql = format!(
        "SELECT i.{} as category, COUNT(*) as cnt FROM incidents i WHERE {} GROUP BY i.{} ORDER BY cnt DESC",
        safe_column, wc, safe_column
    );
    let mut query = sqlx::query(&sql);
    for p in &params {
        query = query.bind(p);
    }
    let rows = query
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(|r| CategoryCount {
        category: r.get::<Option<String>, _>("category").unwrap_or_else(|| "Unknown".to_string()),
        count: r.get::<i64, _>("cnt"),
        previous_count: None,
    }).collect())
}

async fn incidents_by_service(db: &SqlitePool, range: &DateRange, filters: &MetricFilters) -> AppResult<Vec<CategoryCount>> {
    let (wc, params) = build_where_clause(range, filters);
    let sql = format!(
        "SELECT s.name as category, COUNT(*) as cnt FROM incidents i LEFT JOIN services s ON i.service_id = s.id WHERE {} GROUP BY s.name ORDER BY cnt DESC",
        wc
    );
    let mut query = sqlx::query(&sql);
    for p in &params {
        query = query.bind(p);
    }
    let rows = query
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(|r| CategoryCount {
        category: r.get::<Option<String>, _>("category").unwrap_or_else(|| "Unknown Service".to_string()),
        count: r.get::<i64, _>("cnt"),
        previous_count: None,
    }).collect())
}

async fn downtime_by_service(db: &SqlitePool, range: &DateRange, filters: &MetricFilters) -> AppResult<Vec<ServiceDowntime>> {
    let (wc, params) = build_where_clause(range, filters);
    // Use COALESCE to include active incidents: for unresolved incidents, compute duration from started_at to now
    let sql = format!(
        "SELECT i.service_id, s.name as service_name, COALESCE(SUM(COALESCE(i.duration_minutes, CAST((julianday('now') - julianday(i.started_at)) * 1440 AS INTEGER))), 0) as total_min FROM incidents i LEFT JOIN services s ON i.service_id = s.id WHERE {} GROUP BY i.service_id, s.name ORDER BY total_min DESC",
        wc
    );
    let mut query = sqlx::query(&sql);
    for p in &params {
        query = query.bind(p);
    }
    let rows = query
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows.iter().map(|r| {
        let total: i64 = r.get("total_min");
        ServiceDowntime {
            service_id: r.get::<Option<String>, _>("service_id").unwrap_or_default(),
            service_name: r.get::<Option<String>, _>("service_name").unwrap_or_else(|| "Unknown Service".to_string()),
            total_minutes: total,
            formatted: format_minutes(total as f64),
        }
    }).collect())
}

pub async fn get_dashboard_data(
    db: &SqlitePool,
    current_range: &DateRange,
    previous_range: Option<&DateRange>,
    filters: &MetricFilters,
    period_label: &str,
) -> AppResult<DashboardData> {
    let cur_mttr = calc_mttr(db, current_range, filters).await?;
    let cur_mtta = calc_mtta(db, current_range, filters).await?;
    let cur_recurrence = calc_recurrence_rate(db, current_range, filters).await?;
    let cur_tickets = calc_avg_tickets(db, current_range, filters).await?;
    let total = count_incidents(db, current_range, filters).await?;

    let (prev_mttr, prev_mtta, prev_recurrence, prev_tickets) = if let Some(prev) = previous_range {
        (
            Some(calc_mttr(db, prev, filters).await?),
            Some(calc_mtta(db, prev, filters).await?),
            Some(calc_recurrence_rate(db, prev, filters).await?),
            Some(calc_avg_tickets(db, prev, filters).await?),
        )
    } else {
        (None, None, None, None)
    };

    let mut by_severity = incidents_by_category(db, current_range, filters, "severity").await?;
    let mut by_impact = incidents_by_category(db, current_range, filters, "impact").await?;
    let mut by_svc = incidents_by_service(db, current_range, filters).await?;

    // Add previous counts if available
    if let Some(prev) = previous_range {
        let prev_sev = incidents_by_category(db, prev, filters, "severity").await?;
        let prev_imp = incidents_by_category(db, prev, filters, "impact").await?;
        let prev_svc = incidents_by_service(db, prev, filters).await?;

        for item in &mut by_severity {
            item.previous_count = prev_sev.iter().find(|p| p.category == item.category).map(|p| p.count);
        }
        for item in &mut by_impact {
            item.previous_count = prev_imp.iter().find(|p| p.category == item.category).map(|p| p.count);
        }
        for item in &mut by_svc {
            item.previous_count = prev_svc.iter().find(|p| p.category == item.category).map(|p| p.count);
        }
    }

    let downtime = downtime_by_service(db, current_range, filters).await?;

    // Build trends from last 4 quarters
    let trends = build_quarterly_trends(db, filters).await?;

    Ok(DashboardData {
        mttr: MetricResult {
            value: cur_mttr,
            previous_value: prev_mttr,
            trend: calculate_trend(cur_mttr, prev_mttr),
            formatted_value: if total == 0 { "\u{2014}".to_string() } else { format_minutes(cur_mttr) },
        },
        mtta: MetricResult {
            value: cur_mtta,
            previous_value: prev_mtta,
            trend: calculate_trend(cur_mtta, prev_mtta),
            formatted_value: if total == 0 { "\u{2014}".to_string() } else { format_minutes(cur_mtta) },
        },
        recurrence_rate: MetricResult {
            value: cur_recurrence,
            previous_value: prev_recurrence,
            trend: calculate_trend(cur_recurrence, prev_recurrence),
            formatted_value: if total == 0 { "\u{2014}".to_string() } else { format_percentage(cur_recurrence) },
        },
        avg_tickets: MetricResult {
            value: cur_tickets,
            previous_value: prev_tickets,
            trend: calculate_trend(cur_tickets, prev_tickets),
            formatted_value: if total == 0 { "\u{2014}".to_string() } else { format_decimal(cur_tickets) },
        },
        by_severity,
        by_impact,
        by_service: by_svc,
        downtime_by_service: downtime,
        trends,
        total_incidents: total,
        period_label: period_label.to_string(),
    })
}

async fn build_quarterly_trends(db: &SqlitePool, filters: &MetricFilters) -> AppResult<QuarterlyTrends> {
    // Get last 4 quarters
    let rows = sqlx::query(
        "SELECT * FROM quarter_config ORDER BY fiscal_year DESC, quarter_number DESC LIMIT 4"
    )
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let mut quarters = Vec::new();
    let mut mttr_vals = Vec::new();
    let mut mtta_vals = Vec::new();
    let mut count_vals = Vec::new();
    let mut recurrence_vals = Vec::new();
    let mut ticket_vals = Vec::new();

    // Reverse so oldest is first
    let mut quarter_data: Vec<_> = rows.iter().collect();
    quarter_data.reverse();

    for row in quarter_data {
        let label: String = row.get::<Option<String>, _>("label").unwrap_or_default();
        let start: String = row.get::<Option<String>, _>("start_date").unwrap_or_default();
        let end: String = row.get::<Option<String>, _>("end_date").unwrap_or_default();

        let range = DateRange { start, end };
        quarters.push(label);
        mttr_vals.push(calc_mttr(db, &range, filters).await?);
        mtta_vals.push(calc_mtta(db, &range, filters).await?);
        count_vals.push(count_incidents(db, &range, filters).await?);
        recurrence_vals.push(calc_recurrence_rate(db, &range, filters).await?);
        ticket_vals.push(calc_avg_tickets(db, &range, filters).await?);
    }

    Ok(QuarterlyTrends {
        quarters,
        mttr: mttr_vals,
        mtta: mtta_vals,
        incident_count: count_vals,
        recurrence_rate: recurrence_vals,
        avg_tickets: ticket_vals,
    })
}

// Exported function to get dashboard data by quarter ID
pub async fn get_dashboard_data_for_quarter(
    db: &SqlitePool,
    quarter_id: Option<&str>,
    filters: &MetricFilters,
) -> AppResult<DashboardData> {
    if let Some(qid) = quarter_id {
        let q = sqlx::query("SELECT * FROM quarter_config WHERE id = ?")
            .bind(qid)
            .fetch_optional(db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("Quarter '{}' not found", qid)))?;

        let start: String = q.get::<Option<String>, _>("start_date").unwrap_or_default();
        let end: String = q.get::<Option<String>, _>("end_date").unwrap_or_default();
        let label: String = q.get::<Option<String>, _>("label").unwrap_or_default();
        let fy: i64 = q.get::<i64, _>("fiscal_year");
        let qn: i64 = q.get::<i64, _>("quarter_number");

        let current_range = DateRange { start, end };

        // Find previous quarter
        let prev_q = if qn == 1 { 4 } else { qn - 1 };
        let prev_fy = if qn == 1 { fy - 1 } else { fy };
        let prev_row = sqlx::query(
            "SELECT * FROM quarter_config WHERE fiscal_year = ? AND quarter_number = ?"
        )
        .bind(prev_fy)
        .bind(prev_q)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let previous_range = prev_row.map(|pr| DateRange {
            start: pr.get::<Option<String>, _>("start_date").unwrap_or_default(),
            end: pr.get::<Option<String>, _>("end_date").unwrap_or_default(),
        });

        get_dashboard_data(db, &current_range, previous_range.as_ref(), filters, &label).await
    } else {
        // No quarter specified -- use current quarter based on today's date
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let row = sqlx::query(
            "SELECT * FROM quarter_config WHERE start_date <= ? AND end_date >= ? LIMIT 1"
        )
        .bind(&today)
        .bind(&today)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(q) = row {
            let qid: String = q.get::<Option<String>, _>("id").unwrap_or_default();
            // Recurse with the found quarter ID
            Box::pin(get_dashboard_data_for_quarter(db, Some(&qid), filters)).await
        } else {
            // No quarter found for today, return empty
            Ok(DashboardData {
                mttr: MetricResult::no_data(),
                mtta: MetricResult::no_data(),
                recurrence_rate: MetricResult::no_data(),
                avg_tickets: MetricResult::no_data(),
                by_severity: vec![],
                by_impact: vec![],
                by_service: vec![],
                downtime_by_service: vec![],
                trends: QuarterlyTrends {
                    quarters: vec![],
                    mttr: vec![],
                    mtta: vec![],
                    incident_count: vec![],
                    recurrence_rate: vec![],
                    avg_tickets: vec![],
                },
                total_incidents: 0,
                period_label: "No quarter configured".to_string(),
            })
        }
    }
}
