# Implementation Plan: M3 Backup, Restore, and CSV Export

**Branch**: `main` (spec directory `003-m3-backup-export`) | **Date**: 2026-10-29 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/003-m3-backup-export/spec.md`

## Summary

Fill in the four M3 stubs left by M1 in `planner-core`: `export_json` (R21), `restore_json` (R22),
`export_attendance_csv` and `export_assessments_csv` (R23). Export reads every row of the seven
exported tables into serde row structs and serializes one JSON document. Restore parses and
validates the whole document first, then replaces all data in one transaction with foreign-key
checks deferred to commit, so any failure rolls back and leaves the database unchanged. CSV uses the
`csv` crate with its defaults. No schema, rule, or API change: the §5 signatures stay exactly as
stubbed and are already frozen by `crates/core/tests/api_surface.rs`. The R22 pre-restore file copy
is not built here (spec FR-014; human decision 2026-10-29): M3 adds no core API for it, and it is
built in the M4 app layer. `docs/spec.md` §1–§5 is the plan; nothing in §1 is re-decided.

## Technical Context

**Language/Version**: Rust stable, edition 2021, MSRV 1.90, `Cargo.lock` committed (§1)

**Primary Dependencies**: `rusqlite` (`bundled`), `chrono`, `serde`, `thiserror` already in `planner-core`. Adds `serde_json` and `csv`, both already listed in `[workspace.dependencies]` and named in §1 ("Serialization: `serde`, `serde_json`, `csv`"); only `crates/core/Cargo.toml` gains two lines (research R-D1)

**Storage**: SQLite, schema of migration `001_init.sql` unchanged; no new migration (§3). `reminder_log` is neither exported nor touched by restore

**Testing**: `cargo test --workspace`; integration tests `crates/core/tests/m3_backup_export.rs` (R21), `m3_backup_restore.rs` (R22, T30), and `m3_csv.rs` (R23, T25) on `Planner::open_in_memory()` with the §8 fixture from `tests/common/mod.rs`; pure helpers (if any) get unit tests in their module

**Target Platform**: macOS, Windows, Linux desktop (core is platform-independent; CI runs Linux)

**Project Type**: Rust workspace: library crate `planner-core` + desktop binary `app` (app untouched in M3)

**Performance Goals**: N/A (one term, ~10 courses, a few hundred sessions; whole document held in memory)

**Constraints**: core reads no clock (`now` is a parameter); core opens no file and no dialog (callers pass the JSON string and the `Write` sink); restore is a single transaction; no panics on user input (malformed JSON is `Error::Validation`)

**Scale/Scope**: one active term, ~10 courses, ~30 slots, ≤ ~1,000 sessions

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Status |
| --- | --- | --- |
| I. Spec is source of truth | Implements R21–R23 and the §5 signatures as written. The 2026-10-29 human decision (R22 file copy lives in M4) is already in spec.md FR-014 and changes no rule, schema, or API. The `serde_json` and `csv` dependencies are named in §1 | PASS |
| II. Test-first, traceable | T25, T30, and the backup tests (`r21_`/`r22_` names) are written by the tester agent before implementation; rule IDs in doc comments; in-memory SQLite only | PASS |
| III. Pure, clock-free core | `export_json` takes `now`; CSV and restore take no clock; no file or dialog access in core | PASS |
| IV. Append-only migrations | No migration added or edited | PASS |
| V. Gate before done | Milestone ends with the gate; summary pasted | PASS (pending run) |

**Post-design re-check (after Phase 1)**: PASS. Research decisions R-D1–R-D11 only fix details that
§3–§5 and spec.md leave open (JSON value types, `exported_at` format, validation order, restore
mechanics, CSV encoding and sort); none adds a requirement. Router decisions of 2026-10-29 are
recorded in R-D2, R-D3, R-D4, R-D5, R-D8, R-D9.

## Project Structure

### Documentation (this feature)

```text
specs/003-m3-backup-export/
├── plan.md              # This file
├── research.md          # Phase 0: decisions R-D1–R-D11
├── data-model.md        # Phase 1: backup document shape, restore order, CSV columns (no schema change)
├── quickstart.md        # Phase 1: how to validate M3
├── contracts/
│   └── core-api-m3.md   # Phase 1: behavior of the four stubbed §5 methods
├── checklists/          # requirements.md (specify)
└── tasks.md             # Phase 2 (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/core/
├── Cargo.toml             # + serde_json.workspace, csv.workspace
├── src/
│   ├── backup.rs          # export_json (R21), restore_json (R22); private row structs and
│   │                      #   validation helpers; a small #[cfg(test)] module for the
│   │                      #   reminder_log check. due_reminders stub stays (M5)
│   └── export.rs          # export_attendance_csv, export_assessments_csv (R23)
└── tests/                 # written by the tester agent, test-first
    ├── m3_backup_export.rs  # R21 shape, identity, ordering, empty database
    ├── m3_backup_restore.rs # R22 replace-not-merge, all-or-nothing, T30
    └── m3_csv.rs            # R23, T25
```

**Structure Decision**: the §2 / M1 layout. `db.rs` already exposes the crate-private codecs
`fmt_date`, `fmt_time`, `fmt_datetime`, `parse_date`, `parse_time`, `parse_datetime` and
`SCHEMA_VERSION`; M3 reuses them and edits none of `db.rs`, `model.rs`, `lib.rs`, `error.rs`, the
migration, or `crates/app`. Row structs for the backup are private to `backup.rs` (no public type is
added, so the public API is unchanged).

## Complexity Tracking

No constitution violations. Nothing to justify.
