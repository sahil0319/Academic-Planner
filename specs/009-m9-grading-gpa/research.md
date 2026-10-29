# Research: M9 Credits, Rule Sets, Marks, GPA

Every decision below fills a gap that `docs/spec.md` §13.3 leaves to implementation. None adds a
rule, changes the schema, or changes a §13.3 signature. Two points that do interpret a rule are
listed first as open questions for the router; the artifacts use the recommended option and mark
it "pending router confirmation".

## Open questions (router decision)

### OQ-1: What counts as "missing items" (R38) and "fully assessed" (R40)

R36 defines the assessed fraction as (scores entered, capped at K) / K. R38 says "non-final
components with missing items count those as 0, and the result lists what was assumed (e.g.
'assumes 0 for 1 quiz not entered')". T39 says "a quiz missing → listed as assumed 0" for best 3
of 4 with scores 12/11/13/14. Removing one score leaves 3 = K scores, so the R36 assessed
fraction is already 1.

| Option | Missing items | Fully assessed (R40 "final") | T39 "quiz missing" |
| --- | --- | --- | --- |
| A (recommended) | `max(item_count − entered, 0)` | attendance pct known and no missing items in any items component | lists "1 quiz not entered" (passes) |
| B | `max(best_count − entered, 0)` | every assessed fraction = 1 | lists nothing with 3 of 4 entered (T39 fails as written) |
| C | A for R38, B for R40 | assessed fraction 1 | passes, but GPA can say "final" while a quiz is still missing |

Recommendation: **A**. T39 forces A for R38, and using the same notion for R40 means a GPA is
"final" only when nothing is left to enter. Secured marks are the same under every option (R36:
best K of the entered scores, missing ones as 0). Affected tasks: T39 test, R40 `is_final`.

### OQ-2: Which component an assessment may count toward

§13.3 UI says "Counts toward (a component of the course's rule set)". R36 needs the component's
marks (which need the course's credits) to check `score ≤ item_max`. Core has no explicit rule.

| Option | Behavior |
| --- | --- |
| A (recommended) | `create_assessment`/`update_assessment` return `Error::Validation` when `component_id` is not a component of the course's current rule set (so an ungraded course without a rule set cannot link). `course_marks` counts only links to components of the course's current rule set; links left over after the course's rule set changes are ignored, not rejected. A score without a component is stored and not counted. |
| B | Core accepts any existing component id; only the UI restricts the choice; `course_marks` ignores foreign links. The R36 `score ≤ item_max` check is skipped when the course is not graded. |

Recommendation: **A**. It keeps the R36 check always computable and matches the UI text. It adds
no new error kind, only a validation case. Affected tasks: T43 setup, assessment save.

## Decisions

### D1: Public type shapes

- Decision: the types in data-model.md (`RuleSet`, `RuleComponent`, `ComponentKind`,
  `AttendanceBand`, `RuleSetInput`, `RuleComponentInput`, `GradeRow`, `CourseMarks`,
  `ComponentMarks`, `FinalOutlook`, `NeededRow`, `Needed`, `Assumption`, `GpaSummary`,
  `GpaStatus`).
- Rationale: §13.3 fixes the method signatures and names `RuleSet`, `RuleSetInput`, `GradeRow`,
  `CourseMarks`, `GpaSummary` but not their fields; the fields carry exactly the values R33–R40
  produce. Display strings stay in the view model (§2 rule).
- Alternatives: returning preformatted strings from core (rejected: formatting belongs in
  `viewmodel.rs` with unit tests).

### D2: Where the seeds live

- Decision: `crates/core/src/migrations/002_grading.sql` holds the §13.3 DDL verbatim.
  `crates/core/src/grading_seed.sql` holds the seed `INSERT`s with fixed ids (rule sets 1 Theory,
  2 Lab; components 1–7 in §13.3 order; positions 0.. in listed order). `MIGRATIONS[1]` is
  `concat!(include_str!("migrations/002_grading.sql"), include_str!("grading_seed.sql"))`; the R41
  version-1 restore runs `grading_seed.sql` after clearing the four tables. The seed file is
  frozen with migration 002: changing defaults later needs a new migration.
- Rationale: one copy of the seed values for migration and restore.
- Alternatives: seeds inside the 002 file plus a Rust copy for restore (two copies drift).
  If `concat!` of `include_str!` is rejected by the compiler, the migration runner may take the
  seed as a second string for entry 2; behavior is the same.
- Verified: the §13.3 DDL applies cleanly with `foreign_keys=ON` on a populated v1 schema (SQLite
  3.53, `PRAGMA foreign_key_check` empty).

### D3: Saving a rule set (R34)

- Input components carry `Option<i64>` ids. An id updates that row in place (it must belong to
  this rule set, else `Error::Validation`); no id inserts; existing components absent from the
  input are removed. `position` = index in the input list.
- Removing a component that has a linked assessment with a score → `Error::Validation` (R34).
  Linked assessments without a score are unlinked (`component_id = NULL`) first, because the
  `assessment.component_id` foreign key has no `ON DELETE` action.
- Bands are replaced per attendance component on every save. For `items` components bands are not
  stored; for `attendance` components `item_count`, `best_count`, `item_max` are stored NULL.
- An `items` component without `item_count` or `best_count` is `Error::Validation` (R34 needs
  1 ≤ best_count ≤ item_count).
- All R34 checks run before any write; the save is one transaction.

### D4: Value-range checks become validation errors

Values that the 002 `CHECK`s would reject are checked first and returned as `Error::Validation`
with a readable message: empty or duplicate component name within a set, weight outside (0, 100],
band min/credit outside 0–100, duplicate band min, `item_max ≤ 0`, course `credits ≤ 0`, unknown
`rule_set_id` on a course, `score < 0`, empty grade letter, grade `min_pct` outside 0–100,
`points < 0`. Same pattern as R2.

### D5: Deleting a rule set (R34)

`NotFound` for an unknown id. `Error::Validation` if any course row (any term, archived or not)
has that `rule_set_id`. Otherwise assessments linked to its components are unlinked and the set
is deleted (components and bands cascade).

### D6: Grade scale (R39)

`save_grade_scale` validates the whole list (unique letters, unique min_pct, a 0 row, points
non-increasing when sorted by min_pct descending) and replaces all rows in one transaction.
`grade_scale()` returns rows sorted by `min_pct` descending.

### D7: Rounding

- Projected pct: round half away from zero to 1 decimal (the R13 helper's rule), then letter
  lookup (R37). 79.9 → A, 80.0 → A+ (T40).
- Needed score: `ceil` to 1 decimal with a 1e-9 tolerance (`(x * 10.0 - 1e-9).ceil() / 10.0`) so
  float noise such as 111.00000000000001 does not become 111.1 (R38).
- GPA: round half away from zero to 2 decimals (R40).
- Earned and possible marks are not rounded in core; the view model shows 1 decimal for earned
  marks and drops a trailing ".0" for possible marks ("39.0 of 45", "27.0 of 30", "37.5").

### D8: Attendance not yet assessed (R35)

Earned 0, assessed fraction 0. It adds 0 to secured marks in R38 and is not listed as an
assumption (R38 lists missing items only).

### D9: More scores than items

All linked scores count; the best K are chosen from all of them. Missing items =
`max(item_count − entered, 0)` (OQ-1 A). No error for extra scores (none in §13.3).

### D10: Final already scored (R38)

"The final has a score" = at least one assessment linked to the final component has a score. Then
`FinalOutlook::Actual` carries the R37 letter (or none if nothing is assessed) instead of the
needed table.

### D11: `course_marks` on an ungraded course

Returns `CourseMarks { graded: false, .. }` with empty components and no outlook; unknown id →
`Error::NotFound`. Archived courses are computed normally (they are only excluded from R40).

### D12: `term_gpa` without an active term

Returns `GpaSummary { gpa: None, status: Projected, included: 0, left_out: 0 }`; not an error. The
UI shows "—".

### D13: Backup v2 (R41)

The four arrays are appended after `settings` in the JSON (existing key order unchanged). Rows are
ordered by id; bands by (component_id, min_pct); grade rows by min_pct descending, so
export → restore → export is byte-equal except `exported_at` (T42). New course and assessment keys
are always written; on read they default to null when absent. For `schema_version` 2 the four
arrays are required (`Error::Validation` like other format errors); for `schema_version` 1 they
are ignored if present, courses restore ungraded, and `grading_seed.sql` runs. Restored grading
rows are checked only by the DB constraints and foreign keys (as for every other table in R22),
not by R34/R39. The delete list gains `attendance_band`, `rule_component`, `rule_set`,
`grade_scale`.

### D14: Existing tests whose expectations change

Migration 002 and R41 change three pinned values: `crates/core/tests/migrations.rs` asserts
`SCHEMA_VERSION == 1` and fakes a "newer" DB with `user_version` 2;
`crates/core/tests/m3_backup_export.rs` asserts `schema_version` 1 and the exact top-level key
list; `crates/core/tests/m3_backup_restore.rs` (T30) rejects `schema_version` 2 as "newer". The
tester updates them to `SCHEMA_VERSION`, 2, the new key list, and `SCHEMA_VERSION + 1` before the
builder adds migration 002. 32 struct literals of `CourseInput`/`AssessmentInput` in core tests
and 4 in the app gain the new fields (mechanical).

## Arithmetic check (T37–T41 against R33–R40)

3-credit Theory: total 300; Attendance 30, Quizzes 45, Mid 75, Final 150 (sum 300).

- T37: best 3 of {12, 11, 13, 14} = 39; 39 / (3 × 15) × 45 = **39.0 of 45**. Matches.
- T38: 87% → band ≥85 → 90%; 0.90 × 30 = **27.0 of 30**. Matches.
- T39: 92% → band ≥90 → 100% → 30; quizzes 39; mid item_max NULL → 75, 60/75 × 75 = 60.
  Non-final secured = 129. A+: 0.80 × 300 − 129 = 111 → 111 / 150 × 150 = **111.0**. A: 225 −
  129 = 96 → **96.0**. Matches. Float check: both evaluate to exactly 111.0 and 96.0 in f64.
  Variant "a quiz missing" (scores 12, 11, 13): best 3 = 36, secured 126 → A+ 114.0, A 99.0, and
  the assumption "1 quiz not entered" under OQ-1 A.
- T40: 79.9 → A (75 ≤ 79.9 < 80); 80.0 → A+. Matches.
- T41: (4.00 × 3 + 3.00 × 1.5) / 4.5 = 16.5 / 4.5 = 3.6667 → **3.67**, projected. Matches.
  A test setup that reaches the letters: 3-credit Theory with mid 60/75 only → 80.0% → A+;
  1.5-credit Theory (total 150, mid marks 37.5) with mid 23.25/37.5 only → 62.0% → B; no final
  scores, so "projected".

No mismatch between the §13.5 numbers and R33–R40.
