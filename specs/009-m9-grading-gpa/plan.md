# Implementation Plan: M9 Credits, Rule Sets, Marks, GPA

**Branch**: `009-m9-grading-gpa` (spec directory; work happens on the current branch) | **Date**: 2026-10-29 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/009-m9-grading-gpa/spec.md`

## Summary

Implement `docs/spec.md` §13.3 as written: migration 002 (credits and rule set on `course`,
`rule_set`, `rule_component`, `attendance_band`, `grade_scale`, component and score on
`assessment`) with the seeded defaults; rule-set and grade-scale CRUD with the R34/R39
validation; `course_marks` (R35–R38) and `term_gpa` (R40); backup schema version 2 with
version-1 restore defaults (R41); then the UI (Rules page, course and assessment form fields,
marks card, needed-in-final table, GPA header). The work is split into a **core track** (crates/core,
gate-green on its own) and an **app track** (crates/app) that starts once the core API exists and
the M8 app work has merged. Open interpretation points are listed in research.md §Open questions
(OQ-1, OQ-2); the plan uses the recommended option for each, marked "pending router
confirmation".

## Technical Context

**Language/Version**: Stable Rust, edition 2021 (§1).

**Primary Dependencies**: `rusqlite` (bundled), `serde`, `serde_json`, `chrono`, `thiserror` in
core; `slint` and `anyhow` in app (§1). No new dependency.

**Storage**: SQLite; migration 002 is a new file `crates/core/src/migrations/002_grading.sql`
(DDL exactly as §13.3) plus the seed file `crates/core/src/grading_seed.sql`; migration 001 is
not edited. `SCHEMA_VERSION` becomes 2 because it equals the number of embedded migrations. R26
writes `pre-migrate-v1-to-v2.sqlite` when an existing v1 file is opened.

**Testing**: `cargo test --workspace`. Core integration tests in `crates/core/tests/` with
in-memory SQLite (T36–T43 plus rule tests named `r34_…` etc.); pure computation unit tests in
`crates/core/src/marks.rs`; app view-model unit tests in `crates/app/src/viewmodel/tests.rs`.

**Target Platform**: macOS, Windows, Linux desktop (unchanged).

**Project Type**: Desktop app: library crate `planner_core` plus binary crate `app` (§2).

**Performance Goals**: None beyond current behavior; marks are computed on demand from a handful
of rows per course.

**Constraints**: core reads no clock, has no Slint; `.slint` files contain bindings only;
`Theme.*` only; every mutation commits immediately; floating-point results are rounded exactly as
R36–R40 say (1 decimal for pct and needed score, 2 decimals for GPA).

**Scale/Scope**: 9 rules (R33–R41), 8 tests (T36–T43), 7 public methods, 1 migration, 1 new page.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | How |
| --- | --- | --- |
| I. Spec is source of truth | Pass | Schema, seeds, rules, and signatures copied from §13.3. Struct shapes behind the signatures are filled in (data-model.md) without new behavior. Interpretation points are raised as open questions, not decided silently. |
| II. Test-first, traceable | Pass | Tester tasks for T36–T43 and the rule tests precede each implementation task; tests named `t36_…`; doc comments cite R33–R41. |
| III. Pure, clock-free core | Pass | No new time input is needed; marks use R13 counts already in the DB. View-model formatting is pure and unit-tested. |
| IV. Append-only migrations | Pass | New `002_grading.sql`; `001_init.sql` untouched. The seed file is frozen with migration 002 (research D2). |
| V. Gate before done | Pass | Final task runs the §7 gate plus the theme grep check. |

Re-check after Phase 1: still Pass. No complexity exceptions.

## Project Structure

### Documentation (this feature)

```text
specs/009-m9-grading-gpa/
├── spec.md
├── plan.md              # this file
├── research.md          # decisions D1–D14, open questions OQ-1, OQ-2, arithmetic check
├── data-model.md        # schema 002, seeds, Rust types
├── quickstart.md        # validation run guide
├── contracts/
│   ├── core-api.md      # public methods, input validation, errors
│   ├── core-backup.md   # R41 JSON v2 and v1 restore
│   └── ui-grading.md    # Rules page, forms, marks card, needed table, GPA header, view-model fns
├── checklists/
│   ├── requirements.md
│   └── grading-rules.md # speckit-checklist for R33–R41 and T36–T43
└── tasks.md
```

### Source Code (repository root)

```text
crates/core/
├── src/
│   ├── migrations/002_grading.sql   # NEW: §13.3 DDL verbatim
│   ├── grading_seed.sql             # NEW: seeded rule sets, components, bands, grade scale
│   ├── db.rs                        # MIGRATIONS gains entry 2; course/assessment columns (R33, R36)
│   ├── model.rs                     # new types; CourseInput/AssessmentInput gain fields
│   ├── grading.rs                   # NEW: rule set + grade scale CRUD and validation (R34, R39)
│   ├── marks.rs                     # NEW: pure computation + course_marks / term_gpa (R35–R38, R40)
│   ├── backup.rs                    # R41
│   └── lib.rs                       # module declarations
└── tests/
    ├── m9_rules.rs                  # NEW: T36, T43, R34/R39 seeds and validation
    ├── m9_marks.rs                  # NEW: T37–T41
    ├── m9_backup.rs                 # NEW: T42, migration 002 upgrade
    ├── common/mod.rs                # fixture helpers (graded course, linked scores)
    └── (existing tests: mechanical field additions; version pins 1 -> 2)

crates/app/
├── ui/rules.slint                   # NEW: Rules page
├── ui/courses.slint                 # course form fields, marks card, needed table, GPA header
├── ui/assessments.slint             # "Counts toward", score "x / max"
├── ui/app-window.slint, types.slint # nav entry, row structs
├── src/pages/rules.rs               # NEW
├── src/pages/{courses,assessments,mod}.rs
├── src/viewmodel.rs                 # formatting and form parsing (pure)
└── src/viewmodel/tests.rs
```

**Structure Decision**: two existing crates (§2). Core logic goes in two new modules (`grading.rs`
for persistence and validation, `marks.rs` for computation) so `db.rs` does not grow further.

## Track split for parallel building

- **Core track** (tasks tagged `[core]`): migration, seeds, types, CRUD, validation, marks, GPA,
  backup, API surface test. It must be gate-green with no app change except the mechanical
  compile fix: four struct literals in `crates/app/src/viewmodel.rs` and its tests gain
  `credits: None, rule_set_id: None` / `component_id: None, score: None`. That fix is a separate
  task so the M8 builder can rebase over a one-line-per-literal diff.
- **App track** (tasks tagged `[app]`): starts after the core public API (types and signatures)
  is merged **and** after the M8 app work is merged, because both milestones edit
  `viewmodel.rs`, `assessments.slint`, `courses.slint`, and the assessment form parser. tasks.md
  lists the exact core task each app task depends on.

## Complexity Tracking

None.
