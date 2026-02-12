//! Integration test helpers for E2E testing
//! Provides utilities for setting up test databases and fixtures

use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

/// Initialize an in-memory SQLite database for testing
pub async fn setup_test_db() -> SqlitePool {
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

/// Create a test incident with sensible defaults
pub async fn create_test_incident(
    db: &SqlitePool,
    title: &str,
    severity: &str,
) -> String {
    let id = Uuid::new_v4().to_string();
    let service_id = create_test_service(db, "Test Service").await;

    sqlx::query(
        r#"
        INSERT INTO incidents (
            id, title, service_id, severity, impact, status,
            started_at, detected_at, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(title)
    .bind(&service_id)
    .bind(severity)
    .bind("high")
    .bind("active")
    .bind("2025-01-15T10:00:00Z")
    .bind("2025-01-15T10:05:00Z")
    .bind("2025-01-15T10:00:00Z")
    .bind("2025-01-15T10:00:00Z")
    .execute(db)
    .await
    .expect("Failed to insert test incident");

    id
}

/// Create a test service with sensible defaults
pub async fn create_test_service(db: &SqlitePool, name: &str) -> String {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO services (id, name, description, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(name)
    .bind("Test service for integration tests")
    .bind("2025-01-15T10:00:00Z")
    .bind("2025-01-15T10:00:00Z")
    .execute(db)
    .await
    .expect("Failed to insert test service");

    id
}

/// Create multiple test incidents for dashboard testing
pub async fn create_test_incidents_for_dashboard(db: &SqlitePool, count: usize) -> Vec<String> {
    let mut ids = Vec::new();
    let service_id = create_test_service(db, "Dashboard Test Service").await;

    for i in 0..count {
        let id = Uuid::new_v4().to_string();
        let severity = match i % 5 {
            0 => "P0",
            1 => "P1",
            2 => "P2",
            3 => "P3",
            _ => "P4",
        };

        sqlx::query(
            r#"
            INSERT INTO incidents (
                id, title, service_id, severity, impact, status,
                started_at, detected_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(format!("Test Incident {}", i))
        .bind(&service_id)
        .bind(severity)
        .bind("high")
        .bind("active")
        .bind("2025-01-15T10:00:00Z")
        .bind("2025-01-15T10:05:00Z")
        .bind("2025-01-15T10:00:00Z")
        .bind("2025-01-15T10:00:00Z")
        .execute(db)
        .await
        .expect("Failed to insert test incident");

        ids.push(id);
    }

    ids
}
