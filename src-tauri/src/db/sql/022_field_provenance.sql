-- Migration 022: Field-level provenance ("where did this come from?")

CREATE TABLE IF NOT EXISTS field_provenance (
    id TEXT PRIMARY KEY NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    field_name TEXT NOT NULL,
    source_type TEXT NOT NULL CHECK(source_type IN ('manual', 'import', 'computed', 'ai')),
    source_ref TEXT NOT NULL DEFAULT '',
    source_version TEXT NOT NULL DEFAULT '',
    input_hash TEXT NOT NULL DEFAULT '',
    meta_json TEXT NOT NULL DEFAULT '{}',
    recorded_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_field_provenance_entity ON field_provenance(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_field_provenance_field ON field_provenance(entity_type, field_name);
CREATE INDEX IF NOT EXISTS idx_field_provenance_recorded_at ON field_provenance(recorded_at DESC);

