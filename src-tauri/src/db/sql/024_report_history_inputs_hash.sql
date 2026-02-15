-- Migration 024: Report history inputs hash and versioning for repeatability

ALTER TABLE report_history ADD COLUMN inputs_hash TEXT NOT NULL DEFAULT '';
ALTER TABLE report_history ADD COLUMN report_version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE report_history ADD COLUMN quarter_finalized_at TEXT;

