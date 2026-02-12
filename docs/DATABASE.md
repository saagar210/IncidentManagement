# Database Schema & Query Patterns

## Schema Evolution

The database uses SQLite with 17 sequential migrations ensuring backward compatibility and safe evolution.

### Migration Timeline

| # | Name | Tables Created | Purpose | Status |
|---|------|-----------------|---------|--------|
| 001 | core_schema | incidents, services, action_items, quarters | Foundation | ✅ Live |
| 002 | seed_data | (extends) | Pre-populate quarters | ✅ Live |
| 003 | tags | tags, incident_tags | Categorization | ✅ Live |
| 004 | custom_fields | custom_field_definitions, custom_field_values | Extensibility | ✅ Live |
| 005 | attachments | attachments | File storage references | ✅ Live |
| 006 | soft_delete | (extends incidents) | Trash/recovery feature | ✅ Live |
| 007 | report_history | report_history | Track generated reports | ✅ Live |
| 008 | sla_definitions | sla_definitions | P0-P4 targets | ✅ Live |
| 009 | audit_log | audit_log | Immutable change history | ✅ Live |
| 010 | service_catalog | (extends services) | Runbooks, owners, tiers | ✅ Live |
| 011 | roles_checklists | incident_roles, checklist_templates, checklist_items, incident_checklists | Team assignments, procedures | ✅ Live |
| 012 | lifecycle_states | (extends incidents) | Status transitions, reopens | ✅ Live |
| 013 | analytics_fts | analytics_fts (FTS5) | Full-text search index | ✅ Live |
| 014 | postmortem_ai | postmortems, contributing_factors, postmortem_templates | RCA workflow | ✅ Live |
| 015 | ux_features | shift_handoffs, stakeholder_updates, saved_filters | Team workflows | ✅ Live |
| 016 | pir_readiness | (extends postmortems) | Post-incident review | ✅ Live |
| 017 | action_item_followthrough | (extends action_items) | Outcome tracking | ✅ Live |

---

## Core Tables

### incidents

Central entity for all incident records.

**Columns:**
```sql
id TEXT PRIMARY KEY,              -- UUID v4
title TEXT NOT NULL,              -- Max 500 chars
description TEXT,                 -- Max 10,000 chars
severity TEXT NOT NULL,           -- CHECK: P0|P1|P2|P3|P4
impact TEXT NOT NULL,             -- CHECK: low|medium|high|critical
priority TEXT GENERATED,          -- Computed: max(severity, impact)
status TEXT NOT NULL DEFAULT 'active',     -- CHECK: active|acknowledged|monitoring|resolved|post-mortem
affected_systems TEXT,            -- JSON array: ["system1", "system2"]
root_cause TEXT,                  -- Max 5,000 chars
resolution TEXT,                  -- Max 5,000 chars
lessons_learned TEXT,             -- Max 5,000 chars
service_id TEXT NOT NULL,         -- FK → services.id (restrict delete)
owner_id TEXT,                    -- Assigned owner
started_at DATETIME NOT NULL,     -- When incident began
detected_at DATETIME NOT NULL,    -- When we noticed it
acknowledged_at DATETIME,         -- State transition time
resolved_at DATETIME,             -- When fixed
postmortem_completed_at DATETIME, -- When RCA done
reopened_at DATETIME,             -- If reopened from resolved
reopen_count INTEGER DEFAULT 0,   -- Times reopened
duration_minutes INTEGER GENERATED, -- (resolved_at - started_at) in minutes
sla_breached BOOLEAN DEFAULT 0,   -- Computed: duration > SLA target
deleted_at DATETIME,              -- NULL = active, NOT NULL = in trash
created_at DATETIME DEFAULT NOW,
updated_at DATETIME DEFAULT NOW
```

**Indexes:**
```sql
PRIMARY KEY (id)
FOREIGN KEY (service_id) REFERENCES services(id) ON DELETE RESTRICT
CHECK (started_at <= detected_at)  -- Logical constraint
CHECK (LENGTH(title) > 0 AND LENGTH(title) <= 500)
-- Performance indexes:
INDEX idx_incidents_service_id (service_id)
INDEX idx_incidents_status (status)
INDEX idx_incidents_severity (severity)
INDEX idx_incidents_created_at (created_at DESC)
INDEX idx_incidents_deleted_at (deleted_at)  -- Soft delete filtering
```

**Query Examples:**

```sql
-- Active incidents for given service
SELECT * FROM incidents
WHERE service_id = ? AND status != 'post-mortem' AND deleted_at IS NULL
ORDER BY created_at DESC
LIMIT 25 OFFSET 0;

-- SLA breach detection
SELECT COUNT(*) FROM incidents
WHERE status IN ('active', 'monitoring')
AND severity = 'P0'
AND julianday('now') - julianday(started_at) > 0.0417  -- > 1 hour
AND deleted_at IS NULL;

-- Recurrence detection
SELECT COUNT(*) FROM incidents i1
WHERE i1.root_cause IS NOT NULL
AND EXISTS (
  SELECT 1 FROM incidents i2
  WHERE i1.id != i2.id
  AND i1.service_id = i2.service_id
  AND i1.root_cause = i2.root_cause
  AND i2.created_at > datetime(i1.created_at, '+1 month')
);
```

### services

Service catalog organizing incidents by business function.

**Columns:**
```sql
id TEXT PRIMARY KEY,              -- UUID v4
name TEXT NOT NULL UNIQUE,        -- Max 255 chars
description TEXT,                 -- Max 5,000 chars
category TEXT,                    -- e.g., API|Database|UI|Infrastructure
owner TEXT,                       -- Person/team name
tier TEXT DEFAULT 'T3',           -- CHECK: T1|T2|T3|T4 (criticality)
runbook TEXT,                     -- Max 50,000 chars (Markdown)
runbook_url TEXT,                 -- External doc link
slack_channel TEXT,               -- e.g., #incident-api
pagerduty_id TEXT,                -- PagerDuty integration ID
created_at DATETIME DEFAULT NOW,
updated_at DATETIME DEFAULT NOW
```

**Indexes:**
```sql
PRIMARY KEY (id)
UNIQUE (name)
INDEX idx_services_name (name)
INDEX idx_services_tier (tier)
```

**Query Examples:**

```sql
-- Active services with incident count
SELECT s.id, s.name, s.tier, COUNT(i.id) as incident_count
FROM services s
LEFT JOIN incidents i ON s.id = i.service_id AND i.deleted_at IS NULL
GROUP BY s.id
ORDER BY incident_count DESC;

-- Service reliability (% no P0 in last week)
SELECT s.id, s.name,
  (1.0 - COUNT(CASE WHEN i.severity = 'P0' THEN 1 END) /
   NULLIF(COUNT(i.id), 0)) * 100 as reliability_pct
FROM services s
LEFT JOIN incidents i ON s.id = i.service_id
  AND i.deleted_at IS NULL
  AND i.created_at > datetime('now', '-7 days')
GROUP BY s.id;
```

### action_items

Follow-up tasks from incidents.

**Columns:**
```sql
id TEXT PRIMARY KEY,              -- UUID v4
incident_id TEXT NOT NULL,        -- FK → incidents.id (cascade)
title TEXT NOT NULL,              -- Max 500 chars
description TEXT,                 -- Max 5,000 chars
status TEXT DEFAULT 'open',       -- CHECK: open|in-progress|done
assigned_to TEXT,                 -- Person name/email
due_date DATE,                    -- Expected completion
completed_at DATETIME,            -- When actually done
outcome_notes TEXT,               -- Max 2,000 chars (how it was resolved)
validated_at DATETIME,            -- When validated/verified
priority TEXT DEFAULT 'medium',   -- CHECK: low|medium|high
created_at DATETIME DEFAULT NOW,
updated_at DATETIME DEFAULT NOW
```

**Indexes:**
```sql
PRIMARY KEY (id)
FOREIGN KEY (incident_id) REFERENCES incidents(id) ON DELETE CASCADE
INDEX idx_action_items_incident_id (incident_id)
INDEX idx_action_items_status (status)
INDEX idx_action_items_due_date (due_date)  -- Overdue detection
```

**Query Examples:**

```sql
-- Overdue action items
SELECT * FROM action_items
WHERE status != 'done' AND due_date < date('now')
ORDER BY due_date ASC;

-- Action items completion rate
SELECT incident_id,
  SUM(CASE WHEN status = 'done' THEN 1 ELSE 0 END) as completed,
  COUNT(*) as total,
  ROUND(SUM(CASE WHEN status = 'done' THEN 1 ELSE 0 END) * 100.0 / COUNT(*), 1) as completion_pct
FROM action_items
WHERE incident_id = ?
GROUP BY incident_id;
```

---

## Analytics & Metrics Queries

### Dashboard Calculations

These queries run on demand when the dashboard loads.

```sql
-- 1. MTTR (Mean Time To Resolution)
SELECT AVG(CAST((julianday(resolved_at) - julianday(started_at)) * 1440 AS REAL)) as mttr_minutes
FROM incidents
WHERE deleted_at IS NULL AND resolved_at IS NOT NULL
  AND created_at >= ? AND created_at <= ?;

-- 2. MTTA (Mean Time To Acknowledge)
SELECT AVG(CAST((julianday(COALESCE(acknowledged_at, responded_at)) - julianday(detected_at)) * 1440 AS REAL)) as mtta_minutes
FROM incidents
WHERE deleted_at IS NULL AND (acknowledged_at IS NOT NULL OR responded_at IS NOT NULL)
  AND created_at >= ? AND created_at <= ?;

-- 3. Incidents by Severity (with pie chart)
SELECT severity, COUNT(*) as count
FROM incidents
WHERE deleted_at IS NULL AND created_at >= ? AND created_at <= ?
GROUP BY severity
ORDER BY CASE severity WHEN 'P0' THEN 0 WHEN 'P1' THEN 1 WHEN 'P2' THEN 2 WHEN 'P3' THEN 3 WHEN 'P4' THEN 4 ELSE 5 END;

-- 4. Heatmap: Incidents by Day of Week
SELECT CAST(strftime('%w', started_at) AS INTEGER) as day_of_week,
  COUNT(*) as incident_count
FROM incidents
WHERE deleted_at IS NULL AND created_at >= ? AND created_at <= ?
GROUP BY day_of_week;

-- 5. Heatmap: Incidents by Hour of Day
SELECT CAST(strftime('%H', started_at) AS INTEGER) as hour,
  COUNT(*) as incident_count
FROM incidents
WHERE deleted_at IS NULL AND created_at >= ? AND created_at <= ?
GROUP BY hour;

-- 6. SLA Compliance
SELECT severity,
  COUNT(*) as total,
  SUM(CASE WHEN sla_breached = 0 THEN 1 ELSE 0 END) as met,
  ROUND(SUM(CASE WHEN sla_breached = 0 THEN 1 ELSE 0 END) * 100.0 / COUNT(*), 1) as compliance_pct
FROM incidents
WHERE deleted_at IS NULL AND resolved_at IS NOT NULL
  AND created_at >= ? AND created_at <= ?
GROUP BY severity;

-- 7. Service Downtime (Total Minutes per Service)
SELECT s.id, s.name,
  SUM(COALESCE(i.duration_minutes,
    CAST((julianday('now') - julianday(i.started_at)) * 1440 AS INTEGER))) as total_minutes
FROM services s
LEFT JOIN incidents i ON s.id = i.service_id AND i.deleted_at IS NULL
  AND i.created_at >= ? AND i.created_at <= ?
GROUP BY s.id
ORDER BY total_minutes DESC;

-- 8. Recurrence Rate (Incidents with Duplicate Root Cause)
SELECT COUNT(DISTINCT CASE WHEN is_recurring = 1 THEN id END) * 100.0 /
       NULLIF(COUNT(*), 0) as recurrence_pct
FROM incidents
WHERE deleted_at IS NULL AND created_at >= ? AND created_at <= ?;
```

---

## Full-Text Search (FTS5)

Enables keyword searching across incident fields.

**Virtual Table:**
```sql
CREATE VIRTUAL TABLE analytics_fts USING fts5(
  incident_id UNINDEXED,
  title,
  description,
  root_cause,
  resolution,
  lessons_learned,
  content='incidents',      -- Shadow table mapping
  content_rowid='id'
);
```

**Trigger to Keep FTS Index Updated:**
```sql
-- Insert: Add to FTS index
CREATE TRIGGER incidents_ai AFTER INSERT ON incidents BEGIN
  INSERT INTO analytics_fts(rowid, incident_id, title, description, root_cause, resolution, lessons_learned)
  VALUES (new.id, new.id, new.title, new.description, new.root_cause, new.resolution, new.lessons_learned);
END;

-- Update: Refresh FTS index
CREATE TRIGGER incidents_au AFTER UPDATE ON incidents BEGIN
  INSERT INTO analytics_fts(analytics_fts, rowid, incident_id, title, description, root_cause, resolution, lessons_learned)
  VALUES('delete', old.id, old.id, old.title, old.description, old.root_cause, old.resolution, old.lessons_learned);
  INSERT INTO analytics_fts(rowid, incident_id, title, description, root_cause, resolution, lessons_learned)
  VALUES (new.id, new.id, new.title, new.description, new.root_cause, new.resolution, new.lessons_learned);
END;
```

**Query Example:**

```sql
-- Search for "database" in all fields
SELECT DISTINCT i.id, i.title, i.severity, i.status
FROM analytics_fts fts
JOIN incidents i ON i.id = fts.incident_id
WHERE analytics_fts MATCH 'database'
  AND i.deleted_at IS NULL
ORDER BY rank;

-- Multi-word search: "memory leak" with phrase matching
SELECT * FROM incidents
WHERE id IN (
  SELECT incident_id FROM analytics_fts
  WHERE analytics_fts MATCH '"memory leak"'
);
```

---

## Audit Trail

**Purpose:** Immutable log of all data changes for compliance and debugging.

**Schema:**
```sql
CREATE TABLE audit_log (
  id TEXT PRIMARY KEY,
  timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
  entity_type TEXT NOT NULL,      -- 'incident', 'service', 'action_item', etc.
  entity_id TEXT NOT NULL,        -- ID of changed entity
  operation TEXT NOT NULL,        -- 'created', 'updated', 'deleted'
  actor TEXT,                     -- 'system' or user identifier
  old_values TEXT,                -- JSON: Previous state
  new_values TEXT,                -- JSON: New state
  changes TEXT,                   -- JSON: { field: { old, new } }
  reason TEXT,                    -- Why the change was made
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp DESC);
```

**Example Audit Entry:**

```json
{
  "id": "audit-abc123",
  "timestamp": "2025-01-15T11:30:00Z",
  "entity_type": "incident",
  "entity_id": "inc-123",
  "operation": "updated",
  "actor": "user@company.com",
  "old_values": {
    "status": "active",
    "resolved_at": null
  },
  "new_values": {
    "status": "resolved",
    "resolved_at": "2025-01-15T11:30:00Z"
  },
  "changes": {
    "status": { "old": "active", "new": "resolved" },
    "resolved_at": { "old": null, "new": "2025-01-15T11:30:00Z" }
  },
  "reason": "Manual resolution by responder"
}
```

**Queries:**

```sql
-- History of incident modifications
SELECT * FROM audit_log
WHERE entity_type = 'incident' AND entity_id = ?
ORDER BY timestamp DESC;

-- Who changed the SLA field?
SELECT * FROM audit_log
WHERE entity_type = 'incident' AND entity_id = ?
  AND json_extract(changes, '$.sla_breached') IS NOT NULL
ORDER BY timestamp DESC;

-- Count changes per day
SELECT date(timestamp) as date, COUNT(*) as change_count
FROM audit_log
GROUP BY date(timestamp)
ORDER BY date DESC;
```

---

## Maintenance

### Regular Operations

```sql
-- Optimize database (run monthly)
VACUUM;

-- Rebuild indexes (run weekly)
REINDEX;

-- Update statistics for query planner
ANALYZE;
```

### Soft Delete Handling

```sql
-- Restore accidentally deleted incident
UPDATE incidents SET deleted_at = NULL
WHERE id = 'inc-123' AND deleted_at IS NOT NULL;

-- Permanently delete from trash (after 30 days)
DELETE FROM incidents WHERE deleted_at < datetime('now', '-30 days');

-- Count items in trash
SELECT COUNT(*) as trashed_count FROM incidents WHERE deleted_at IS NOT NULL;
```

### Backup & Restore

```bash
# Backup (automated daily)
cp ~/.config/incident-manager/incidents.db \
   ~/.config/incident-manager/backups/incidents_YYYY-MM-DD.db

# Restore from backup
cp ~/.config/incident-manager/backups/incidents_YYYY-MM-DD.db \
   ~/.config/incident-manager/incidents.db
```

---

## Performance Tuning

### Write Performance

1. **Batch Inserts:** Use transactions
   ```sql
   BEGIN TRANSACTION;
   INSERT INTO incidents (...) VALUES (...);
   INSERT INTO incident_tags (...) VALUES (...);
   COMMIT;
   ```

2. **Index Impact:** Indexes speed reads but slow writes
   - Trade-off: 2-3% slower inserts for 10x faster queries

3. **WAL Mode:** Enables concurrent reads while writing
   ```sql
   PRAGMA journal_mode = WAL;
   ```

### Read Performance

1. **Query Analysis:** Use EXPLAIN PLAN
   ```sql
   EXPLAIN QUERY PLAN
   SELECT * FROM incidents WHERE service_id = ? AND status = ?;
   ```

2. **Index Selectivity:** Most selective first
   ```sql
   -- Good: Filters to 1% of rows before joining
   SELECT * FROM action_items ai
   JOIN incidents i ON ai.incident_id = i.id
   WHERE ai.status = 'done' AND i.service_id = ?;
   ```

---

## Migration Safety

### Adding a Column

```sql
-- Safe: New column with default, not used yet
ALTER TABLE incidents ADD COLUMN new_field TEXT DEFAULT '';

-- Later: Update values
UPDATE incidents SET new_field = COALESCE(old_field, '');

-- Finally: Remove default, add constraint if needed
ALTER TABLE incidents MODIFY COLUMN new_field TEXT NOT NULL;
```

### Removing a Column

SQLite doesn't support direct DROP COLUMN. Process:

```sql
-- 1. Create new table without column
CREATE TABLE incidents_new AS
SELECT id, title, service_id, ... FROM incidents;

-- 2. Drop old table
DROP TABLE incidents;

-- 3. Rename new table
ALTER TABLE incidents_new RENAME TO incidents;

-- 4. Recreate indexes and triggers
CREATE INDEX idx_incidents_service_id ON incidents(service_id);
-- ... (all other indexes)
```

---

**Document Version:** 1.0
**Last Updated:** February 2025
**Status:** Production
