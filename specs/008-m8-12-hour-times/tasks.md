---

description: "Task list for M8 12-hour times"
---

# Tasks: M8 12-Hour Times

**Input**: Design documents from `specs/008-m8-12-hour-times/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [quickstart.md](quickstart.md), [contracts/viewmodel-time.md](contracts/viewmodel-time.md), [contracts/ui-time-field.md](contracts/ui-time-field.md)

**Tests**: Required. T34 and T35 (app) come from `docs/spec.md` §13.5; the pre-fill, round-trip,
edge, and `times_to_12h` tests come from contracts/viewmodel-time.md. The **tester** writes them
before the builder implements (constitution II). No test touches a database.

**Start condition**: M7 is committed (it is: `cd079f3`). One milestone at a time. App crate
only: no file under `crates/core/` changes.

**Organization**: three rounds. Round 1 (builder) makes the new signatures and stubs compile.
Round 2 (tester) writes the tests. Round 3 (builder) implements, wires every page, and runs the
gate. Story labels map to spec.md: US1 display (P1), US2 input (P1), US3 edit pre-fill (P2).

## Format: `[ID] [P?] [Story] (owner) Description`

- **[P]**: can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: user story from spec.md (US1–US3)
- **(builder)** / **(tester)** / **(human)**: who does the task

## Path Conventions

Edited: `crates/app/src/viewmodel.rs`, `crates/app/src/viewmodel/tests.rs`,
`crates/app/src/pages/{timetable.rs, courses.rs, assessments.rs}`,
`crates/app/ui/{components.slint, timetable.slint, courses.slint, assessments.slint}`, and
`crates/app/src/notify.rs` only if OQ-1 = B. No new source files. Not edited: anything in
`crates/core/`, `crates/app/ui/theme.slint` (no new tokens), `docs/spec.md`, `DESIGN.md`,
`.claude/state/` (router only).

---

## Phase 1: Round 1, builder: signatures and stubs (blocking)

- [x] T001 (builder) In crates/app/src/viewmodel.rs per contracts/viewmodel-time.md: add `pub struct TimeText<'a> { pub digits: &'a str, pub pm: bool }` (`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`, doc `/// R32: one time field as typed: the digits text and the AM/PM toggle (pm == false is AM).`); change `parse_time` to `pub fn parse_time(digits: &str, pm: bool) -> Result<NaiveTime, String>` with body `todo!("M8: R32")`; add stubs `pub fn format_time_input(t: NaiveTime) -> (String, bool)` and `pub fn times_to_12h(text: &str) -> String`, both `todo!("M8")`, each with `#[allow(dead_code)]` until wired; set `TIME_FORMAT_ERR` to `"Enter the time as digits, e.g. 930 (hour 1–12, minutes 00–59)"`; change the time parameters of `parse_slot_form` (`start`, `end`), `parse_makeup_form` (`start`, `end`), and `parse_assessment_form` (`due_time`) from `&str` to `TimeText<'_>`, calling `parse_time(x.digits, x.pm)` and keeping parameter order and error keys (`start`, `end`, `due_time`). Leave `format_time` and `error_text` bodies unchanged in this round.
- [x] T002 [P] (builder) Add in-out `bool` properties (default false) next to the paired text properties: `rs-start-pm`, `rs-end-pm`, `slot-start-pm`, `slot-end-pm` in `TimetableState` (crates/app/ui/timetable.slint); `mk-start-pm`, `mk-end-pm` in `CoursesState` (crates/app/ui/courses.slint); `due-time-pm` in `AssessmentsState` (crates/app/ui/assessments.slint). No other `.slint` change in this round.
- [x] T003 (builder) Make the pages compile against T001/T002: in crates/app/src/pages/timetable.rs (reschedule save near line 252, slot save near line 397), crates/app/src/pages/courses.rs (makeup save near line 280), and crates/app/src/pages/assessments.rs (save near line 268), pass `vm::TimeText { digits: &text, pm: g.get_<field>_pm() }` for each time field. Pre-fill and clear changes wait for round 3. Needs T001, T002
- [x] T004 (builder) Compile-only update of crates/app/src/viewmodel/tests.rs: rewrite existing call sites to the new signatures (`parse_time("09:30")` → `parse_time("09:30", false)`; form time arguments wrapped as `TimeText { digits: "…", pm: false }`) with no change to any expected value or assertion; the tester revises expectations in round 2. Then run `cargo test --workspace --no-run` and `cargo clippy --workspace --all-targets -- -D warnings`; both must succeed (test failures are expected and not run here). Needs T001, T003

**Checkpoint**: workspace and tests compile; clippy clean; the tester can start.

---

## Phase 2: Round 2, tester: tests first

All in crates/app/src/viewmodel/tests.rs (one file, so in order). Doc comments cite R31/R32.

- [x] T005 [US2] (tester) T34 per contracts/viewmodel-time.md: `t34_am_inputs_parse` (AM: `9`→09:00, `930`→09:30, `0930`→09:30, `1215`→00:15, `12:15`→00:15), `t34_pm_inputs_parse` (PM: `1215`→12:15, `130`→13:30), `t34_bad_inputs_are_errors` (`1260`, `1300`, `0`, `""` with both `pm = false` and `pm = true` → `Err(TIME_FORMAT_ERR)`). Needs T004
- [x] T006 [US1] (tester) T35: `t35_times_display_12_hour` (00:00, 09:05, 12:00, 13:30 → "12:00 AM", "9:05 AM", "12:00 PM", "1:30 PM"). Update existing expectations: `format_time_drops_seconds` → "9:05 AM"; `format_time_range_uses_en_dash` → "9:00 AM–10:30 AM"; `format_datetime_date_and_time` → "October 30, 2026, 11:59 PM"; any row-builder test (session row, slot row, assessment row) that asserts a 24-hour time. Needs T005
- [x] T007 [US1] (tester) `times_to_12h` and `error_text`: "slot end time 09:00 is not after its start time 10:30" → "slot end time 9:00 AM is not after its start time 10:30 AM"; "due October 30, 2026, 23:59" → "due October 30, 2026, 11:59 PM"; text with no time and an ISO date `2026-10-30` unchanged; a non-time token such as `24:00` or `123:45` unchanged; `error_text(&Error::Validation("slot end time 09:00 is not after its start time 10:30".into()))` → "Slot end time 9:00 AM is not after its start time 10:30 AM". Needs T006
- [x] T008 [US2] (tester) Parser edges and existing input tests: ` 930 ` (trimmed) and `9:30` AM → 09:30; `12` AM → 00:00 and `12` PM → 12:00; `13` PM, `030`, `12345`, `9a`, `9.30` → error. Replace the old `TIME_ERR` literal with the new constant text, rewrite `parse_time_accepts_hh_mm` / `parse_time_rejects_bad_input_with_format_hint` for the new rules, and change form-parser tests to 12-hour inputs (for example `TimeText { digits: "200", pm: true }` for 14:00, `"1159"` PM for 23:59) keeping their original intent (valid input, all errors at once with keys `start`/`end`/`due_time`, end-before-start passed to core). Needs T007
- [x] T009 [US3] (tester) Pre-fill: `format_time_input` gives 09:30 → ("930", false), 13:30 → ("130", true), 00:15 → ("1215", false), 12:05 → ("1205", true), 00:00 → ("1200", false), 12:00 → ("1200", true); round trip: for all 1,440 minutes `t`, `let (d, pm) = format_time_input(t); parse_time(&d, pm) == Ok(t)` (SC-005). Needs T008
- [x] T010 (tester) Run `cargo test -p app` and report which tests fail (expected: the T34, T35, T007, T008, T009 tests, through `todo!` or 24-hour output); every other app test passes. Needs T009

---

## Phase 3: Round 3, builder: implement

### View-model functions (make round 2 green)

- [x] T011 [US1] (builder) Implement `format_time` in crates/app/src/viewmodel.rs: `h % 12` shown as 12 when 0, no leading zero, two-digit minutes, `AM` when hour < 12 else `PM`, seconds dropped. Doc comments with `/// R31:` on `format_time`, `format_time_range`, `format_datetime` (bodies of the latter two unchanged). Needs T010
- [x] T012 [US1] (builder) Implement `times_to_12h` (rewrite each standalone `HH:MM` token: two digits, `:`, two digits, a valid time, no digit directly before or after; other text unchanged) and apply it in `error_text` to `Error::Validation` messages after the existing capitalisation. Remove its `#[allow(dead_code)]`. Doc `/// R31:` citing the T09 `display_status_text` precedent. Needs T011
- [x] T013 [US2] (builder) Implement `parse_time(digits, pm)` per data-model.md "Parse rules": trim; remove every `:`; 1–4 ASCII digits else error; 1–2 digits = hour with minute 0; 3–4 digits = hour from all but the last two digits, minute from the last two; hour 1–12 and minute 0–59 else error; AM → `hour % 12`, PM → `hour % 12 + 12`; every error is `TIME_FORMAT_ERR`. Doc `/// R32:`. Needs T011
- [x] T014 [US3] (builder) Implement `format_time_input`: `hmm` = 12-hour hour without a leading zero followed by two-digit minutes, plus `pm = hour >= 12`. Doc `/// R32:`. Run `cargo test -p app`; all tests pass. Needs T013

### UI components and page wiring

- [x] T015 [US2] (builder) In crates/app/ui/components.slint add `export component AmPmToggle` (`in-out property <bool> pm`; two text segments "AM"/"PM" on a `Theme.surface-alt` pill, active segment `Theme.surface` with a `Theme.border` border, modelled on `AppearanceSegment`/`AppearanceSwitch`: `accessible-role: button`, `accessible-label` "AM"/"PM", TouchArea click, FocusScope with Enter/Space; a segment click sets `pm` to false/true) and `export component TimeField inherits VerticalLayout` (`label`, in-out `text`, in-out `pm`, `error`; same label, bordered single-line input, and inline error as `Field`, with `AmPmToggle` beside the input in the input row). Only existing `Theme.*` tokens; no new token in theme.slint; no raw hex or px. Needs T002
- [x] T016 [US2] (builder) crates/app/ui/timetable.slint: replace the four time `Field`s (reschedule near lines 216/221, slot form near lines 355/360) with `TimeField`, binding `text <=>` and `pm <=>` to the paired `TimetableState` properties and keeping the `error` bindings; labels "Start" and "End" (drop "(HH:MM)"); import `TimeField`. Needs T015
- [x] T017 [P] [US2] (builder) crates/app/ui/courses.slint: replace the two makeup time `Field`s (near lines 355/360) with `TimeField` bound to `mk-start`/`mk-start-pm` and `mk-end`/`mk-end-pm`; labels "Start" and "End"; import `TimeField`. Needs T015
- [x] T018 [P] [US2] (builder) crates/app/ui/assessments.slint: replace the due-time `Field` (near line 313) with `TimeField` bound to `due-time`/`due-time-pm`; label "Due time"; import `TimeField`. Needs T015
- [ ] T019 [US3] (builder) crates/app/src/pages/timetable.rs: reschedule open (near line 195) and slot edit open (near line 373) set text and pm from `vm::format_time_input(...)`; new/cleared slot form (near line 321) sets text empty and both `*-pm` false. Needs T014, T016
- [ ] T020 [P] [US3] (builder) crates/app/src/pages/courses.rs: `clear_makeup_form` also sets `mk-start-pm` and `mk-end-pm` to false. Needs T017
- [ ] T021 [P] [US3] (builder) crates/app/src/pages/assessments.rs: new form (near line 202) sets `due-time` empty and `due-time-pm` false; edit open (near line 234) sets both from `vm::format_time_input(assessment.due_at.time())`. Needs T014, T018
- [ ] T022 [US1] (builder) Display audit: confirm every displayed time goes through `format_time` / `format_time_range` / `format_datetime` / `error_text` (`grep -rn '%H' crates/app/src` shows only `pre_restore_file_name`, a file name); crates/app/src/pages/dashboard.rs needs no change. Fix any other 24-hour display path found. Needs T012
- [ ] T023 (builder) **Only if the human answers OQ-1 = B** (router decides): in crates/app/src/notify.rs pass `reminder.title` through `crate::viewmodel::times_to_12h` before showing it; doc `/// R31:`. Skip and mark "skipped (OQ-1 = A)" otherwise. Needs T012

---

## Phase 4: Gate and Done-when

- [ ] T024 (builder) Remove leftover `#[allow(dead_code)]` from T001; run `cargo test -p app t34_`, `cargo test -p app t35_`, the gate `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`, and the theme grep `! grep -rnE '#[0-9a-fA-F]{6}|[0-9]+px' crates/app/ui --include=*.slint | grep -v theme.slint`; confirm `git diff --stat` shows no file under `crates/core/`; paste the summary. Needs T019–T022 (and T023 if kept)
- [ ] T025 (human) Manual checks from quickstart.md "Manual" steps 1–7 with `cargo run -p app` (every page 12-hour, each of the 7 time fields saves `930` + PM as 9:30 PM, pre-fill on edit, keyboard toggle, backup/CSV still 24-hour); report any failure to the router. Needs T024

---

## Dependencies & Execution Order

- Round 1 (T001–T004) → round 2 (T005–T010) → round 3 (T011–T023) → T024 → T025.
- Round 1: T002 runs in parallel with T001; T003 needs both; T004 needs T003.
- Round 2 is one file, so tasks run in order.
- Round 3: T011 → T012/T013 → T014. UI: T015 → T016/T017/T018 (parallel) → T019/T020/T021.
  T022 and T023 need T012.

## Parallel Examples

- T001 (viewmodel.rs) with T002 (.slint state properties).
- T017 (courses.slint) and T018 (assessments.slint) after T015, alongside T016.
- T020 (courses.rs) and T021 (assessments.rs) after their `.slint` tasks.

## Implementation Strategy

1. MVP = US1 (display) + US2 (input): both P1, delivered together in round 3 since input and
   display share `viewmodel.rs` and the toggle.
2. US3 (pre-fill on edit) follows in the same round (T014, T019–T021).
3. Gate (T024), then the human manual pass (T025).

## Owners

- **builder**: T001–T004, T011–T024 (T023 conditional on OQ-1)
- **tester**: T005–T010
- **human**: T025
