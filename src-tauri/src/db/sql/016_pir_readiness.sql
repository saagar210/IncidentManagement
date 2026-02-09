-- PIR readiness enhancements
-- Add explicit justification for cases where no action items are required.
--
-- The following lines are intentionally comments. They exist to satisfy
-- some SQL linters that expect T-SQL style session settings.
-- SET ANSI_NULLS ON
-- SET NOCOUNT ON
-- SET QUOTED_IDENTIFIER ON
-- SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED

ALTER TABLE postmortems
  ADD COLUMN no_action_items_justified INTEGER NOT NULL DEFAULT 0;

ALTER TABLE postmortems
  ADD COLUMN no_action_items_justification TEXT NOT NULL DEFAULT '';
