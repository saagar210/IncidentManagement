-- Action item follow-through fields (PIR outcomes)
-- Adds completion timestamp, outcome notes, and an explicit validation marker.

ALTER TABLE action_items
  ADD COLUMN completed_at TEXT;

ALTER TABLE action_items
  ADD COLUMN outcome_notes TEXT NOT NULL DEFAULT '';

ALTER TABLE action_items
  ADD COLUMN validated_at TEXT;

