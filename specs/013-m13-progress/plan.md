# Implementation Plan: M13 Progress Page

**Branch**: `013-m13-progress` (spec directory; work happens on the current branch) | **Date**: 2026-10-31 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/013-m13-progress/spec.md`

## Summary

A read-only **Progress** page: a top line (term GPA and status counts) and one card per
non-archived course of the active term with attendance, a target bar, classes you can still miss,
marks, the best reachable grade, and a status chip (R61–R65). `docs/spec.md` §13.8 is the plan.

- **No core change.** The page reads `active_term`, `all_summaries`, `course_marks`,
  `grade_scale`, and `term_gpa`, which exist since M1/M9. No schema change, no migration, no new
  core API.
- **All logic in `crates/app/src/viewmodel.rs`** as pure functions, tested in
  `crates/app/src/viewmodel/tests.rs` (T62–T65 plus `r6x_…` rule tests). Tests come first.
- **New page** `crates/app/src/pages/progress.rs` (`refresh` only) and `crates/app/ui/progress.slint`
  (`ProgressState` global and `ProgressPage`), wired into `app-window.slint`, `controller.rs`, and
  `main.rs` like the other pages. Nav item between Dashboard and Timetable; page index **7** (no
  existing index changes).

Interpretations are listed in [decisions.md](decisions.md) (D1–D14). None changes a rule, the
schema, or the API.

## Technical Context

**Language/Version**: Rust stable, edition 2021; Slint 1.16 markup.

**Primary Dependencies**: existing `planner_core`, `slint`, `chrono`. No new crate.

**Storage**: none touched (read-only page; no schema, migration, or settings change).

**Testing**: `cargo test --workspace`. New unit tests in `crates/app/src/viewmodel/tests.rs`; no
database (core types are built directly, as the M9 view-model tests do). The §12 theme grep.

**Target Platform**: macOS, Windows, Linux desktop (unchanged).

**Project Type**: desktop app (Cargo workspace: `crates/core`, `crates/app`).

**Performance Goals**: one `course_marks` call per course on each refresh; a term has a handful of
courses, so no caching.

**Constraints**: `Theme.*` only, no raw hex or px in `.slint` (grep scans comments too); square
corners; 40px rows (`Theme.row-height`); no logic in `.slint` beyond bindings; no toast; no
system clock in core.

**Scale/Scope**: 1 new page, ~12 view-model functions, ~2 structs, 1 Slint struct, 1 global.

## Constitution Check

| Principle | Status |
| --- | --- |
| I. Spec is the source of truth | Pass. Texts and rules copied from §13.8; wording gaps resolved in decisions.md without new behaviour. Visuals from DESIGN.md "Progress card" and M17 tokens; no new token. |
| II. Test-first, traceable | Pass. T62–T65 and `r61_…`–`r65_…` tests are written (tasks T002–T004) before the bodies (T005). Doc comments cite R61–R65. No database in these tests. |
| III. Pure, clock-free core | Pass. Core untouched. Every text, fraction, status, and order is a view-model function; `progress.slint` only binds. |
| IV. Append-only migrations | Pass. No migration. |
| V. Gate before done | Pass. Final task runs the gate and the theme grep and pastes the summary. |

Post-design re-check: unchanged, pass. No Complexity Tracking entries.

## Design

### Data flow (pages/progress.rs `refresh`)

1. `active_term()`; error → `ProgressState.message`; `None` → `has-term = false`, empty state.
2. `term_gpa()`, `grade_scale()`, `all_summaries()` (non-archived courses of the active term, by
   code, R3/R15).
3. For each summary: `course_marks(summary.course_id)`; build
   `viewmodel::progress_card(&summary, &marks, &scale)`.
4. `viewmodel::progress_view(&gpa, cards)` sorts and counts; copy into `ProgressState` via
   `bridge::progress_card`.
5. Any core error → `viewmodel::error_text` into `ProgressState.message` and stop (as Dashboard).

All planner borrows end before the Slint setters run (as `dashboard::fill_calendar` does).

### View-model contract (crates/app/src/viewmodel.rs, new section "M13: Progress page")

```rust
/// R65: card status; declaration order is the R61 card order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProgressStatus { AtRisk, Watch, OnTrack, NotEnoughData }

/// R65: "At risk", "Watch", "On track", "Not enough data".
pub fn progress_status_label(status: ProgressStatus) -> &'static str;
/// R65: On track = Attended, Watch = Pending, At risk = Absent, Not enough data = Cancelled.
pub fn progress_chip(status: ProgressStatus) -> ChipKind;
/// R65: `points` is the grade points of the projected letter (None when no projected letter).
pub fn progress_status(below_target: bool, pct: Option<f64>, target: Option<i64>,
                       projected_pct: Option<f64>, points: Option<f64>) -> ProgressStatus;
/// R65: points of `letter` in the grade scale; None when the letter is None or not in the scale.
pub fn letter_points(letter: Option<&str>, scale: &[GradeRow]) -> Option<f64>;

/// R63: k = floor((100(a + r) - t(a + b + r)) / 100) in i64 (`div_euclid`); "" = no line.
pub fn miss_line(attended: u32, absent: u32, remaining: u32, target: Option<i64>) -> String;
/// R62: "87.5% · 14 of 16 classes", or "No classes marked yet" when attended + absent = 0.
pub fn attendance_line(attended: u32, absent: u32, pct: Option<f64>) -> String;
/// R62: "Target 75%", or "No target".
pub fn progress_target_text(target: Option<i64>) -> String;
/// R62: bar fill 0.0..=1.0 = pct / 100, clamped; 0.0 when pct is None (empty bar).
pub fn bar_fraction(pct: Option<f64>) -> f32;
/// R62: marker position 0.0..=1.0 = target / 100, clamped; None = no marker.
pub fn target_fraction(target: Option<i64>) -> Option<f32>;

/// R64: "Not graded" | "No marks entered yet" | "Secured 129.0 of 150 assessed marks".
pub fn progress_marks_line(marks: &CourseMarks) -> String;
/// R64, R38: "A needs 96.0 of 150 in the final" | "A+ is already secured" | "Final grade: A";
/// "" when there is no outlook (not graded) or every row is Not reachable.
pub fn best_grade_line(outlook: Option<&FinalOutlook>) -> String;

/// R61: "1 on track · 1 to watch · 1 at risk" (Not enough data is not counted).
pub fn progress_counts_text(statuses: &[ProgressStatus]) -> String;
/// R61: empty-state text; `has_term` false → no-term text, else no-courses text.
pub fn progress_empty_text(has_term: bool) -> &'static str;

/// R61–R65: one card.
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressCardView {
    pub code: String,            // sort key and title
    pub name: String,
    pub status: ProgressStatus,
    pub status_label: String,    // progress_status_label
    pub chip: ChipKind,          // progress_chip
    pub attendance_figure: String, // format_pct(pct): "87.5%" or "—"
    pub attendance_line: String,   // attendance_line
    pub target_text: String,       // progress_target_text
    pub bar_fill: f32,             // bar_fraction
    pub has_marker: bool,          // target_fraction(..).is_some()
    pub marker_at: f32,            // target_fraction(..).unwrap_or(0.0)
    pub miss_line: String,         // miss_line(attended, absent, pending, target)
    pub marks_figure: String,      // format_projected(projected_pct, letter) when graded, else "—"
    pub marks_line: String,        // progress_marks_line
    pub best_grade_line: String,   // best_grade_line(marks.outlook.as_ref())
}
pub fn progress_card(summary: &AttendanceSummary, marks: &CourseMarks, scale: &[GradeRow])
    -> ProgressCardView;

/// R61: header texts and cards sorted by (status, code).
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressView { pub gpa_text: String, pub counts_text: String,
                          pub cards: Vec<ProgressCardView> }
/// R61: gpa_text = gpa_header_text(gpa) (R40, reused); counts from the cards.
pub fn progress_view(gpa: &GpaSummary, cards: Vec<ProgressCardView>) -> ProgressView;
```

Key rule details:

- **R63** checks in this order: no target → `""`; `r = 0` → "No classes left to mark"; then
  `k ≥ r`, `0 < k < r`, `k = 0`, `k < 0`. `k` uses `i64::div_euclid` so a negative numerator
  floors (−50 / 100 = −1, out of reach), not truncates to 0. Worked values for T62: (10,2,8,75)
  k = 3; (10,4,4,75) k = 0; (5,5,2,75) k = −2; (20,0,2,75) k = 5; (3,1,0,75) r = 0.
- **R65** with `points = letter_points(marks.letter, scale)` only when `marks.projected_pct` is
  Some: At risk if `below_target` or `points == Some(0.0)`; else Watch if
  (`pct`, `target`) both Some and `pct < target + 5.0`, or `points < 3.0`; else On track if `pct`
  or `projected_pct` is Some; else Not enough data. Ungraded courses have no projected pct.
- **R64** secured uses one decimal (`fixed(secured, 1)`), assessed uses `format_mark` (same
  pattern as `format_marks`); the score uses `needed_text` (one decimal) and the final item max
  `format_mark`. `FinalOutlook::Actual` reuses `actual_grade_text`.
- **R61** the sort is stable on `(status, code)`; `all_summaries` is already by code.

Reused, unchanged: `format_pct`, `format_projected`, `format_mark`, `fixed`, `needed_text`,
`actual_grade_text`, `gpa_header_text`, `count_text`, `error_text`, `ChipKind`.

### Slint contract

- `crates/app/ui/types.slint`: `export struct ProgressCardRow { code, name, status-label: string,
  chip: Chip, attendance-figure, attendance-line, target-text: string, bar-fill: float,
  has-marker: bool, marker-at: float, miss-line, marks-figure, marks-line, best-grade-line: string }`
  (exported from `app-window.slint` like the other row structs).
- `crates/app/ui/progress.slint`: `export global ProgressState { in property <string> message;
  in property <bool> has-term: true; in property <string> empty-text; in property <string>
  gpa-text; in property <string> counts-text; in property <[ProgressCardRow]> cards; }` and
  `export component ProgressPage`. No callbacks (read-only).
- Page layout (DESIGN.md "Progress card"): `PageTitle` "Progress"; message line when non-empty;
  when `!has-term || cards.length == 0` one `EmptyState { text: empty-text }`; otherwise the top
  line (gpa-text and counts-text in body) and one card per row: surface fill, square
  (`Theme.radius-none`), `Theme.space-lg` padding; title row `Theme.row-height` tall with code and
  name in body-emphasis and `StatusChip` right; two figure columns (Attendance, Marks): label
  caption, numeral figure (`Theme.numeral-md-*`), body-small detail (target-text / marks-line);
  the bar row: track `Theme.progress-track`, height `Theme.progress-height`, fill
  `Theme.progress-fill` with `width: parent.width * bar-fill`, marker `Theme.progress-target`
  `Theme.progress-target-width` wide at `x: (parent.width - Theme.progress-target-width) *
  marker-at` when `has-marker`, attendance-line printed beside the bar; then miss-line and
  best-grade-line in body, each only when non-empty.
- `ProgressBar` is a local component in `progress.slint` (only this page uses it).
- `app-window.slint`: import `ProgressPage, ProgressState`, export `ProgressState`, add the
  `NavItem "Progress"` (`current-page == 7`) right after Dashboard, and
  `if AppState.current-page == 7: ProgressPage { }`.

### Rust wiring

- `crates/app/src/bridge.rs`: `pub fn progress_card(c: &vm::ProgressCardView) -> ProgressCardRow`
  (no id, so no fallible conversion; chip via `bridge::chip`).
- `crates/app/src/pages/progress.rs`: `pub fn refresh(app: &App)` per the data flow; no `wire`
  (no callbacks). `pages/mod.rs` adds `pub mod progress;`.
- `crates/app/src/controller.rs`: `refresh_all` calls `pages::progress::refresh`; `show_message`
  maps page 7 to `ProgressState`. Not added to `tick` (D3).
- `crates/app/src/main.rs`: nothing new beyond the module path, since `page_selected` already
  calls `refresh_all` (refresh on page open) and every mutation calls `App::changed` →
  `refresh_all` (refresh after data changes).

## Project Structure

### Documentation (this feature)

```text
specs/013-m13-progress/
├── spec.md          # R61–R65, T62–T65 copied from docs/spec.md §13.8
├── plan.md          # this file (research and contracts folded in)
├── decisions.md     # D1–D14; builder appends build-time decisions
├── tasks.md         # speckit-tasks output
└── analysis.md      # speckit-analyze output
```

### Source Code (repository root)

```text
crates/app/src/
├── viewmodel.rs           # + "M13: Progress page" section (functions above)
├── viewmodel/tests.rs     # + T62–T65 and r61_–r65_ tests
├── bridge.rs              # + progress_card
├── controller.rs          # refresh_all, show_message page 7
├── pages/mod.rs           # + pub mod progress
└── pages/progress.rs      # new: refresh
crates/app/ui/
├── types.slint            # + ProgressCardRow
├── progress.slint         # new: ProgressState, ProgressBar, ProgressPage
└── app-window.slint       # nav item, page 7, exports
```

**Structure Decision**: existing two-crate workspace; only `crates/app` changes. `crates/core` is
not edited.

## Complexity Tracking

None.
