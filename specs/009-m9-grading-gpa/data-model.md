# Data Model: M9 Credits, Rule Sets, Marks, GPA

## Schema: migration 002

`crates/core/src/migrations/002_grading.sql` is the SQL block of `docs/spec.md` §13.3 copied
verbatim (two `ALTER TABLE course`, `CREATE TABLE rule_set`, `rule_component`,
`attendance_band`, `grade_scale`, two `ALTER TABLE assessment`). Migration 001 is not edited.
`SCHEMA_VERSION` = 2.

## Seeds: `crates/core/src/grading_seed.sql`

Run once by migration 002 and by a version-1 restore (research D2). Fixed ids.

| rule_set.id | name |
| --- | --- |
| 1 | Theory |
| 2 | Lab |

| id | set | position | name | kind | weight | item_count | best_count | item_max | is_final |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 1 | 0 | Attendance | attendance | 10 | NULL | NULL | NULL | 0 |
| 2 | 1 | 1 | Quizzes | items | 15 | 4 | 3 | 15 | 0 |
| 3 | 1 | 2 | Mid | items | 25 | 1 | 1 | NULL | 0 |
| 4 | 1 | 3 | Final | items | 50 | 1 | 1 | NULL | 1 |
| 5 | 2 | 0 | Attendance | attendance | 30 | NULL | NULL | NULL | 0 |
| 6 | 2 | 1 | Lab work | items | 40 | 1 | 1 | NULL | 0 |
| 7 | 2 | 2 | Lab final | items | 30 | 1 | 1 | NULL | 1 |

Attendance bands for components 1 and 5 (min_pct → credit_pct): 90 → 100, 85 → 90, 80 → 80,
75 → 70, 70 → 60, 0 → 0.

Grade scale (letter, min_pct, points): A+ 80 4.00, A 75 3.75, A- 70 3.50, B+ 65 3.25, B 60 3.00,
B- 55 2.75, C+ 50 2.50, C 45 2.25, D 40 2.00, F 0 0.00.

## Rust types (crates/core/src/model.rs)

Derives follow the existing types (`Debug, Clone, PartialEq`, plus `Copy, Eq` on enums).

```rust
// Existing inputs gain (R33, R36):
pub struct CourseInput {      /* existing fields */ pub credits: Option<f64>, pub rule_set_id: Option<i64> }
pub struct Course {           /* existing fields */ pub credits: Option<f64>, pub rule_set_id: Option<i64> }
pub struct AssessmentInput {  /* existing fields */ pub component_id: Option<i64>, pub score: Option<f64> }
pub struct Assessment {       /* existing fields */ pub component_id: Option<i64>, pub score: Option<f64> }

pub enum ComponentKind { Attendance, Items }            // TEXT 'attendance' | 'items'

pub struct AttendanceBand { pub min_pct: f64, pub credit_pct: f64 }

pub struct RuleComponent {
    pub id: i64,
    pub rule_set_id: i64,
    pub position: i64,
    pub name: String,
    pub kind: ComponentKind,
    pub weight_pct: f64,
    pub item_count: Option<u32>,     // items only
    pub best_count: Option<u32>,     // items only
    pub item_max: Option<f64>,       // items only; None = the component's marks for the course
    pub is_final: bool,
    pub bands: Vec<AttendanceBand>,  // attendance only; sorted by min_pct descending
}

pub struct RuleSet { pub id: i64, pub name: String, pub components: Vec<RuleComponent> } // by position

pub struct RuleComponentInput {
    pub id: Option<i64>,             // Some = update in place (R34); None = new
    pub name: String,
    pub kind: ComponentKind,
    pub weight_pct: f64,
    pub item_count: Option<u32>,
    pub best_count: Option<u32>,
    pub item_max: Option<f64>,
    pub is_final: bool,
    pub bands: Vec<AttendanceBand>,
}

pub struct RuleSetInput { pub id: Option<i64>, pub name: String, pub components: Vec<RuleComponentInput> }

pub struct GradeRow { pub letter: String, pub min_pct: f64, pub points: f64 }

pub struct ComponentMarks {
    pub component_id: i64,
    pub name: String,
    pub kind: ComponentKind,
    pub is_final: bool,
    pub marks: f64,                  // possible: weight_pct / 100 × total (R33)
    pub earned: f64,                 // R35 / R36
    pub assessed_fraction: f64,      // 0..=1 (R35 / R36)
    pub item_max: Option<f64>,       // items: effective item max (NULL → marks); attendance: None
    pub scores_entered: u32,         // items: linked assessments with a score
    pub missing_items: u32,          // items: max(item_count − entered, 0) (OQ-1 A); attendance: 0
}

pub enum Needed { Secured, NotReachable, Score(f64) }   // R38; Score rounded up to 1 decimal

pub struct NeededRow { pub letter: String, pub min_pct: f64, pub needed: Needed }

pub struct Assumption { pub component_name: String, pub missing_items: u32 } // "assumes 0 for …"

pub enum FinalOutlook {
    Needed { rows: Vec<NeededRow>, final_item_max: f64, assumptions: Vec<Assumption> },
    Actual { letter: Option<String> },               // the final already has a score (R38)
}

pub struct CourseMarks {
    pub course_id: i64,
    pub graded: bool,                // R33
    pub total: f64,                  // 100 × credits; 0 when not graded
    pub components: Vec<ComponentMarks>,
    pub secured: f64,                // R37
    pub assessed: f64,               // R37
    pub projected_pct: Option<f64>,  // R37, 1 decimal
    pub letter: Option<String>,      // R37
    pub fully_assessed: bool,        // R40 (OQ-1 A)
    pub outlook: Option<FinalOutlook>, // None when not graded
}

pub enum GpaStatus { Projected, Final }

pub struct GpaSummary {
    pub gpa: Option<f64>,            // R40, 2 decimals
    pub status: GpaStatus,
    pub included: u32,               // graded courses with a projected pct
    pub left_out: u32,               // graded courses without marks
}
```

## Relationships and invariants

- `course.rule_set_id` → `rule_set.id`; a set used by any course cannot be deleted (R34).
- `rule_component.rule_set_id` → `rule_set.id` `ON DELETE CASCADE`; `attendance_band.component_id`
  → `rule_component.id` `ON DELETE CASCADE`.
- `assessment.component_id` → `rule_component.id` (no action): core unlinks unscored assessments
  before removing a component and refuses removal when a linked assessment has a score (R34,
  research D3).
- Under OQ-2 A, a linked component always belongs to the course's rule set at save time.
- Graded course: `credits IS NOT NULL AND rule_set_id IS NOT NULL` (R33).

## Computation (crates/core/src/marks.rs, pure functions)

- `component_marks = weight_pct / 100 × 100 × credits` (R33).
- Attendance: `pct` from R13 counts; band = highest `min_pct ≤ pct`; earned = credit_pct / 100 ×
  marks, fraction 1; `pct` None → earned 0, fraction 0 (R35).
- Items: scores of linked assessments with a score; best K summed; earned = sum / (K × item_max) ×
  marks; fraction = min(entered, K) / K (R36).
- Course: secured = Σ earned; assessed = Σ marks × fraction; projected = round1(secured / assessed
  × 100) or None; letter by highest `min_pct ≤ projected` (R37).
- Needed: per grade row, need = min_pct / 100 × total − Σ earned of non-final components; score =
  ceil1(need / final_marks × final_item_max); ≤ 0 → Secured; > final_item_max → NotReachable
  (R38).
- GPA: over non-archived graded courses of the active term with `projected_pct`: round2(Σ points ×
  credits / Σ credits); Final iff every included course is `fully_assessed` (R40).
