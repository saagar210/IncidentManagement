-- Migration 019: Quarter finalization (freeze snapshot + overrides)

CREATE TABLE IF NOT EXISTS quarter_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    quarter_id TEXT NOT NULL REFERENCES quarter_config(id) ON DELETE CASCADE,
    schema_version INTEGER NOT NULL DEFAULT 1,
    inputs_hash TEXT NOT NULL,
    snapshot_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(quarter_id)
);

CREATE TABLE IF NOT EXISTS quarter_readiness_overrides (
    id TEXT PRIMARY KEY NOT NULL,
    quarter_id TEXT NOT NULL REFERENCES quarter_config(id) ON DELETE CASCADE,
    rule_key TEXT NOT NULL,
    incident_id TEXT NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    reason TEXT NOT NULL,
    approved_by TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(quarter_id, rule_key, incident_id)
);

CREATE INDEX IF NOT EXISTS idx_quarter_overrides_quarter ON quarter_readiness_overrides(quarter_id);
CREATE INDEX IF NOT EXISTS idx_quarter_overrides_incident ON quarter_readiness_overrides(incident_id);

CREATE TABLE IF NOT EXISTS quarter_finalizations (
    quarter_id TEXT PRIMARY KEY NOT NULL REFERENCES quarter_config(id) ON DELETE CASCADE,
    finalized_at TEXT NOT NULL,
    finalized_by TEXT NOT NULL,
    snapshot_id TEXT NOT NULL REFERENCES quarter_snapshots(id) ON DELETE RESTRICT,
    inputs_hash TEXT NOT NULL,
    notes TEXT NOT NULL DEFAULT ''
);

