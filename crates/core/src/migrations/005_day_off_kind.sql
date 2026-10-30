-- M15 (R71, R72): a day off has a kind. Every kind skips classes; rows that exist before this
-- migration stay 'holiday' through the column default.
-- This is migration 004 of docs/spec.md §13.10, renumbered to the next free number (D1).
ALTER TABLE holiday ADD COLUMN kind TEXT NOT NULL DEFAULT 'holiday'
  CHECK (kind IN ('holiday','midterm','final'));
