# Implementation Plan: M2 Sessions, Exceptions, Attendance, and Dashboard

**Branch**: `main` (spec directory `002-m2-sessions-attendance`) | **Date**: 2026-10-29 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/002-m2-sessions-attendance/spec.md`

## Summary

Fill in every M2 stub left by M1 in `planner-core`: session sync (R4, R5, with the R3 archived-course
exemption), slot change and end (R7, syncing with the effective date / day after last date as
today), cancel / reschedule / makeup (R8–R10), status marking (R11, R12), session views with
`display_status` and `is_past` (§5), attendance summaries (R13–R15), the dashboard (R16–R20), and
the `Upcoming`/`Overdue` filters of `list_assessments` (R18, R19). Exam periods are holidays
(FR-006a) and need no code beyond R4/R5. No schema, rule, or API change: the §5 signatures are
frozen by `crates/core/tests/api_surface.rs`. `docs/spec.md` §1–§5 is the plan; nothing in §1 is
re-decided.

## Technical Context

**Language/Version**: Rust stable, edition 2021, MSRV 1.90, `Cargo.lock` committed (§1)

**Primary Dependencies**: `rusqlite` (`bundled`), `chrono` (naive types), `thiserror` — already in use; no new dependency

**Storage**: SQLite, schema of migration `001_init.sql` unchanged; no new migration (§3)

**Testing**: `cargo test --workspace`; integration tests in `crates/core/tests/m2_*.rs` on `Planner::open_in_memory()` with the §8 fixture from `tests/common/mod.rs`; pure helpers (display status, percentage rounding) get unit tests in their module

**Target Platform**: macOS, Windows, Linux desktop (core is platform-independent; CI runs Linux)

**Project Type**: Rust workspace: library crate `planner-core` + desktop binary `app` (app untouched in M2)

**Performance Goals**: N/A (one term, ~10 courses, a few hundred sessions; sync may rebuild its expected set in memory on every run)

**Constraints**: core reads no clock (`today`/`now` are parameters); every mutation commits before returning; multi-step mutations (sync, change_slot, end_slot, reschedule) run in one transaction; no panics on user input

**Scale/Scope**: one active term, ~10 courses, ~30 slots, ≤ ~1,000 sessions

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Status |
| --- | --- | --- |
| I. Spec is source of truth | Implements R3–R20 and §5 as written. Human decisions of 2026-10-29 (slot-change sync date, R3 over R5, exam periods as holidays) are already in spec.md and change no rule, schema, or API. | PASS |
| II. Test-first, traceable | All 25 T-tests plus the User Story 7 scenarios are written (tester agent) before implementation; names `t04_…`; rule IDs in doc comments; in-memory SQLite only | PASS |
| III. Pure, clock-free core | Every time-dependent method takes `today`/`now`; display-status and percentage formatting are pure core functions with unit tests | PASS |
| IV. Append-only migrations | No migration added or edited | PASS |
| V. Gate before done | Milestone ends with the gate; summary pasted | PASS (pending run) |

**Post-design re-check (after Phase 1)**: PASS. Research decisions R-D1–R-D12 only fix behavior
§4/§5 leave open (ordering, error variants, no-active-term behavior, minute precision); none adds a
requirement. Router decisions of 2026-10-29 (session room fallback, `change_slot` `valid_from`,
`all_summaries` scope, `Error::Validation` for R9/R12) are recorded in R-D3, R-D4, R-D6, R-D7, R-D9.

## Project Structure

### Documentation (this feature)

```text
specs/002-m2-sessions-attendance/
├── plan.md              # This file
├── research.md          # Phase 0: decisions R-D1–R-D12
├── data-model.md        # Phase 1: session lifecycle and derived values (no schema change)
├── quickstart.md        # Phase 1: how to validate M2
├── contracts/
│   └── core-api-m2.md   # Phase 1: M2 behavior of the stubbed §5 methods
├── checklists/          # requirements.md (specify), rules.md (checklist)
└── tasks.md             # Phase 2 (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/core/
├── src/
│   ├── db.rs              # + SESSION_COLS, session_from_row, load_session, start/end
│   │                      #   date-time helpers; list_assessments Upcoming/Overdue (R18, R19)
│   ├── schedule.rs        # sync_sessions + shared sync_in(tx, today) (R3, R4, R5);
│   │                      #   change_slot, end_slot (R7); set_status (R11, R12);
│   │                      #   reschedule (R9); add_makeup (R10); sessions_between,
│   │                      #   sessions_for_course, display_status (§5)
│   ├── attendance.rs      # attendance_summary, all_summaries, pct/below-target helpers (R13–R15)
│   └── dashboard.rs       # dashboard (R16–R20, R3)
└── tests/                 # written by the tester agent, test-first
    ├── m2_sync.rs         # T04–T07, T14
    ├── m2_slot_changes.rs # T10, T11, T27
    ├── m2_exceptions.rs   # T08, T09, T19, T22
    ├── m2_attendance.rs   # T01–T03, T12, T21, T23
    ├── m2_dashboard.rs    # T13, T16–T18, T29
    ├── m2_archive.rs      # T15, T28
    └── m2_exam_periods.rs # User Story 7 (FR-006a)
```

**Structure Decision**: the §2 / M1 layout. Each area's `impl Planner` block stays in its module;
row decoding and shared helpers go in `db.rs` beside the M1 codecs. `model.rs`, `lib.rs`,
`error.rs`, and the migration are not edited.

## Complexity Tracking

No constitution violations. Nothing to justify.
