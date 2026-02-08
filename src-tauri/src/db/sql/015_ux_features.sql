-- Migration 015: UX Features — stakeholder updates, shift handoffs

CREATE TABLE IF NOT EXISTS stakeholder_updates (
    id TEXT PRIMARY KEY NOT NULL,
    incident_id TEXT NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    content TEXT NOT NULL DEFAULT '',
    update_type TEXT NOT NULL DEFAULT 'status' CHECK(update_type IN ('status', 'initial', 'final', 'custom')),
    generated_by TEXT NOT NULL DEFAULT 'manual' CHECK(generated_by IN ('manual', 'template', 'ai')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_stakeholder_updates_incident ON stakeholder_updates(incident_id);

CREATE TABLE IF NOT EXISTS shift_handoffs (
    id TEXT PRIMARY KEY NOT NULL,
    shift_end_time TEXT DEFAULT (datetime('now')),
    content TEXT NOT NULL DEFAULT '{}',
    created_by TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
