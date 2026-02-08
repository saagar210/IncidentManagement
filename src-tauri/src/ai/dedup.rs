use crate::ai::similar::SimilarIncident;
use crate::error::AppResult;
use sqlx::{Row, SqlitePool};

/// Check for potential duplicate incidents by searching open incidents
/// in the same service using FTS5 title matching.
pub async fn check_duplicates(
    db: &SqlitePool,
    title: &str,
    service_id: &str,
) -> AppResult<Vec<SimilarIncident>> {
    if title.trim().is_empty() {
        return Ok(vec![]);
    }

    // Build FTS5 query — each word gets prefix matching
    let fts_query = title
        .replace('"', "\"\"")
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|w| format!("\"{}\"*", w))
        .collect::<Vec<_>>()
        .join(" OR ");

    if fts_query.is_empty() {
        return Ok(vec![]);
    }

    let rows = sqlx::query(
        "SELECT i.id, i.title, COALESCE(s.name, 'Unknown') as service_name, \
                i.severity, i.status, rank
         FROM incidents_fts
         JOIN incidents i ON i.rowid = incidents_fts.rowid
         LEFT JOIN services s ON i.service_id = s.id
         WHERE incidents_fts MATCH ?1
           AND i.deleted_at IS NULL
           AND i.service_id = ?2
           AND i.status NOT IN ('Resolved', 'Post-Mortem')
         ORDER BY rank
         LIMIT 5",
    )
    .bind(&fts_query)
    .bind(service_id)
    .fetch_all(db)
    .await;

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
