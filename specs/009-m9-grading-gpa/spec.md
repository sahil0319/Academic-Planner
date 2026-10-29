# Feature Specification: M9 Credits, Rule Sets, Marks, GPA

**Feature Branch**: `009-m9-grading-gpa` (spec directory; work happens on the current branch)

**Created**: 2026-10-29

**Status**: Draft

**Input**: User description: "Milestone M9 from docs/spec.md §13.3: rules R33–R41, tests T36–T43.
Do not add requirements."

**Source of truth**: `docs/spec.md` §7 (M9 row), §13.3 (migration 002, seeded defaults, R33–R41,
API additions, UI) and §13.5 (T36–T43). Everything in §1–§12 still applies, in particular R13
(attendance percentage), R21–R23 (backup and CSV, amended by R41), R25–R26 (migrations), and the
`Theme.*`-only UI rule. This feature implements the §13.3 text as written and adds no rule.

## User Scenarios & Testing *(mandatory)*

The user is a student who already tracks courses, attendance, and assessments in the app. They
want to know how many marks they have secured in each course, what grade that projects to, what
they need in the final exam for each grade, and their term GPA.

### User Story 1 - Grading rules exist and are validated (Priority: P1)

A fresh or upgraded database contains the seeded "Theory" and "Lab" rule sets, their attendance
bands, and the UGC Bangladesh grade scale. The student can edit rule sets, components, bands, and
the grade scale; invalid rule sets and grade scales are refused with a clear message.

**Why this priority**: every mark, needed-in-final value, and GPA depends on these rules.

**Independent Test**: open an in-memory planner, list the rule sets and grade scale and compare
them with §13.3; save the invalid rule sets of T36 and see each refused.

**Acceptance Scenarios**:

1. **Given** a new database, **When** the rule sets are listed, **Then** "Theory" has Attendance
   10% (attendance), Quizzes 15% (best 3 of 4, each out of 15), Mid 25% (1 of 1), Final 50% (1 of
   1, final), "Lab" has Attendance 30%, Lab work 40% (1 of 1), Lab final 30% (1 of 1, final), and
   both attendance components carry the bands ≥90→100, ≥85→90, ≥80→80, ≥75→70, ≥70→60, ≥0→0.
2. **Given** a new database, **When** the grade scale is read, **Then** it has the ten UGC rows
   A+ 80 4.00 down to F 0 0.00.
3. **Given** a rule set whose weights sum to 95, or with two final components, or with best 5 of
   4, or whose attendance component has no band with min 0, **When** it is saved, **Then** each
   save fails with a validation error (T36).
4. **Given** a rule set used by a course, **When** it is deleted, **Then** a validation error is
   returned (T43).
5. **Given** a grade scale with a duplicate letter or min, no 0 row, or points that rise as the
   minimum falls, **When** it is saved, **Then** a validation error is returned (R39).

---

### User Story 2 - Course marks and projected grade (Priority: P1)

The student gives a course credits and a rule set, links assessments to components ("counts
toward") and enters scores. The course shows earned marks per component, secured marks, the
projected percentage, and the projected letter.

**Why this priority**: this is the core value of the milestone.

**Independent Test**: a 3-credit Theory course with quiz scores and attendance gives the marks of
T37 and T38; projected percentages 79.9 and 80.0 map to A and A+ (T40).

**Acceptance Scenarios**:

1. **Given** a 3-credit Theory course with quiz scores 12, 11, 13, 14, **When** its marks are
   read, **Then** Quizzes shows 39.0 of 45 (T37).
2. **Given** a 3-credit Theory course with 87% attendance and the default bands, **When** its
   marks are read, **Then** Attendance shows 27.0 of 30 (T38).
3. **Given** projected percentages 79.9 and 80.0, **When** the letter is looked up, **Then** they
   give A and A+ (T40).
4. **Given** a score above the component's item maximum, **When** the assessment is saved,
   **Then** a validation error is returned (T43).
5. **Given** a course without credits or without a rule set, **When** its marks are read,
   **Then** it is reported as not graded.

---

### User Story 3 - Needed in final (Priority: P1)

Before the final exam the student sees, for each grade, the final score they need, or "Secured",
or "Not reachable", together with any assumptions made about items not yet entered.

**Why this priority**: it is the decision the student actually acts on before the final.

**Independent Test**: T39 on a 3-credit Theory course.

**Acceptance Scenarios**:

1. **Given** a 3-credit Theory course with attendance 92%, quizzes 12/11/13/14, mid 60 of 75, and
   no final score, **When** needed-in-final is read, **Then** A+ needs 111.0 and A needs 96.0 out
   of 150 (T39).
2. **Given** the same course with one quiz score not entered, **When** needed-in-final is read,
   **Then** the result lists that it assumes 0 for the missing quiz (T39).
3. **Given** a grade whose need is at or below zero, **Then** it shows "Secured"; **given** a need
   above the final's item maximum, **Then** it shows "Not reachable" (R38).
4. **Given** the final already has a score, **Then** the actual grade is shown instead (R38).

---

### User Story 4 - Term GPA (Priority: P2)

The Courses page header shows the term GPA and whether it is projected or final, and how many
graded courses were left out for lack of marks.

**Why this priority**: a summary of US2 across courses.

**Independent Test**: T41.

**Acceptance Scenarios**:

1. **Given** a 3-credit course projecting A+ (4.00) and a 1.5-credit course projecting B (3.00)
   in the active term, **When** the term GPA is read, **Then** it is 3.67 and "projected" (T41).
2. **Given** no graded course with a projected percentage, **Then** the GPA is none (R40).

---

### User Story 5 - Backups carry grading data (Priority: P2)

Backups include rule sets, components, bands, the grade scale, and the new course and assessment
fields. Old (version 1) backups still restore.

**Why this priority**: R21/R22 already promise lossless backups; M9 must keep that promise.

**Independent Test**: T42.

**Acceptance Scenarios**:

1. **Given** a database with grading data, **When** it is exported, restored, and exported again,
   **Then** both exports are equal except `exported_at` (T42).
2. **Given** a version-1 backup, **When** it is restored, **Then** courses are ungraded and the
   rule sets, components, bands, and grade scale equal the seeded defaults (T42).

---

### User Story 6 - Screens for grading (Priority: P2)

A Rules page edits rule sets (components and bands) and the grade scale. The course form takes
credits and a rule set; the assessment form takes "Counts toward" and a score "x / max"; course
detail shows a marks card and a "Needed in final" table; the Courses page header shows the term
GPA.

**Why this priority**: the user-facing surface of US1–US4; it can follow the core work.

**Independent Test**: view-model unit tests for the formatting and form parsing; a manual look at
the pages.

**Acceptance Scenarios**:

1. **Given** a graded course, **When** its detail opens, **Then** the marks card lists each
   component with earned and possible marks, the secured total, and the projected grade.
2. **Given** a graded course whose final has no score, **Then** the "Needed in final" table lists
   each grade with its needed score, "Secured", or "Not reachable", and any assumptions.
3. **Given** the Courses page, **Then** its header shows the term GPA with "projected" or "final".

---

### Edge Cases

- Credits may be fractional (1.5); total marks = 100 × credits (R33).
- A course missing credits or a rule set is "not graded" and is left out of R37–R40 (R33).
- Attendance with no attended or absent sessions (R13 pct none) is "not yet assessed" (R35).
- Fewer item scores than best K: missing items count as 0 for secured marks; the assessed fraction
  is (scores entered, capped at K) / K (R36).
- Nothing assessed: projected percentage is none and there is no letter (R37).
- Needed score is rounded up to 1 decimal; ≤ 0 is "Secured"; above the final's item maximum is
  "Not reachable" (R38).
- The projected percentage is rounded to 1 decimal before the letter lookup, so 79.9 is A and
  80.0 is A+ (R37, T40).
- Removing a component that has linked scored assessments is an error; deleting a rule set that a
  course uses is an error (R34).
- Restoring a version-1 backup: courses ungraded, the four new tables reset to the seeded defaults
  (R41).
- CSV exports are unchanged (R41, R23).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001** (Migration 002): The database schema gains exactly the migration 002 of §13.3 as a
  new numbered migration; migration 001 is not edited. A new or upgraded database contains the
  seeded rule sets, components, bands, and grade scale of §13.3. Upgrading an existing database
  follows R25/R26 (pre-migration copy).
- **FR-002** (R33): A course is graded when it has credits and a rule set. Total marks = 100 ×
  credits; a component's marks = weight_pct / 100 × total. Ungraded courses are left out of
  R37–R40.
- **FR-003** (R34): Saving a rule set is validated: non-empty unique name; weights sum to 100
  (±0.01); at most one attendance component; exactly one final component and it is `items`; for
  `items`, 1 ≤ best_count ≤ item_count; an attendance component has at least one band and a band
  with min_pct 0. Components are updated in place by id. Removing a component that has linked
  scored assessments is an error. Deleting a rule set that a course uses is an error. All errors
  are validation errors.
- **FR-004** (R35): The attendance component's earned fraction is the credit_pct of the band with
  the highest min_pct ≤ the course's R13 pct; with no pct the component is not yet assessed.
- **FR-005** (R36): An items component's earned fraction = (sum of the best K scores) / (K ×
  item_max), item_max NULL meaning the component's marks. A score above item_max is a validation
  error. With fewer than K scores, missing items count as 0; the assessed fraction is (scores
  entered, capped at K) / K.
- **FR-006** (R37): Course marks report per component earned marks and marks possible; secured =
  sum of earned; assessed = sum of (component marks × assessed fraction); projected pct = secured
  / assessed × 100 (none if nothing assessed); the letter is the grade row with the highest
  min_pct ≤ projected pct rounded to 1 decimal.
- **FR-007** (R38): Needed in final, per grade row: need = min_pct / 100 × total − secured marks
  of all non-final components; needed score = need / final marks × final item_max, rounded up to
  1 decimal; "Secured" if ≤ 0, "Not reachable" if above item_max, else the score. Missing items
  of non-final components count as 0 and the result lists what was assumed. If the final already
  has a score, the actual grade is shown instead.
- **FR-008** (R39): Saving the grade scale is validated: unique letters, unique min_pct, a row
  with min_pct 0, points non-increasing as min_pct decreases.
- **FR-009** (R40): Term GPA over non-archived graded courses in the active term with a projected
  pct: Σ(points × credits) / Σ credits, rounded to 2 decimals; none if no course qualifies;
  "final" when every component of every included course is fully assessed, else "projected"; it
  reports how many graded courses were left out for lack of marks.
- **FR-010** (R41): The backup JSON gains `rule_sets`, `rule_components`, `attendance_bands`,
  `grade_scale`; course and assessment rows carry the new columns; `schema_version` is 2.
  Restoring a version-1 backup leaves courses ungraded and sets the four new tables to the seeded
  defaults. CSV formats are unchanged.
- **FR-011** (API): Course input gains credits and rule set; assessment input gains component and
  score; the planner gains list/save/delete rule set, read/save grade scale, course marks, and
  term GPA, with the signatures of §13.3.
- **FR-012** (UI): A Rules page (rule sets with components and bands; the grade scale); course
  form credits and rule set; assessment form "Counts toward" and score "x / max"; a marks card and
  a "Needed in final" table on course detail; the term GPA on the Courses page header. Formatting
  lives in view-model functions with unit tests; styling uses theme tokens only.

### Key Entities

- **Rule set**: a named grading scheme (e.g. Theory, Lab) made of ordered components.
- **Rule component**: a part of a rule set with a weight; kind attendance or items; for items, N
  held, best K counted, each out of item_max (or the component's marks); one is the final.
- **Attendance band**: maps a minimum attendance percentage to a credit percentage for an
  attendance component.
- **Grade row**: a letter with its minimum percentage and grade points (the grade scale).
- **Course (extended)**: gains credits and a rule set; graded only with both.
- **Assessment (extended)**: gains the component it counts toward and its score.
- **Course marks**: per-component earned and possible marks, secured, assessed, projected pct,
  letter, and the needed-in-final table with assumptions.
- **GPA summary**: GPA value, projected or final, and the count of graded courses left out.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: T36–T43 pass, together with every earlier test, and the §7 gate passes.
- **SC-002**: For the T39 course the student sees A+ 111.0 and A 96.0 out of 150 without any
  manual arithmetic.
- **SC-003**: An existing version-1 database upgrades with its data intact, leaves a
  pre-migration copy, and has the seeded rules afterwards.
- **SC-004**: A version-1 backup restores without error on the new version.
- **SC-005**: No raw colors or pixel sizes appear outside the theme file (theme grep check).

## Assumptions

- Seed values are exactly those in §13.3; the band and Lab values are placeholders the user edits
  later.
- The order of components follows `position`; seeded positions follow the order listed in §13.3.
- Rule set, component, band, grade row, course marks, and GPA summary shapes beyond the §13.3
  signatures are decided in the plan (data-model.md) without adding behavior.
- Exam weeks remain holidays entered by the user (no change in this milestone).

## Router decisions (2026-10-29)

- OQ-1 = A: for R38 "missing items" = item_count − entered (a pending quiz could still raise the best-K total, so it is listed); for R40 the GPA is "final" only when no component has missing items.
- OQ-2 = A: core rejects (Error::Validation) an assessment whose component is not in the course's rule set, including when the course is ungraded. This clarifies R34/R36 and does not change them.
