ALTER TABLE slot ADD COLUMN week_parity TEXT CHECK (week_parity IN ('odd','even'));  -- NULL = every week
