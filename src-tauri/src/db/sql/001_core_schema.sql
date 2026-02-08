-- services table
CREATE TABLE IF NOT EXISTS services (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL CHECK (category IN (
        'Communication', 'Infrastructure', 'Development',
        'Productivity', 'Security', 'Other'
    )),
    default_severity TEXT NOT NULL CHECK (default_severity IN ('Critical', 'High', 'Medium', 'Low')),
    default_impact TEXT NOT NULL CHECK (default_impact IN ('Critical', 'High', 'Medium', 'Low')),
    description TEXT DEFAULT '',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- incidents table
CREATE TABLE IF NOT EXISTS incidents (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    service_id TEXT NOT NULL REFERENCES services(id),
    severity TEXT NOT NULL CHECK (severity IN ('Critical', 'High', 'Medium', 'Low')),
    impact TEXT NOT NULL CHECK (impact IN ('Critical', 'High', 'Medium', 'Low')),
    status TEXT NOT NULL CHECK (status IN ('Active', 'Monitoring', 'Resolved', 'Post-Mortem')),
    started_at TEXT NOT NULL,
    detected_at TEXT NOT NULL,
    responded_at TEXT,
    resolved_at TEXT,
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
    recurrence_of TEXT REFERENCES incidents(id),
    lessons_learned TEXT DEFAULT '',
    action_items TEXT DEFAULT '',
    external_ref TEXT DEFAULT '',
    notes TEXT DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- action_items table
CREATE TABLE IF NOT EXISTS action_items (
    id TEXT PRIMARY KEY NOT NULL,
    incident_id TEXT NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT DEFAULT '',
    status TEXT NOT NULL CHECK (status IN ('Open', 'In-Progress', 'Done')) DEFAULT 'Open',
    owner TEXT DEFAULT '',
    due_date TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- quarter_config table
CREATE TABLE IF NOT EXISTS quarter_config (
    id TEXT PRIMARY KEY NOT NULL,
    fiscal_year INTEGER NOT NULL,
    quarter_number INTEGER NOT NULL CHECK (quarter_number BETWEEN 1 AND 4),
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    label TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(fiscal_year, quarter_number)
);

-- import_templates table
CREATE TABLE IF NOT EXISTS import_templates (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    column_mapping TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- app_settings table
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_incidents_service_id ON incidents(service_id);
CREATE INDEX IF NOT EXISTS idx_incidents_severity ON incidents(severity);
CREATE INDEX IF NOT EXISTS idx_incidents_impact ON incidents(impact);
CREATE INDEX IF NOT EXISTS idx_incidents_status ON incidents(status);
CREATE INDEX IF NOT EXISTS idx_incidents_started_at ON incidents(started_at);
CREATE INDEX IF NOT EXISTS idx_incidents_resolved_at ON incidents(resolved_at);
CREATE INDEX IF NOT EXISTS idx_incidents_recurrence_of ON incidents(recurrence_of);
CREATE INDEX IF NOT EXISTS idx_action_items_incident_id ON action_items(incident_id);
CREATE INDEX IF NOT EXISTS idx_action_items_status ON action_items(status);
