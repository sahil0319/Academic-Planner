---

description: "Task list for M1 core storage and setup data"
---

# Tasks: M1 Core Storage and Setup Data

**Input**: Design documents from `specs/001-m1-core-storage/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/core-api.md](contracts/core-api.md)

**Tests**: Required. The constitution (II) and `AGENTS.md` require T-tests to be written before the
rule they cover. Test names follow `t20_fresh_db_reopen_is_noop`; tests for rules without a T-id use
the rule id, e.g. `r01_second_term_becomes_only_active`. All tests use in-memory SQLite (constitution
II): `Planner::open_in_memory()`, or for T20/T26, which need a reopen, `Planner::open` on a
shared-cache in-memory URI `file:<unique name>?mode=memory&cache=shared` kept alive by a "keeper"
`rusqlite::Connection` held for the whole test. No test creates a DB file.

**Organization**: Grouped by user story. Phase order differs from spec priority in one place:
US3 (validation, T24) runs after US2 and US4 because T24 exercises the term, course, slot, and
assessment create/update operations those phases build.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: user story from spec.md (US1–US5)

## Path Conventions

Rust workspace. Core crate: `crates/core/src/`, integration tests: `crates/core/tests/`.
`crates/app` is not changed in M1.

---

## Phase 1: Setup

**Purpose**: dependencies and module skeleton

- [X] T001 Add `rusqlite`, `chrono`, `thiserror`, `directories`, `serde` as `{ workspace = true }` dependencies in crates/core/Cargo.toml; run `cargo build -p planner-core` and commit the updated Cargo.lock
- [X] T002 Create empty modules `error`, `model`, `db`, `schedule`, `attendance`, `dashboard`, `backup`, `export` (files under crates/core/src/) and wire them in crates/core/src/lib.rs with `pub use` re-exports of `Planner`, `Error`, `Result`, all model types, `resolve_db_path`, `default_db_path`

---

## Phase 2: Foundational (blocking prerequisites)

**Purpose**: every §5 type and signature exists so tests compile; M1 bodies are `todo!()` until their story

**⚠️ CRITICAL**: no story work starts until this phase builds cleanly under clippy

- [X] T003 [P] Define `Error` (thiserror) and `pub type Result<T>` in crates/core/src/error.rs with variants exactly per data-model.md "Errors": `Validation(String)`, `NotFound { entity: &'static str, id: i64 }`, `NoActiveTerm`, `SchemaTooNew { found: i64, supported: i64 }` with message "database schema version {found} is newer than this app supports ({supported}); update the app", `NoDataDir`, `Corrupt(String)`, `Db(#[from] rusqlite::Error)`, `Io(#[from] std::io::Error)`
- [X] T004 [P] Define all types in crates/core/src/model.rs exactly per data-model.md: enums `Status`, `SessionKind`, `AssessmentKind`, `ReminderKind`, `AssessmentFilter { Upcoming { now }, Overdue { now }, Completed, All }`; `TermInput { id: Option<i64>, name, start_date, end_date }`, `Term` (incl. `is_active: bool`, `holidays: Vec<Holiday>`), `Holiday { date, label: Option<String> }`, `CourseInput` (`min_attendance_pct: Option<i64>`), `Course` (`archived: bool`), `SlotInput`/`Slot` (`weekday: chrono::Weekday`, `valid_to: Option<NaiveDate>`), `AssessmentInput` (`remind_minutes_before: Option<u32>`), `Assessment` (`completed: bool`, `course_code: String`), `Settings { class_reminder_minutes: u32, upcoming_window_days: u32, notifications_enabled: bool }`, and M2+ types `Session`, `SessionView`, `MakeupInput`, `SyncReport`, `AttendanceSummary`, `Dashboard`, `Reminder`. Derive `Debug, Clone, PartialEq` on all and `Serialize, Deserialize` on records; doc comment each type with its §3 table
- [X] T005 Add `pub struct Planner { conn: rusqlite::Connection }` and private text codecs in crates/core/src/db.rs: `NaiveDate` ↔ `%Y-%m-%d`, `NaiveTime` ↔ `%H:%M`, `NaiveDateTime` ↔ `%Y-%m-%dT%H:%M` (parse failure → `Error::Corrupt`), weekday ↔ ISO 1–7, and enum ↔ text for `Status`/`SessionKind`/`AssessmentKind`/`ReminderKind`; unit tests in the same file for round-trips and for seconds being dropped (research D4)
- [X] T006 Add every `impl Planner` method from contracts/core-api.md with its exact signature: M1 methods in crates/core/src/db.rs with body `todo!("M1")`; `change_slot`, `end_slot`, `sync_sessions`, `sessions_between`, `sessions_for_course`, `set_status`, `reschedule`, `add_makeup` in crates/core/src/schedule.rs; `attendance_summary`, `all_summaries` in crates/core/src/attendance.rs; `dashboard` in crates/core/src/dashboard.rs; `export_json`, `restore_json`, `due_reminders` in crates/core/src/backup.rs; `export_attendance_csv`, `export_assessments_csv` in crates/core/src/export.rs. Out-of-scope bodies are `todo!("<milestone>: <rule ids>")` using the table's rule column (e.g. `todo!("M2: R4, R5")`); each has a `///` doc comment naming its rules
- [X] T007 Create fixture helpers in crates/core/tests/common/mod.rs per §8: `fixture_term()` = term 2026-01-05 → 2026-01-25; `fixture_course()` = `CSE101`, target 75; `slot_a()` = Mon 09:00–10:00 valid_from 2026-01-05; `slot_b()` = Wed 11:00–12:00 valid_from 2026-01-05; plus `d("YYYY-MM-DD")`, `t("HH:MM")`, `dt("YYYY-MM-DDTHH:MM")` parse helpers

**Checkpoint**: `cargo clippy --workspace --all-targets -- -D warnings` passes. Commit "M1: core types and API stubs".

---

## Phase 3: User Story 1 - Planner data file opens safely (Priority: P1) 🎯 MVP

**Goal**: migration 001, open/open_in_memory, R24 path, R25 migrate / no-op / refuse newer

**Independent Test**: `cargo test -p planner-core --test migrations` and `--lib r24`

### Tests for User Story 1 (write first, confirm they fail)

- [X] T008 [P] [US1] Write crates/core/tests/migrations.rs: `t20_fresh_db_migrated_and_reopen_is_noop` (keeper connection on `file:t20?mode=memory&cache=shared`; `Planner::open` on that URI → keeper sees `PRAGMA user_version` = 1 and all §3 tables; drop the planner; record `sqlite_master` rows, `user_version`, and `setting` rows; open again → all unchanged; also `open_in_memory` reports version 1); `t26_newer_schema_version_refused` (keeper on `file:t26?mode=memory&cache=shared` sets `PRAGMA user_version = 2`; `Planner::open` returns `Error::SchemaTooNew { found: 2, supported: 1 }` whose `to_string()` contains "newer"; afterwards the keeper still sees `user_version` 2 and an empty `sqlite_master`)
- [X] T009 [P] [US1] Write unit tests in crates/core/src/db.rs: `r24_override_wins`, `r24_default_is_data_dir_planner_sqlite`, `r24_no_override_no_data_dir_is_error` for `resolve_db_path`, and `r24_ensure_parent_dir_creates_missing_dirs` for the private `ensure_parent_dir(path)` helper (unique nested path under `std::env::temp_dir()`, no SQLite file created, removed afterwards)

### Implementation for User Story 1

- [X] T010 [US1] Write crates/core/src/migrations/001_init.sql: the `docs/spec.md` §3 SQL verbatim, then `INSERT INTO setting (key, value) VALUES ('class_reminder_minutes','15'), ('upcoming_window_days','7'), ('notifications_enabled','1');`
- [X] T011 [US1] Implement in crates/core/src/db.rs per research D1/D2: `const MIGRATIONS: &[&str] = &[include_str!("migrations/001_init.sql")]`; on connect, in this order: `PRAGMA foreign_keys = ON`; read `user_version`; if greater than `MIGRATIONS.len()` return `SchemaTooNew` (no write has happened); `PRAGMA journal_mode = WAL`; apply each pending migration in its own transaction that also sets `user_version`. `open_in_memory()`; `open(path)` calls `ensure_parent_dir(path)` (`std::fs::create_dir_all` on a non-empty parent) then connects. Doc comments cite R25
- [X] T012 [US1] Implement `resolve_db_path(env_override: Option<OsString>, data_dir: Option<PathBuf>) -> Result<PathBuf>` and `default_db_path()` (reads `ACADEMIC_PLANNER_DB`, `ProjectDirs::from("", "", "Academic Planner")`, file `planner.sqlite`) in crates/core/src/db.rs per research D5. Doc comments cite R24

**Checkpoint**: T008, T009 pass. Commit "M1: migration 001 and open (R24, R25)".

---

## Phase 4: User Story 2 - Term, holidays, and courses (Priority: P1)

**Goal**: R1 active term, holidays, course CRUD, R3 archive in lists

**Independent Test**: `cargo test -p planner-core --test terms_courses`

### Tests for User Story 2 (write first)

- [X] T013 [US2] Write crates/core/tests/terms_courses.rs: `r01_no_term_means_no_active_term`; `r01_saved_term_becomes_active`; `r01_second_term_becomes_only_active` (count `is_active = 1` rows via `active_term` and a second save); `r01_update_term_keeps_id_and_active`; `set_holidays_replaces_set` (set two, then one → only the one; `Term.holidays` sorted by date; unknown term → `NotFound`); `courses_create_update_list_in_active_term` (list ordered by code; courses of an inactive term not listed); `create_course_without_active_term_fails` (`Error::NoActiveTerm`); `update_missing_course_is_not_found`; `r03_archived_hidden_from_default_list` (archive → absent from `list_courses(false)`, present in `list_courses(true)` with all fields unchanged; unarchive → back in default list)

### Implementation for User Story 2

- [X] T014 [US2] Implement `active_term`, `save_term` (one transaction: `UPDATE term SET is_active = 0`, then insert or update with `is_active = 1`; update of missing id → `NotFound`), and `set_holidays` (one transaction: delete the term's holidays, insert the given ones; missing term → `NotFound`) in crates/core/src/db.rs; load `Term.holidays` sorted by date. Doc comments cite R1
- [X] T015 [US2] Implement `create_course` (active term id; none → `NoActiveTerm`), `update_course` (missing → `NotFound`), `set_archived` (missing → `NotFound`), `list_courses(include_archived)` (active term only, ordered by code; empty when no active term) in crates/core/src/db.rs. Doc comments cite R1, R3

**Checkpoint**: T013 passes. Commit "M1: terms, holidays, courses (R1, R3)".

---

## Phase 5: User Story 4 - Slots and settings (Priority: P2)

**Goal**: slot add/list-on-date, settings read/save with §3 defaults

**Independent Test**: `cargo test -p planner-core --test slots_settings`

### Tests for User Story 4 (write first)

- [X] T016 [US4] Write crates/core/tests/slots_settings.rs using the §8 fixture: `add_slot_returns_stored_values`; `list_slots_inclusive_validity` (slot with `valid_from` 2026-01-12, `valid_to` 2026-01-19: listed on 01-12 and 01-19, not on 01-11 or 01-20); `list_slots_open_end_until_term_end` (`valid_to = None`: listed on 2026-01-25, not on 2026-01-26); `list_slots_excludes_archived_course`; `list_slots_ordered_by_weekday_then_start`; `settings_defaults_on_fresh_db` (15, 7, true); `save_settings_round_trip`

### Implementation for User Story 4

- [X] T017 [US4] Implement `add_slot` (insert; unknown course surfaces the FK error) and `list_slots(on)` per research D9 (`valid_from <= on AND on <= coalesce(valid_to, term.end_date)`, active term, non-archived courses, ordered by weekday, start_time) in crates/core/src/db.rs
- [X] T018 [US4] Implement `settings()` (read the three keys; missing key → §3 default; unparsable value → `Corrupt`) and `save_settings` (one transaction, upsert the three keys; bool as `'1'`/`'0'`) in crates/core/src/db.rs

**Checkpoint**: T016 passes. Commit "M1: slots and settings".

---

## Phase 6: User Story 3 - Invalid input rejected (Priority: P1)

**Goal**: assessment storage (FR-013a) and every R2 validation case (T24)

**Independent Test**: `cargo test -p planner-core --test assessments --test validation`

### Tests for User Story 3 (write first)

- [X] T019 [P] [US3] Write crates/core/tests/assessments.rs: `assessment_create_update_round_trip` (includes `course_code`); `set_completed_toggles`; `delete_assessment_removes_row`; `list_all_sorted_by_due_at`; `list_completed_only_completed`; `list_excludes_archived_course`; missing id on update/set_completed/delete → `NotFound`
- [X] T020 [P] [US3] Write crates/core/tests/validation.rs, one test per R2 case, each asserting `Err(Error::Validation(_))` and that the stored row count is unchanged: `t24_term_end_before_start`, `t24_slot_end_not_after_start` (equal and earlier), `t24_attendance_target_out_of_range` (-1 and 101, on create and update), `t24_empty_course_code`, `t24_empty_course_name` (empty and whitespace-only), `t24_empty_assessment_title` (create and update), `t24_duplicate_course_code_in_term` (create, and update to another course's code). Boundary tests that must succeed: `t24_term_end_equal_start_is_valid`, `t24_target_bounds_and_none_are_valid` (0, 100, None), `t24_same_code_in_other_term_is_valid`, `t24_update_keeping_own_code_is_valid`

### Implementation for User Story 3

- [X] T021 [US3] Implement `create_assessment`, `update_assessment`, `set_completed`, `delete_assessment`, and `list_assessments` in crates/core/src/db.rs: `All` and `Completed` filters return active-term, non-archived-course assessments sorted by `due_at` with `course_code` joined; `Upcoming`/`Overdue` arms are `todo!("M2: R18")` / `todo!("M2: R19")`
- [X] T022 [US3] Add private `validate(&self) -> Result<()>` for `TermInput`, `CourseInput`, `SlotInput`, `AssessmentInput` (in crates/core/src/model.rs) per research D7: term `end_date < start_date`; slot `end_time <= start_time`; `min_attendance_pct` outside 0–100; `code`/`name`/`title` empty after `trim()`. Call each before any write in `save_term`, `create_course`, `update_course`, `add_slot`, `create_assessment`, `update_assessment`; in `create_course`/`update_course` check duplicate code (`term_id = ? AND code = ? AND id != ?`) inside the write transaction. All return `Error::Validation(msg)`. Doc comments cite R2

**Checkpoint**: T019, T020 pass. Commit "M1: assessments and R2 validation (T24)".

---

## Phase 7: User Story 5 - Full core API exists (Priority: P2)

**Goal**: freeze every §5 signature so M2 and M4 can proceed in parallel

**Independent Test**: `cargo test -p planner-core --test api_surface`

- [ ] T023 [US5] Write crates/core/tests/api_surface.rs: one `#[test] fn api_signatures_match_spec_section_5()` that binds every method in contracts/core-api.md to an explicit `fn` pointer type (e.g. `let _: fn(&Planner, NaiveDate, NaiveDate, NaiveDateTime) -> Result<Vec<SessionView>> = Planner::sessions_between;`, generic CSV methods as `Planner::export_attendance_csv::<Vec<u8>>`), plus `resolve_db_path` and `default_db_path`. Never calls a stub
- [ ] T024 [US5] Grep crates/core/src for `todo!(` and confirm the only remaining ones are the out-of-scope methods and the `Upcoming`/`Overdue` arms, each with a milestone + rule message; no `todo!("M1")` remains

**Checkpoint**: Commit "M1: API surface check".

---

## Phase 8: Polish & gate

- [ ] T025 Run `cargo doc -p planner-core --no-deps` and fix any broken doc links; confirm every public item has a doc comment
- [ ] T026 Run the gate `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` and paste its summary; confirm T20, T24, T26 pass (§7 M1 done-when)
- [ ] T027 Update .claude/state/progress.json (all tasks) and .claude/state/checkpoint.md (last green gate hash, next milestone M2); commit "M1: milestone complete"

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (T001–T002) → Foundational (T003–T007) → US1 (T008–T012)
- US2 (T013–T015) and US4 (T016–T018) both need US1 (`open_in_memory`); US4 tests use the term/course
  from US2, so run US2 first
- US3 (T019–T022) needs US2 and US4 (T24 exercises term, course, slot, and assessment writes)
- US5 (T023–T024) needs Foundational only for T023; T024 needs all story phases
- Polish (T025–T027) last

### Within Each Story

- Tests first; confirm they fail (they panic on `todo!("M1")`), then implement
- Commit at each checkpoint; update progress.json after every task

### Parallel Opportunities

- T003 ∥ T004 (different files)
- T008 ∥ T009 (different files)
- T019 ∥ T020 (different files)
- T023 can be written any time after T006

## Parallel Example: User Story 1

```text
Task: "Write crates/core/tests/migrations.rs (t20, t26)"
Task: "Write r24_* unit tests in crates/core/src/db.rs"
```

## Implementation Strategy

### MVP first

Phases 1–3 deliver a safely opened, migrated data file (T20, T26): the base everything else needs.

### Incremental delivery

US1 → US2 → US4 → US3 → US5, with the gate's clippy step kept green at every checkpoint so the
final gate (T026) is a formality.

## Notes

- Single agent, sequential: [P] marks independence, not a request to spawn subagents (§12.4).
- No task adds a requirement outside `docs/spec.md` plus the two recorded human decisions.
