-- SLA Definitions: map priority levels to response/resolve time targets
CREATE TABLE sla_definitions (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    priority TEXT NOT NULL,
    response_time_minutes INTEGER NOT NULL,
    resolve_time_minutes INTEGER NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Unique constraint: one active SLA definition per priority
CREATE UNIQUE INDEX idx_sla_definitions_priority_active
ON sla_definitions(priority) WHERE is_active = 1;

-- Seed default SLA definitions matching P0-P4
INSERT INTO sla_definitions (id, name, priority, response_time_minutes, resolve_time_minutes, is_active) VALUES
    ('sla-p0', 'P0 - Critical', 'P0', 15, 60, 1),
    ('sla-p1', 'P1 - High', 'P1', 30, 240, 1),
    ('sla-p2', 'P2 - Medium', 'P2', 60, 480, 1),
    ('sla-p3', 'P3 - Low', 'P3', 120, 1440, 1),
    ('sla-p4', 'P4 - Informational', 'P4', 480, 2880, 1);
