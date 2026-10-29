# Implementation Plan: M10 Courses and Timetable Import

**Branch**: `010-m10-course-import` (spec directory; work happens on the current branch) | **Date**: 2026-10-29 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/010-m10-course-import/spec.md`

## Summary

Three parts. Core and app can be built in parallel once the core types and stubs exist (Phase 2).

1. **Core import module (R42–R48; T44–T50).** A new private module `crates/core/src/import/`
   with three layers:
   - `parse.rs` (R42): reads the pasted text into a `serde_json::Value` and walks it by hand, so
     every error and warning carries its JSON path, absent and `null` stay distinct, and unknown
     keys become warnings. It needs no database. Output: a parsed document plus issues.
   - `plan.rs` (R43–R47): reads the database (active term, its courses including archived, their
     slots valid on the effective date, rule set names) and builds the `ImportPlan`: term to
     create, courses to add, courses to update with field changes, slot changes, warnings, errors.
   - `apply.rs` (R48): `apply_import` opens one transaction, rebuilds the plan inside it from the
     same input, refuses it if it has errors, writes it (term, courses, slot ends, new slots), syncs
     with `min(effective_from, today)` (R7 semantics, research D11), and commits.
   Public types live in `import/types.rs` and are re-exported from `lib.rs`. The two methods are
   exactly the §13.4 signatures. No new `Error` variant: errors travel inside `ImportPlan`, and
   `apply_import` turns them into one `Error::Validation` listing every `path: message`
   (research D5). No migration; `SCHEMA_VERSION` is whatever M9 sets.
2. **Docs (R50).** `docs/import/template.json` (the §13.4 example) with a core test that it
   previews with no errors and no warnings on a fresh database with M9's seeded rule sets, and
   `docs/import/ai-prompt.md` with the period → time rule for merged cells and the user's
   7th-semester table as a worked example in 24-hour times (the table comes from the human, OQ-2).
3. **App Import panel (§13.4 UI, R49, R50).** "Import…" on the Courses page opens a panel
   (`ui/import.slint`, `pages/import.rs`): paste box, Copy template, Copy AI prompt, effective date
   (default today), Preview, Apply. All shown text comes from pure functions in a new
   `crates/app/src/viewmodel/import.rs` with unit tests. Both docs files are embedded with
   `include_str!`. Copy uses Slint's own `TextInput` clipboard functions (research D17).

`docs/spec.md` §13.4 is the plan: no rule, schema, or API change. The only interpretations are
recorded in research.md; three of them are open questions for the human (OQ-1 to OQ-3).

## Dependencies and start condition

- **M9 core merged first** (migration 002; `CourseInput.credits: Option<f64>`,
  `CourseInput.rule_set_id: Option<i64>`; the matching `Course` fields; table `rule_set(id, name)`
  with seeded "Theory" and "Lab"; `list_rule_sets()`). M10 names these as in `docs/spec.md` §13.3.
  If M9 lands different Rust field names, the builder uses M9's names; the plan's logic does not
  change.
- **M7**: R26 is untouched (no migration in M10).
- **M8**: the preview formats times with `viewmodel::format_time`, which M8 makes 12-hour (R31).
  The import format itself stays 24-hour.
- **Merge hot spots with M8/M9** (both edit `viewmodel.rs`, `viewmodel/tests.rs`,
  `courses.slint`, `pages/courses.rs`): M10 puts its code in new files and touches the shared
  files only for one-line hooks (a `pub mod import;`, an "Import…" button, a panel instance).

## Technical Context

**Language/Version**: Rust stable, edition 2021 (workspace settings unchanged).

**Primary Dependencies**: core: `serde_json` (already a dependency; `Value` API), `chrono`,
`rusqlite`. App: Slint 1.16 (existing). No new crate in either (research D17).

**Storage**: SQLite, no schema change. Writes go to `term`, `course`, `slot`, `session` (via sync)
only through `apply_import`.

**Testing**: core integration tests in `crates/core/tests/m10_import.rs` (T44–T50 named
`t44_…`–`t50_…`, plus contract tests for R42–R47 robustness) on `Planner::open_in_memory()`;
the R50 template test in the same file reads the file with
`include_str!("../../../docs/import/template.json")`. App unit tests in
`crates/app/src/viewmodel/import/tests.rs`. "Database unchanged" is checked by comparing
`export_json(fixed_now)` before and after.

**Target Platform**: macOS, Windows, Linux desktop (unchanged).

**Project Type**: desktop app (Cargo workspace: `crates/core` library, `crates/app` Slint binary).

**Performance Goals**: none beyond interactive use; a term has tens of courses and slots.

**Constraints**: core reads no clock (today and the effective date are passed in); one
transaction for apply; nothing written on any error; `Theme.*` only in `.slint`.

**Scale/Scope**: 2 public core methods, 12 public data types (contracts/core-import.md), 1 panel, 2 docs files.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Status |
|---|---|---|
| I. Spec is source of truth | Implements §13.4 as written; signatures exactly as §13.4; no schema change; interpretations listed in research.md, rule gaps raised as OQ-1–OQ-3 | Pass |
| I. Theme only | Panel uses `Theme.*`; theme grep check in the gate task | Pass |
| II. Test-first, traceable | Every core rule has tester tasks before builder tasks; test names `t44_…`; doc comments cite R42–R50; in-memory SQLite only | Pass |
| III. Pure, clock-free core | `today` and `effective_from` are parameters; no file or clipboard access in core; template test reads the file at compile time | Pass |
| III. No logic in `.slint` | Panel binds to view-model output; enable/disable state computed in `viewmodel/import.rs` | Pass |
| IV. Append-only migrations | No migration | Pass |
| V. Gate before done | Final task runs the gate and the theme grep | Pass |

Post-design re-check (after Phase 1): unchanged, all pass. No complexity tracking needed.

## Project Structure

### Documentation (this feature)

```text
specs/010-m10-course-import/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── import-format.md      # the JSON format, known keys, null/absent, day/time grammar
│   ├── core-import.md        # preview_import / apply_import, plan types, messages, paths
│   ├── ui-import.md          # Import panel, view-model functions
│   └── import-docs.md        # template.json and ai-prompt.md contents
├── checklists/requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/core/src/
├── lib.rs                    # edit: mod import; pub use import::{types re-exported}
├── import/
│   ├── mod.rs                # impl Planner { preview_import, apply_import }
│   ├── types.rs              # public plan/report types (contracts/core-import.md)
│   ├── parse.rs              # R42: Value walk, paths, tri-state fields, day/time/date parsing
│   ├── plan.rs               # R43–R47: matching, merge, slot-set comparison, term handling
│   └── apply.rs              # R48: one transaction, writes, sync
├── db.rs, schedule.rs        # reuse pub(crate) helpers (insert_slot, sync_in, active_term_id,
│                             #   load_course, fmt_*); add small pub(crate) helpers if needed
crates/core/tests/
├── m10_import.rs             # T44–T50 + robustness contract tests + R50 template test
└── api_surface.rs            # edit: bind the two new signatures
docs/import/
├── template.json             # R50
└── ai-prompt.md              # R50
crates/app/src/
├── viewmodel.rs              # edit: `pub mod import;` only
├── viewmodel/import.rs       # pure formatting + can_apply
├── viewmodel/import/tests.rs # unit tests
├── pages/import.rs           # wiring: preview, apply, copy, effective date, refresh
├── pages/mod.rs, controller.rs or main.rs   # edit: register the page wiring
crates/app/ui/
├── import.slint              # ImportPanel component (Theme.* only)
├── types.slint               # edit: ImportLine struct(s)
└── courses.slint             # edit: "Import…" button + panel instance
```

**Structure Decision**: one new core module folder and new app files, so M8/M9 edits to the same
crates rarely conflict. Exact helper names inside core are the builder's choice.

## Phase 0 and Phase 1 outputs

- [research.md](research.md): decisions D1–D20 and open questions OQ-1–OQ-3.
- [data-model.md](data-model.md): in-memory types; no schema change.
- [contracts/](contracts/): format, core API, UI, docs.
- [quickstart.md](quickstart.md): how to validate.

## Complexity Tracking

None.
