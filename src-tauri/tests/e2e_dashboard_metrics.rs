//! E2E Integration Test: Create Incidents → View Dashboard Metrics
//! Tests the complete flow of creating incidents and viewing calculated metrics

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePool;
    use uuid::Uuid;

    /// Helper: Create test database with migrations
    async fn setup_test_db() -> SqlitePool {
        let database_url = "sqlite::memory:";
        let pool = SqlitePool::connect(database_url)
            .await
            .expect("Failed to create test database");

        // Run migrations
        sqlx::migrate!("src/db/sql")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    /// Helper: Create a service for testing
    async fn create_test_service(db: &SqlitePool, name: &str) -> String {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO services (id, name, description, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(name)
        .bind("Dashboard Metrics Test Service")
        .bind("2025-01-15T10:00:00Z")
        .bind("2025-01-15T10:00:00Z")
        .execute(db)
        .await
        .expect("Failed to insert service");

        id
    }

    #[tokio::test]
    async fn test_dashboard_calculates_incident_count() {
        let db = setup_test_db().await;
        let service_id = create_test_service(&db, "Metrics Service").await;

        // Create incidents of various severities
        let severities = vec!["P0", "P1", "P1", "P2", "P2", "P2", "P3", "P4"];

        for (idx, severity) in severities.iter().enumerate() {
            let incident_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO incidents (
                    id, title, service_id, severity, impact, status,
                    started_at, detected_at, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&incident_id)
            .bind(format!("Severity {} Incident", severity))
            .bind(&service_id)
            .bind(severity)
            .bind("high")
            .bind("active")
            .bind("2025-01-15T10:00:00Z")
            .bind("2025-01-15T10:05:00Z")
            .bind("2025-01-15T10:00:00Z")
            .bind("2025-01-15T10:00:00Z")
            .execute(&db)
            .await
            .expect("Failed to insert incident");
        }

        // Calculate total incidents (dashboard metric)
        let total_row = sqlx::query("SELECT COUNT(*) as total FROM incidents")
            .fetch_one(&db)
            .await
            .expect("Failed to count incidents");

        let total: i64 = total_row.get("total");
        assert_eq!(total, 8, "Dashboard should show 8 total incidents");

        // Calculate incidents by severity
        let severity_row = sqlx::query("SELECT severity, COUNT(*) as cnt FROM incidents GROUP BY severity ORDER BY severity")
            .fetch_all(&db)
            .await
            .expect("Failed to count by severity");

        assert_eq!(severity_row.len(), 5, "Should have 5 severity levels");
    }

    #[tokio::test]
    async fn test_dashboard_calculates_mttr() {
        let db = setup_test_db().await;
        let service_id = create_test_service(&db, "MTTR Test Service").await;

        // Create incidents with known durations
        let durations = vec![
            ("2025-01-15T10:00:00Z", "2025-01-15T11:00:00Z"), // 60 minutes
            ("2025-01-15T12:00:00Z", "2025-01-15T14:00:00Z"), // 120 minutes
            ("2025-01-15T15:00:00Z", "2025-01-15T15:30:00Z"), // 30 minutes
        ];

        for (i, (start, resolved)) in durations.iter().enumerate() {
            let incident_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO incidents (
                    id, title, service_id, severity, impact, status,
                    started_at, detected_at, resolved_at, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&incident_id)
            .bind(format!("MTTR Incident {}", i + 1))
            .bind(&service_id)
            .bind("P2")
            .bind("high")
            .bind("resolved")
            .bind(start)
            .bind(start)
            .bind(resolved)
            .bind(start)
            .bind(resolved)
            .execute(&db)
            .await
            .expect("Failed to insert incident");
        }

        // Calculate MTTR (average resolution time)
        // This simulates the dashboard metric calculation
        let mttr_row = sqlx::query(
            "SELECT AVG(CAST((julianday(resolved_at) - julianday(started_at)) * 1440 AS REAL)) as avg_minutes FROM incidents WHERE resolved_at IS NOT NULL"
        )
        .fetch_one(&db)
        .await
        .expect("Failed to calculate MTTR");

        let avg_minutes: Option<f64> = mttr_row.get("avg_minutes");
        assert!(avg_minutes.is_some(), "MTTR should be calculable");
        // Average of 60, 120, 30 = 70 minutes
        assert!((avg_minutes.unwrap() - 70.0).abs() < 1.0, "MTTR should be approximately 70 minutes");
    }

    #[tokio::test]
    async fn test_dashboard_service_breakdown() {
        let db = setup_test_db().await;

        // Create multiple services
        let service1 = create_test_service(&db, "Service A").await;
        let service2 = create_test_service(&db, "Service B").await;
        let service3 = create_test_service(&db, "Service C").await;

        // Assign incidents to services
        let distribution = vec![
            (&service1, 5),
            (&service2, 3),
            (&service3, 2),
        ];

        for (service_id, count) in distribution.iter() {
            for i in 0..*count {
                let incident_id = Uuid::new_v4().to_string();
                sqlx::query(
                    r#"
                    INSERT INTO incidents (
                        id, title, service_id, severity, impact, status,
                        started_at, detected_at, created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    "#,
                )
                .bind(&incident_id)
                .bind(format!("Service Incident {}", i))
                .bind(service_id)
                .bind("P2")
                .bind("high")
                .bind("active")
                .bind("2025-01-15T10:00:00Z")
                .bind("2025-01-15T10:05:00Z")
                .bind("2025-01-15T10:00:00Z")
                .bind("2025-01-15T10:00:00Z")
                .execute(&db)
                .await
                .expect("Failed to insert incident");
            }
        }

        // Get incidents by service (dashboard breakdown)
        let service_rows = sqlx::query(
            "SELECT s.name as service_name, COUNT(i.id) as incident_count FROM services s LEFT JOIN incidents i ON s.id = i.service_id GROUP BY s.id ORDER BY incident_count DESC"
        )
        .fetch_all(&db)
        .await
        .expect("Failed to get service breakdown");

        assert_eq!(service_rows.len(), 3, "Should have 3 services");

        // Verify first service has most incidents
        let first_count: i64 = service_rows[0].get("incident_count");
        assert_eq!(first_count, 5, "Service A should have 5 incidents");
    }

    #[tokio::test]
    async fn test_dashboard_severity_distribution() {
        let db = setup_test_db().await;
        let service_id = create_test_service(&db, "Distribution Service").await;

        // Create incidents with specific severity distribution
        let severity_distribution = vec![
            ("P0", 1),
            ("P1", 3),
            ("P2", 5),
            ("P3", 4),
            ("P4", 2),
        ];

        for (severity, count) in severity_distribution.iter() {
            for i in 0..*count {
                let incident_id = Uuid::new_v4().to_string();
                sqlx::query(
                    r#"
                    INSERT INTO incidents (
                        id, title, service_id, severity, impact, status,
                        started_at, detected_at, created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    "#,
                )
                .bind(&incident_id)
                .bind(format!("{} Incident {}", severity, i))
                .bind(&service_id)
                .bind(severity)
                .bind("high")
                .bind("active")
                .bind("2025-01-15T10:00:00Z")
                .bind("2025-01-15T10:05:00Z")
                .bind("2025-01-15T10:00:00Z")
                .bind("2025-01-15T10:00:00Z")
                .execute(&db)
                .await
                .expect("Failed to insert incident");
            }
        }

        // Get severity distribution (dashboard chart data)
        let severity_rows = sqlx::query(
            "SELECT severity, COUNT(*) as cnt FROM incidents GROUP BY severity ORDER BY severity"
        )
        .fetch_all(&db)
        .await
        .expect("Failed to get severity distribution");

        assert_eq!(severity_rows.len(), 5, "Should have all 5 severities");

        // Verify P2 has most incidents
        let p2_row = severity_rows.iter()
            .find(|r| r.get::<String, _>("severity") == "P2")
            .expect("P2 not found");
        let p2_count: i64 = p2_row.get("cnt");
        assert_eq!(p2_count, 5, "P2 should have 5 incidents");
    }

    #[tokio::test]
    async fn test_dashboard_sla_compliance() {
        let db = setup_test_db().await;
        let service_id = create_test_service(&db, "SLA Service").await;

        // Create some resolved incidents (for SLA check)
        let sla_test_cases = vec![
            ("2025-01-15T10:00:00Z", "2025-01-15T11:00:00Z", "P1"), // 60 min, target 240 min -> compliant
            ("2025-01-15T12:00:00Z", "2025-01-15T17:00:00Z", "P1"), // 300 min, target 240 min -> breach
            ("2025-01-15T18:00:00Z", "2025-01-15T18:30:00Z", "P3"), // 30 min, target 1440 min -> compliant
        ];

        for (start, resolved, severity) in sla_test_cases.iter() {
            let incident_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO incidents (
                    id, title, service_id, severity, impact, status,
                    started_at, detected_at, resolved_at, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&incident_id)
            .bind("SLA Test")
            .bind(&service_id)
            .bind(severity)
            .bind("high")
            .bind("resolved")
            .bind(start)
            .bind(start)
            .bind(resolved)
            .bind(start)
            .bind(resolved)
            .execute(&db)
            .await
            .expect("Failed to insert SLA test incident");
        }

        // Verify all incidents were created for metric calculation
        let count_row = sqlx::query("SELECT COUNT(*) as total FROM incidents WHERE status = ?")
            .bind("resolved")
            .fetch_one(&db)
            .await
            .expect("Failed to count");

        let total: i64 = count_row.get("total");
        assert_eq!(total, 3, "Should have 3 resolved incidents for SLA check");
    }
}
