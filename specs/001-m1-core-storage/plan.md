# Implementation Plan: M1 Core Storage and Setup Data

**Branch**: `main` (spec directory `001-m1-core-storage`) | **Date**: 2026-10-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/001-m1-core-storage/spec.md`

## Summary

Build the storage foundation of `planner-core`: migration 001 (exact `docs/spec.md` §3 schema),
`Planner::open`/`open_in_memory` with forward-only migrations tracked by `PRAGMA user_version`
(R25), the data-file location rule (R24), every §5 type and method signature, and working
implementations for terms, holidays, courses, slots (add/list), settings, and assessments (human
decision, spec FR-013a). R1–R3 are enforced; all R2 validation returns `Error::Validation`.
Every method outside M1 scope is a `todo!()` stub with its rule IDs, so M2 (core) and M4 (UI) can
start in parallel. `docs/spec.md` §1–§5 is the plan; nothing in §1 is re-decided.

## Technical Context

**Language/Version**: Rust stable, edition 2021, MSRV 1.90 (`rust-version` in workspace), `Cargo.lock` committed (§1)

**Primary Dependencies**: `rusqlite` (`bundled`), `chrono` (naive types), `thiserror`, `directories`, `serde` (derives on model types for M3) — all already in `[workspace.dependencies]`

**Storage**: SQLite file, `PRAGMA foreign_keys=ON`, `journal_mode=WAL`; schema version in `PRAGMA user_version` (§1)

**Testing**: `cargo test --workspace`; integration tests in `crates/core/tests/` on `Planner::open_in_memory()`; file-based tests (T20, T26) use a temp file under the OS temp dir, never the user DB

**Target Platform**: macOS, Windows, Linux desktop (core is platform-independent; CI runs Linux)

**Project Type**: Rust workspace: library crate `planner-core` + desktop binary `app`

**Performance Goals**: N/A for M1 (a single student's data: tens of courses, hundreds of rows)

**Constraints**: core reads no clock (clippy `disallowed-methods`), no Slint, no file dialogs; every mutation commits immediately; no panics on user input

**Scale/Scope**: one active term, ~10 courses, ~30 slots

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Status |
|---|---|---|
| I. Spec is source of truth | Plan implements §3 schema verbatim and §5 signatures. Two human-approved spec decisions: assessment CRUD in M1 (FR-013a); `now` parameter on `sessions_between`/`sessions_for_course` for `is_past` (applied to `docs/spec.md` §5, 2026-10-28). No other additions. | PASS |
| II. Test-first, traceable | T20, T24, T26 written before the code they cover; names `t20_…`, `t24_…`, `t26_…`; rule IDs in doc comments; tests use in-memory SQLite (temp file only where a file is required by the test) | PASS |
| III. Pure, clock-free core | No Slint/clock/dialogs in core; `AssessmentFilter` carries `now` instead of reading the clock; R24 resolution is a pure function plus a thin env/`ProjectDirs` wrapper | PASS |
| IV. Append-only migrations | `001_init.sql` is new; nothing previously applied is edited | PASS |
| V. Gate before done | Milestone ends with the gate; summary pasted | PASS (pending run) |

**Post-design re-check (after Phase 1)**: PASS. Contracts add no rules; all type fields derive from
§3 columns, plus `Term.holidays`, `Assessment.course_code`, and `SessionView` fields named in §5
(see [research.md](research.md) D6–D8).

## Project Structure

### Documentation (this feature)

```text
specs/001-m1-core-storage/
├── plan.md              # This file
├── research.md          # Phase 0: decisions D1–D12
├── data-model.md        # Phase 1: entities, field types, validation
├── quickstart.md        # Phase 1: how to validate M1
├── contracts/
│   └── core-api.md      # Phase 1: Rust API contract (types + methods, M1 vs stub)
├── checklists/requirements.md
└── tasks.md             # Phase 2 (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/core/
├── Cargo.toml                 # add rusqlite, chrono, thiserror, directories, serde
├── src/
│   ├── lib.rs                 # module wiring + re-exports (Planner, model types, Error, Result)
│   ├── error.rs               # Error enum (thiserror), Result alias
│   ├── model.rs               # all §5 types: inputs, records, enums, views, Settings, reports
│   ├── db.rs                  # Planner struct, open/open_in_memory, migrations runner, R24 path,
│   │                          #   date/time text codecs, terms/holidays/courses/slots/settings/
│   │                          #   assessments implementations
│   ├── migrations/
│   │   └── 001_init.sql       # §3 schema verbatim + setting defaults
│   ├── schedule.rs            # stubs: change_slot, end_slot, sync_sessions, sessions_*, set_status,
│   │                          #   reschedule, add_makeup (M2)
│   ├── attendance.rs          # stubs: attendance_summary, all_summaries (M2)
│   ├── dashboard.rs           # stub: dashboard (M2)
│   ├── backup.rs              # stubs: export_json, restore_json (M3); due_reminders (M5)
│   └── export.rs              # stubs: export_attendance_csv, export_assessments_csv (M3)
└── tests/
    ├── common/mod.rs          # fixture helpers (§8 fixture term/course/slots)
    ├── migrations.rs          # t20, t26
    ├── validation.rs          # t24
    └── setup_crud.rs          # R1, R3, holidays, slots list, settings, assessments CRUD
crates/app/                    # unchanged in M1
```

**Structure Decision**: the §2 layout. Each `impl Planner` block lives in the module named for its
area (Rust allows multiple `impl` blocks per type across modules of one crate), so M2/M3 fill in
their own files without touching `db.rs`.

## Complexity Tracking

No constitution violations. Nothing to justify.
