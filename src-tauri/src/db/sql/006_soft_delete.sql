ALTER TABLE incidents ADD COLUMN deleted_at TEXT DEFAULT NULL;
CREATE INDEX idx_incidents_deleted_at ON incidents(deleted_at)