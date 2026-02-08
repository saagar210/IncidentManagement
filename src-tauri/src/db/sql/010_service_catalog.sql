-- Migration 010: Service Catalog Enhancement
-- Adds owner, tier, runbook to services; creates service_dependencies table

ALTER TABLE services ADD COLUMN owner TEXT NOT NULL DEFAULT '';

ALTER TABLE services ADD COLUMN tier TEXT NOT NULL DEFAULT 'T3' CHECK(tier IN ('T1', 'T2', 'T3', 'T4'));

ALTER TABLE services ADD COLUMN runbook TEXT NOT NULL DEFAULT '';

CREATE TABLE service_dependencies (
    id TEXT PRIMARY KEY,
    service_id TEXT NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    depends_on_service_id TEXT NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    dependency_type TEXT NOT NULL DEFAULT 'runtime' CHECK(dependency_type IN ('runtime', 'build', 'data', 'optional')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    CHECK(service_id != depends_on_service_id),
    UNIQUE(service_id, depends_on_service_id)
)
