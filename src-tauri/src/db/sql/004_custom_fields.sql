CREATE TABLE custom_field_definitions (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    field_type TEXT NOT NULL CHECK (field_type IN ('text', 'number', 'select')),
    options TEXT DEFAULT '',
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE TABLE custom_field_values (
    incident_id TEXT NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    field_id TEXT NOT NULL REFERENCES custom_field_definitions(id) ON DELETE CASCADE,
    value TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (incident_id, field_id)
)