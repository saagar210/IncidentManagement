-- Migration 012: Expanded Lifecycle States
-- HIGH RISK: Rebuilds the incidents table to add Acknowledged status,
-- new timestamp columns, and reopen support.
--
-- State transition graph (directed):
--   Active → Acknowledged, Monitoring, Resolved
--   Acknowledged → Active, Monitoring, Resolved
--   Monitoring → Active, Acknowledged, Resolved
--   Resolved → Active (reopen), Post-Mortem
--   Post-Mortem → Active (reopen)

PRAGMA foreign_keys = OFF;

-- Step 1: Create the new table with expanded schema
CREATE TABLE incidents_new (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    service_id TEXT NOT NULL REFERENCES services(id),
    severity TEXT NOT NULL CHECK (severity IN ('Critical', 'High', 'Medium', 'Low')),
    impact TEXT NOT NULL CHECK (impact IN ('Critical', 'High', 'Medium', 'Low')),
    status TEXT NOT NULL CHECK (status IN ('Active', 'Acknowledged', 'Monitoring', 'Resolved', 'Post-Mortem')),
    started_at TEXT NOT NULL,
    detected_at TEXT NOT NULL,
    acknowledged_at TEXT,
    first_response_at TEXT,
    mitigation_started_at TEXT,
    responded_at TEXT,
    resolved_at TEXT,
    reopened_at TEXT,
    reopen_count INTEGER NOT NULL DEFAULT 0,
    duration_minutes INTEGER GENERATED ALWAYS AS (
        CASE
            WHEN resolved_at IS NOT NULL AND started_at IS NOT NULL
            THEN CAST((julianday(resolved_at) - julianday(started_at)) * 1440 AS INTEGER)
            ELSE NULL
        END
    ) STORED,
    root_cause TEXT DEFAULT '',
    resolution TEXT DEFAULT '',
    tickets_submitted INTEGER DEFAULT 0,
    affected_users INTEGER DEFAULT 0,
    is_recurring INTEGER NOT NULL DEFAULT 0,
    recurrence_of TEXT REFERENCES incidents_new(id),
    lessons_learned TEXT DEFAULT '',
    action_items TEXT DEFAULT '',
    external_ref TEXT DEFAULT '',
    notes TEXT DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    deleted_at TEXT DEFAULT NULL
);

-- Step 2: Copy data from old table, mapping existing statuses
-- All 4 existing statuses are valid in the new schema, so direct copy works.
INSERT INTO incidents_new (
    id, title, service_id, severity, impact, status,
    started_at, detected_at, responded_at, resolved_at,
    reopen_count,
    root_cause, resolution, tickets_submitted, affected_users,
    is_recurring, recurrence_of, lessons_learned, action_items,
    external_ref, notes, created_at, updated_at, deleted_at
)
SELECT
    id, title, service_id, severity, impact, status,
    started_at, detected_at, responded_at, resolved_at,
    0,
    root_cause, resolution, tickets_submitted, affected_users,
    is_recurring, recurrence_of, lessons_learned, action_items,
    external_ref, notes, created_at, updated_at, deleted_at
FROM incidents;

-- Step 3: Drop old table and rename new
DROP TABLE incidents;

ALTER TABLE incidents_new RENAME TO incidents;

-- Step 4: Recreate all indexes
CREATE INDEX idx_incidents_service_id ON incidents(service_id);
CREATE INDEX idx_incidents_severity ON incidents(severity);
CREATE INDEX idx_incidents_impact ON incidents(impact);
CREATE INDEX idx_incidents_status ON incidents(status);
CREATE INDEX idx_incidents_started_at ON incidents(started_at);
CREATE INDEX idx_incidents_resolved_at ON incidents(resolved_at);
CREATE INDEX idx_incidents_recurrence_of ON incidents(recurrence_of);
CREATE INDEX idx_incidents_deleted_at ON incidents(deleted_at);

PRAGMA foreign_keys = ON;

PRAGMA integrity_check
