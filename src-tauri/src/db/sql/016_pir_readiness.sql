-- PIR readiness enhancements
-- Add explicit justification for cases where no action items are required.

ALTER TABLE postmortems
  ADD COLUMN no_action_items_justified INTEGER NOT NULL DEFAULT 0;

ALTER TABLE postmortems
  ADD COLUMN no_action_items_justification TEXT NOT NULL DEFAULT '';

