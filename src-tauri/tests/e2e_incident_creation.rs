//! E2E Integration Test: Create Incident → View in List
//! Tests the complete flow of creating an incident and verifying it appears in the list

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
    async fn create_test_service(db: &SqlitePool) -> String {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO services (id, name, description, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind("Test Service")
        .bind("Integration test service")
        .bind("2025-01-15T10:00:00Z")
        .bind("2025-01-15T10:00:00Z")
        .execute(db)
        .await
        .expect("Failed to insert service");

        id
    }

    #[tokio::test]
    async fn test_create_incident_appears_in_list() {
        let db = setup_test_db().await;
        let service_id = create_test_service(&db).await;

        // Step 1: Create incident
        let incident_id = Uuid::new_v4().to_string();
        let title = "Database Connection Failure";
        let severity = "P1";

        sqlx::query(
            r#"
            INSERT INTO incidents (
                id, title, service_id, severity, impact, status,
                started_at, detected_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&incident_id)
        .bind(title)
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

        // Step 2: Query incidents (simulate list view)
        let row = sqlx::query("SELECT id, title, severity, status FROM incidents WHERE id = ?")
            .bind(&incident_id)
            .fetch_one(&db)
            .await
            .expect("Incident not found in database");

        // Step 3: Verify incident appears with correct data
        let retrieved_id: String = row.get("id");
        let retrieved_title: String = row.get("title");
        let retrieved_severity: String = row.get("severity");
        let retrieved_status: String = row.get("status");

        assert_eq!(retrieved_id, incident_id);
        assert_eq!(retrieved_title, title);
        assert_eq!(retrieved_severity, severity);
        assert_eq!(retrieved_status, "active");
    }

    #[tokio::test]
    async fn test_create_multiple_incidents_list_all() {
        let db = setup_test_db().await;
        let service_id = create_test_service(&db).await;

        // Create 3 incidents
        for i in 0..3 {
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
            .bind(format!("Incident {}", i))
            .bind(&service_id)
            .bind("P2")
            .bind("medium")
            .bind("active")
            .bind("2025-01-15T10:00:00Z")
            .bind("2025-01-15T10:05:00Z")
            .bind("2025-01-15T10:00:00Z")
            .bind("2025-01-15T10:00:00Z")
            .execute(&db)
            .await
            .expect("Failed to insert incident");
        }

        // List all incidents (with no filter)
        let rows = sqlx::query("SELECT COUNT(*) as count FROM incidents")
            .fetch_one(&db)
            .await
            .expect("Failed to count incidents");

        let count: i64 = rows.get("count");
        assert_eq!(count, 3, "Should have exactly 3 incidents");
    }

    #[tokio::test]
    async fn test_create_incident_with_details() {
        let db = setup_test_db().await;
        let service_id = create_test_service(&db).await;

        let incident_id = Uuid::new_v4().to_string();
        let root_cause = "Connection pool exhaustion due to leaked connections";
        let resolution = "Restarted service and added monitoring";

        sqlx::query(
            r#"
            INSERT INTO incidents (
                id, title, service_id, severity, impact, status,
                root_cause, resolution,
                started_at, detected_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&incident_id)
        .bind("Connection Pool Leak")
        .bind(&service_id)
        .bind("P0")
        .bind("critical")
        .bind("active")
        .bind(root_cause)
        .bind(resolution)
        .bind("2025-01-15T10:00:00Z")
        .bind("2025-01-15T10:05:00Z")
        .bind("2025-01-15T10:00:00Z")
        .bind("2025-01-15T10:00:00Z")
        .execute(&db)
        .await
        .expect("Failed to insert incident");

        // Retrieve and verify all details
        let row = sqlx::query(
            "SELECT id, title, severity, root_cause, resolution FROM incidents WHERE id = ?",
        )
        .bind(&incident_id)
        .fetch_one(&db)
        .await
        .expect("Incident not found");

        assert_eq!(row.get::<String, _>("id"), incident_id);
        assert_eq!(row.get::<String, _>("title"), "Connection Pool Leak");
        assert_eq!(row.get::<String, _>("severity"), "P0");
        assert_eq!(row.get::<Option<String>, _>("root_cause"), Some(root_cause.to_string()));
        assert_eq!(row.get::<Option<String>, _>("resolution"), Some(resolution.to_string()));
    }

    #[tokio::test]
    async fn test_incident_status_transitions() {
        let db = setup_test_db().await;
        let service_id = create_test_service(&db).await;

        let incident_id = Uuid::new_v4().to_string();

        // Create with "active" status
        sqlx::query(
            r#"
            INSERT INTO incidents (
                id, title, service_id, severity, impact, status,
                started_at, detected_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&incident_id)
        .bind("Status Test")
        .bind(&service_id)
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

        // Transition to "acknowledged"
        sqlx::query("UPDATE incidents SET status = ? WHERE id = ?")
            .bind("acknowledged")
            .bind(&incident_id)
            .execute(&db)
            .await
            .expect("Failed to update status");

        // Verify transition
        let row = sqlx::query("SELECT status FROM incidents WHERE id = ?")
            .bind(&incident_id)
            .fetch_one(&db)
            .await
            .expect("Incident not found");

        let status: String = row.get("status");
        assert_eq!(status, "acknowledged");
    }
}
