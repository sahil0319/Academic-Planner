---

description: "Task list for M3 backup, restore, and CSV export"
---

# Tasks: M3 Backup, Restore, and CSV Export

**Input**: Design documents from `specs/003-m3-backup-export/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/core-api-m3.md](contracts/core-api-m3.md)

**Tests**: Required (constitution II: T-tests before the rule). All test tasks are in Phase 2 and are
done by the **tester** agent before any implementation task. Implementation tasks are done by the
**builder** agent and make those tests pass. The builder does not edit the test files except to fix
a test that contradicts spec.md (then ask the human).

**Organization**: Phase 1 setup, Phase 2 tests first (labels trace each test to its story), then one
phase per user story with implementation only (US1 and US2 P1, US3 P2), then polish. Test names are
`t25_…` / `t30_…` for matrix tests and `r21_…` / `r22_…` / `r23_…` for the backup tests; all use
`Planner::open_in_memory()`.

## Format: `[ID] [P?] [Story] (role) Description`

- **[P]**: can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: user story from spec.md (US1–US3)
- **(tester)** / **(builder)**: which agent does the task

## Path Conventions

Rust workspace, package `planner-core`. Core: `crates/core/src/`; integration tests:
`crates/core/tests/`. Only `backup.rs`, `export.rs`, and `crates/core/Cargo.toml` (plus `Cargo.lock`)
change. `db.rs`, `model.rs`, `lib.rs`, `error.rs`, the migration, `tests/common/mod.rs`, and
`crates/app` are not edited. §5 signatures are frozen (`crates/core/tests/api_surface.rs`). No core
API is added for the R22 file copy (FR-014, built in M4).

---

## Phase 1: Setup

**Purpose**: dependencies the tests and the implementation both need

- [x] T001 (builder) Add `serde_json.workspace = true` and `csv.workspace = true` to `[dependencies]` in crates/core/Cargo.toml (both already in root `[workspace.dependencies]`, named in §1; research R-D1) and let `Cargo.lock` update; run `cargo build -p planner-core`

---

## Phase 2: Tests first (blocking: no implementation task starts until T006 passes)

**Purpose**: write every M3 test while the methods are still `todo!("M3: …")` stubs. T002, T003,
T004, and T005 touch different files and run in parallel; T006 follows all of them.

Each backup test builds data with M1/M2 public methods using the helpers in
crates/core/tests/common/mod.rs (do not edit that file), builds bad documents by exporting a good
one, parsing it with `serde_json::Value`, mutating it, and serializing it back, and compares
database state by `export_json` with the same `now` before and after. Every failing restore asserts
`Error::Validation` (research R-D4, R-D6) and an unchanged export.

- [x] T002 [P] [US1] (tester) Write crates/core/tests/m3_backup_export.rs (R21, FR-001–FR-004) with: `r21_export_has_exact_top_level_keys` (keys exactly `format`, `schema_version`, `exported_at`, `terms`, `holidays`, `courses`, `slots`, `sessions`, `assessments`, `settings`; `format == "academic-planner-backup"`; `schema_version == 1`; `exported_at == "2026-01-25T18:30"` for `now = 2026-01-25 18:30`); `r21_export_rows_keep_ids_and_value_types` (on the §8 fixture synced, with holiday, a rescheduled 01-12 session plus attended makeup, an assessment, changed settings: every term/holiday/course/slot/session/assessment appears once with its stored id and §3 column keys; `is_active`, `archived`, `completed` are JSON booleans; NULL columns are JSON `null`; dates and times are the stored strings; the makeup's `replaces_session_id` equals the original's id; `settings` is an object of key to string with the changed values); `r21_export_collections_ordered` (rows by `id`, holidays by `(term_id, date)` entered out of order, `settings` object holds all keys); `r21_export_includes_archived_inactive_and_all_statuses` (archived course, an inactive earlier term, pending / attended / absent / cancelled sessions all present); `r21_export_empty_database` (fresh `open_in_memory()`: all six arrays empty, `settings` holds exactly `class_reminder_minutes = "15"`, `upcoming_window_days = "7"`, `notifications_enabled = "1"`); `r21_export_is_deterministic` (two exports with the same `now` are byte-identical)
- [x] T003 [P] [US2] (tester) Write crates/core/tests/m3_backup_restore.rs (R22, FR-005–FR-009) with: `t30_export_restore_into_empty_export_equal_except_exported_at` (export from the populated fixture at `now1`, restore into `open_in_memory()`, export at `now2`; parse both, drop `exported_at`, equal; and `exported_at` values differ); `t30_bad_format_rejected_and_db_unchanged` (format wrong, format missing, format not a string); `t30_newer_schema_version_rejected_and_db_unchanged` (`schema_version` 2; also missing; also non-integer); `r22_restore_accepts_equal_and_lower_schema_version` (versions 1 and 0 restore); `r22_restore_replaces_not_merges` (planner A populated, planner B with different rows incl. other ids; restore A's backup into B; B's export equals A's modulo `exported_at`, no B-only row remains); `r22_restore_preserves_ids_and_links` (course→term, slot→course, session→slot, makeup `replaces_session_id`, assessment→course all intact; `sessions_for_course` still shows "Rescheduled → …" on the original); `r22_restore_makeup_with_lower_id_than_original` (hand-edit the document so the makeup row has a lower id than the session it replaces; restore succeeds — proves deferred foreign keys, R-D5); `r22_invalid_json_rejected_and_db_unchanged` (garbage text, empty string, root `[]`); `r22_missing_collection_rejected_and_db_unchanged` (remove `sessions`; also remove `settings`); `r22_ill_typed_field_rejected_and_db_unchanged` (`completed: 1`, `is_active: "yes"`, `id: "x"`, a collection that is an object); `r22_malformed_date_rejected_and_db_unchanged` (term `start_date: "01/05/2026"`, session `start_time: "9am"`, assessment `due_at: "2026-01-05"`); `r22_dangling_reference_rolls_back` (session `course_id: 999`; session `replaces_session_id: 999`; slot `course_id: 999`); `r22_constraint_violation_rolls_back` (two active terms; session with `end_time <= start_time`; duplicate `(slot_id, date)`); each failure case compares export before and after
- [x] T004 [P] [US3] (tester) Write crates/core/tests/m3_csv.rs (R23, FR-010–FR-013) with byte-exact comparisons against expected text using CRLF record terminators (research R-D8): `t25_attendance_csv_header_and_rows_exact` (fixture synced; 01-12 rescheduled to a makeup on 01-13 marked attended with a note; expected full text starts with `date,start,end,course_code,course_name,kind,status,replaces_date,note`; 01-12 row `cancelled` with empty `replaces_date`; 01-13 row `makeup`, `attended`, `replaces_date = 2026-01-12`; rows in date then start-time order); `t25_assessments_csv_header_and_rows_exact` (two assessments entered in reverse due order, one completed; header `course_code,kind,title,due_at,completed,notes`; `completed` as `true`/`false`; `due_at` as `YYYY-MM-DDTHH:MM`; rows in due order); `r23_csv_empty_planner_is_header_only` (both exports); `r23_csv_quotes_comma_quote_and_line_break` (note, title, and course name containing `,`, `"`, and a line break; output quoted with doubled quotes; parsing the output back with the `csv` reader returns the original text); `r23_attendance_csv_includes_archived_all_statuses_all_terms` (archived course, pending / attended / absent / cancelled sessions, a second inactive term's course); `r23_attendance_csv_ties_ordered_by_course_code` (two courses with a session on the same date and start time: order by course code, then session id); `r23_assessments_csv_ties_ordered_by_course_code` (same due time); `r23_csv_missing_values_are_empty` (no note, no notes, no replaced session); `r23_csv_write_failure_is_io_error` (a `Write` that always fails returns `Error::Io`)
- [x] T005 [P] [US2] (tester) Append a `#[cfg(test)] mod tests` to crates/core/src/backup.rs (above or below the existing stub impl, leaving the stubs untouched) with `r22_restore_leaves_reminder_log_untouched`: on `open_in_memory()` insert a `reminder_log` row through `p.conn`, restore a valid backup, assert the row is still there; and a failing restore also leaves it (R22, spec edge case; research R-D5, R-D11). The test fails on the stub until T008/T009
- [x] T006 (tester) Run `cargo test -p planner-core --no-fail-fast` and confirm every new M3 test binary compiles, every M3 test fails only by panicking on a `todo!("M3…")` (no compile errors, no assertion failures in M1/M2 tests, `api_surface.rs` still green); note the failing count. Needs T001–T005

**Checkpoint**: tests written and failing for the right reason. Commit "M3: tests for R21–R23, T25, T30".

---

## Phase 3: User Story 1 - Back up everything to one file (Priority: P1) 🎯 MVP

**Goal**: `export_json` produces the versioned, ordered, id-preserving document.

**Independent Test**: `cargo test -p planner-core --test m3_backup_export` passes.

- [ ] T007 [US1] (builder) In crates/core/src/backup.rs add the private serde row structs (`TermRow`, `HolidayRow`, `CourseRow`, `SlotRow`, `SessionRow`, `AssessmentRow`) with field order and names equal to the §3 columns, flags as `bool`, nullable columns as `Option<…>` (JSON `null`), dates/times/date-times as `String`, and a private `Backup` struct (`format`, `schema_version: i64`, `exported_at`, the six collections, `settings: BTreeMap<String, String>`) deriving `Serialize` and `Deserialize`; implement `export_json(now)` per research R-D2, R-D3, R-D10 and [data-model.md](data-model.md): one `unchecked_transaction()`, one `SELECT` per table with `ORDER BY id` (holidays `ORDER BY term_id, date`, settings `ORDER BY key`), `format = "academic-planner-backup"`, `schema_version = SCHEMA_VERSION`, `exported_at = fmt_datetime(now)`, serialize with `serde_json::to_string_pretty`; map flags with `!= 0`. Doc comment `/// R21: …`. Needs T001, T006

**Checkpoint**: T002's tests pass; `cargo clippy -p planner-core --all-targets -- -D warnings` passes.

---

## Phase 4: User Story 2 - Restore a backup, all or nothing (Priority: P1)

**Goal**: `restore_json` validates everything first, then replaces all data in one transaction.

**Independent Test**: `cargo test -p planner-core --test m3_backup_restore` and
`cargo test -p planner-core --lib backup` pass.

- [ ] T008 [US2] (builder) In crates/core/src/backup.rs add a private `parse_backup(json: &str) -> Result<Backup>` following research R-D4 exactly: `serde_json::from_str::<Value>`; non-object root, invalid JSON → `Error::Validation`; `format` must be the string `"academic-planner-backup"`; `schema_version` must be an integer, present, and `<= SCHEMA_VERSION` ("schema_version greater than the app's is rejected; equal or less is accepted"); then `serde_json::from_value::<Backup>` (all seven collections required, every field well-typed, unknown extra keys ignored; map `serde_json::Error` to `Error::Validation` naming the problem); then check every date, time, and date-time text value with `parse_date`, `parse_time`, `parse_datetime` from `crate::db` (term `start_date`/`end_date`; holiday `date`; course none; slot `start_time`/`end_time`/`valid_from`/`valid_to`; session `date`/`start_time`/`end_time`; assessment `due_at`), `Error::Validation` on failure. Needs T007
- [ ] T009 [US2] (builder) In crates/core/src/backup.rs implement `restore_json(json)` per research R-D5–R-D7: call `parse_backup` first; open `self.conn.transaction()`; `PRAGMA defer_foreign_keys = ON`; `DELETE FROM` `session`, `assessment`, `slot`, `course`, `holiday`, `term`, `setting` in that order (never `reminder_log`); `INSERT` every row with its id in order `term`, `holiday`, `course`, `slot`, `session` (by id), `assessment`, `setting` (flags written as `1`/`0`); `commit()`; map `rusqlite::Error::SqliteFailure` with `ErrorCode::ConstraintViolation` (including at commit) to `Error::Validation("backup data rejected: …")`, any other error through the existing `From`; the transaction is dropped (rolled back) on every error path; no sync, no clock. Doc comment `/// R22: …` and a line stating `reminder_log` is untouched. Needs T008

**Checkpoint**: T003 and T005 pass (together with T002); a failed restore leaves the export identical.

---

## Phase 5: User Story 3 - Open attendance and assessments in a spreadsheet (Priority: P2)

**Goal**: both CSV exports with exact headers, sorted rows, correct quoting.

**Independent Test**: `cargo test -p planner-core --test m3_csv` passes.

**Parallel note**: T010 and T011 touch only crates/core/src/export.rs, so they run in parallel with
the whole US1/US2 chain (T007–T009) once T006 passes; T011 follows T010 (same file).

- [ ] T010 [P] [US3] (builder) Implement `export_attendance_csv(w)` in crates/core/src/export.rs per research R-D8, R-D9 and [data-model.md](data-model.md): `csv::WriterBuilder::new().from_writer(w)` with defaults (CRLF, quote when needed); write the header record `date,start,end,course_code,course_name,kind,status,replaces_date,note`; query `session s JOIN course c ON c.id = s.course_id LEFT JOIN session r ON r.id = s.replaces_session_id` (all sessions, all courses, terms, statuses) `ORDER BY s.date, s.start_time, c.code, s.id`; `replaces_date` = `r.date` or empty; NULL note empty; `kind`/`status` as stored; flush; map `csv::Error` to `Error::Io`. Doc comment `/// R23: …`. Needs T001, T006
- [ ] T011 [US3] (builder) Implement `export_assessments_csv(w)` in crates/core/src/export.rs: same writer setup; header `course_code,kind,title,due_at,completed,notes`; query `assessment a JOIN course c ON c.id = a.course_id` `ORDER BY a.due_at, c.code, a.id`; `completed` as `true`/`false`; `due_at` as stored; NULL notes empty; flush. Doc comment `/// R23: …`. Needs T010

**Checkpoint**: T004 passes.

---

## Phase 6: Polish & Gate

- [ ] T012 (builder) Confirm no `todo!("M3` remains in crates/core/src (`due_reminders` keeps its `todo!("M5…")`); confirm every new public-facing method has a rule-ID doc comment (R21, R22, R23) and `crates/core/tests/api_surface.rs` is unchanged and green. Needs T007–T011
- [ ] T013 (builder) Run the gate `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` and paste the summary; run the commands in [quickstart.md](quickstart.md). Do not edit `.claude/state/` (the router does). Needs T012

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (T001)**: no dependencies; can run in parallel with writing T002–T005 (different files), but
  those tests only compile after T001.
- **Phase 2**: T002, T003, T004, T005 in parallel (different files); T006 after T001–T005.
- **Phases 3–5**: all need T006. Chain A: T007 → T008 → T009 (all backup.rs). Chain B: T010 → T011
  (export.rs). Chain A and chain B are file-disjoint and run in parallel.
- **Phase 6**: after T007–T011.

### Story dependencies

- US1 (T007) before US2 (T008, T009): the restore tests build their documents with `export_json`.
- US3 is independent of US1/US2.

### Parallel opportunities

| Group | Tasks | Role |
| --- | --- | --- |
| Test authoring (disjoint files) | T002, T003, T004, T005 (and T001) | tester (T001 builder) |
| Implementation chains (disjoint files) | T007→T008→T009 and T010→T011 | builder |

### Role summary

- **tester**: T002, T003, T004, T005, T006
- **builder**: T001, T007, T008, T009, T010, T011, T012, T013

## Implementation Strategy

1. T001 and the four test tasks first; T006 proves the tests fail only on `todo!`.
2. MVP: US1 (backup export), then US2 (restore). US3 can be done by a second builder in parallel.
3. Gate at the end; the router commits per phase ("M3: …").
