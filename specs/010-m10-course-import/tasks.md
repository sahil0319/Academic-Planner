---

description: "Task list for M10 courses and timetable import"
---

# Tasks: M10 Courses and Timetable Import

**Input**: Design documents from `specs/010-m10-course-import/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [quickstart.md](quickstart.md), [contracts/import-format.md](contracts/import-format.md), [contracts/core-import.md](contracts/core-import.md), [contracts/ui-import.md](contracts/ui-import.md), [contracts/import-docs.md](contracts/import-docs.md)

**Tests**: Required. T44–T50 come from `docs/spec.md` §13.5 and are named `t44_…`–`t50_…`; the
robustness tests come from the contracts (named `r42_…`–`r47_…`, `r50_…`). Every test task is done
by the **tester** before the matching builder task (constitution II), and the tester confirms the
new tests fail on `todo!` first. Core tests use `Planner::open_in_memory()` only; "database
unchanged" means `export_json(fixed_now)` is equal before and after.

**Start condition**: M9's core is merged (migration 002, `CourseInput.credits`,
`CourseInput.rule_set_id`, `rule_set` table seeded with "Theory" and "Lab", `list_rule_sets`). If
M9 uses different Rust field names than `docs/spec.md` §13.3, use M9's names. M8's
`viewmodel::format_time` (12-hour) is used by the app tasks; if M8 is not merged yet, the app
view-model tests assert through `format_time` instead of literal strings for times.

**Organization**: Phase 1 setup, Phase 2 core types and stubs (unblocks tester and the app track),
then the core stories in build order US3 (validation, every path goes through it) → US1 (first
import) → US2 (re-import), then the app panel US4, the AI prompt US5, and the gate. All three core
stories are P1; US3 goes first only because US1 and US2 call its parser.

## Format: `[ID] [P?] [Story] (owner) [crate] Description`

- **[P]**: can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: user story from spec.md (US1–US5)
- **(tester)** / **(builder)** / **(human)**: who does the task
- **[core]** / **[app]**: the crate (or crate-owned docs) the task touches. `docs/import/template.json`
  is `[core]` (a core test validates it); `docs/import/ai-prompt.md` is `[app]` (only the app uses it).

## Parallel tracks

- **Core track**: T002 → T003–T007 → T008 → T009–T012 → T013 → T014–T019 → T020 → T021.
- **App track**: after T002, T022 → T023 → T024 → T025 run beside the core track (they compile
  against the core types and stubs; the panel shows real plans once T013/T021 land).
- **Docs**: T012 (template) with US1; T026 (AI prompt body) after T012; T027 (worked example) after T001.

---

## Phase 1: Setup

- [x] T001 (human) [app] Provide the 7th-semester timetable for the worked example in
  `docs/import/ai-prompt.md` (OQ-2): the period header with each period's start and end time, and
  the course grid including at least one merged cell (text or a photo the builder can transcribe).
  Also answer OQ-1 (ambiguous code match: error, recommended / lowest id) and OQ-3 (which slots
  R45 ends: every slot not ended before E, recommended / only slots valid on E). Until answered,
  the builder uses the recommended options. Blocks only T027.

---

## Phase 2: Foundational (blocking)

- [ ] T002 (builder) [core] Core types and stubs so tests and the app compile: create
  crates/core/src/import/mod.rs (`impl Planner { preview_import, apply_import }` with the exact
  §13.4 signatures, bodies `todo!("M10")`, doc comments citing R42–R48), crates/core/src/import/types.rs
  (every public type of contracts/core-import.md: `ImportPlan` with `is_empty()`, `ImportIssue`
  with `Display` "path: message", `ImportTerm`, `CourseAdd`, `RuleSetRef`, `CourseUpdate`,
  `ImportField`, `FieldValue`, `FieldChange`, `SlotChange`, `ImportSlot`, `ImportReport`), empty
  crates/core/src/import/{parse.rs, plan.rs, apply.rs}; in crates/core/src/lib.rs add `mod import;`
  and re-export the types; in crates/core/tests/api_surface.rs bind
  `fn(&Planner, &str, NaiveDate) -> Result<ImportPlan>` and
  `fn(&mut Planner, &str, NaiveDate, NaiveDate) -> Result<ImportReport>`. Type and field names are
  frozen after this task. Gate-clean (fmt, clippy). Needs M9 core merged.

**Checkpoint**: tester (core) and the app track can start.

---

## Phase 3: User Story 3 - See every problem before anything is written (P1, R42) — build first

**Goal**: the whole document is validated, all errors are collected with JSON paths, unknown keys
are warnings, nothing is written on error.

**Independent test**: `cargo test -p planner-core --test m10_import r42_ t48_`.

- [ ] T003 (tester) [core] Create crates/core/tests/m10_import.rs with helpers: `fresh()` (in-memory
  planner, no term; M9 seeds rule sets), `with_term(name, start, end)` (active term via
  `save_term`), `add_course(code, archived)`, `add_slot(course, day, start, end, room, from)`,
  `snapshot(&Planner) -> String` (`export_json` at a fixed `NaiveDateTime`), `d("2026-01-14")`,
  and `doc(json!(…))` building import text with `serde_json::json!` (add `serde_json` to
  crates/core `[dev-dependencies]` only if not already reachable; it is a normal dependency, so the
  test can use it directly). Needs T002
- [ ] T004 [US3] (tester) [core] In crates/core/tests/m10_import.rs write
  `t48_errors_collected_with_paths_db_unchanged`: one file with a bad day (`"Mo"` at
  `courses[0].slots[0].day`), end ≤ start (`courses[1].slots[0].end`), a duplicate code
  (`courses[2].code` duplicating `courses[0].code` written as `"cse 4711"` vs `"CSE 4711"`), and an
  unknown rule set (`courses[1].rule_set: "Foo"`); assert `preview_import` returns exactly those
  four errors (by path, any order), empty `add`/`update`/`term`; `apply_import` returns
  `Error::Validation` whose message contains all four paths; `snapshot` unchanged. Needs T003
- [ ] T005 [US3] (tester) [core] Shape tests in crates/core/tests/m10_import.rs, one test each,
  asserting the path and that `apply_import` writes nothing: `r42_not_json_is_document_error`
  (path `document`, message has line and column), `r42_root_not_object`, `r42_format_missing_or_wrong`,
  `r42_version_not_1` (`2` and `"1"`), `r42_courses_missing_or_not_list`, `r42_course_not_object`,
  `r42_code_missing_null_or_blank`, `r42_new_course_needs_name` (absent → "required for a new
  course"; `null` and `"  "` → error), `r42_slots_null_is_error` ("use [] to end all slots"),
  `r42_term_malformed_with_active_term_is_error` (bad `term.start` while an active term exists),
  `r42_term_end_before_start`, `r42_errors_collected_across_courses` (errors in `courses[0]` and
  `courses[3]` both reported). Needs T003
- [ ] T006 [US3] (tester) [core] Value tests in crates/core/tests/m10_import.rs:
  `r42_day_variants_accepted` (`"Mon"`, `"monday"`, `"MONDAY"`, `" Mon "`, `"1"`, `1` all →
  Monday in the planned slot; `"Sun"`, `7` → Sunday), `r42_day_variants_rejected` (`"Mo"`,
  `"Tues"`, `0`, `8`, `"0"`, `1.5`, `true` → error at that `.day` path), `r42_times`
  (`"9:30"`, `"09:30"`, `"23:59"` accepted; `"24:00"`, `"9:5"`, `"10:30 AM"`, `"10:30:00"`, `930`
  rejected at the `.start`/`.end` path), `r42_dates` (`"2026-1-5"`, `"05/01/2026"` rejected),
  `r42_credits` (`0`, `-1`, `"3"` rejected; `1.5` accepted), `r42_min_attendance_pct` (`101`, `-1`,
  `75.5`, `"75"` rejected; `0`, `100` accepted), `r42_rule_set_unknown_lists_names` (message
  contains "Theory" and "Lab"). Needs T003
- [ ] T007 [US3] (tester) [core] Warning tests in crates/core/tests/m10_import.rs:
  `r42_unknown_keys_warn_not_error`: unknown keys at root (`comment`), `term` (`id`), course
  (`teacher`, `notes`, `archived`), and slot (`type`) each produce one warning with its exact path
  (e.g. `courses[0].slots[1].type`), no errors, and the plan is otherwise the same as without
  them; apply succeeds and ignores them (course `notes` unchanged). Needs T003
- [ ] T008 [US3] (builder) [core] Implement R42 in crates/core/src/import/parse.rs (research
  D2–D4, D7–D10; contracts/import-format.md): `Value` walk, path builder, tri-state `Field<T>`,
  own `H:MM`/`HH:MM` time parser, day parser, number rules, duplicate detection by normalized code
  (research D6), unknown-key warnings, issue ordering per D4. In crates/core/src/import/plan.rs
  start `build_plan(conn, doc, effective_from)` with the DB checks that need reads: rule set lookup
  by exact name (error lists names) and the "term required when there is no active term" error;
  wire `preview_import` to it (on errors: return the plan with only `errors`/`warnings`). Exit:
  every `preview_import` error and warning assertion in T004–T007 passes. Three kinds of assertion
  need T013 and stay red until then: `apply_import` results (T004, T005, T007), and the planned-slot
  weekday checks in T006. Doc comments cite R42. Needs T004, T005, T006, T007

**Checkpoint**: invalid files are fully reported; nothing can be written yet.

---

## Phase 4: User Story 1 - First import into an empty planner (P1, R47, R48, R50)

**Goal**: a valid file creates the term, courses, and slots in one transaction and syncs.

**Independent test**: `cargo test -p planner-core --test m10_import t44_ t50_ r47_ r50_`.

- [ ] T009 [US1] (tester) [core] In crates/core/tests/m10_import.rs write
  `t44_import_into_db_with_no_term`: file with `term` (2026-01-05 to 2026-05-01) and two courses
  (one with `credits: 3, rule_set: "Theory"`, one with `rule_set: "Lab"`, slots on Mon and Fri);
  `apply_import(json, 2026-01-14, today 2026-01-14)`; assert the active term matches the file, both
  courses exist with all file fields (credits, rule set id, instructor/room `null` → `None`,
  `min_attendance_pct`), each slot has `valid_from` 2026-01-14 and `valid_to` `None`, regular
  sessions exist on every matching weekday from 2026-01-14 to 2026-05-01 and none before.
  Also `r47_term_required_without_active_term` (no `term` → error at `term`, snapshot unchanged)
  and `r47_term_only_file_creates_term` (`courses: []`). Needs T008
- [ ] T010 [US1] (tester) [core] In crates/core/tests/m10_import.rs write
  `t50_preview_writes_nothing_apply_matches`: on a DB with an active term and one existing course
  not in the file, preview a file that adds two courses with slots; snapshot equal before and after
  preview; then `apply_import` with the same input; assert `report.plan == preview` and the DB
  reflects exactly that plan (counts of courses and slots). The update path of T50 is in T014.
  Also `r48_apply_with_errors_writes_nothing` (Validation error, snapshot unchanged). Needs T008
- [ ] T011 [US1] (tester) [core] In crates/core/tests/m10_import.rs write
  `r50_template_previews_cleanly`: `include_str!("../../../docs/import/template.json")`, preview on
  `fresh()` with effective 2026-01-05: no errors, no warnings, term planned, one course `CSE 4711`
  with two slots. Needs T003
- [ ] T012 [US1] (builder) [core] Create docs/import/template.json with exactly the
  `docs/spec.md` §13.4 example (contracts/import-docs.md), pretty-printed with two-space indent.
  Needs T011
- [ ] T013 [US1] (builder) [core] Implement the add path and apply: in
  crates/core/src/import/plan.rs, new-course planning (code as written, trimmed; absent/`null` →
  `None`; slots from E), term planning (R47: create when no active term), `ImportPlan::is_empty`;
  in crates/core/src/import/apply.rs, `apply_import` per contracts/core-import.md "Apply write
  order": one transaction, re-plan through the transaction (research D13), `Error::Validation`
  listing every `path: message` on errors, empty plan → nothing written and no sync, insert term
  (active, as `save_term` does), insert courses with M9's `credits`/`rule_set_id`, insert slots via
  `insert_slot`, then `sync_in(tx, min(effective_from, today))` (research D12), commit; return
  `ImportReport { plan, sync }`. Add `pub(crate)` helpers in crates/core/src/db.rs only if needed.
  Doc comments cite R47, R48. T009–T011 and every remaining T004–T007 assertion pass. Needs T009, T010, T012

**Checkpoint**: MVP — a fresh planner can be filled from one file.

---

## Phase 5: User Story 2 - Re-import safely (P1, R43–R46)

**Goal**: matching, field merge, slot-set comparison, R7-style slot changes, idempotence.

**Independent test**: `cargo test -p planner-core --test m10_import t45_ t46_ t47_ t49_ r43_ r44_ r45_ r47_`.

- [ ] T014 [US2] (tester) [core] In crates/core/tests/m10_import.rs write
  `t45_same_file_twice_is_empty`: apply a file (term + 3 courses, one with `credits: 1.5`, one with
  slots listed out of day order, one with a slot `room`) at E; preview the same text at the same E:
  `is_empty()`, no slot changes, no field changes; snapshot before and after a second
  `apply_import` is equal and `report.sync` is zero. Repeat with the archived variant (one course
  archived between the imports) to confirm only the archived warning appears and the plan is still
  empty. Also `t50_update_path_apply_matches_preview`: a changed file (one field change and one slot
  change on a matched course): snapshot equal after preview, `report.plan == preview` after apply.
  Needs T013
- [ ] T015 [US2] (tester) [core] In crates/core/tests/m10_import.rs write
  `t46_slot_time_change_effective_0114`: term from 2026-01-05, course with Mon 10:30–11:45 imported
  at 2026-01-05 and synced; mark the 2026-01-05 session attended; re-import with Mon 11:00–12:15 at
  effective 2026-01-14, today 2026-01-14; assert sessions dated before 2026-01-14 keep 10:30–11:45
  (the attended one unchanged), sessions from 2026-01-14 are 11:00–12:15, no date has two regular
  sessions, the old slot has `valid_to` 2026-01-13. Also `r45_backdated_effective_removes_pending_only`
  (2026-01-05 is a Monday; effective 2026-01-14, today 2026-01-21, research D12): variant A, the
  old-time 2026-01-19 session is pending → it is removed and a new-time 2026-01-19 session exists;
  variant B, it is marked attended first → it is kept unchanged and the new-time 2026-01-19 session
  also exists (R5, R7: non-pending sessions are never removed) and
  `r45_future_slot_also_ended` (OQ-3 default: slot from an earlier import with a later E is ended;
  no duplicate sessions). Needs T013
- [ ] T016 [US2] (tester) [core] In crates/core/tests/m10_import.rs write
  `t47_code_match_ignores_case_and_space` (DB `CSE 4711`, file `cse4711` with a new name: one
  course, code still `CSE 4711`, name updated), `t49_course_absent_from_file_unchanged` (DB course
  B with slots and sessions not in the file: course, slots, sessions identical after apply),
  `r43_ambiguous_match_is_error` (OQ-1 default: DB `CSE 4711` and `CSE4711`, file `cse 4711` →
  error at `courses[0].code`), `r43_match_includes_archived` (archived match, no duplicate created).
  Needs T013
- [ ] T017 [US2] (tester) [core] In crates/core/tests/m10_import.rs write
  `r44_absent_keeps_null_clears`: existing course with every field set; file A omits `instructor`,
  `room`, `credits`, `rule_set`, `min_attendance_pct`, `name` → plan has no field change and apply
  keeps all; file B sets each to `null` → one `FieldChange` per field with `new` = `None`, apply
  clears them (course becomes ungraded per R33); file C sets same values as stored (credits `3`
  vs stored `3.0`) → no change; `notes` set in the DB is kept in all cases.
  `r44_archived_match_warns_and_stays_archived`: warning at `courses[0]`, fields and slots updated,
  `archived` still true. Needs T013
- [ ] T018 [US2] (tester) [core] In crates/core/tests/m10_import.rs write
  `r45_slot_set_equal_any_order_no_change`, `r45_slot_room_difference_is_change` (`null` vs
  `"301"`), `r45_absent_room_equals_null_room`, `r45_empty_slots_ends_all` (`"slots": []` →
  every current slot `valid_to = E − 1`, nothing added, pending sessions from E removed),
  `r45_missing_slots_key_untouched` (field change only; slots identical),
  `r45_duplicate_identical_slots_counted` (file lists the same slot twice vs DB once → change).
  Needs T013
- [ ] T019 [US2] (tester) [core] In crates/core/tests/m10_import.rs write
  `r47_active_term_differs_warns` (file term name or dates differ → warning at `term`, active term
  unchanged, no term in plan) and `r47_active_term_equal_no_warning`. Needs T013
- [ ] T020 [US2] (builder) [core] Implement matching and merge in crates/core/src/import/plan.rs:
  load active-term courses including archived, normalized-code index (research D6), ambiguous
  match error (OQ-1 default), `FieldChange` list per research D14 (present and different only;
  absent keep; `null` clear), archived warning, term-differs warning (research D15/D16); in
  crates/core/src/import/apply.rs write changed fields onto the stored course (keep `notes`,
  `archived`, code). Doc comments cite R43, R44, R46, R47. T016, T017, T019 pass. Needs T016, T017, T019
- [ ] T021 [US2] (builder) [core] Implement R45 in crates/core/src/import/plan.rs and
  crates/core/src/import/apply.rs (research D11): slots valid on E per course, multiset comparison
  of `(weekday, start, end, room)` after sorting, "current" slots = not ended before E (OQ-3
  default), `SlotChange { end, add }` only when different, apply sets `valid_to = E − 1` and
  inserts the file slots from E before the sync. Doc comments cite R45 and R7. T014, T015, T018
  pass and all earlier M10 tests still pass. Needs T014, T015, T018, T020

**Checkpoint**: core complete; `cargo test -p planner-core` green.

---

## Phase 6: User Story 4 - Import panel on the Courses page (P2, §13.4 UI, R49, R50)

**Goal**: paste, copy template/prompt, effective date, preview, apply, all text from the view model.

**Independent test**: `cargo test -p app viewmodel::import` plus the manual check T029.

- [ ] T022 [P] [US4] (builder) [app] App stubs: create crates/app/src/viewmodel/import.rs with the
  function signatures of contracts/ui-import.md (`issue_line`, `errors_heading`, `plan_summary`,
  `term_line`, `course_add_line`, `field_change_line`, `slot_line`, `slot_change_lines`,
  `import_report_text`, `can_apply`, and the `PreviewState` struct holding the previewed text, date
  text, `has_errors`, `is_empty`), bodies `todo!("M10")`, `#[cfg(test)] mod tests;`, empty
  crates/app/src/viewmodel/import/tests.rs; add `pub mod import;` to crates/app/src/viewmodel.rs.
  `#[allow(dead_code)]` only on stub items until T025. Needs T002
- [ ] T023 [US4] (tester) [app] In crates/app/src/viewmodel/import/tests.rs write unit tests for
  every row of the contracts/ui-import.md table: `issue_line` exact text; `errors_heading(1)` and
  `(3)`; `plan_summary` for term+adds+updates, singular/plural ("1 course", "2 courses"), and the
  empty plan ("No changes. Everything already matches."); `term_line`; `course_add_line` with and
  without credits/rule set; `field_change_line` for each `ImportField` including `None` → `—` and
  credits `1.5`; `slot_line` with and without room (times via `format_time_range`);
  `slot_change_lines` "Ends after …"/"From …" with effective 2026-01-14; `import_report_text`;
  `can_apply` false when no preview, text changed, date changed, errors, or empty plan, true
  otherwise. Confirm they fail on `todo!`. Needs T022
- [ ] T024 [US4] (builder) [app] Implement crates/app/src/viewmodel/import.rs per
  contracts/ui-import.md, reusing `format_date`, `format_time_range`, `weekday_name`, `or_dash`
  from crates/app/src/viewmodel.rs. T023 passes. Needs T023
- [ ] T025 [US4] (builder) [app] UI and wiring: create crates/app/ui/import.slint (`export global
  ImportState` with the properties and callbacks of contracts/ui-import.md and
  `export component ImportPanel`: paste box, Copy template / Copy AI prompt with two read-only
  `TextInput`s and `select-all()`/`copy()`/`clear-selection()` (research D17), effective date
  field with inline error, Preview, result lines, Apply bound to `can-apply`; `Theme.*` only,
  bindings only); add the line struct(s) it needs to crates/app/ui/types.slint; add an
  "Import…" button and the panel instance to crates/app/ui/courses.slint (minimal edit; M9 also
  edits this file); create crates/app/src/pages/import.rs (`wire`, `refresh`; `TEMPLATE` and
  `AI_PROMPT` via `include_str!("../../../../docs/import/…")` (research D18); preview → 
  `preview_import`; apply → `apply_import(text, date, today)` then `App::refresh_all`; editing text
  or date clears the result (research D19); date defaults to today (R49)); register it in
  crates/app/src/pages/mod.rs and the startup wiring in crates/app/src/controller.rs or
  crates/app/src/main.rs (wherever the other pages' `wire` is called). Remove the T022
  `#[allow(dead_code)]`. Run the theme grep from quickstart.md. Needs T024, T012, T026

**Checkpoint**: the panel imports the template end to end.

---

## Phase 7: User Story 5 - Convert a timetable with an AI assistant (P3, R50)

**Independent test**: read the file against contracts/import-docs.md items 1–6.

- [ ] T026 [US5] (builder) [app] Write docs/import/ai-prompt.md per contracts/import-docs.md
  items 1–4 and 6: the "convert my timetable to this format…" request, output-only-JSON
  instruction, the template, the known keys and rules (24-hour `HH:MM`, `day` forms, dates,
  `rule_set` "Theory"/"Lab" or `null`, unknown → `null`), and the period → time rule for merged
  cells (first period's start to last period's end). Leave a section heading
  "Worked example: 7th semester" for T027. Needs T012
- [ ] T027 [US5] (builder) [app] Fill the worked example in docs/import/ai-prompt.md
  (contracts/import-docs.md item 5) from the table the human gave in T001: period header with
  times, the grid with its merged cell(s), and the resulting JSON in 24-hour times; check the JSON
  previews with no errors (paste it into the app on a scratch DB, or a throwaway local test that
  is not committed). Needs T001, T026

---

## Phase 8: Polish, gate, Done-when

- [ ] T028 (builder) [core] [app] Run the gate `cargo fmt --check && cargo clippy --workspace
  --all-targets -- -D warnings && cargo test --workspace`, the theme grep, `cargo test -p
  planner-core --test m10_import`, and `cargo test -p app viewmodel::import`; confirm every M10
  function has a rule-ID doc comment and no `todo!("M10")` remains; paste the summary. Needs T021,
  T025, T027
- [ ] T029 (human) [app] Run the quickstart.md manual check (steps 1–6) on a scratch DB via
  `ACADEMIC_PLANNER_DB`, including Copy AI prompt and reading the prompt's worked example. Needs T028

---

## Dependencies & Execution Order

- Phase 1 T001 (human) runs any time; it blocks only T027 (and confirms OQ-1/OQ-3 defaults).
- T002 blocks everything else except T001.
- Core: T003 → {T004, T005, T006, T007} → T008 → {T009, T010, T011} → T012 → T013 →
  {T014, T015, T016, T017, T018, T019} → T020 → T021.
- App: T002 → T022 → T023 → T024 → T025 (T025 also needs T012 and T026 because it embeds both
  files).
- T026 needs T012; T027 needs T001 and T026.
- T028 needs the core track, T025, and T027; T029 needs T028.

## Parallel Examples

- Core test tasks share crates/core/tests/m10_import.rs, so they are not marked [P]; one tester
  writes them in order. They are independent test functions, so their order inside a phase is free.
- After T002: the builder can do T022 → T024 (app) while the tester writes core tests T003–T007.

## Implementation Strategy

1. MVP: T001–T013 (validation + first import) — a fresh planner can be filled from the template.
2. Re-import safety: T014–T021 (idempotence, matching, merge, slot changes).
3. Panel and prompt: T022–T027, then the gate T028 and the manual check T029.

## Task counts

| Owner | [core] | [app] | Total |
|---|---|---|---|
| tester | 14 (T003–T007, T009–T011, T014–T019) | 1 (T023) | 15 |
| builder | 6 (T002, T008, T012, T013, T020, T021) | 5 (T022, T024, T025, T026, T027) | 11 |
| builder, both crates | T028 (gate) | | 1 |
| human | 0 | 2 (T001, T029) | 2 |
| **Total** | | | **29** |
