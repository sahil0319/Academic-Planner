# Research: M3 Backup, Restore, and CSV Export

All technology choices are fixed by `docs/spec.md` §1, and the rules by §4. This file records only
the M3 design decisions that §3–§5 and spec.md leave open. No item re-decides §1 or changes a rule,
the schema, or the API. R-D2, R-D3, R-D4, R-D5, R-D8, and R-D9 are router decisions of 2026-10-29.

## R-D1. Dependencies

- **Decision**: add `serde_json.workspace = true` and `csv.workspace = true` to
  `crates/core/Cargo.toml`. Both already exist in root `[workspace.dependencies]`; `serde` (with
  `derive`) is already a core dependency.
- **Rationale**: §1 fixes "Serialization: `serde`, `serde_json`, `csv`". Typed serde row structs give
  free type checking on restore ("well-typed collections").
- **Alternatives**: hand-built JSON via `serde_json::Value` — rejected, no type checking; hand-rolled
  CSV quoting — rejected, §1 names `csv`.

## R-D2. `exported_at` format

- **Decision**: `fmt_datetime(now)`, i.e. the stored format `YYYY-MM-DDTHH:MM`.
- **Rationale**: one date-time format everywhere (§1); the existing helper is reused.
- **Alternatives**: RFC 3339 with seconds — rejected, §1 forbids time zones and stores minutes only.

## R-D3. JSON value types

- **Decision**: one object per row; keys are the §3 column names. INTEGER flags (`term.is_active`,
  `course.archived`, `assessment.completed`) are JSON booleans; nullable columns are JSON `null`;
  dates, times, and date-times are the stored strings; all other INTEGER columns stay numbers;
  `settings` is an object mapping key to string value. Collections are ordered by `id`;
  `holidays` by `(term_id, date)`; `settings` by key. Top-level key order: `format`,
  `schema_version`, `exported_at`, `terms`, `holidays`, `courses`, `slots`, `sessions`,
  `assessments`, `settings`.
- **Rationale**: matches the R21 shape literally and is stable (FR-004), so T30 can compare
  documents after dropping `exported_at`.
- **Alternatives**: flags as 0/1 — rejected, booleans are what a reader of the file expects and
  restore maps them back to 0/1 exactly.

## R-D4. Restore validation order, all before touching the database

- **Decision**: `restore_json` (1) parses the text with `serde_json::from_str::<Value>`; invalid
  JSON or a non-object root is `Error::Validation`; (2) checks `format` equals
  `"academic-planner-backup"` (missing or other value is `Error::Validation`); (3) checks
  `schema_version` is present, is an integer, and is `<= SCHEMA_VERSION` (missing, non-integer, or
  greater is `Error::Validation`; equal or less is accepted); (4) deserializes the whole value into
  the private `Backup` struct, which requires all seven collections present and every row
  well-typed (a missing or ill-typed collection or field is `Error::Validation`); (5) checks that
  every date, time, and date-time text value parses with the stored-format helpers
  (`parse_date`, `parse_time`, `parse_datetime`); a bad value is `Error::Validation`. Only then does
  it open the transaction. Unknown extra keys are ignored. `exported_at` must be present as a string; its content is not parsed
  and is not stored.
- **Rationale**: spec FR-005, FR-006, FR-008. Step 5 keeps text the app could not read back
  (`Error::Corrupt` on later reads) out of the database, which is part of "on any error, the DB is
  unchanged" and "well-typed". Format is checked before version so a non-backup file never reports a
  version error.
- **Alternatives**: rely on SQLite constraints only — rejected, SQLite stores any text in a TEXT
  column, so a malformed date would restore "successfully" and fail later.

## R-D5. Restore transaction

- **Decision**: one `rusqlite` transaction. Inside it: `PRAGMA defer_foreign_keys = ON` (resets
  automatically at commit or rollback); delete all rows of `session`, `assessment`, `slot`,
  `course`, `holiday`, `term`, `setting` in that child-first order; insert every row with its
  stored `id` in order `term`, `holiday`, `course`, `slot`, `session` (by id), `assessment`,
  `setting`; commit. `reminder_log` is not touched. Document the app-level statement
  "reminder_log stays as is" in the doc comment.
- **Rationale**: deferred checking lets a makeup be inserted before the session it replaces (the
  self-reference `replaces_session_id`) without an ordering sort, and still makes a dangling
  reference fail at commit. Delete order is FK-safe even without deferral.
- **Alternatives**: `PRAGMA foreign_keys=OFF` around the restore — rejected, the pragma is a no-op
  inside a transaction and would leave the connection unsafe if the call returned early; topological
  sort of sessions — rejected, more code than deferral.

## R-D6. Error mapping in restore

- **Decision**: document problems (steps 1–5 of R-D4) are `Error::Validation` with a message that
  names the problem. During apply and commit, a SQLite constraint violation (CHECK, UNIQUE, NOT NULL,
  PRIMARY KEY, or FOREIGN KEY at commit) maps to `Error::Validation` ("backup data rejected:
  …") after rollback; any other database error stays `Error::Db`. The transaction is dropped
  (rolled back) on every error path.
- **Rationale**: a broken backup is invalid input from the user's point of view (R2 style), the
  caller shows one message; a real I/O failure is still reported as a database error.
- **Alternatives**: always `Error::Db` — rejected, the UI could not tell "bad file" from "bad disk".

## R-D7. Restored data is exactly the backup

- **Decision**: restore does not call sync, does not clamp or repair rows, and does not read the
  clock. A backup whose `terms` has one active term restores one active term; the partial unique
  index `one_active_term` rejects a backup with two (constraint rule above, rolled back).
- **Rationale**: spec Assumptions ("Restore does not sync sessions or touch the clock").

## R-D8. CSV encoding

- **Decision**: `csv::WriterBuilder::new().from_writer(w)` with its defaults: comma delimiter, quote
  only when needed (comma, double quote, line break), quotes doubled, record terminator CRLF.
  Header written explicitly as the first record. `completed` is `true` / `false`. `due_at` is the
  stored `YYYY-MM-DDTHH:MM`. A missing value (NULL note, NULL notes, no replaced session) is an empty
  field. The writer is flushed before returning; write errors map to `Error::Io`.
- **Rationale**: spec FR-010–FR-013; CRLF is the `csv` crate default and what spreadsheets accept.
  `true`/`false` matches the JSON booleans of R-D3.
- **Alternatives**: LF terminator, `1`/`0` — rejected, no benefit, and CRLF is the default.

## R-D9. CSV scope and sort

- **Decision**: attendance rows cover all sessions of all courses, all terms, all statuses,
  archived courses included. Sort: `date`, `start_time`, course `code`, session `id`. Assessment
  rows cover all assessments, sorted by `due_at`, course `code`, assessment `id`. Sorting is done in
  SQL (`ORDER BY`), reading `date`, `start_time`, and `due_at` as stored text, which sorts
  chronologically because of the zero-padded formats. `replaces_date` is the `date` of the session
  whose id is `replaces_session_id`, taken by `LEFT JOIN session r ON r.id = s.replaces_session_id`;
  empty when NULL. `kind` and `status` are the stored values.
- **Rationale**: spec FR-010–FR-012 and the edge cases; stable output across runs.

## R-D10. Consistent snapshot on export

- **Decision**: `export_json` reads all seven tables inside one read transaction opened with
  `self.conn.unchecked_transaction()` (usable through `&self`), so the document is one snapshot.
  Each CSV export is one query.
- **Rationale**: cheap, and it keeps the document internally consistent if a later milestone moves
  work to a second thread.
- **Alternatives**: sequential reads without a transaction — acceptable today because core has
  exclusive access to the connection, rejected as weaker for no saving.

## R-D11. Test layout

- **Decision**: three integration test files (`m3_backup_export.rs`, `m3_backup_restore.rs`,
  `m3_csv.rs`), each building its own data (no edit to `tests/common/mod.rs`), so the three test
  tasks are file-disjoint and can run in parallel. `reminder_log` is private to the crate, so the
  "restore leaves `reminder_log` untouched" check is a unit test in `backup.rs`.
- **Rationale**: tests use in-memory SQLite only (constitution II); integration tests cannot reach
  the crate-private connection.
