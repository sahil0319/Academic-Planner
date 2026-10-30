# Feature Specification: M13 Progress Page

**Feature Branch**: `013-m13-progress` (spec directory; work happens on the current branch)

**Created**: 2026-10-31

**Status**: Draft

**Input**: "Milestone M13 from docs/spec.md §13.8: rules R61–R65, tests T62–T65. Do not add
requirements."

**Source of truth**: `docs/spec.md` §13.8 (R61–R65, T62–T65), §13.12 R85 (nav order), §4 R13–R15
(attendance summary), §13.3 R33, R37, R38, R40 (marks, needed in final, term GPA). Visuals:
`DESIGN.md` "Progress card", "Status chip", "Section card", "Empty state", "Numeral" type, and the
M17 `progress-*` tokens in `crates/app/ui/theme.slint`. This file copies the rules so the plan and
tasks can cite them; if the two differ, `docs/spec.md` wins.

## Scope

A second dashboard that shows how each course is going so far. It reads existing data only:
no schema change, no new core API, no core change. It uses `all_summaries`, `course_marks`,
`grade_scale`, and `term_gpa`. Every computation is a pure function in
`crates/app/src/viewmodel.rs` with unit tests (§13.8 intro). The page is read-only: it has no
form, no button that changes data, and no toast.

## Rules (copied from docs/spec.md §13.8)

- **R61** A new sidebar page **Progress**, between Dashboard and Timetable. The top line shows the
  term GPA (R40) and the counts "N on track · N to watch · N at risk". Below it, one card per
  non-archived course of the active term, ordered At risk, Watch, On track, Not enough data, then
  by code. No active term or no courses → an empty state.
- **R62** Attendance on a card: the R13 percentage with "attended of marked" counts
  ("87.5% · 14 of 16 classes"), the target, and a bar from 0 to 100% with a marker at the target
  (no marker without a target). No marked class yet → "No classes marked yet" and an empty bar.
- **R63** Classes you can still miss: with target `t`, attended `a`, absent `b`, and `r` = the
  course's pending sessions (R15, past and future),
  `k = floor((100 × (a + r) − t × (a + b + r)) / 100)` in integer arithmetic. No target → no line.
  `r = 0` → "No classes left to mark". `k ≥ r` → "You can miss all {r} remaining classes and stay
  at or above {t}%". `0 < k < r` → "You can miss {k} of the {r} remaining classes and stay at or
  above {t}%". `k = 0` → "Attend all {r} remaining classes to stay at or above {t}%". `k < 0` →
  "{t}% is out of reach, even attending all {r} remaining classes".
- **R64** Marks on a card (graded courses, R33): "Secured {secured} of {assessed} assessed marks"
  and the projected percentage with its letter (R37), e.g. "86.7% · A+". Not graded →
  "Not graded"; nothing assessed → "No marks entered yet". Best grade still reachable, from R38:
  the first grade row (highest first) that is not "Not reachable": "Secured" → "{letter} is
  already secured"; a score → "{letter} needs {score} of {final item max} in the final". If the
  final already has a score (R38 actual), "Final grade: {letter}".
- **R65** Status chip per course: **At risk** when below target (R14) or the projected letter's
  grade points are 0; otherwise **Watch** when the attendance percentage is less than 5 points
  above the target, or the projected letter's grade points are below 3.00; otherwise **On track**
  when there is an attendance percentage or a projected percentage; otherwise **Not enough data**.
  Chip colours: On track = attended, Watch = pending, At risk = absent, Not enough data =
  cancelled.
- **R85** (§13.12) Nav order Dashboard, Progress, Timetable, Courses, Assessments, Rules,
  Settings, About.

**UI** (§13.8): the card layout is the DESIGN.md "Progress card". It uses the M17 `Theme.*`
tokens, including `progress-track`, `progress-fill`, `progress-target`, `progress-height`, and
`progress-target-width`. Square corners, 40px rows, no raw hex or px outside `theme.slint`.

## User Scenarios & Testing

The user is a student who wants one screen that says, per course, whether attendance and marks
are on track and what is still possible.

### User Story 1 - See every course's progress at a glance (Priority: P1)

The user opens **Progress** from the sidebar. The top line shows the term GPA and how many
courses are on track, to watch, and at risk. Below, one card per course, the riskiest first.

**Independent Test**: T65 (order and header counts) plus T63 (status) in
`crates/app/src/viewmodel/tests.rs`; then open the page in the running app.

**Acceptance Scenarios**:

1. **Given** four courses with each status, **When** the page is built, **Then** the cards are
   ordered At risk, Watch, On track, Not enough data and the header reads
   "1 on track · 1 to watch · 1 at risk" (T65, R61).
2. **Given** the T63 inputs, **When** the status is computed, **Then** the statuses are At risk;
   Watch; On track; Watch; At risk; Not enough data (T63, R65).
3. **Given** no active term, or an active term with no non-archived course, **When** the page is
   opened, **Then** an empty state is shown instead of the header and cards (R61).
4. **Given** the sidebar, **When** it is shown, **Then** Progress sits between Dashboard and
   Timetable (R61, R85).

### User Story 2 - Attendance and classes I can still miss (Priority: P1)

Each card shows the attendance percentage with counts, the target, a bar with a target marker,
and how many remaining classes the user can miss.

**Independent Test**: T62 in `crates/app/src/viewmodel/tests.rs`.

**Acceptance Scenarios**:

1. **Given** (a, b, r, t) = (10, 2, 8, 75), (10, 4, 4, 75), (5, 5, 2, 75), (20, 0, 2, 75),
   (3, 1, 0, 75), and no target, **When** the miss line is built, **Then** it reads miss 3 of 8;
   attend all 4; out of reach; miss all 2; no classes left; no line (T62, R63).
2. **Given** no attended or absent class, **When** the card is built, **Then** it shows
   "No classes marked yet" and an empty bar (R62).
3. **Given** a course without a target, **When** the bar is drawn, **Then** it has no marker
   (R62).

### User Story 3 - Marks and best reachable grade (Priority: P1)

Each graded card shows secured and assessed marks, the projected percentage with its letter, and
the best grade still reachable.

**Independent Test**: T64 in `crates/app/src/viewmodel/tests.rs`.

**Acceptance Scenarios**:

1. **Given** needed rows A+ not reachable and A 96.0 of 150, **When** the best-grade line is
   built, **Then** it reads "A needs 96.0 of 150 in the final" (T64, R64).
2. **Given** the first row is secured, **Then** "A+ is already secured" (T64).
3. **Given** the final is scored with A, **Then** "Final grade: A" (T64).
4. **Given** an ungraded course, **Then** "Not graded"; **given** a graded course with nothing
   assessed, **then** "No marks entered yet" (R64).

## Requirements

No functional requirement beyond R61–R65 and R85 above. Constraints from the constitution:
pure view-model functions with unit tests written first (Principle II, III); `.slint` holds only
bindings (Principle III); `Theme.*` only (Principle I); gate passes (Principle V).

## Success Criteria

- T62–T65 pass in `cargo test --workspace`.
- The gate passes: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings &&
  cargo test --workspace`.
- The §12 theme grep finds no raw hex or px in `crates/app/ui/progress.slint` or the changed
  `app-window.slint`.
- `crates/core` is unchanged; no migration file is added.
