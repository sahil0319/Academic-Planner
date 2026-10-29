# Implementation Plan: M8 12-Hour Times

**Branch**: `008-m8-12-hour-times` (spec directory; work happens on the current branch) | **Date**: 2026-10-29 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/008-m8-12-hour-times/spec.md`

## Summary

App crate only. `docs/spec.md` §1–§5, §6 ("Date/time inputs"), and §13.2 are the plan; nothing in
§1 is re-decided (Clock row still holds: the new functions are pure). No core, schema, backup,
CSV, or §5 change.

1. **Display (R31, T35).** Change the body of `viewmodel::format_time` to `h:mm AM/PM`. Every
   displayed time already flows through it (`format_time_range` for session and slot rows,
   `format_datetime` for assessment rows, `format_time` for the Dashboard day panel's due times),
   so no page changes for display (research D1). Core's R2 slot message is rewritten in
   `error_text` with a new pure `times_to_12h` (research D5).
2. **Input (R32, T34).** Replace the strict `HH:MM` `parse_time` with
   `parse_time(digits, pm)`; the three form parsers take a new `TimeText { digits, pm }` per time
   field (keeps clippy's 7-argument limit; research D3). A new `format_time_input` gives the
   `hmm` + toggle pre-fill (research D4).
3. **UI.** Two new components in `components.slint`, `AmPmToggle` and `TimeField`, built from the
   `AppearanceSegment` pattern and existing `Theme.*` tokens (research D6). The seven time fields
   on Timetable (reschedule, Rules slot form), Courses (Add makeup class), and Assessments switch
   from `Field` to `TimeField`, with new `*-pm` bool properties on the page state globals
   (data-model.md). Pages pre-fill, clear, and read them (contracts/ui-time-field.md).
4. **Open question OQ-1** (notification text, spec.md): one conditional task applies
   `times_to_12h` in `notify.rs` if the human picks option B.

Work runs in three rounds: (1) builder signatures, stubs, and state properties so everything
compiles; (2) tester writes T34, T35, and the other view-model tests and updates existing
expectations; (3) builder implements, builds the components, wires every page, and runs the gate
plus the theme grep (research D9).

## Technical Context

**Language/Version**: Rust (workspace toolchain, edition as in the workspace `Cargo.toml`)

**Primary Dependencies**: `chrono` (`NaiveTime`, `NaiveDateTime`), Slint (workspace version); no
new dependency

**Storage**: unchanged SQLite schema v1; times stay `HH:MM` text (FR-004)

**Testing**: `cargo test` unit tests in `crates/app/src/viewmodel/tests.rs`; no DB needed

**Target Platform**: macOS, Windows, Linux desktop (as M6)

**Project Type**: desktop app (Cargo workspace: `crates/core`, `crates/app`)

**Performance Goals**: n/a (string formatting of a few hundred rows)

**Constraints**: `.slint` holds bindings only; `Theme.*` only, theme grep clean; clippy
`-D warnings` (including `too_many_arguments`)

**Scale/Scope**: 1 view-model file, 1 test file, 4 page files (+ `notify.rs` if OQ-1 = B),
4 `.slint` files; 7 time inputs

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status |
|---|---|
| I. Spec is source of truth | Pass. R31/R32/T34/T35 as written; naive choices listed in spec Assumptions; OQ-1 left to the human; no new rule, schema, or API. |
| II. Test-first, traceable | Pass. Round 2 writes `t34_*`/`t35_*` before round 3 implements; doc comments cite R31/R32. No DB in these tests. |
| III. Pure, clock-free core | Pass. Core untouched; new functions are pure and live in `viewmodel.rs`; `.slint` changes are bindings and callbacks only. |
| IV. Append-only migrations | Pass. No migration. |
| V. Gate before done | Pass. Final task runs the gate and the theme grep and pastes the summary. |

Post-design re-check: unchanged, all pass. No Complexity Tracking entries.

## Project Structure

### Documentation (this feature)

```text
specs/008-m8-12-hour-times/
├── spec.md
├── plan.md                 # this file
├── research.md             # D1–D9
├── data-model.md           # parse/display rules, TimeText, new Slint properties
├── quickstart.md           # automated + manual checks
├── contracts/
│   ├── viewmodel-time.md   # function signatures and tests
│   └── ui-time-field.md    # components, fields per page, page wiring
├── checklists/
│   └── requirements.md
└── tasks.md                # /speckit-tasks output
```

### Source Code (repository root)

```text
crates/app/
├── src/
│   ├── viewmodel.rs        # format_time (12h), parse_time(digits, pm), TimeText,
│   │                       # format_time_input, times_to_12h, error_text, form parsers
│   ├── viewmodel/tests.rs  # T34, T35, pre-fill, round trip, times_to_12h, updated tests
│   ├── notify.rs           # only if OQ-1 = B
│   └── pages/
│       ├── timetable.rs    # reschedule + slot form: pre-fill, clear, TimeText
│       ├── courses.rs      # makeup form: clear, TimeText
│       ├── assessments.rs  # due time: pre-fill, clear, TimeText
│       └── dashboard.rs    # no change expected (uses format_time)
└── ui/
    ├── components.slint    # AmPmToggle, TimeField
    ├── timetable.slint     # 4 TimeFields + rs/slot *-pm properties
    ├── courses.slint       # 2 TimeFields + mk *-pm properties
    └── assessments.slint   # 1 TimeField + due-time-pm property
```

**Structure Decision**: existing app layout; no new files outside `specs/`. Core is not touched.

## Complexity Tracking

None.
