-- M15 (R71, R72): a day off has a kind. Every kind skips classes; rows that exist before this
-- migration stay 'holiday' through the column default.
-- This is spec migration 004 of docs/spec.md §13.10, renumbered to 005 after M12 and M19 took 003 and 004 (D1).
ALTER TABLE holiday ADD COLUMN kind TEXT NOT NULL DEFAULT 'holiday'
  CHECK (kind IN ('holiday','midterm','final'));
