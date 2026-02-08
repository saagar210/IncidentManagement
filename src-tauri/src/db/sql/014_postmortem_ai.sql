-- Contributing factors (normalized from root_cause)
CREATE TABLE IF NOT EXISTS contributing_factors (
    id TEXT PRIMARY KEY,
    incident_id TEXT NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    category TEXT NOT NULL CHECK(category IN ('Process', 'Tooling', 'Communication', 'Human Factors', 'External')),
    description TEXT NOT NULL DEFAULT '',
    is_root INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_contributing_factors_incident ON contributing_factors(incident_id);

-- Post-mortem templates
CREATE TABLE IF NOT EXISTS postmortem_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    incident_type TEXT NOT NULL CHECK(incident_type IN ('Outage', 'Degradation', 'Security', 'General')),
    template_content TEXT NOT NULL DEFAULT '{}',
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Post-mortem documents
CREATE TABLE IF NOT EXISTS postmortems (
    id TEXT PRIMARY KEY,
    incident_id TEXT NOT NULL UNIQUE REFERENCES incidents(id) ON DELETE CASCADE,
    template_id TEXT REFERENCES postmortem_templates(id) ON DELETE SET NULL,
    content TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft', 'review', 'final')),
    reminder_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_postmortems_incident ON postmortems(incident_id);
CREATE INDEX IF NOT EXISTS idx_postmortems_status ON postmortems(status);

-- Seed default templates
INSERT OR IGNORE INTO postmortem_templates (id, name, incident_type, template_content, is_default)
VALUES
('pmt-general', 'General Post-Mortem', 'General', '{"sections":["Summary","Impact","Timeline","Root Cause Analysis","Contributing Factors","Action Items","Lessons Learned"]}', 1),
('pmt-security', 'Security Incident Post-Mortem', 'Security', '{"sections":["Summary","Impact","Attack Vector","Timeline","Containment Actions","Root Cause Analysis","Contributing Factors","Remediation Steps","Action Items","Lessons Learned"]}', 0);

-- Migrate existing root_cause data into contributing_factors
INSERT INTO contributing_factors (id, incident_id, category, description, is_root)
SELECT
    'cf-' || hex(randomblob(8)),
    id,
    'Process',
    root_cause,
    1
FROM incidents
WHERE root_cause IS NOT NULL AND root_cause != '' AND deleted_at IS NULL
