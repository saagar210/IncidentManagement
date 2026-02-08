use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarIncident {
    pub id: String,
    pub title: String,
    pub service_name: String,
    pub severity: String,
    pub status: String,
    pub rank: f64,
}

pub async fn find_similar(
    db: &SqlitePool,
    query: &str,
    exclude_id: Option<&str>,
    limit: i32,
) -> AppResult<Vec<SimilarIncident>> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    // Build FTS5 query - each word gets prefix matching
    let fts_query = query
        .replace('"', "\"\"")
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|w| format!("\"{}\"*", w))
        .collect::<Vec<_>>()
        .join(" OR ");

    if fts_query.is_empty() {
        return Ok(vec![]);
    }

    let rows = if let Some(eid) = exclude_id {
        sqlx::query(
            "SELECT i.id, i.title, s.name as service_name, i.severity, i.status, rank
             FROM incidents_fts
             JOIN incidents i ON i.rowid = incidents_fts.rowid
             LEFT JOIN services s ON i.service_id = s.id
             WHERE incidents_fts MATCH ?1
               AND i.deleted_at IS NULL
               AND i.id != ?2
             ORDER BY rank
             LIMIT ?3",
        )
        .bind(&fts_query)
        .bind(eid)
        .bind(limit)
        .fetch_all(db)
        .await
    } else {
        sqlx::query(
            "SELECT i.id, i.title, s.name as service_name, i.severity, i.status, rank
             FROM incidents_fts
             JOIN incidents i ON i.rowid = incidents_fts.rowid
             LEFT JOIN services s ON i.service_id = s.id
             WHERE incidents_fts MATCH ?1
               AND i.deleted_at IS NULL
             ORDER BY rank
             LIMIT ?2",
        )
        .bind(&fts_query)
        .bind(limit)
        .fetch_all(db)
        .await
    };

    match rows {
        Ok(rows) => Ok(rows
            .iter()
            .map(|r| SimilarIncident {
                id: r.get("id"),
                title: r.get("title"),
                service_name: r
                    .get::<Option<String>, _>("service_name")
                    .unwrap_or_else(|| "Unknown".to_string()),
                severity: r.get("severity"),
                status: r.get("status"),
                rank: r.get::<f64, _>("rank"),
            })
            .collect()),
        Err(_) => Ok(vec![]), // FTS5 table might not exist yet
    }
}
