# Research: M1 Core Storage and Setup Data

All technology choices are fixed by `docs/spec.md` §1. This file records only the M1 design
decisions that §1–§5 leave open. No item re-decides §1.

## D1. Migration runner

- **Decision**: `const MIGRATIONS: &[&str] = &[include_str!("migrations/001_init.sql")]`. On open:
  set `foreign_keys`, read `user_version` (= `v`). If `v > MIGRATIONS.len()` return
  `Error::SchemaTooNew { found, supported }` before any write. Otherwise apply
  `MIGRATIONS[v..]` in order, each inside its own transaction that also sets `user_version` to its
  number. Applying nothing when `v == len` makes reopen a no-op (R25, T20).
- **Rationale**: §1 "hand-rolled, embedded SQL, `PRAGMA user_version`". Version check before any
  write guarantees T26's "file untouched".
- **Alternatives**: `rusqlite_migration` / `refinery` crates — rejected, §1 says hand-rolled.

## D2. Pragmas

- **Decision**: `PRAGMA foreign_keys = ON` first on every connection; `PRAGMA journal_mode = WAL`
  only after the version check passes (WAL rewrites the file header, so a refused newer DB must
  not get it), then migrations. For in-memory DBs SQLite reports `memory` for journal mode; that is accepted.
- **Rationale**: §1. WAL is a per-file setting; `foreign_keys` is per-connection.

## D3. Setting defaults

- **Decision**: `001_init.sql` ends with
  `INSERT INTO setting VALUES ('class_reminder_minutes','15'),('upcoming_window_days','7'),('notifications_enabled','1');`.
  `settings()` falls back to the same defaults for any missing key.
- **Rationale**: §3 lists the defaults beside the `setting` table; a fresh DB then matches §3.

## D4. Date and time encoding

- **Decision**: private codecs in `db.rs`: `NaiveDate` ↔ `%Y-%m-%d`, `NaiveTime` ↔ `%H:%M`,
  `NaiveDateTime` ↔ `%Y-%m-%dT%H:%M`. Bound as `String`; parsed on read (a malformed stored value
  becomes `Error::Corrupt`). Seconds are dropped on write.
- **Rationale**: §1 exact text formats. rusqlite's `chrono` feature writes `HH:MM:SS`, which would
  break the format and the lexical comparisons used by later SQL.

## D5. R24 data-file location

- **Decision**: in `db.rs`:
  `pub fn resolve_db_path(env_override: Option<OsString>, data_dir: Option<PathBuf>) -> Result<PathBuf>`
  (pure: override wins; else `data_dir/planner.sqlite`; neither → `Error::NoDataDir`) and
  `pub fn default_db_path() -> Result<PathBuf>` (reads `ACADEMIC_PLANNER_DB` and
  `ProjectDirs::from("", "", "Academic Planner")`). `Planner::open` creates the parent directory
  if missing. The app calls `default_db_path()` when it starts using the DB (M4); M1 does not
  change `crates/app`.
- **Testing**: the directory step is a private `ensure_parent_dir(path)` helper unit-tested on a
  temp directory (no SQLite file is created), keeping all DB tests in memory (constitution II).
- **Rationale**: pure function is unit-testable without env-var races or touching the user DB.
  Creating the directory in `open` covers both the default and the override path.
- **Alternatives**: put it in the app — rejected: core is where tested logic lives (§2 rule).

## D6. Active-term scoping (R1)

- **Decision**: `save_term` runs one transaction: `UPDATE term SET is_active = 0`, then insert
  (`id = None`) or update (`id = Some`) with `is_active = 1`. `create_course` takes the active
  term's id; with no active term it returns `Error::NoActiveTerm`. `list_courses`, `list_slots`,
  and `list_assessments` return only rows of the active term (empty when none).
  `Term` carries `holidays: Vec<Holiday>` so the Settings page (M4) can show them; §5 has no
  separate holiday reader.
- **Rationale**: R1 "operate on the active term"; §5 comment "create or update; sets active".

## D7. Validation (R2, T24)

- **Decision**: each `*Input` has a private `validate(&self) -> Result<()>` called before any write:
  term `end_date < start_date`; slot `end_time <= start_time`; `min_attendance_pct` outside 0–100;
  `code`/`name`/`title` empty after `trim()`. Duplicate course code: query
  `course WHERE term_id = ? AND code = ? AND id != ?` inside the write transaction. All return
  `Error::Validation(String)` with a human-readable message. Nothing is stored on error.
- **Rationale**: R2 lists exactly these cases; whitespace-only text is treated as empty because it
  displays as empty. `min_attendance_pct: Option<i64>` keeps -1 and 101 representable so T24 can
  test both sides.
- **Type-level guards** (not R2 cases, made unrepresentable instead of validated): weekday is
  `chrono::Weekday`; `remind_minutes_before` is `Option<u32>`.

## D8. Archived courses (R3) in M1

- **Decision**: `list_courses(false)`, `list_slots`, and `list_assessments` exclude archived
  courses; `list_courses(true)` includes them. Archiving changes only the flag; nothing is deleted.
  Sync/dashboard parts of R3 are M2 (T15, T28).
- **Rationale**: R3 "hides it from … timetable, and default lists".

## D9. `list_slots(on)` (§5)

- **Decision**: slots where `valid_from <= on` and `on <= coalesce(valid_to, term.end_date)` for
  non-archived courses of the active term, ordered by weekday, start time.
- **Rationale**: §3 comments: both ends inclusive; `NULL` valid_to = until term end.

## D10. Assessments (FR-013a)

- **Decision**: create/update/set_completed/delete fully implemented in M1, with R2 title check.
  `Assessment` carries the joined `course_code` for lists. `AssessmentFilter` is
  `Upcoming { now }`, `Overdue { now }`, `Completed`, `All`; `All` and `Completed` work in M1
  (sorted by `due_at`); `Upcoming`/`Overdue` arms are `todo!("M2: R18")`/`todo!("M2: R19")`
  until T16/T17 are written first in M2.
- **Rationale**: list filters need `now` but core reads no clock, so the variants carry it.
  Test-first rule: R18/R19 logic waits for its T-tests.

## D11. `is_past` (human decision 2026-10-28)

- **Decision**: `sessions_between(from, to, now)` and `sessions_for_course(course_id, now)`;
  `is_past = now >= session start`. Applied to `docs/spec.md` §5. Implemented in M2.

## D12. Stubs

- **Decision**: every out-of-scope method has its final signature and body
  `todo!("M2: R4, R5")` (milestone + rule IDs). `SyncReport`, `Session`, `SessionView`,
  `MakeupInput`, `AttendanceSummary`, `Dashboard`, `Reminder` are fully defined now so the UI can
  bind to them.
- **Rationale**: §5 "stub with `todo!()` in M1 so UI work can start in parallel".
