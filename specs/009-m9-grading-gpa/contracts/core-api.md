# Contract: core API additions (M9)

Signatures are copied from `docs/spec.md` §13.3 and must match exactly. Types are in
[data-model.md](../data-model.md). All errors use the existing `Error` enum; no new variant.

```rust
// CourseInput gains: credits: Option<f64>, rule_set_id: Option<i64>
// AssessmentInput gains: component_id: Option<i64>, score: Option<f64>
pub fn list_rule_sets(&self) -> Result<Vec<RuleSet>>;                 // with components and bands
pub fn save_rule_set(&mut self, input: RuleSetInput) -> Result<RuleSet>; // create or update
pub fn delete_rule_set(&mut self, id: i64) -> Result<()>;
pub fn grade_scale(&self) -> Result<Vec<GradeRow>>;
pub fn save_grade_scale(&mut self, rows: &[GradeRow]) -> Result<()>;
pub fn course_marks(&self, course_id: i64) -> Result<CourseMarks>;     // R35–R38
pub fn term_gpa(&self) -> Result<GpaSummary>;                          // R40
```

## Existing methods, changed behavior

| Method | Change |
| --- | --- |
| `create_course`, `update_course` | Store `credits`, `rule_set_id`. `credits ≤ 0` → Validation; unknown `rule_set_id` → Validation. Either may be None (not graded, R33). |
| `list_courses` | `Course` carries `credits`, `rule_set_id`. |
| `create_assessment`, `update_assessment` | Store `component_id`, `score`. `score < 0` → Validation. OQ-2 A: `component_id` not in the course's rule set → Validation. R36: linked `score > item_max` (NULL item_max = component's marks for that course) → Validation (T43). |
| `list_assessments`, `dashboard` | `Assessment` carries `component_id`, `score`. |
| `export_json`, `restore_json` | See [core-backup.md](core-backup.md). |
| CSV exports | Unchanged (R41, R23). |

## New methods

| Method | Behavior | Errors |
| --- | --- | --- |
| `list_rule_sets` | All sets ordered by id (seeds Theory, Lab first); components by position; bands by min_pct descending. | Db |
| `save_rule_set` | `id: None` creates, `Some` updates. R34 checks (below) before any write; one transaction; returns the stored set. | Validation (R34, research D3/D4); NotFound for unknown set id |
| `delete_rule_set` | Unlinks assessments linked to its components, deletes the set (cascade). | NotFound; Validation if any course uses it (T43) |
| `grade_scale` | Rows by min_pct descending. | Db |
| `save_grade_scale` | R39 checks, then replaces all rows in one transaction. | Validation |
| `course_marks` | R33, R35–R38 per data-model.md "Computation". Ungraded → `graded: false`. | NotFound |
| `term_gpa` | R40. No active term → `gpa: None`. | Db |

## R34 checks (each `Error::Validation`)

1. Name non-empty after trim; unique among other rule sets.
2. Σ weight_pct within 100 ± 0.01; each weight in (0, 100]; component names non-empty and unique
   within the set.
3. At most one `attendance` component.
4. Exactly one `is_final` component, and it is `items`.
5. For `items`: `item_count` and `best_count` present and 1 ≤ best_count ≤ item_count; `item_max`
   None or > 0.
6. For `attendance`: at least one band; a band with min_pct 0; band values 0–100; band min_pct
   unique.
7. A component id in the input that belongs to another set → Validation.
8. Removing a component that has a linked assessment with a score → Validation.

## R39 checks (each `Error::Validation`)

Letters non-empty and unique; min_pct unique and in 0–100; a row with min_pct 0; points ≥ 0 and
non-increasing when rows are sorted by min_pct descending.

## API surface test

`crates/core/tests/api_surface.rs` gains one `let _: fn(...) -> ... = Planner::...;` line per new
method so a signature drift fails to compile.
