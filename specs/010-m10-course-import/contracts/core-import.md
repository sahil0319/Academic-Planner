# Contract: Core Import API (R42–R48)

## Methods (exact §13.4 signatures)

```rust
impl Planner {
    /// R42–R48: parses and validates the whole document and returns the plan. Writes nothing.
    pub fn preview_import(&self, json: &str, effective_from: NaiveDate) -> Result<ImportPlan>;

    /// R48: applies the plan `preview_import` returns for the same input, in one transaction,
    /// then syncs (R6) with min(effective_from, today). Errors in the plan → Error::Validation
    /// listing every "path: message"; nothing written.
    pub fn apply_import(&mut self, json: &str, effective_from: NaiveDate, today: NaiveDate)
        -> Result<ImportReport>;
}
```

`crates/core/tests/api_surface.rs` binds both:

```rust
let _: fn(&Planner, &str, NaiveDate) -> Result<ImportPlan> = Planner::preview_import;
let _: fn(&mut Planner, &str, NaiveDate, NaiveDate) -> Result<ImportReport> = Planner::apply_import;
```

## Public types (re-exported from `planner_core`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ImportPlan {
    pub term: Option<ImportTerm>,          // R47: term to create
    pub add: Vec<CourseAdd>,
    pub update: Vec<CourseUpdate>,
    pub warnings: Vec<ImportIssue>,
    pub errors: Vec<ImportIssue>,
}
impl ImportPlan { pub fn is_empty(&self) -> bool; }   // no term, no add, no update

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportIssue { pub path: String, pub message: String }   // Display: "path: message"

#[derive(Debug, Clone, PartialEq)]
pub struct ImportTerm { pub name: String, pub start_date: NaiveDate, pub end_date: NaiveDate }

#[derive(Debug, Clone, PartialEq)]
pub struct CourseAdd {
    pub code: String, pub name: String,
    pub instructor: Option<String>, pub room: Option<String>,
    pub min_attendance_pct: Option<i64>,
    pub credits: Option<f64>, pub rule_set: Option<RuleSetRef>,
    pub slots: Vec<ImportSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSetRef { pub id: i64, pub name: String }

#[derive(Debug, Clone, PartialEq)]
pub struct CourseUpdate {
    pub course_id: i64, pub code: String, pub archived: bool,
    pub changes: Vec<FieldChange>,
    pub slots: Option<SlotChange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportField { Name, Credits, RuleSet, Instructor, Room, MinAttendancePct }

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Text(Option<String>),        // name, instructor, room
    Integer(Option<i64>),        // min_attendance_pct
    Number(Option<f64>),         // credits
    RuleSet(Option<RuleSetRef>), // rule_set
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldChange { pub field: ImportField, pub old: FieldValue, pub new: FieldValue }

#[derive(Debug, Clone, PartialEq)]
pub struct SlotChange { pub end: Vec<Slot>, pub add: Vec<ImportSlot> }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSlot { pub weekday: Weekday, pub start: NaiveTime, pub end: NaiveTime, pub room: Option<String> }

#[derive(Debug, Clone, PartialEq)]
pub struct ImportReport { pub plan: ImportPlan, pub sync: SyncReport }
```

Field and type names may be adjusted by the builder before the tester writes tests (Phase 2);
after Phase 2 they are frozen for M10.

## Behavior

| Rule | Behavior | Tests |
|---|---|---|
| R42 | All issues collected with paths (contracts/import-format.md); any error → `errors` non-empty, change lists empty; apply returns `Error::Validation` and writes nothing | T48, robustness tests |
| R42 | Unknown keys → `warnings`; never errors | robustness |
| R43 | Normalized code match (research D6) against active-term courses incl. archived; file duplicates by normalized code are errors; OQ-1 ambiguous → error | T47 |
| R44 | Present-and-different fields → `FieldChange`; absent keep; `null` clear; archived match → warning, stays archived | robustness |
| R45 | Multiset comparison on slots valid on E; else end current slots at E − 1 (OQ-3) and add file slots from E; `[]` ends all; missing untouched; new course slots from E | T46, T45 |
| R46 | Courses not in file: untouched | T49 |
| R47 | No active term: `term` required, created active; active term: ignored, warning if different | T44 |
| R48 | Preview writes nothing; apply = same plan, one transaction, then sync `min(E, today)`; empty plan → no writes, no sync | T44, T45, T50 |

`update` lists a matched course only if `changes` is non-empty or `slots` is `Some`. A
`SlotChange` with both lists empty never appears.

## Apply write order (one transaction)

1. Re-plan through the transaction; errors → rollback, `Error::Validation`.
2. Empty plan → rollback (nothing written), return the report with a zero `SyncReport`.
3. Insert the term (active) if planned.
4. Insert new courses (term = active term), then their slots (`valid_from = E`, `valid_to = NULL`).
5. For each update: write changed fields (others, including `notes` and `archived`, kept); set
   `valid_to = E − 1` on each slot in `slots.end`; insert `slots.add` from E.
6. `sync_in(tx, min(E, today))`; commit.
