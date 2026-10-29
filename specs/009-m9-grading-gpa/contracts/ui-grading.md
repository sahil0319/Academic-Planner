# Contract: grading UI (M9, app crate)

`docs/spec.md` §13.3 "UI". Every string derived from data comes from a pure function in
`crates/app/src/viewmodel.rs` with a unit test in `crates/app/src/viewmodel/tests.rs`. `.slint`
files only bind and call back. Colors, sizes, spacing, and radii use `Theme.*` only. No new core
API: the app uses the §13.3 methods and `ComponentMarks.item_max` for the effective item max.

## Navigation

A new **Rules** page. The nav button appears after Assessments; its page index is 6 so existing
indices 0–5 (Dashboard … About) do not change. (The Timetable page keeps its own "Rules" tab for
slots; the names come from §6 and §13.3.)

## Rules page (`ui/rules.slint`, `src/pages/rules.rs`)

- Rule sets list (name, component summary). Select one to edit; "New rule set"; "Delete" (the R34
  error text is shown inline when a course uses it).
- Component rows: name, kind (Attendance / Items), weight %, for Items: items held, best counted,
  each out of (empty = the component's marks), "Final" toggle; add, remove, move up/down
  (position = row order).
- Attendance bands per attendance component: rows "min % → credit %"; add, remove.
- Save shows core's `Error::Validation` message inline (R34).
- Grade scale table: letter, min %, points; add, remove, Save (R39 errors inline).

## Course form and detail (`ui/courses.slint`, `src/pages/courses.rs`)

- Form gains **Credits** (text, empty = none) and **Rule set** (choice: "Not graded" + rule set
  names).
- Detail gains a **Marks** card when graded: one row per component (name, earned of possible),
  secured total, projected percentage and letter. Ungraded: a single line saying the course is not
  graded and needs credits and a rule set.
- **Needed in final** table (graded, `FinalOutlook::Needed`): one row per grade letter with the
  needed score of the final's item max, "Secured", or "Not reachable"; assumption lines below.
  With `FinalOutlook::Actual`: one line with the actual grade.
- Courses page header: term GPA with "projected" or "final" and the left-out count when > 0.

## Assessment form (`ui/assessments.slint`, `src/pages/assessments.rs`)

- **Counts toward**: choice of "Not counted" + the components of the selected course's rule set
  (empty list when the course is not graded).
- **Score**: text field followed by "/ max", where max is the component's effective item max.
  Empty = no score.

## View-model functions (pure, unit-tested)

| Function | Example output |
| --- | --- |
| `format_mark(f64) -> String` | `45.0` → `"45"`, `37.5` → `"37.5"`, `1.5` → `"1.5"` |
| `format_marks(earned, possible) -> String` | `"39.0 of 45"`, `"27.0 of 30"` (T37/T38 values) |
| `format_projected(pct, letter) -> String` | `"80.0% · A+"`; None → `"—"` |
| `needed_text(&Needed) -> String` | `"111.0"`, `"Secured"`, `"Not reachable"` |
| `needed_rows(&FinalOutlook) -> Vec<NeededRowView>` | letter + text, scale order |
| `assumption_text(&Assumption) -> String` | `"Quizzes: assumes 0 for 1 item not entered"`; 2 → `"2 items"` |
| `actual_grade_text(Option<&str>) -> String` | `"Final grade: A"`; None → `"Final grade: —"` |
| `gpa_header_text(&GpaSummary) -> String` | `"Term GPA 3.67 (projected)"`, `"Term GPA 3.67 (final)"`, None → `"Term GPA —"`; left_out 1 → appends `" · 1 graded course without marks"` |
| `component_summary(&RuleComponent) -> String` | `"Attendance · 10%"`, `"Items · 15% · best 3 of 4, out of 15"`, `"Items · 50% · 1 of 1 · final"` |
| `band_text(&AttendanceBand) -> String` | `"≥ 90% → 100%"` |
| `score_max_label(Option<f64>) -> String` | `Some(15.0)` → `"/ 15"`; None → `""` |
| `parse_course_form(…, credits, rule_set_choice)` | credits empty → None; not a number or ≤ 0 → inline error; choice 0 → None |
| `parse_assessment_form(…, component_choice, score)` | score empty → None; not a number or < 0 → inline error; choice 0 → None |
| `parse_component_row(…) -> Result<RuleComponentInput, FormErrors>` | numbers parsed; empty item max → None; range checks stay in core |
| `parse_band_row(min, credit)`, `parse_grade_row(letter, min, points)` | number parse errors inline; R34/R39 checks stay in core |

Exact wording may be tuned during implementation; the unit tests pin whatever is chosen.
