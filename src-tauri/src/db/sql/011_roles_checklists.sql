-- Migration 011: Incident Roles & Checklists

CREATE TABLE incident_roles (
    id TEXT PRIMARY KEY,
    incident_id TEXT NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK(role IN ('Incident Commander', 'Communications Lead', 'Technical Lead', 'Scribe', 'SME')),
    assignee TEXT NOT NULL,
    is_primary INTEGER NOT NULL DEFAULT 1,
    assigned_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    unassigned_at TEXT,
    UNIQUE(incident_id, role, assignee)
);

CREATE INDEX idx_incident_roles_incident ON incident_roles(incident_id);

CREATE TABLE checklist_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    service_id TEXT REFERENCES services(id) ON DELETE SET NULL,
    incident_type TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE checklist_template_items (
    id TEXT PRIMARY KEY,
    template_id TEXT NOT NULL REFERENCES checklist_templates(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_checklist_template_items_template ON checklist_template_items(template_id);

CREATE TABLE incident_checklists (
    id TEXT PRIMARY KEY,
    incident_id TEXT NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    template_id TEXT REFERENCES checklist_templates(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_incident_checklists_incident ON incident_checklists(incident_id);

CREATE TABLE checklist_items (
    id TEXT PRIMARY KEY,
    checklist_id TEXT NOT NULL REFERENCES incident_checklists(id) ON DELETE CASCADE,
    template_item_id TEXT REFERENCES checklist_template_items(id) ON DELETE SET NULL,
    label TEXT NOT NULL,
    is_checked INTEGER NOT NULL DEFAULT 0,
    checked_at TEXT,
    checked_by TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_checklist_items_checklist ON checklist_items(checklist_id)
