---

description: "Task list for M5 reminders"
---

# Tasks: M5 Reminders

**Input**: Design documents from `specs/005-m5-reminders/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/core-api-m5.md](contracts/core-api-m5.md), [contracts/app-notify.md](contracts/app-notify.md)

**Tests**: Required (constitution II: tests before the rule; §7 "reminder dedupe tests pass"). All
core test tasks are in Phase 2 and are done by the **tester** before any implementation task. The
**builder** makes them pass and does not edit the test file except to fix a test that contradicts
spec.md (then ask the human). M5 has no T-IDs, so tests are named `m5_…`; each test's doc comment
cites the spec FR id(s) and §5, e.g. `/// FR-001, §5: a due reminder is returned once and logged.`

**Organization**: Phase 1 setup (none needed), Phase 2 tests first, then one phase per user story
(all P1), then polish. All core tests use `Planner::open_in_memory()` through the §8 fixture.

## Format: `[ID] [P?] [Story] (role) Description`

- **[P]**: can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: user story from spec.md (US1–US3)
- **(tester)** / **(builder)** / **(human)**: who does the task

## Path Conventions

Core: `crates/core/src/`, integration tests `crates/core/tests/`; app: `crates/app/src/`. Files that
change: new `crates/core/src/reminders.rs`, `crates/core/src/backup.rs` (stub removed),
`crates/core/src/lib.rs` (`mod reminders;`), new `crates/core/tests/m5_reminders.rs`,
`crates/app/Cargo.toml`, `Cargo.lock`, new `crates/app/src/notify.rs`, `crates/app/src/main.rs`,
`crates/app/src/controller.rs`, and `.github/workflows/ci.yml` only if research R-1's check
requires it. Not edited: `model.rs`, `db.rs`, migrations, `tests/common/mod.rs`,
`tests/api_surface.rs`, any `.slint` file, `viewmodel.rs`. §5 signatures are frozen.

---

## Phase 1: Setup

No shared setup: core needs no new dependency, and the app dependency belongs to US3 (T009).

---

## Phase 2: Tests first (blocking: no core implementation task starts until T004 passes)

**Purpose**: write every M5 core test while `due_reminders` is still `todo!("M5: reminders")`.
T001–T003 write the same file, so they run in order.

All tests live in crates/core/tests/m5_reminders.rs, start with `mod common;` and `use common::*;`,
and use `synced_fixture()` (term 2026-01-05 → 2026-01-25; `CSE101` "Intro to Programming", course
room `A-101`; slot A Mon 09:00–10:00, slot B Wed 11:00–12:00; synced on 2026-01-05, so slot A has
sessions on 01-05, 01-12, 01-19 and slot B on 01-07, 01-14, 01-21), `regular_on`, `makeup_input`,
`assessment_input`, `dt`, and `d`. For a `now` with seconds use
`d("2026-01-05").and_hms_opt(9, 0, 30).unwrap()`. Change settings with
`p.save_settings(&Settings { class_reminder_minutes: n, ..Settings::default() })`. Expected session
titles follow [contracts/core-api-m5.md](contracts/core-api-m5.md), e.g.
`"CSE101 Intro to Programming at 09:00 in A-101"`.

- [X] T001 [US1] (tester) Create crates/core/tests/m5_reminders.rs with the session tests: `m5_session_reminder_fires_once` (at `2026-01-05T08:45` returns exactly one `Reminder { kind: Session, ref_id: <01-05 session id>, fire_at: 2026-01-05T08:45, title: "CSE101 Intro to Programming at 09:00 in A-101" }`; calls at 08:46 and 08:59 return empty); `m5_session_reminder_not_before_fire_at` (08:44 returns empty; then 08:45 returns the reminder, proving the early call logged nothing); `m5_missed_session_reminder_skipped` (first call at 2026-01-05T09:00 returns empty, 09:30 empty; the 01-07 11:00 class still fires once at 2026-01-07T10:45); `m5_cancelled_session_no_reminder` (`set_status(<01-05 id>, Status::Cancelled, dt("2026-01-05T08:00"))`, then 08:45 empty); `m5_rescheduled_session_reminds_makeup_only` (`reschedule(<01-05 id>, makeup_input(cse.id, "2026-01-06", "14:00"))`; 2026-01-05T08:45 empty; 2026-01-06T13:45 returns one `Session` reminder with `ref_id` = makeup id and title `"CSE101 Intro to Programming at 14:00 in A-101"`); `m5_makeup_session_reminded` (`add_makeup(makeup_input(cse.id, "2026-01-08", "16:00"))`; 2026-01-08T15:45 returns it once); `m5_marked_session_no_reminder` (`class_reminder_minutes = 0`; mark the 01-05 session `Attended` at `2026-01-05T09:00`; call at 09:00:30 returns empty: only `pending` sessions are reminded); `m5_class_reminder_minutes_setting_used` (setting 30: 08:29 empty, 08:30 returns one with `fire_at` 08:30, 08:45 empty); `m5_zero_lead_session_reminder` (setting 0: 08:59 empty; 09:00:30 returns one with `fire_at` 2026-01-05T09:00; 09:00:45 empty; first call for the 01-07 class at 2026-01-07T11:01 returns empty because the one-minute window has passed). Doc comments cite FR-001, FR-002, FR-004, FR-005, §5. Fails on the stub
- [X] T002 [US2] (tester) Append the assessment tests to crates/core/tests/m5_reminders.rs, each creating an assessment with `create_assessment(AssessmentInput { remind_minutes_before: Some(m), ..assessment_input(cse.id, AssessmentKind::Assignment, "HW1", "2026-01-07T23:59") })`: `m5_assessment_reminder_fires_once` (m = 60: 22:58 empty; 22:59 returns exactly one `Reminder { kind: Assessment, ref_id: <id>, fire_at: 2026-01-07T22:59, title: "CSE101 Assignment: HW1 due 2026-01-07 23:59" }`; 23:30 empty); `m5_assessment_without_reminder_minutes_skipped` (`remind_minutes_before: None`: calls at 22:59, 23:30, 23:58 all empty); `m5_completed_assessment_no_reminder` (m = 60, `set_completed(id, true)`, 22:59 empty); `m5_missed_assessment_reminder_skipped` (m = 60, first call at 2026-01-07T23:59 empty, 2026-01-08T00:00 empty); `m5_assessment_due_change_fires_new_reminder` (m = 60, fire at 22:59; `update_assessment` to due `2026-01-08T23:59`; 2026-01-07T23:00 empty; 2026-01-08T22:59 returns one with `fire_at` 2026-01-08T22:59 and the new due text; a second call at 23:00 empty); `m5_zero_lead_assessment_reminder` (m = 0: 23:58 empty; 23:59 returns one with `fire_at` 23:59; 23:59:40 empty). Doc comments cite FR-001, FR-003, FR-004, FR-005, §5. Needs T001
- [X] T003 (tester) Append the cross-cutting tests to crates/core/tests/m5_reminders.rs: `m5_several_due_reminders_ordered` (add course `course_input("MTH201")` with a Monday 09:00–10:00 slot and sync again so its sessions have higher ids; add CSE101 assessments due 2026-01-05T09:00 with m = 20 (fire 08:40) and m = 15 (fire 08:45); one call at 08:50 returns exactly four reminders in the order: assessment fire 08:40, CSE101 session, MTH201 session (title `"MTH201 MTH201 course at 09:00 in A-101"`), assessment fire 08:45; a second call at 08:51 empty); `m5_archived_course_no_reminders` (assessment due 2026-01-05T12:00 with m = 60; `set_archived(cse.id, true)`; 08:45 and 11:00 empty); `m5_no_active_term_no_reminders` (fresh `Planner::open_in_memory()`: `Ok` and empty); `m5_inactive_term_no_reminders` (assessment due 2026-01-07T23:59 m = 60; then `save_term` a new term 2026-02-01 → 2026-02-28 which becomes active; 2026-01-05T08:45 and 2026-01-07T22:59 empty, R1); `m5_notifications_switch_ignored_by_core` (`notifications_enabled: false`; 08:45 still returns the session reminder: the app, not core, gates the call); `m5_restore_does_not_refire` (fire at 08:45; `export_json(dt("2026-01-05T08:45"))`; `restore_json` of that text; calls at 08:45 and 08:46 return empty: restore leaves `reminder_log` untouched). Doc comments cite the FR ids and spec edge cases. Needs T002
- [X] T004 (tester) Run `cargo test -p planner-core --no-fail-fast` and confirm crates/core/tests/m5_reminders.rs compiles, every `m5_*` test fails only by panicking on `todo!("M5: reminders")`, and every earlier test (including `api_surface.rs` and `backup.rs`'s `r22_restore_leaves_reminder_log_untouched`) stays green; note the failing count. Run `cargo clippy -p planner-core --all-targets -- -D warnings`. Needs T003

**Checkpoint**: tests written and failing for the right reason. Router commits "M5: reminder tests".

---

## Phase 3: User Story 1 - Class reminder before each class (Priority: P1) 🎯 MVP

**Goal**: `due_reminders` returns each pending session's reminder once, inside its window.

**Independent Test**: the US1 tests from T001 pass (`cargo test -p planner-core --test m5_reminders m5_` with the session tests green).

- [X] T005 [US1] (builder) Move the stub: create crates/core/src/reminders.rs (module doc: "Reminders (M5): due reminders logged in `reminder_log`.") with `impl Planner { pub fn due_reminders(&mut self, now: NaiveDateTime) -> Result<Vec<Reminder>> }`, delete the stub and its `Reminder` import from crates/core/src/backup.rs and change backup.rs's module doc to "JSON backup and restore (M3).", add `mod reminders;` to crates/core/src/lib.rs. Signature unchanged (`tests/api_surface.rs` must still compile). Needs T004
- [X] T006 [US1] (builder) Implement `due_reminders` for sessions in crates/core/src/reminders.rs per [contracts/core-api-m5.md](contracts/core-api-m5.md) and research R-2–R-5: read `class_reminder_minutes` via `self.settings()`; open `self.conn.transaction()`; select pending sessions (`session.status = 'pending'`, regular and makeup) joined to `course` (`archived = 0`) and `term` (`is_active = 1`) with `course.code`, `course.name`, `coalesce(session.room, course.room)`, optionally pre-filtered to `date || 'T' || start_time >= fmt_datetime(now - 1 minute)`; parse with `db::parse_date`/`parse_time`; `fire_at = T - lead` (`checked_sub`, skip on `None`); keep when `fire_at <= now && now < max(T, fire_at + 1 minute)`; for each kept candidate `INSERT OR IGNORE INTO reminder_log (kind, ref_id, fire_at) VALUES ('session', ?, fmt_datetime(fire_at))` and keep it only when the statement changed 1 row; `commit()`; sort by `(fire_at, kind rank Session = 0 / Assessment = 1, ref_id)`; title `"{code} {name} at {HH:MM}"` plus `" in {room}"` when the room is not null and not blank. Structure the code so T007 adds assessment candidates to the same candidate list before the insert loop. Doc comments: `/// §5, FR-001: …` on the method, FR-002/FR-004 on helpers. Needs T005

**Checkpoint**: every session test of T001 passes; `cargo clippy -p planner-core --all-targets -- -D warnings` passes.

---

## Phase 4: User Story 2 - Assessment reminder (Priority: P1)

**Goal**: incomplete assessments with `remind_minutes_before` set are reminded once.

**Independent Test**: T001–T003 all pass: `cargo test -p planner-core --test m5_reminders`.

- [ ] T007 [US2] (builder) Extend `due_reminders` in crates/core/src/reminders.rs with assessment candidates: select `assessment` rows with `completed = 0` and `remind_minutes_before IS NOT NULL` joined to `course` (`archived = 0`) and `term` (`is_active = 1`), optionally pre-filtered to `due_at >= fmt_datetime(now - 1 minute)`; `T = due_at` (`db::parse_datetime`), lead = `remind_minutes_before`; same window, `INSERT OR IGNORE` with kind `'assessment'`, ordering, and transaction as T006; title `"{code} {Kind}: {title} due {YYYY-MM-DD HH:MM}"` with a private core mapping `Quiz`/`Exam`/`Assignment`/`Other` (research R-5). Doc comments cite FR-003 and FR-005. Needs T006

**Checkpoint**: all `m5_*` tests pass; the full core suite is green.

---

## Phase 5: User Story 3 - Notifications while the app runs (Priority: P1)

**Goal**: the 60-second timer shows due reminders as desktop notifications when the switch is on.

**Independent Test**: the manual check in [quickstart.md](quickstart.md) (T012).

**Parallel note**: T008 and T009 touch only app files and `Cargo.lock`, so they can start as soon as
T004 is done, in parallel with T005–T007; T010 and T011 need the app to build against core (the
signature already exists, so they do not wait for T007 to compile, but the manual check does).

- [ ] T008 [P] [US3] (builder) Add `notify-rust = "4.18.2"` (default features) to `[dependencies]` in crates/app/Cargo.toml; run `cargo build -p app` on Rust 1.90 and let `Cargo.lock` update (CI uses `--locked`); run `cargo tree -p app -i libdbus-sys`: if it reports a match, add `libdbus-1-dev` to the apt-get line in .github/workflows/ci.yml, otherwise leave ci.yml unchanged (research R-1). If 4.18.2 does not build on 1.90, use the newest 4.x that does and note it in the commit message. Needs T004
- [ ] T009 [US3] (builder) Create crates/app/src/notify.rs per [contracts/app-notify.md](contracts/app-notify.md): module doc citing §2, §7, FR-006–FR-008; `pub fn show(reminder: &Reminder)` that clones `reminder.title`, spawns a detached `std::thread`, and in it runs `notify_rust::Notification::new().summary("Academic Planner").body(&title).show()`, writing any error to stderr with `eprintln!` and never panicking or retrying; add `mod notify;` to crates/app/src/main.rs. Needs T008
- [ ] T010 [US3] (builder) In crates/app/src/controller.rs add a private `fn check_reminders(&self, now: NaiveDateTime)` and call it at the end of `tick()` with the same `now` (after the existing R6 sync/refresh, so a new day's sessions exist): read `self.planner().settings()` (on error `show_message(&viewmodel::error_text(&e))` and return); if `!notifications_enabled` return without calling core (US3.2); otherwise `self.planner_mut().due_reminders(now)`, drop the borrow, then `notify::show(&r)` for each reminder in order; on error show the message. Doc comments cite FR-006, FR-008, §7. Update the timer comment in crates/app/src/main.rs to say it also checks reminders. No `.slint` change, no new viewmodel function (research R-6). Needs T009
- [ ] T011 [US3] (builder) Run `cargo clippy -p app --all-targets -- -D warnings` and `cargo test -p app`; confirm no `notify_rust` use outside crates/app/src/notify.rs and no `notify-rust` in crates/core/Cargo.toml. Needs T010

**Checkpoint**: app builds; existing app tests green.

---

## Phase 6: Polish & Gate

- [ ] T012 (human) Manual notification check per [quickstart.md](quickstart.md) "Manual notification check" with a throwaway `ACADEMIC_PLANNER_DB`: one notification appears within 60 seconds of `start - 15 minutes`, none repeats for the same class. Record OS and time in `.claude/state/checkpoint.md` (router). If unattended, leave pending for the human (FR-010). Needs T007, T011
- [ ] T013 (builder) Run `speckit-converge` against spec.md, plan.md, and tasks.md; confirm no `todo!(` remains in crates/core/src, every new item has an FR/§ doc comment, `tests/api_surface.rs` is unchanged; append any leftover work as new tasks and do it. Needs T007, T011
- [ ] T014 (builder) Run the gate `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` and paste the summary; run the automated commands in [quickstart.md](quickstart.md). Do not edit `.claude/state/` (the router does). Needs T013

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 2**: T001 → T002 → T003 → T004 (one file).
- **Phases 3–4 (core)**: T005 → T006 → T007 (all need T004).
- **Phase 5 (app)**: T008 → T009 → T010 → T011; T008 may start after T004, in parallel with T005–T007.
- **Phase 6**: T012 and T013 after T007 and T011; T014 after T013.

### Story dependencies

- US1 (T006) before US2 (T007): same function and file; T007 extends T006's candidate list.
- US3 depends only on the frozen §5 signature to compile; its manual check needs US1.

### Parallel opportunities

| Group | Tasks | Role |
| --- | --- | --- |
| Core chain and app chain (disjoint files) | T005→T006→T007 and T008→T009→T010→T011 | builder |

### Role summary

- **tester**: T001, T002, T003, T004
- **builder**: T005, T006, T007, T008, T009, T010, T011, T013, T014
- **human**: T012

## Implementation Strategy

1. Tests first (T001–T004); T004 proves they fail only on `todo!`.
2. MVP: US1 (session reminders, T005–T006), then US2 (T007). The app chain (T008–T011) can run
   beside it.
3. Converge, then the gate; the human's manual notification check closes the §7 done-when. The
   router commits per phase ("M5: …").
