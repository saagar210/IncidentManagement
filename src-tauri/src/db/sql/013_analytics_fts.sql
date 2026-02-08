-- Migration 013: Analytics & Full-Text Search
-- Adds saved filter presets and FTS5 virtual table for full-text search

CREATE TABLE IF NOT EXISTS saved_filters (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    filters TEXT NOT NULL DEFAULT '{}',
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- FTS5 virtual table for full-text search across incidents
CREATE VIRTUAL TABLE IF NOT EXISTS incidents_fts USING fts5(
    title,
    root_cause,
    resolution,
    lessons_learned,
    notes,
    content=incidents,
    content_rowid=rowid
);

-- Backfill existing data into FTS5
INSERT INTO incidents_fts(rowid, title, root_cause, resolution, lessons_learned, notes)
    SELECT rowid, title,
        COALESCE(root_cause, ''),
        COALESCE(resolution, ''),
        COALESCE(lessons_learned, ''),
        COALESCE(notes, '')
    FROM incidents;

-- Triggers to keep FTS5 in sync with incidents table
CREATE TRIGGER incidents_fts_insert AFTER INSERT ON incidents BEGIN
    INSERT INTO incidents_fts(rowid, title, root_cause, resolution, lessons_learned, notes)
    VALUES (new.rowid, new.title, COALESCE(new.root_cause, ''), COALESCE(new.resolution, ''), COALESCE(new.lessons_learned, ''), COALESCE(new.notes, ''));
END;

CREATE TRIGGER incidents_fts_update AFTER UPDATE ON incidents BEGIN
    INSERT INTO incidents_fts(incidents_fts, rowid, title, root_cause, resolution, lessons_learned, notes)
    VALUES ('delete', old.rowid, old.title, COALESCE(old.root_cause, ''), COALESCE(old.resolution, ''), COALESCE(old.lessons_learned, ''), COALESCE(old.notes, ''));
    INSERT INTO incidents_fts(rowid, title, root_cause, resolution, lessons_learned, notes)
    VALUES (new.rowid, new.title, COALESCE(new.root_cause, ''), COALESCE(new.resolution, ''), COALESCE(new.lessons_learned, ''), COALESCE(new.notes, ''));
END;

CREATE TRIGGER incidents_fts_delete AFTER DELETE ON incidents BEGIN
    INSERT INTO incidents_fts(incidents_fts, rowid, title, root_cause, resolution, lessons_learned, notes)
    VALUES ('delete', old.rowid, old.title, COALESCE(old.root_cause, ''), COALESCE(old.resolution, ''), COALESCE(old.lessons_learned, ''), COALESCE(old.notes, ''));
END;
