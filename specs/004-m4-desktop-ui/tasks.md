---

description: "Task list for M4 desktop UI"
---

# Tasks: M4 Desktop UI

**Input**: Design documents from `specs/004-m4-desktop-ui/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/viewmodel.md](contracts/viewmodel.md),
[contracts/ui-bridge.md](contracts/ui-bridge.md), [quickstart.md](quickstart.md)

**Tests**: Required (constitution II, spec FR-005, SC-003). The tester writes the view-model tests
(Phase 3) and the pre-restore copy tests (Phase 6) before the matching implementation. The builder
does not edit tester-owned test files except to fix a test that contradicts spec.md or a contract
(then ask the router). App test names are descriptive (no `t_` prefix required); doc comments cite
rule IDs.

**Organization**: Phases follow the router's layer order so builders can work in parallel per page:
dependencies → theme + CI grep → view model (tests first) → shell and controller → one task per
page → R22 pre-restore copy → R6 timer → converge and gate. Story labels (`[US1]`–`[US6]`, from
spec.md) trace page tasks to user stories; a page that serves several stories carries its main one
and names the rest in the description.

## Format: `- [ ] [ID] [P?] [Story?] [owner] Description`

- **[P]**: can run in parallel with the other [P] tasks of the same phase (no shared file, no
  dependency on an incomplete task)
- **[Story]**: user story from spec.md; setup, foundational, and polish tasks have none
- **[tester]** / **[builder]**: which agent does the task; **[router]** / **[human]** for checks only
  they can run

## Path Conventions

Rust workspace. App crate: `crates/app/` (`src/`, `ui/`, `build.rs`, `Cargo.toml`). CI:
`.github/workflows/ci.yml`. `crates/core/` is **not edited** (§5 API, schema, and rules frozen).
Every builder task ends with `cargo build -p app` and
`cargo clippy -p app --all-targets -- -D warnings` passing.

---

## Phase 1: Setup (dependencies)

**Purpose**: everything later phases compile against.

- [X] T001 [builder] In crates/app/Cargo.toml add `rfd` (latest release that builds on Rust 1.90, default features; record the exact version), `chrono = { workspace = true, features = ["clock"] }`, and `rusqlite.workspace = true` (research R-D1, R-D2, R-D3); in crates/app/build.rs switch to `slint_build::compile_with_config("ui/app-window.slint", slint_build::CompilerConfiguration::new().with_style("fluent-light".into()))` (research R-D7); let `Cargo.lock` update; run `cargo build -p app`. If the Linux CI build later fails for a missing system library, add it to the existing `apt-get install` line in .github/workflows/ci.yml

---

## Phase 2: Theme (FR-001, FR-002)

**Purpose**: every visual value exists in `Theme` before any page is built.

- [X] T002 [P] [builder] Rewrite crates/app/ui/theme.slint as `export global Theme` with one `out property` per DESIGN.md token, values copied exactly from the DESIGN.md front matter (research R-D13, data-model.md "Theme"): the 21 colors (`primary` #B4532F … `below-target-bg` #F6E3D6); `font-heading: "Georgia"`, `font-body: ""`; `title-lg-size` 22px / `title-lg-weight` 400, `headline-md` 16px/400, `body-md` 13px/400, `body-sm` 12px/400, `label-md` 12px/600, `label-sm` 11px/600 (line heights as comments); `radius-none` 0px, `radius-sm` 6px, `radius-md` 10px, `radius-full` 9999px; `space-xs` 4px … `space-xxl` 32px; `nav-width` 208px, `row-height` 34px, `input-height` 30px, `window-width` 1100px, `window-height` 720px; `divider-width` 1px, `border-width` 1px. Keep the names `neutral`, `window-width`, `window-height` used by crates/app/ui/app-window.slint. Header comment: "Design tokens from DESIGN.md. The only place colors and sizes are defined."
- [X] T003 [P] [builder] In .github/workflows/ci.yml add a step named "Theme grep check" to the `check` job (after Format) running exactly `! grep -rnE '#[0-9a-fA-F]{6}|[0-9]+px' crates/app/ui --include=*.slint | grep -v theme.slint` (§12.3, FR-002); run the command locally and confirm exit code 0

---

## Phase 3: View model, tests first (FR-005, SC-003)

**Purpose**: all formatting, parsing, and validation as pure functions, proven by unit tests before
any page uses them. Contract: [contracts/viewmodel.md](contracts/viewmodel.md).

- [X] T004 [tester] Create crates/app/src/viewmodel.rs with every public type and function signature from contracts/viewmodel.md, types complete, function bodies `todo!()`, a module doc comment citing §2, rule-ID doc comments (`/// R13: …`), a temporary `#![allow(dead_code)] // removed in T026 once pages use every item`, and `#[cfg(test)] mod tests;`; add `mod viewmodel;` to crates/app/src/main.rs. `cargo build -p app` passes
- [X] T005 [tester] Write crates/app/src/viewmodel/tests.rs with at least one test per public item of contracts/viewmodel.md, using every example in its tables as a case, building core types directly (no database). Must include: `format_pct` (`50.0%`, `66.7%`, `100.0%`, `0.0%`, `—` for `None`, R13); `status_label`/`chip_kind` for all four statuses and a rescheduled original (`display_status` "Rescheduled → 2026-01-15" with `Cancelled` chip); `can_mark_attendance` true/false from `is_past` (R12); `parse_date`/`parse_time` accepted and rejected inputs (including empty) with exact error texts that state the format (FR-007); `parse_target`, `parse_optional_minutes`, `parse_count`; every `parse_*_form` reporting all field errors at once and passing R2-owned checks through to core (empty course code is `Ok`); `expand_holiday_range` single day, 5-day inclusive range with one label, `end < start` error; `merge_holidays` overlap (one holiday per date, range label wins, sorted); `remove_holiday` removes only one date; `week_start` for Monday, mid-week, and Sunday 2026-11-01 → 2026-10-26; `shift_week` across a month and a year end; `week_title`; `group_week` order and titles; `banner_line` "CSE101 — 66.7% (target 75%)" (R14); `summary_view`; `dashboard_view` keeps order and builds banner lines; row builders' "—" for missing values; `slot_row` validity text both forms; `assessment_filter` per choice; `error_text` for `Validation` (capitalized) and `SchemaTooNew` (R25); `pre_restore_file_name` "pre-restore-20261029-1430.sqlite" and `free_pre_restore_name` with no clash, one clash (`-2`), two clashes (`-3`), and unrelated names (R22, Q2); `backup_file_name`; `date_rolled_over` at 23:59 same day (false) and 00:00 next day (true) (R6)
- [X] T006 [tester] Run `cargo test -p app --no-fail-fast` and confirm the tests compile and every view-model test fails only by panicking on `todo!()`; `cargo clippy -p app --all-targets -- -D warnings` passes; all core tests still green. Needs T004, T005
- [X] T007 [builder] Implement in crates/app/src/viewmodel.rs the sections "Status and percentages", "Dates and times", "Text and numbers", and "Forms" of contracts/viewmodel.md (research R-D9, R-D10, R-D11 for exact strictness and messages). Needs T006
- [X] T008 [builder] Implement in crates/app/src/viewmodel.rs the sections "Holidays", "Rows", "Assessment filters", "Week view", "Dashboard", and "Errors, files, timer" of contracts/viewmodel.md; `cargo test -p app` fully green. Needs T007

**Checkpoint**: view model complete and green. Router commits "M4: view model with unit tests".

---

## Phase 4: Shell and controller (foundational, blocks every page)

**Purpose**: shared Slint types, components, the window with nav and stub pages, and the Rust
controller, so each page task afterwards touches only its own two files. Contract:
[contracts/ui-bridge.md](contracts/ui-bridge.md).

- [ ] T009 [builder] Create crates/app/ui/types.slint with the enums `Chip`, `StatusAction`, the structs `SessionRow`, `DayGroup`, `AssessmentRow`, `CourseRow`, `SlotRow`, `HolidayRow`, `SummaryView`, and `global AppState` exactly as in contracts/ui-bridge.md
- [ ] T010 [builder] (FR-003, FR-007) Create crates/app/ui/components.slint with `NavItem`, `PrimaryButton`, `SecondaryButton` (`enabled`), `Field` (label above, `in-out text`, `error` below in `Theme.error` with error border when non-empty), `StatusChip` (pill, text always shown, colors by `Chip`), `SectionCard`, `ListRow`, `EmptyState`, `Banner` (`lines: [string]`, below-target colors), and `PageTitle`, built from base elements with every color, size, spacing, radius, and font from `Theme.*` per DESIGN.md "Components" (research R-D7). Needs T002, T009
- [ ] T011 [builder] Create stub page files crates/app/ui/dashboard.slint, crates/app/ui/timetable.slint, crates/app/ui/courses.slint, crates/app/ui/assessments.slint, crates/app/ui/settings.slint, each exporting `global <Name>State { in property <string> message; }` and `component <Name>Page` with a `PageTitle` and an `EmptyState`; rewrite crates/app/ui/app-window.slint: `Theme.nav-width` nav column on `Theme.surface-alt` with five `NavItem`s bound to `AppState.current-page` and calling `AppState.page-selected`, the current page in a scrollable `Theme.neutral` content area with `Theme.space-xl` padding, and only the `AppState.startup-error` text when it is non-empty (FR-004, FR-008); re-export `AppState` and the five page globals. Run the theme grep check. Needs T010
- [ ] T012 [builder] (FR-006, FR-009) Create crates/app/src/controller.rs with `App` and `UiState` as in data-model.md "UI state" and the methods listed in contracts/ui-bridge.md "Controller": `now()`, `today()`, `planner()`, `planner_mut()`, `window()`, `state()`, `changed(sync: bool)` (R6: `sync_sessions(today)` then `refresh_all`; a sync error goes to the current page's `message` and the already-committed mutation stands), `refresh_all()` (fresh `now`, calls every `pages::*::refresh`). Doc comments cite R6. Needs T008, T011
- [ ] T013 [builder] Create crates/app/src/bridge.rs (viewmodel rows → Slint structs, `i64 → i32` via `i32::try_from`, `ChipKind → Chip`, `StatusAction → planner_core::Status`; research R-D18) and crates/app/src/pages/mod.rs plus stub modules crates/app/src/pages/dashboard.rs, timetable.rs, courses.rs, assessments.rs, settings.rs, each with empty `pub fn refresh(app: &App)` and `pub fn wire(app: &Rc<App>)`. Needs T012
- [ ] T014 [builder] Rewrite crates/app/src/main.rs: `fn local_now() -> NaiveDateTime` as the only clock read, annotated `#[allow(clippy::disallowed_methods)]` with the reason comment (research R-D1); declare modules; resolve the path with `planner_core::default_db_path()`; on any open error (incl. `SchemaTooNew`, R25) set `AppState.startup-error` to `viewmodel::error_text` and run the window without pages (FR-008, no panic); otherwise build `App`, sync with today (R6 start trigger), call every `pages::*::wire`, connect `AppState.page-selected` to `refresh_all`, `refresh_all`, run. `cargo run -p app` with `ACADEMIC_PLANNER_DB` set to a scratch file shows the nav and five empty pages (quickstart 5). Needs T013

**Checkpoint**: window, nav, empty pages; gate green. Router commits "M4: shell, components, controller".

---

## Phase 5: Pages (one task per page; all [P])

**Purpose**: the five §6 pages. Each task owns exactly `crates/app/ui/<page>.slint` and
`crates/app/src/pages/<page>.rs`, follows its section of contracts/ui-bridge.md, uses only
`components.slint` + `Theme.*`, takes every displayed string from `viewmodel.rs` via `bridge.rs`
(no formatting in `.slint` or in the page module), shows each list's empty-state sentence, and runs
the theme grep check before finishing. A page task needing a new shared component or struct asks the
router instead of editing shared files. All need T014.

- [ ] T015 [P] [US3] [builder] Dashboard in crates/app/ui/dashboard.slint and crates/app/src/pages/dashboard.rs (FR-011, FR-012, FR-013, SC-002): below-target `Banner` hidden when `banner` is empty (R14); sections "Today's classes" (R16), "Next class" (R17, `has-next`), "Upcoming" (R18), "Overdue" (R19), "Awaiting attendance" (R20) with Attended / Absent (enabled only when `can-mark`, R12) / Cancelled buttons calling `set_status(id, status, now)` then `changed(false)`; every section has its empty-state sentence. Independent test: US3 scenarios 1–5
- [ ] T016 [P] [US4] [builder] Timetable in crates/app/ui/timetable.slint and crates/app/src/pages/timetable.rs (FR-014, FR-015, FR-016; also US1 scenarios 4–6): tabs Week / Rules; Week: previous / next / today buttons over `UiState.week_start`, `week_title`, `sessions_between(week_start, week_end, now)` → `group_week` (research R-D17), rows with time, course, room, `StatusChip` + `status-label` (incl. "Rescheduled → {date}"); selecting a row opens an inline panel with Attended / Absent (disabled unless `can-mark`, R12), Pending, Cancel (R8), the stored note read-only, and a reschedule form (date, start, end, room, optional note; pre-filled per research R-D16) → `parse_makeup_form` with the session's course id → `reschedule` (R9; core error shown in `message`) → `changed(false)`; week empty state (US4-6). Rules: `list_slots(today)` rows; Add slot form (course combo, weekday combo, start, end, room, valid from pre-filled with the active term start, valid to optional) → `add_slot` → `changed(true)`; Edit asks for an effective date pre-filled with today → `change_slot` → `changed(true)` (R7); End asks for the last date pre-filled with today → `end_slot` → `changed(true)` (R5); field errors under fields, core R2 errors as the form error; Rules empty state
- [ ] T017 [P] [US2] [builder] Courses in crates/app/ui/courses.slint and crates/app/src/pages/courses.rs (FR-017, FR-018, FR-019; also US1 scenarios 3 and 7, US4 scenario 5): list from `list_courses(show_archived)` with the show-archived toggle; create / edit form (code, name, instructor, room, notes, target) → `create_course` / `update_course` → `changed(true)`; Archive / Unarchive → `set_archived` → `changed(true)` (R3, R6); course detail: `summary_view(attendance_summary(id))` with counts, pct or "—", target, and a `Below target` chip when `below-target` (R13–R15), the full `sessions_for_course(id, now)` list with Attended / Absent (disabled unless `can-mark`) / Cancelled / Pending buttons → `set_status` → `changed(false)`, and an "Add makeup class" form (date, start, end, room, note) → `parse_makeup_form` → `add_makeup` → `changed(false)` (R10); the session list scrolls; empty states for no courses and no sessions
- [ ] T018 [P] [US5] [builder] Assessments in crates/app/ui/assessments.slint and crates/app/src/pages/assessments.rs (FR-020): filter control upcoming / overdue / completed / all → `list_assessments(assessment_filter(choice, now))`; rows with course code, kind, title, due, notes, and a complete `CheckBox` → `set_completed` → `changed(false)`; create / edit form (course combo, kind combo quiz / exam / assignment / other, title, due date, due time, notes, reminder minutes) → `parse_assessment_form` → `create_assessment` / `update_assessment` → `changed(false)`; Delete → `delete_assessment` → `changed(false)`; empty state per filter. Independent test: US5 scenarios 1–4
- [ ] T019 [P] [US6] [builder] Settings without restore in crates/app/ui/settings.slint and crates/app/src/pages/settings.rs (FR-021, FR-022, FR-023, FR-024, FR-026, FR-027; also US1 scenarios 1–2): term form (name, start, end; editing the active term passes its id) → `parse_term_form` → `save_term` → `changed(true)` (R1, R2, R6); holiday list (`holiday_row`, a Remove button per date → `remove_holiday` → `set_holidays` → `changed(true)`) and holiday form (start date, optional end date, label) → `parse_holiday_form` → `merge_holidays` with the active term's holidays → `set_holidays` → `changed(true)`; without an active term show `error_text(&Error::NoActiveTerm)` (human decision §6; spec edge cases); settings form (class reminder minutes, upcoming window days, notifications `Switch`) → `parse_settings_form` → `save_settings` → `changed(false)`; Backup → `rfd` save dialog with default name `backup_file_name(now)` → write `export_json(now)` (R21); Export CSV → folder dialog → write `attendance.csv` and `assessments.csv` (R23); DB path text via `db_path_text` (R24); Restore button and the restore callbacks declared but left as no-ops (wired in T022); cancelling any dialog does nothing

**Checkpoint**: all pages work on a scratch DB; gate green. Router commits per page ("M4: dashboard page", …).

---

## Phase 6: R22 pre-restore copy and restore flow (FR-025, SC-007)

- [ ] T020 [builder] Create crates/app/src/restore.rs with `pub fn pre_restore_copy(db_path: &Path, now: NaiveDateTime) -> anyhow::Result<PathBuf>` (body `todo!()`, doc comment `/// R22: …`, temporary `#![allow(dead_code)]` removed in T026) and add `mod restore;` to crates/app/src/main.rs. No test file: per Principle II (in-memory SQLite only, Q1 = B) the copy's file behavior is verified manually in quickstart §6; its file name and same-minute suffix are covered by the `free_pre_restore_name` view-model tests. `cargo build -p app` passes. Needs T014
- [ ] T021 [builder] Implement `pre_restore_copy` in crates/app/src/restore.rs per research R-D3/R-D4: `{db_path.parent()}/backups` created if missing, existing file names listed, name from `viewmodel::free_pre_restore_name(now, &existing)`, `VACUUM INTO` on a second `rusqlite::Connection` to `db_path`, returns the copy path. `cargo test -p app` green; quickstart §6 restore checks pass by hand. Needs T020
- [ ] T022 [US6] [builder] Wire Restore in crates/app/ui/settings.slint and crates/app/src/pages/settings.rs per contracts/ui-bridge.md "Restore flow": `restore-pick` → `rfd` open dialog (json) → store path, show the inline confirm panel warning that all current data will be replaced; `restore-cancel` → clear, nothing copied; `restore-confirm` → read file (error → message, stop) → `pre_restore_copy(db_path, now)` (error → message, restore not attempted) → `restore_json` (error → `error_text`, data unchanged) → on success `changed(true)` (R6 after restore) and message `restored_message(copy)`. Needs T019, T021

---

## Phase 7: R6 date-rollover timer (FR-010)

- [ ] T023 [builder] Add `pub fn tick(&self)` to crates/app/src/controller.rs (`now = clock()`; if `viewmodel::date_rolled_over(last_sync, now)` sync with `now.date()` and store it; then `refresh_all`, which also refreshes now-dependent lists) and in crates/app/src/main.rs start a `slint::Timer` in `TimerMode::Repeated` every 60 s calling `tick`, kept alive until the window closes (research R-D15). Doc comments cite R6. Needs T014 (may run after Phase 5 to avoid editing main.rs during T020)

---

## Phase 8: Converge and gate

- [ ] T024 [builder] (FR-002, SC-004) Run quickstart steps 2 and 4: theme grep check (zero hits) and clock grep (exactly one hit, in crates/app/src/main.rs under the allow). Fix any hit in the offending `.slint`/`.rs` file by using `Theme.*` or the clock parameter. Needs T015–T023
- [ ] T025 [builder] Review every `.slint` file under crates/app/ui for logic beyond bindings and callbacks (string building, conditionals that compute text, arithmetic on data) and move it into crates/app/src/viewmodel.rs with tests requested from the tester first (FR-005). Needs T024
- [ ] T026 [builder] Remove the temporary `#![allow(dead_code)]` from crates/app/src/viewmodel.rs and crates/app/src/restore.rs; wire or delete each item then reported unused (deleting a contract item also deletes its tests and updates contracts/viewmodel.md, with router approval). Needs T025
- [ ] T027 [builder] Run the gate `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` and `npx -y @google/design.md@0.4.0 lint DESIGN.md`; paste both summaries (FR-028, constitution V). Needs T026
- [ ] T028 [router] Screenshot critique per §12.3 (FR-029, SC-005): one screenshot per page plus one empty-state screenshot (scratch DB), `impeccable` against DESIGN.md; builder fixes each high-severity item in crates/app/ui/theme.slint or layout only (never hardcoded values), then T027 is re-run. If `impeccable` is not installed, the router raises it with the human. Needs T027
- [ ] T029 [human] Run quickstart steps 6–8 (§9 smoke checklist plus holiday range, pre-restore copy, same-minute suffix, inline format errors, R12 enablement, rollover, newer DB refused) and record pass/fail (FR-030, SC-001, SC-006, SC-007). Needs T028

---

## Dependencies & Execution Order

- Phase 1 → Phase 2 (T002 ∥ T003) → Phase 3 (T004 → T005 → T006 → T007 → T008) → Phase 4
  (T009 → T010 → T011 → T012 → T013 → T014) → Phase 5 (T015 ∥ T016 ∥ T017 ∥ T018 ∥ T019) →
  Phase 6 (T020 → T021 → T022; T022 also needs T019) → Phase 7 (T023) → Phase 8 (T024 → … → T029).
- T020 and T023 both edit crates/app/src/main.rs; run them in sequence, not in parallel.
- Phase 3 can start right after T001 (it does not need the theme); Phase 2 and Phase 3 may overlap
  (different files) except that T004 edits main.rs, which no Phase 2 task touches.

### User story coverage

| Story | Tasks |
| --- | --- |
| US1 term, courses, timetable (P1) | T016 (Rules), T017, T019 (term) |
| US2 attendance and percentage (P1) | T015, T016, T017 |
| US3 dashboard (P1) | T015 |
| US4 week view exceptions (P2) | T016, T017 (makeup) |
| US5 assessments (P2) | T018 |
| US6 settings, holidays, backup, restore, export (P2) | T019, T020–T022 |
| Cross-cutting (R6 sync, theme, view model, gate) | T001–T014, T023–T029 |

## Parallel Example

```text
After T014, launch five builders at once:
  T015 dashboard.slint + pages/dashboard.rs
  T016 timetable.slint + pages/timetable.rs
  T017 courses.slint + pages/courses.rs
  T018 assessments.slint + pages/assessments.rs
  T019 settings.slint + pages/settings.rs
Meanwhile a builder can do T020 (restore.rs skeleton, one line in main.rs).
```

## Implementation Strategy

1. MVP slice: Phases 1–4, then T019 (term) + T017 (courses) + T016 (timetable) + T015 (dashboard)
   = §9 steps up to "course shows 50.0%".
2. Then T018 and Phases 6–7 for the rest of §9.
3. Converge (Phase 8). M4 is done only when the gate, grep check, DESIGN.md lint, critique, and the
   human §9 run all pass.
