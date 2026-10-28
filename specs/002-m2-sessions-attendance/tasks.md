---

description: "Task list for M2 sessions, exceptions, attendance, and dashboard"
---

# Tasks: M2 Sessions, Exceptions, Attendance, and Dashboard

**Input**: Design documents from `specs/002-m2-sessions-attendance/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/core-api-m2.md](contracts/core-api-m2.md)

**Tests**: Required (constitution II: T-tests before the rule). All M2 integration tests were
written first by the tester agent (commit 6b3358a) on `Planner::open_in_memory()` with the helpers
in crates/core/tests/common/mod.rs; their tasks are marked done. Implementation tasks below make
them pass. Do not edit the test files except to fix a test that contradicts spec.md (then ask).

**Organization**: Grouped by user story in priority order (US1, US2, US3 P1; US4, US5, US7 P2;
US6 P3). US7 runs after US5 because its scenarios read the dashboard and summaries; US6 runs last
because it touches sync, dashboard, and summaries.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: user story from spec.md (US1–US7)

## Path Conventions

Rust workspace, package `planner-core`. Core: `crates/core/src/`; integration tests:
`crates/core/tests/`. Only `db.rs`, `schedule.rs`, `attendance.rs`, `dashboard.rs` change;
`model.rs`, `lib.rs`, `error.rs`, the migration, and `crates/app` are not edited. §5 signatures are
frozen (crates/core/tests/api_surface.rs).

---

## Phase 1: Setup

**Purpose**: test-first baseline

- [ ] T001 Run `cargo test -p planner-core --no-fail-fast` and confirm every crates/core/tests/m2_*.rs binary compiles and its tests fail only by panicking on a `todo!("M2…")` (no compile errors, no assertion failures in M1 tests); note the failing count

---

## Phase 2: Foundational (blocking prerequisites)

**Purpose**: session row access and the session readers/writers every test helper uses
(`sessions_of` → `sessions_for_course`; `mark` → `set_status`; `add_marked_makeups` → `add_makeup`)

**⚠️ CRITICAL**: no story work starts until this phase builds cleanly under clippy

- [ ] T002 Add to crates/core/src/db.rs: `pub(crate) const SESSION_COLS`, `pub(crate) fn session_from_row(&Row) -> rusqlite::Result<Session>` (using `col_date`, `col_time`, `col_status`, `col_session_kind`), `pub(crate) fn load_session(conn, id) -> Result<Session>` (`Error::NotFound { entity: "session", id }`), `pub(crate) fn session_start(&Session) -> NaiveDateTime` / `session_end` (`date.and_time(start_time | end_time)`), and `pub(crate) fn to_minute(NaiveDateTime) -> NaiveDateTime` dropping seconds (research R-D12), with unit tests for `to_minute` in the existing `#[cfg(test)]` module
- [ ] T003 [P] Add a pure `pub(crate) fn display_status(status: Status, replaced_by: Option<NaiveDate>) -> String` in crates/core/src/schedule.rs returning "Pending" / "Attended" / "Absent" / "Cancelled", or "Rescheduled → YYYY-MM-DD" (U+2192) only when `status == Cancelled` and `replaced_by` is `Some`; unit tests in the same file for all five outputs and for `replaced_by` being ignored when not cancelled (§5, FR-018)
- [ ] T004 Implement `sessions_between(from, to, now)` and `sessions_for_course(course_id, now)` in crates/core/src/schedule.rs via one private view builder: join `course` for code/name; `replaced_by` = date of the highest-id makeup whose `replaces_session_id` is the session; `is_past = to_minute(now) >= session_start`. `sessions_between`: dates `from..=to`, non-archived courses of the active term, ordered by date, start time, id; no active term → empty. `sessions_for_course`: every session of the course, archived or not, same order; unknown course → `Error::NotFound { entity: "course" }` (research R-D8; doc comments cite R3, §5)
- [ ] T005 Implement `set_status(id, status, now)` in crates/core/src/schedule.rs: load the session (`NotFound`); for `Attended`/`Absent`, if `to_minute(now) < session_start` return `Error::Validation` and write nothing; `Pending`/`Cancelled` always written (research R-D6; doc comment `/// R11, R12: …`)
- [ ] T006 Implement `add_makeup(makeup)` in crates/core/src/schedule.rs: `Error::Validation` if `end_time <= start_time`; `Error::NotFound { entity: "course" }` if `course_id` is missing; insert `kind='makeup'`, `slot_id NULL`, `replaces_session_id NULL`, `status='pending'`, `room`/`note` from the input; return the stored `Session` (research R-D7; `/// R10: …`). Extract the insert as a private helper reused by T033

**Checkpoint**: `cargo clippy --workspace --all-targets -- -D warnings` passes; `cargo test -p planner-core --lib` passes. Commit "M2: session rows, views, status, makeups".

---

## Phase 3: User Story 1 - Weekly classes appear as dated sessions (Priority: P1) 🎯 MVP

**Goal**: sync turns slots into dated regular sessions, idempotently, honoring holidays (R4, R5)

**Independent Test**: `cargo test -p planner-core --test m2_sync`

### Tests (written first)

- [X] T007 [P] [US1] T04 in crates/core/tests/m2_sync.rs: `t04_sync_generates_six_sessions`, `t04_session_copies_slot_room`
- [X] T008 [P] [US1] T05 in crates/core/tests/m2_sync.rs: `t05_sync_twice_same_rows_same_ids`
- [X] T009 [P] [US1] T06 in crates/core/tests/m2_sync.rs: `t06_holiday_before_sync_skips_that_day`
- [X] T010 [P] [US1] T07 in crates/core/tests/m2_sync.rs: `t07_holiday_added_after_sync`
- [X] T011 [P] [US1] T14 in crates/core/tests/m2_sync.rs: `t14_session_on_term_end_date_is_generated`, `t14_open_ended_slot_reaches_term_end_date`

### Implementation

- [ ] T012 [US1] Add private `fn sync_in(conn: &Connection, today: NaiveDate) -> Result<SyncReport>` in crates/core/src/schedule.rs, generation half (research R-D1): no active term → `SyncReport::default()`; load term dates, its holiday dates, and the slots (with `coalesce(slot.room, course.room)`) of **non-archived** courses of the active term; for each slot walk `[max(term.start, valid_from), min(term.end, valid_to ?? term.end)]` inclusive, keep dates with matching weekday that are not holidays, collect the expected `(slot_id, date)` set, and `INSERT OR IGNORE` a `regular`, `pending` session copying start, end, and the resolved room (research R-D3); `created` = rows actually inserted. Doc comment `/// R3, R4: …`
- [ ] T013 [US1] Add the deletion half to `sync_in` in crates/core/src/schedule.rs: delete sessions with `kind='regular' AND status='pending' AND date >= today` whose course is non-archived and in the active term and whose `(slot_id, date)` is not in the expected set; `deleted` = rows removed. Doc comment `/// R5: sync never deletes past or non-pending sessions; FR-006: never sessions of archived courses.`
- [ ] T014 [US1] Implement `sync_sessions(today)` in crates/core/src/schedule.rs: one transaction around `sync_in(&tx, today)`, commit, return the report (doc comment cites R4, R5, R6)

**Checkpoint**: m2_sync passes. Commit "M2: session sync R4, R5".

---

## Phase 4: User Story 2 - Change or end a weekly slot without rewriting history (Priority: P1)

**Goal**: slot changes take effect from a date; past sessions are snapshots (R7)

**Independent Test**: `cargo test -p planner-core --test m2_slot_changes`

### Tests (written first)

- [X] T015 [P] [US2] T10 in crates/core/tests/m2_slot_changes.rs: `t10_change_slot_weekday_effective_mid_term`
- [X] T016 [P] [US2] T11 in crates/core/tests/m2_slot_changes.rs: `t11_change_slot_time_keeps_past_session_times`
- [X] T017 [P] [US2] T27 in crates/core/tests/m2_slot_changes.rs: `t27_end_slot_removes_later_pending_sessions`, `t27_end_slot_syncs_internally`

### Implementation

- [ ] T018 [US2] Implement `change_slot(id, input, effective_from)` in crates/core/src/schedule.rs (research R-D4): `input.validate()?` (R2, as `add_slot`); load the old slot (`NotFound { entity: "slot" }`); `effective_from.pred_opt()` missing → `Error::Validation`; in one transaction set old `valid_to = effective_from - 1 day`, insert the new slot from `input` with `valid_from = effective_from` (input.valid_from ignored), run `sync_in(&tx, effective_from)`, commit, return the new `Slot`. Doc comment `/// R7: …`
- [ ] T019 [US2] Implement `end_slot(id, last_date)` in crates/core/src/schedule.rs (research R-D5): `NotFound { entity: "slot" }` if missing; in one transaction set `valid_to = last_date`, run `sync_in(&tx, last_date + 1 day)`, commit. Doc comment `/// R7, FR-008: …`

**Checkpoint**: m2_slot_changes and m2_sync pass. Commit "M2: change_slot and end_slot R7".

---

## Phase 5: User Story 3 - Mark attendance and see the attendance percentage (Priority: P1)

**Goal**: status marking rules and per-course summaries (R11–R15)

**Independent Test**: `cargo test -p planner-core --test m2_attendance`

### Tests (written first)

- [X] T020 [P] [US3] T01 in crates/core/tests/m2_attendance.rs: `t01_percentage_from_mixed_statuses`
- [X] T021 [P] [US3] T02 in crates/core/tests/m2_attendance.rs: `t02_only_cancelled_and_pending_has_no_percentage`
- [X] T022 [P] [US3] T03 in crates/core/tests/m2_attendance.rs: `t03_below_target_boundaries`
- [X] T023 [P] [US3] T12 in crates/core/tests/m2_attendance.rs: `t12_attended_on_future_session_rejected`, `t12_attended_allowed_at_exact_start`
- [X] T024 [P] [US3] T21 in crates/core/tests/m2_attendance.rs: `t21_course_without_slots_has_empty_summary`
- [X] T025 [P] [US3] T23 in crates/core/tests/m2_attendance.rs: `t23_attended_back_to_pending_updates_counts`

### Implementation

- [ ] T026 [P] [US3] Add pure helpers in crates/core/src/attendance.rs: `pub(crate) fn pct(attended: u32, absent: u32) -> Option<f64>` = `(attended / (attended + absent) × 1000).round() / 10`, `None` when the denominator is 0 (R13); `pub(crate) fn below_target(pct: Option<f64>, target: Option<i64>) -> bool` = `target.is_some() && pct.is_some() && pct < target` on the rounded pct (R14, research R-D9); unit tests: 3/1 → 75.0, 0/0 → None, 2/1 → 66.7, (74.9, 75) → true, (75.0, 75) → false, (Some, None) → false
- [ ] T027 [US3] Implement `attendance_summary(course_id)` in crates/core/src/attendance.rs: load the course (`NotFound { entity: "course" }`; archived allowed), count all its sessions (regular + makeup, past + future) by status, fill `pct`, `target = min_attendance_pct`, `below_target` (R13–R15)
- [ ] T028 [US3] Implement `all_summaries()` in crates/core/src/attendance.rs: `attendance_summary` for each non-archived course of the active term, ordered by code; no active term → empty (R3, R15, research R-D9)

**Checkpoint**: m2_attendance passes. Commit "M2: attendance summaries R13–R15".

---

## Phase 6: User Story 4 - Cancel, reschedule, and add makeup classes (Priority: P2)

**Goal**: exceptions that survive re-sync (R8–R10)

**Independent Test**: `cargo test -p planner-core --test m2_exceptions`

### Tests (written first)

- [X] T029 [P] [US4] T08 in crates/core/tests/m2_exceptions.rs: `t08_cancelled_session_survives_resync`
- [X] T030 [P] [US4] T09 in crates/core/tests/m2_exceptions.rs: `t09_reschedule_creates_makeup_and_cancels_original`
- [X] T031 [P] [US4] T19 in crates/core/tests/m2_exceptions.rs: `t19_reschedule_attended_session_is_error`, `t19_reschedule_absent_session_is_error`
- [X] T032 [P] [US4] T22 in crates/core/tests/m2_exceptions.rs: `t22_standalone_makeup_counts_as_attended`

### Implementation

- [ ] T033 [US4] Implement `reschedule(id, makeup)` in crates/core/src/schedule.rs (research R-D7): in one transaction load the original (`NotFound { entity: "session" }`); `Attended`/`Absent` → `Error::Validation`, nothing written; makeup `end_time <= start_time` → `Error::Validation`; insert the makeup via the T006 helper with `replaces_session_id = id`; set the original to `cancelled`; commit; return the makeup. Doc comment `/// R9: …`
- [ ] T034 [US4] Confirm T08 (cancel survives re-sync, R8) passes with no extra code: `INSERT OR IGNORE` keeps the cancelled row and deletion skips non-pending rows; if not, fix `sync_in` in crates/core/src/schedule.rs only

**Checkpoint**: m2_exceptions passes. Commit "M2: reschedule and makeups R8–R10".

---

## Phase 7: User Story 5 - Dashboard shows what matters now (Priority: P2)

**Goal**: dashboard built for a given now (R16–R20, R14)

**Independent Test**: `cargo test -p planner-core --test m2_dashboard`

### Tests (written first)

- [X] T035 [P] [US5] T13 in crates/core/tests/m2_dashboard.rs: `t13_awaiting_excludes_in_progress_includes_at_end`
- [X] T036 [P] [US5] T16 in crates/core/tests/m2_dashboard.rs: `t16_overdue_only_incomplete_and_strictly_before_now`
- [X] T037 [P] [US5] T17 in crates/core/tests/m2_dashboard.rs: `t17_upcoming_includes_due_at_window_edge`
- [X] T038 [P] [US5] T18 in crates/core/tests/m2_dashboard.rs: `t18_next_class_skips_cancelled_session`, `t18_session_starting_now_is_not_next`
- [X] T039 [P] [US5] T29 in crates/core/tests/m2_dashboard.rs: `t29_today_sorted_by_start_time_across_courses`

### Implementation

- [ ] T040 [P] [US5] Replace the `todo!` arms of `list_assessments` in crates/core/src/db.rs (research R-D11): `Upcoming { now }` = incomplete with `now <= due_at <= now + upcoming_window_days` days (setting via `self.settings()`); `Overdue { now }` = incomplete with `due_at < now`; both with `now` truncated by `to_minute`, M1's non-archived active-term scope, ordered by `due_at`, id (R18, R19)
- [ ] T041 [US5] Implement `dashboard(now)` in crates/core/src/dashboard.rs (research R-D10): no active term → empty `Dashboard`; `today` = `sessions_between(now.date(), now.date(), now)` sorted by start time, course code, id (R16); `next_class` = earliest pending view of a non-archived course with `session_start > to_minute(now)` (R17); `upcoming` / `overdue` from T040 (R18, R19); `awaiting` = pending views of non-archived courses with `session_end <= to_minute(now)`, ordered by date, start, id (R20); `below_target` = `all_summaries()` entries with `below_target` (R14). Doc comment cites R3, R14, R16–R20

**Checkpoint**: m2_dashboard passes. Commit "M2: dashboard R16–R20".

---

## Phase 8: User Story 7 - Midterm and final exam periods have no classes (Priority: P2)

**Goal**: exam periods entered as holidays remove classes and attendance expectations (FR-006a)

**Independent Test**: `cargo test -p planner-core --test m2_exam_periods`

### Tests (written first)

- [X] T042 [P] [US7] User Story 7 scenarios 1–4 in crates/core/tests/m2_exam_periods.rs: `exam_period_holidays_remove_future_pending_sessions`, `exam_period_exams_appear_as_assessments_and_prep_days_are_class_free`, `exam_period_removing_a_date_restores_its_sessions`, `exam_period_keeps_session_cancelled_before_period_was_entered`

### Implementation

- [ ] T043 [US7] Confirm m2_exam_periods passes with no new code (exam periods are holidays, R4/R5); any fix goes in `sync_in` in crates/core/src/schedule.rs, with no schema, API, or rule change

**Checkpoint**: m2_exam_periods passes. Commit only if T043 changed code ("M2: exam periods as holidays").

---

## Phase 9: User Story 6 - Archived courses stop generating classes but keep history (Priority: P3)

**Goal**: archive hides a course and freezes its sessions; unarchive resumes without duplicates (R3, FR-006)

**Independent Test**: `cargo test -p planner-core --test m2_archive`

### Tests (written first)

- [X] T044 [P] [US6] T15 in crates/core/tests/m2_archive.rs: `t15_archive_hides_course_but_keeps_history`, `t15_archived_course_gets_no_new_sessions`
- [X] T045 [P] [US6] T28 in crates/core/tests/m2_archive.rs: `t28_generation_resumes_after_unarchive`, `t28_unarchive_and_sync_creates_no_duplicates`

### Implementation

- [ ] T046 [US6] Confirm m2_archive passes: archived courses are excluded in `sync_in` generation and deletion (T012, T013), `sessions_between`, `dashboard`, and `all_summaries`, while `attendance_summary` and `sessions_for_course` still work for them; fix only in crates/core/src/{schedule.rs, attendance.rs, dashboard.rs}

**Checkpoint**: m2_archive passes. Commit only if T046 changed code ("M2: archived courses R3").

---

## Phase 10: Polish, converge & gate

- [ ] T047 Grep crates/core/src for `todo!("M2` and confirm none remains (only M3/M5 stubs stay); run `cargo doc -p planner-core --no-deps` and confirm every new item has a doc comment citing its rule IDs
- [ ] T048 Run `/speckit-converge` for specs/002-m2-sessions-attendance and complete any tasks it appends to this file
- [ ] T049 Run the gate `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` and paste its summary; confirm T01–T19, T21–T23, T27–T29, the m2_exam_periods tests, and M1's T20, T24, T26 pass (§7 M2 done-when)
- [ ] T050 Update .claude/state/progress.json (all tasks) and .claude/state/checkpoint.md (last green gate hash, next milestone M3); commit "M2: milestone complete"

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (T001) → Foundational (T002–T006): every test file reads sessions through `sessions_for_course`; m2_attendance, m2_exceptions, m2_dashboard use `set_status` and `add_makeup` via helpers
- US1 (T012–T014) needs Foundational; every later story calls `sync_sessions`
- US2 (T018–T019) needs `sync_in` from US1
- US3 (T026–T028) needs Foundational and US1
- US4 (T033–T034) needs US1 and the T006 insert helper; T09/T22 also read summaries (US3)
- US5 (T040–T041) needs US1 and US3 (`all_summaries` for `below_target`)
- US7 (T043) needs US1, US3, US5; US6 (T046) needs US1, US3, US5
- Polish (T047–T050) last

### Within Each Story

- Tests are already written and failing; implement until they pass, then commit at the checkpoint
- Update progress.json after every task

### Parallel Opportunities

- T003 ∥ T002 (different files)
- T026 can be written any time after Setup (pure functions)
- T040 (db.rs) ∥ T033 (schedule.rs)

## Parallel Example: User Story 5

```text
Task: "Replace Upcoming/Overdue todo! arms of list_assessments in crates/core/src/db.rs"
Task: "Implement reschedule in crates/core/src/schedule.rs" (US4, different file)
```

## Implementation Strategy

### MVP first

Phases 1–3 deliver idempotent session sync with holidays (T04–T07, T14): the base every other
story reads.

### Incremental delivery

US1 → US2 → US3 → US4 → US5 → US7 → US6, keeping clippy green at each checkpoint so the final gate
(T049) is a formality.

## Notes

- Single agent, sequential: [P] marks independence, not a request to spawn subagents (§12.4).
- No task adds a requirement outside `docs/spec.md` plus the recorded human and router decisions
  (spec.md, research.md).
