-- Migration 018: Index detected_at for quarter inclusion performance

CREATE INDEX IF NOT EXISTS idx_incidents_detected_at ON incidents(detected_at);
