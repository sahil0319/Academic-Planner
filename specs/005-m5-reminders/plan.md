# Implementation Plan: M5 Reminders

**Branch**: `005-m5-reminders` (spec directory; work happens on the current branch) | **Date**: 2026-10-29 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/005-m5-reminders/spec.md`

## Summary

Implement the frozen §5 method `Planner::due_reminders(&mut self, now) -> Result<Vec<Reminder>>`
in core: it finds the session and assessment reminders whose due window contains `now`
(FR-002–FR-004), records each one in `reminder_log` with `INSERT OR IGNORE` inside one transaction,
and returns only the rows it actually inserted, ordered and titled per FR-005 (FR-001). In the app,
add `notify-rust` and a thin `notify.rs` (§2) that shows one desktop notification per reminder on a
detached thread, and call the reminder check from the existing 60-second R6 timer (`App::tick`),
gated by `settings().notifications_enabled` (FR-006–FR-008). `docs/spec.md` §1–§5 is the plan;
nothing in §1 is re-decided, and no rule, schema, or §5 signature changes.

## Technical Context

**Language/Version**: Rust stable, edition 2021, `rust-version = "1.90"` (workspace), resolver 3 (§1)

**Primary Dependencies**: core: `rusqlite` (bundled), `chrono` (naive local types), `thiserror`;
app: `slint` 1.16, `anyhow`, new `notify-rust = "4.18.2"` with default features (§1 Notifications
row; MSRV 1.89, default feature `z` = pure-Rust `zbus`, so no `libdbus` on Linux; research R-1)

**Storage**: SQLite, migration 001 already has `reminder_log` and the `setting` defaults (§3). No new
migration.

**Testing**: `cargo test --workspace`; new core integration test file
`crates/core/tests/m5_reminders.rs` on the §8 fixture (`tests/common`), in-memory SQLite only.
App notification display is verified by one manual check (FR-010).

**Target Platform**: desktop (macOS, Windows, Linux); CI is Linux only (§7 M0)

**Project Type**: desktop app (Rust workspace: `planner-core` library + `app` binary)

**Performance Goals**: one reminder check per 60-second tick; the check is one small SQLite
transaction and never blocks the UI on notification delivery (FR-008)

**Constraints**: core reads no clock (`now` passed in, §1, `clippy.toml`); core has no notification
code; §5 API and §3 schema frozen; notifications only while the app runs (§10)

**Scale/Scope**: one student, a handful of courses; at most a few reminders per check

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| I. Spec is source of truth | PASS | Only `due_reminders` (already a §5 stub) is implemented; `reminder_log` and settings come from §3; `notify-rust` from §1; `notify.rs` from §2. The behavior details are the dated router decisions recorded in spec.md. No UI visuals change, so DESIGN.md is untouched. |
| II. Test-first, traceable | PASS | Tester writes `m5_*` tests first (no T-IDs exist for M5; doc comments cite spec FR ids and §5). Tests use `Planner::open_in_memory()`. |
| III. Pure, clock-free core | PASS | `due_reminders` takes `now`; titles are built in core because `Reminder.title` is a core field (§5 type). The app adds no formatting, so no new viewmodel function is needed (research R-6). No `.slint` change. |
| IV. Append-only migrations | PASS | No schema change. |
| V. Gate before done | PASS | Final task runs the gate and pastes the summary. |

Post-design re-check: PASS (no violations; Complexity Tracking empty).

## Project Structure

### Documentation (this feature)

```text
specs/005-m5-reminders/
├── spec.md
├── plan.md              # this file
├── research.md          # Phase 0
├── data-model.md        # Phase 1
├── quickstart.md        # Phase 1
├── contracts/
│   ├── core-api-m5.md   # due_reminders behavior contract
│   └── app-notify.md    # notify.rs and the timer hook
├── checklists/requirements.md
└── tasks.md             # /speckit-tasks output
```

### Source Code (repository root)

```text
crates/core/
├── src/
│   ├── reminders.rs     # NEW: impl Planner { due_reminders } and private helpers (moved out of backup.rs)
│   ├── backup.rs        # stub removed; module doc no longer mentions reminders
│   └── lib.rs           # add `mod reminders;`
└── tests/
    └── m5_reminders.rs  # NEW: reminder dedupe and eligibility tests (tester)

crates/app/
├── Cargo.toml           # add notify-rust
└── src/
    ├── notify.rs        # NEW: show(reminder) on a detached thread
    ├── main.rs          # `mod notify;`; timer comment mentions reminders
    └── controller.rs    # tick() runs the reminder check after the R6 work

Cargo.lock               # updated (CI uses --locked)
```

**Structure Decision**: Existing two-crate workspace (§2). `due_reminders` moves from `backup.rs` to a
new private module `reminders.rs` (layout only; the public path `planner_core::Planner::due_reminders`
and its signature are unchanged, so `tests/api_surface.rs` still holds). `.github/workflows/ci.yml`
does not change unless the builder's dependency check finds `libdbus-sys` in the tree (research R-1).

## Complexity Tracking

No violations.
