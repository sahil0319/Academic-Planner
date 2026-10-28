# Implementation Plan: M4 Desktop UI

**Branch**: `main` (spec directory `004-m4-desktop-ui`) | **Date**: 2026-10-29 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/004-m4-desktop-ui/spec.md`

## Summary

Build the full Slint desktop UI (§6) in the `app` crate on top of the frozen §5 core API. Work goes
in this order: dependencies, then `ui/theme.slint` translated by hand from every `DESIGN.md` token
plus the §12.3 theme grep step in CI, then the pure view-model module (`src/viewmodel.rs`, tests
first), then the app controller (state, sync triggers, refresh, error routing), then one `.slint`
page plus one Rust page module per §6 page, then the R22 pre-restore copy, then the 60-second
rollover timer (R6), then converge and gate. `docs/spec.md` §1–§6 is the plan; nothing in §1 is
re-decided. The core crate, schema, rules, and §5 API do not change.

## Technical Context

**Language/Version**: Rust stable, edition 2021, MSRV 1.90, `Cargo.lock` committed (§1)

**Primary Dependencies**: `slint` 1.16 + `slint-build` (already in the app); `planner-core` (path
dependency, already present); `anyhow` (already present). M4 adds to `crates/app/Cargo.toml`:
`rfd` (file dialogs, fixed by §1), `chrono` from the workspace with the extra `clock` feature (the
app is the one place allowed to read the clock), and `rusqlite` from the workspace (only for the
R22 pre-restore copy, research R-D3). No other crates.

**Storage**: SQLite file at the R24 path, opened through `Planner::open`. Schema unchanged. The app
writes three kinds of files only on explicit user action: the JSON backup, the two CSV files, and the
pre-restore copy in `{data_dir}/backups/`.

**Testing**: `cargo test --workspace`. App tests are unit tests inside the binary crate (a bin crate
has no integration-test access to its modules): `crates/app/src/viewmodel/tests.rs` (view-model
functions); the pre-restore copy is checked manually (quickstart §6, Q1 = B). The
screens are verified by the human §9 smoke checklist and the end-of-M4 screenshot critique (§12.3).
All earlier core tests stay green.

**Target Platform**: macOS, Windows, Linux desktop; CI runs Linux only (§7 M0).

**Project Type**: Rust workspace: library `planner-core` + desktop binary `app` (`academic-planner`).

**Performance Goals**: N/A. One term, ~10 courses, ≤ ~1,000 sessions; every core call is
synchronous on the UI thread and every refresh reloads all pages.

**Constraints**: `.slint` files hold only bindings and callbacks (§2); every visual value comes from
`Theme.*` (§6, §12.3 grep); core stays clock-free and dialog-free (§1); the app never panics on user
input or core errors (spec edge cases); every mutation commits immediately (§1).

**Scale/Scope**: 5 pages, ~7 forms, ~10 list types, ~40 view-model functions.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Status |
| --- | --- | --- |
| I. Spec is source of truth | Implements §6 with the 2026-10-29 human decisions already in §6 and spec.md (holiday ranges, note only via reschedule, "Add makeup class", pre-restore copy in the app). No rule, schema, or API change; `rfd` is fixed by §1. Visual values come only from `DESIGN.md` via `theme.slint` | PASS |
| II. Test-first, traceable | View-model tests (including the pre-restore file name and same-minute suffix) are written by the tester before implementation; doc comments cite rule IDs; app tests use no database or in-memory SQLite only and never touch the real user DB. The file-level behavior of the R22 pre-restore copy (WAL data included, backups folder created) is verified manually in quickstart §6 | PASS |
| III. Pure, clock-free core | Core unchanged. The app reads the clock in exactly one function in `main.rs` with a scoped `#[allow(clippy::disallowed_methods)]` (research R-D1). All formatting lives in `src/viewmodel.rs` as pure functions with unit tests; `.slint` files only bind | PASS |
| IV. Append-only migrations | No migration added or edited | PASS |
| V. Gate before done | Milestone ends with the gate plus the theme grep check and DESIGN.md lint; summary pasted | PASS (pending run) |

**Post-design re-check (after Phase 1)**: PASS, no exceptions (router decision 2026-10-29: Q1 = B,
manual check, to avoid a constitution amendment). Research decisions R-D1–R-D18 only fix details that §1–§6, DESIGN.md, and
spec.md leave open (module layout, Slint structure, parse strictness, refresh strategy, file names);
none adds a requirement.

## Project Structure

### Documentation (this feature)

```text
specs/004-m4-desktop-ui/
├── plan.md              # This file
├── research.md          # Phase 0: decisions R-D1–R-D18
├── data-model.md        # Phase 1: view-model rows, form inputs, UI state (no stored entities)
├── quickstart.md        # Phase 1: validation run guide (gate, grep, lint, §9 smoke)
├── contracts/
│   ├── viewmodel.md     # Pure function signatures + expected outputs (tester writes tests from this)
│   └── ui-bridge.md     # Slint globals, structs, callbacks; Rust page-module responsibilities
├── checklists/          # requirements.md (spec quality) + ui.md (speckit-checklist)
└── tasks.md             # speckit-tasks output
```

### Source Code (repository root)

```text
.github/workflows/ci.yml          # + "Theme grep check" step (§12.3)
crates/app/
├── Cargo.toml                    # + rfd, chrono (clock), rusqlite
├── build.rs                      # compile ui/app-window.slint with style "fluent-light"
├── ui/
│   ├── theme.slint               # export global Theme: every DESIGN.md token (only place with hex/px)
│   ├── types.slint               # shared structs/enums + global AppState (page, startup error)
│   ├── components.slint          # NavItem, PrimaryButton, SecondaryButton, Field, StatusChip,
│   │                             # SectionCard, ListRow, EmptyState, Banner, PageTitle
│   ├── app-window.slint          # AppWindow: nav + page switch + startup error; re-exports globals
│   ├── dashboard.slint           # DashboardPage + global DashboardState
│   ├── timetable.slint           # TimetablePage (Week / Rules tabs) + global TimetableState
│   ├── courses.slint             # CoursesPage (list + detail) + global CoursesState
│   ├── assessments.slint         # AssessmentsPage + global AssessmentsState
│   └── settings.slint            # SettingsPage + global SettingsState
└── src/
    ├── main.rs                   # entry, the single clock read, open DB, build window, start timer
    ├── viewmodel.rs              # pure formatting / parsing / validation functions (§2)
    ├── viewmodel/tests.rs        # tester-owned unit tests
    ├── controller.rs             # App: Planner + UI state, sync triggers (R6), refresh_all, errors
    ├── bridge.rs                 # viewmodel row structs -> Slint structs (field copies only)
    ├── restore.rs                # R22 pre-restore copy (VACUUM INTO) + restore flow helper
    └── pages/
        ├── mod.rs
        ├── dashboard.rs          # refresh + callbacks for DashboardState
        ├── timetable.rs
        ├── courses.rs
        ├── assessments.rs
        └── settings.rs
```

**Structure Decision**: Each page owns exactly one `.slint` file and one `src/pages/*.rs` file and
talks to Rust through its own Slint global (research R-D6). The shared files (`theme.slint`,
`types.slint`, `components.slint`, `app-window.slint`, `controller.rs`, `bridge.rs`,
`pages/mod.rs`) are written before the page tasks, with stub globals for every page, so the five
page tasks touch disjoint files and can run in parallel. `crates/core` is not edited.

## Phase outline (feeds tasks.md)

1. **Dependencies**: `rfd`, `chrono` (`clock`), `rusqlite` in the app; `build.rs` style.
2. **Theme**: full `theme.slint` translation (R-D13) + CI grep step; window uses `Theme.*`.
3. **View model**: tester writes `viewmodel/tests.rs` against stub signatures from
   [contracts/viewmodel.md](contracts/viewmodel.md) (red); builder implements (green).
4. **Shell and controller**: `types.slint`, `components.slint`, `app-window.slint` with stub page
   globals; `controller.rs`, `bridge.rs`, `pages/mod.rs`, `main.rs` (clock, open DB with startup
   error, sync on start, refresh_all).
5. **Pages** [parallel]: dashboard, timetable, courses, assessments, settings (settings without
   restore).
6. **R22 restore**: builder implements the copy (manual check, Q1 = B) and wires
   Restore with the confirm panel; post-restore sync.
7. **R6 timer**: 60-second `slint::Timer`; rollover sync + refresh.
8. **Converge and gate**: gate, grep check, DESIGN.md lint, screenshots + critique (router/human),
   §9 smoke (human).

## Complexity Tracking

None. Q1 was first answered A (file database in a temp folder for the pre-restore copy test),
which conflicts with Principle II; the router switched it to B (manual check in quickstart §6) under
the human's 2026-10-29 instruction to take the recommended fix, so no constitution amendment is
needed.
