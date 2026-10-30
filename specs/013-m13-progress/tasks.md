---
description: "Task list for M13 Progress page"
---

# Tasks: M13 Progress Page

**Input**: Design documents from `specs/013-m13-progress/`

**Prerequisites**: [plan.md](plan.md) (design, view-model and Slint contracts), [spec.md](spec.md)
(R61–R65, T62–T65), [decisions.md](decisions.md) (D1–D14)

**Tests**: Required. T62–T65 come from `docs/spec.md` §13.8; rule tests (`r61_…`–`r65_…`) cover
the remaining texts of the plan's view-model contract. All tests are unit tests in
`crates/app/src/viewmodel/tests.rs` that build core types directly (no database). Every tester
task runs before the builder task that implements it (constitution II).

**Scope guard**: no change under `crates/core/`, no migration, no new core API, no new theme token.

## Format: `[ID] [P?] [Story] (owner) Description`

- **[P]**: can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: user story from spec.md: US1 page, order, status (R61, R65); US2 attendance and
  classes you can still miss (R62, R63); US3 marks and best grade (R64)
- **(tester)** / **(builder)** / **(human)**: who does the task
- **Commit**: one commit per task, message prefixed `M13:`. The suggested message is given per
  task. The tester commits failing tests (they panic on the `todo!` stubs, as in M9); the gate is
  green again after T009.

## Path Conventions

New: `crates/app/src/pages/progress.rs`, `crates/app/ui/progress.slint`. Edited:
`crates/app/src/viewmodel.rs`, `crates/app/src/viewmodel/tests.rs`, `crates/app/src/bridge.rs`,
`crates/app/src/controller.rs`, `crates/app/src/pages/mod.rs`, `crates/app/ui/types.slint`,
`crates/app/ui/app-window.slint`, `specs/013-m13-progress/decisions.md`.

---

## Phase 1: Setup

- [ ] T001 (builder) Decisions log: read specs/013-m13-progress/decisions.md (D1–D14) before
  starting. During any later task, append one line per new build-time decision to
  specs/013-m13-progress/decisions.md as `- D15 (builder): …`, `- D16 (builder): …`, and commit
  it with the task that made the decision. A choice that would change a rule, the schema, the
  API, or scope is not decided: stop and report it to the router. No commit for this task.

- [ ] T002 (builder) View-model stubs: in crates/app/src/viewmodel.rs add a section
  `// ---- M13: Progress page ----` with `ProgressStatus` (variants `AtRisk, Watch, OnTrack,
  NotEnoughData` in that order, derives `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord`),
  `ProgressCardView`, `ProgressView`, and the functions `progress_status_label`, `progress_chip`,
  `progress_status`, `letter_points`, `miss_line`, `attendance_line`, `progress_target_text`,
  `bar_fraction`, `target_fraction`, `progress_marks_line`, `best_grade_line`,
  `progress_counts_text`, `progress_empty_text`, `progress_card`, `progress_view`, with the exact
  signatures and field lists of plan.md "View-model contract" and bodies `todo!("M13: R6x")`.
  Doc comments cite R61–R65. Put `#[allow(dead_code)]` only on these new unused items (removed in
  T012). Builds; existing tests green. Commit `M13: view-model stubs for the Progress page`.

---

## Phase 2: Tests first (T62–T65)

All four tasks edit crates/app/src/viewmodel/tests.rs, so they run one after another. Add a
section `// ---- M13: Progress page ----`; add `AttendanceSummary`, `GradeRow`, `NeededRow`,
`Needed`, `GpaSummary`, `GpaStatus` imports as needed. Reuse the existing `course_marks(outlook)`
helper (graded, secured 129.0, assessed 150.0, projected 86.0, letter A+) where it fits. Each
test has a doc comment citing its T or R ID. Run the tests and record which fail (all should
panic on `todo!`).

- [ ] T003 [US2] (tester) T62 and R62/R63 tests. Needs T002.
  - `t62_classes_you_can_still_miss`: `miss_line(a, b, r, t)` gives
    (10, 2, 8, Some(75)) → "You can miss 3 of the 8 remaining classes and stay at or above 75%";
    (10, 4, 4, Some(75)) → "Attend all 4 remaining classes to stay at or above 75%";
    (5, 5, 2, Some(75)) → "75% is out of reach, even attending all 2 remaining classes";
    (20, 0, 2, Some(75)) → "You can miss all 2 remaining classes and stay at or above 75%";
    (3, 1, 0, Some(75)) → "No classes left to mark"; (10, 2, 8, None) → "".
  - `r63_miss_line_floors_negative_numerator` (D4): (6, 3, 2, Some(75)) → numerator −25, k = −1 →
    "75% is out of reach, even attending all 2 remaining classes"; (7, 3, 2, Some(75)) → k = 0 →
    "Attend all 2 remaining classes to stay at or above 75%".
  - `r63_miss_line_singular_class` (D6): (10, 0, 1, Some(75)) → "You can miss all 1 remaining
    class and stay at or above 75%".
  - `r62_attendance_line`: (14, 2, Some(87.5)) → "87.5% · 14 of 16 classes"; (1, 0, Some(100.0))
    → "100.0% · 1 of 1 class" (D6); (0, 0, None) → "No classes marked yet".
  - `r62_target_text_and_bar`: `progress_target_text(Some(75))` == "Target 75%", `(None)` ==
    "No target" (D7); `bar_fraction(Some(87.5))` == 0.875, `(None)` == 0.0, `(Some(100.0))` ==
    1.0; `target_fraction(Some(75))` == Some(0.75), `(None)` == None.
  - Commit `M13: T62 classes-you-can-miss and R62 attendance tests`.

- [ ] T004 [US3] (tester) T64 and R64 tests. Needs T003.
  - `t64_best_reachable_grade`: `best_grade_line(Some(&FinalOutlook::Needed { rows: [A+
    NotReachable, A Score(96.0), A- Score(80.0)], final_item_max: 150.0, assumptions: [] }))` ==
    "A needs 96.0 of 150 in the final"; rows `[A+ Secured, A Secured]` → "A+ is already secured";
    `FinalOutlook::Actual { letter: Some("A") }` → "Final grade: A".
  - `r64_best_grade_line_hidden` (D10): `best_grade_line(None)` == ""; rows all `NotReachable`
    → "".
  - `r64_marks_line`: `course_marks(None)` with `graded: false` → "Not graded"; graded with
    `projected_pct: None` → "No marks entered yet"; `course_marks(None)` → "Secured 129.0 of 150
    assessed marks" (D9).
  - Commit `M13: T64 best reachable grade and R64 marks tests`.

- [ ] T005 [US1] (tester) T63 and R65 tests. Needs T004.
  - `t63_progress_status`: `progress_status(below_target, pct, target, projected_pct, points)`
    gives (true, Some(70.0), Some(75), None, None) → AtRisk; (false, Some(78.0), Some(75), None,
    None) → Watch; (false, Some(90.0), Some(75), Some(75.0), Some(3.0)) → OnTrack; (false,
    Some(90.0), Some(75), Some(72.0), Some(2.75)) → Watch; (false, Some(90.0), Some(75),
    Some(40.0), Some(0.0)) → AtRisk; (false, None, Some(75), None, None) → NotEnoughData.
  - `r65_status_boundaries` (D12): (false, Some(80.0), Some(75), None, None) → OnTrack; (false,
    Some(79.9), Some(75), None, None) → Watch; (false, Some(60.0), None, None, None) → OnTrack;
    (false, None, None, Some(86.0), Some(4.0)) → OnTrack.
  - `r65_letter_points` (D11): a scale `[A+ 80 4.0, B 65 3.0, B- 60 2.75, F 0 0.0]`;
    `letter_points(Some("B"), &scale)` == Some(3.0); `Some("Z")` → None; `None` → None.
  - `r65_label_and_chip`: labels "At risk", "Watch", "On track", "Not enough data"; chips
    Absent, Pending, Attended, Cancelled respectively.
  - Commit `M13: T63 progress status tests`.

- [ ] T006 [US1] (tester) T65 and R61 tests. Needs T005.
  - Helper `summary(code, attended, absent, pending, pct, target, below_target) ->
    AttendanceSummary` (course_name `format!("{code} name")`, cancelled 0).
  - `r61_progress_card_fields`: summary (CSE101, 14, 2, 8, Some(87.5), Some(75), false) with
    `course_marks(None)` and the T005 scale → card: code "CSE101", status OnTrack, status_label
    "On track", chip Attended, attendance_figure "87.5%", attendance_line "87.5% · 14 of 16
    classes", target_text "Target 75%", bar_fill 0.875, has_marker true, marker_at 0.75,
    miss_line `miss_line(14, 2, 8, Some(75))`, marks_figure "86.0% · A+", marks_line "Secured
    129.0 of 150 assessed marks", best_grade_line "". An ungraded course: marks_figure "—",
    marks_line "Not graded". No target: has_marker false, marker_at 0.0, miss_line "".
  - `t65_progress_order_and_counts`: four cards built with `progress_card` (T005 scale) in code
    order AAA101 (On track: 90.0 vs 75, `course_marks(None)` A+ = 4.0), BBB101 (Not enough data:
    no pct, ungraded), CCC101 (At risk: 70.0 vs 75, `below_target` true, ungraded), DDD101
    (Watch: 78.0 vs 75, ungraded); "ungraded" = `CourseMarks { graded: false, projected_pct:
    None, letter: None, outlook: None, .. }`; `progress_view(&gpa, cards)` orders CCC101, DDD101, AAA101,
    BBB101; `counts_text` == "1 on track · 1 to watch · 1 at risk"; `gpa_text` ==
    `gpa_header_text(&gpa)`.
  - `r61_same_status_sorted_by_code`: two At risk cards given as ZZZ101, AAA101 come out AAA101,
    ZZZ101.
  - `r61_empty_text` (D13): `progress_empty_text(false)` == "No term yet. Create one in Settings
    → Term."; `progress_empty_text(true)` == "No courses yet. Add one on the Courses page.";
    `progress_counts_text(&[])` == "0 on track · 0 to watch · 0 at risk".
  - Commit `M13: T65 progress order and header tests`.

---

## Phase 3: User Story 2 - Attendance and classes I can still miss (Priority: P1)

**Goal**: R62 and R63 texts and bar fractions. **Independent test**: T003 tests pass.

- [ ] T007 [US2] (builder) Implement `miss_line`, `attendance_line`, `progress_target_text`,
  `bar_fraction`, `target_fraction` in crates/app/src/viewmodel.rs per plan.md "Key rule
  details" (R63 order D5; `i64::div_euclid` D4; nouns via the existing `count_text`-style
  singular/plural D6; pct text via `fixed(p, 1)` as `format_pct` does; fractions clamped to
  0.0..=1.0). Doc comments `/// R62: …`, `/// R63: …`. T003 tests pass. Commit `M13: attendance
  and classes-you-can-miss lines (R62, R63)`. Needs T006.

---

## Phase 4: User Story 3 - Marks and best reachable grade (Priority: P1)

**Goal**: R64 texts. **Independent test**: T004 tests pass.

- [ ] T008 [US3] (builder) Implement `progress_marks_line` and `best_grade_line` in
  crates/app/src/viewmodel.rs: not graded → "Not graded"; `projected_pct` None → "No marks
  entered yet"; else `format!("Secured {} of {} assessed marks", fixed(secured, 1),
  format_mark(assessed))`; best grade: `FinalOutlook::Needed` → first row whose `needed` is not
  `NotReachable`: `Secured` → "{letter} is already secured", `Score(s)` → "{letter} needs
  {needed_text} of {format_mark(final_item_max)} in the final", none → ""; `Actual` →
  `actual_grade_text`; `None` → "". Doc comments `/// R64: …`. T004 tests pass. Commit `M13:
  marks and best reachable grade lines (R64)`. Needs T007.

---

## Phase 5: User Story 1 - Every course's progress at a glance (Priority: P1)

**Goal**: R65 status, R61 card order and header, and the page itself.
**Independent test**: T005 and T006 tests pass; the page opens from the sidebar between Dashboard
and Timetable.

- [ ] T009 [US1] (builder) Implement `progress_status`, `letter_points`, `progress_status_label`,
  `progress_chip`, `progress_counts_text`, `progress_empty_text`, `progress_card`, and
  `progress_view` in crates/app/src/viewmodel.rs per plan.md (R65 rule detail; points only when
  `projected_pct` is Some, D11; `marks_figure` = `format_projected` when graded and projected,
  else "—", D9; `gpa_text` = `gpa_header_text`, D13; stable sort by `(status, code)`). Doc
  comments `/// R61: …`, `/// R65: …`. All T62–T65 and r61–r65 tests pass; the full gate is green.
  Commit `M13: progress status, cards, and header (R61, R65)`. Needs T008.

- [ ] T010 [US1] (builder) Slint page: add `ProgressCardRow` to crates/app/ui/types.slint (fields
  of plan.md "Slint contract"); create crates/app/ui/progress.slint with `export global
  ProgressState` (message, has-term, empty-text, gpa-text, counts-text, cards; no callbacks), a
  local `ProgressBar` component, and `export component ProgressPage` laid out per plan.md and
  DESIGN.md "Progress card" (reuse `PageTitle`, `EmptyState`, `StatusChip` from components.slint;
  card on `Theme.surface`, `Theme.radius-none`, `Theme.space-lg` padding, title row
  `Theme.row-height`, figures `Theme.numeral-md-size`/`-weight`, captions `Theme.label-sm-*`, bar
  `Theme.progress-track`/`progress-fill`/`progress-target`, `Theme.progress-height`,
  `Theme.progress-target-width`). Bindings only: no string building or conditionals beyond
  `if text != ""` / `if has-marker` / the empty-state condition. In crates/app/ui/app-window.slint
  import and export `ProgressState` (and `ProgressCardRow` in the types export list), add
  `NavItem { text: "Progress"; active: AppState.current-page == 7; … page-selected(7) }`
  directly after the Dashboard item (D1), and `if AppState.current-page == 7: ProgressPage { }`.
  No raw hex or `px` anywhere in these files, comments included. Builds. Commit `M13: Progress
  page layout and nav item (R61, R85)`. Needs T009.

- [ ] T011 [US1] (builder) Rust wiring: `pub fn progress_card(c: &vm::ProgressCardView) ->
  ProgressCardRow` in crates/app/src/bridge.rs (chip via `bridge::chip`); create
  crates/app/src/pages/progress.rs with `pub fn refresh(app: &App)` per plan.md "Data flow" (no
  `wire`, D2; module doc comment cites R61–R65; borrows of the planner end before Slint setters;
  errors via `viewmodel::error_text` into `ProgressState.message`; `has-term`, `empty-text` from
  `progress_empty_text`); `pub mod progress;` in crates/app/src/pages/mod.rs; in
  crates/app/src/controller.rs call `pages::progress::refresh(self)` in `refresh_all` and map
  page `7` to `ProgressState` in `show_message` (not in `tick`, D3). Builds; gate green. Commit
  `M13: wire the Progress page refresh`. Needs T010.

---

## Phase 6: Polish and gate

- [ ] T012 (builder) Remove every `#[allow(dead_code)]` and `todo!` added in T002; confirm
  `git diff --stat main -- crates/core` is empty and no file was added under
  crates/core/src/migrations; run the theme grep `! grep -rnE '#[0-9a-fA-F]{6}|[0-9]+px'
  crates/app/ui --include=*.slint | grep -v theme.slint`; run the gate `cargo fmt --check &&
  cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` and paste the
  summary. Commit `M13: cleanup and gate` only if files changed. Needs T011.

- [ ] T013 (builder) Decisions check: make sure specs/013-m13-progress/decisions.md lists every
  build-time decision (D15+) from T002–T012, one line each, and that D1–D14 still match the code;
  add a line for any mismatch. Commit `M13: decisions log` if changed. Needs T012.

- [ ] T014 (human) Open the app, click Progress (between Dashboard and Timetable) in light and
  dark: cards ordered At risk first, chips coloured per R65, the bar marker at the target, the
  empty state with no courses. Report anything that looks wrong. Needs T013.

---

## Dependencies and Execution Order

- T001 → T002 → T003 → T004 → T005 → T006 (tests all in one file) → T007 → T008 → T009 → T010 →
  T011 → T012 → T013 → T014.
- Story order: US2 (T007) and US3 (T008) first, because `progress_card` in US1 (T009) uses their
  functions; the page (T010, T011) closes US1.
- Gate: red from T003 until T009 (tests panic on stubs, as in M9); green after T009 and at
  T011, T012.

## Parallel Opportunities

- None worth taking: every view-model task edits crates/app/src/viewmodel.rs or its tests.rs,
  and T010 depends on the struct fields fixed in T009. After T009, T010 (Slint) and the bridge
  part of T011 could be drafted at the same time by one builder, but they are committed in order.

## Implementation Strategy

- MVP: T001–T009 (all rules tested and implemented in the view model, gate green), then the page
  (T010, T011). Stop after T012 for the router's review; T014 is the human's look.
