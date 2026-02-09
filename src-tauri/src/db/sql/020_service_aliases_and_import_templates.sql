-- Migration 020: Service aliases + import template metadata

CREATE TABLE IF NOT EXISTS service_aliases (
    id TEXT PRIMARY KEY NOT NULL,
    alias TEXT NOT NULL COLLATE NOCASE,
    service_id TEXT NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(alias)
);

CREATE INDEX IF NOT EXISTS idx_service_aliases_service ON service_aliases(service_id);

-- Import template metadata: support source tagging and schema versioning for safer reuse.
ALTER TABLE import_templates ADD COLUMN source TEXT NOT NULL DEFAULT 'generic';
ALTER TABLE import_templates ADD COLUMN schema_version INTEGER NOT NULL DEFAULT 1;

