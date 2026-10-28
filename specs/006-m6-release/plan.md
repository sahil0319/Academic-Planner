# Implementation Plan: M6 Release Packaging

**Branch**: `006-m6-release` (spec directory; work happens on the current branch) | **Date**: 2026-10-29 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/006-m6-release/spec.md`

## Summary

Ship the finished app as installers. Add a `cargo-packager` configuration to the `app` crate
(FR-001), a `cargo-about` configuration and HTML template that turn `Cargo.lock` into
`THIRD-PARTY-NOTICES.html` (FR-004), and a tag-triggered workflow `.github/workflows/release.yml`
that checks the tag against the workspace version (FR-003), generates the notices once, packages on
macOS (`dmg`), Windows (`nsis`), and Linux (`deb`, `appimage`) runners with the notices bundled,
and attaches everything to one draft GitHub release (FR-002). Write `README.md` (FR-005), point
`NOTICE` at the generated notices, and leave `ci.yml` untouched (FR-006). The human pushes the tag
and installs one artifact (FR-007). `docs/spec.md` §1–§5 is the plan: nothing in §1 is re-decided,
and no rule, schema, or §5 API changes. M6 starts only after M5's gate is green (one milestone at a
time).

## Technical Context

**Language/Version**: Rust stable, edition 2021, `rust-version = "1.90"`, resolver 3, workspace
version `0.1.0`, license `GPL-3.0-or-later` (§1, `Cargo.toml`)

**Primary Dependencies** (tools only; no new crate dependency in the workspace):
`cargo-packager` **0.11.8** (installed with `cargo install --locked`; builds on Rust 1.90, verified
2026-10-29), `cargo-about` **0.9.3** (MSRV 1.88; prebuilt in CI via `taiki-e/install-action`
v2.88.0; local install needs `--features cli`), `softprops/action-gh-release` **v3.0.3**
(commit-pinned). See [research.md](research.md) R-1 to R-4.

**Storage**: N/A. The installed app keeps using the R24 per-user data folder; packaging never places
a database in the install folder (US2 scenario 4).

**Testing**: no T-tests (config and docs only; no rule changes). Verification: local macOS
`cargo packager --release --formats dmg`, local `cargo about generate`, workflow lint (`actionlint`
if available, else a YAML parse), the gate, and the human tag build ([quickstart.md](quickstart.md)).

**Target Platform**: macOS (Apple Silicon, `macos-latest`), Windows x64 (`windows-latest`), Linux
x86_64 (`ubuntu-22.04`, the oldest current GitHub runner, for AppImage portability).

**Project Type**: desktop app (Rust workspace: `planner-core` library + `app` binary
`academic-planner`)

**Performance Goals**: N/A (one release build per tag)

**Constraints**: no macOS signing or notarization (§10); no Windows signing; `ci.yml` behavior
unchanged (FR-006); tag push is a human step (FR-007); agents cannot verify Windows or Linux
packaging locally (spec Assumptions).

**Scale/Scope**: 4 installers + 1 notices file per tag; about 600 Rust crates in the notices.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| I. Spec is source of truth | PASS | Scope is exactly the §7 M6 row plus §10 (Gatekeeper note) and the §1 License row (notices). Router decisions are dated in spec.md; planner decisions are listed in research.md and none touches a rule, the schema, or the API. No UI visuals change; the placeholder icon (R-6) takes its color from DESIGN.md `primary` and is not UI code. |
| II. Test-first, traceable | PASS (N/A) | No rule is implemented, so no T-tests. Verification tasks replace tests; the gate still runs. |
| III. Pure, clock-free core | PASS | `core` is not touched. The only Rust change is one crate attribute in `crates/app/src/main.rs` (R-7). |
| IV. Append-only migrations | PASS | No schema change. |
| V. Gate before done | PASS | Final agent task runs the gate and pastes the summary; done-when also needs the human tag build and install. |

Post-design re-check: PASS (no violations; Complexity Tracking empty).

## Project Structure

### Documentation (this feature)

```text
specs/006-m6-release/
├── spec.md
├── plan.md                       # this file
├── research.md                   # Phase 0: versions and decisions
├── data-model.md                 # Phase 1: release artifacts
├── quickstart.md                 # Phase 1: how to verify locally and on a tag
├── contracts/
│   ├── packager-config.md        # [package.metadata.packager] block
│   ├── notices.md                # about.toml, about.hbs, NOTICE wording
│   ├── release-workflow.md       # release.yml jobs, steps, pins
│   └── readme.md                 # README.md outline and facts
├── checklists/requirements.md
└── tasks.md                      # /speckit-tasks output
```

### Source Code (repository root)

```text
Cargo.toml                        # unchanged (workspace version is the release version)
about.toml                        # NEW: cargo-about accepted licenses
about.hbs                         # NEW: notices HTML template
.gitignore                        # add /THIRD-PARTY-NOTICES.html (generated, not committed)
NOTICE                            # last paragraph points to THIRD-PARTY-NOTICES.html
README.md                         # NEW: US3 contents
docs/screenshots/                 # NEW (human): dashboard, timetable, courses, assessments, settings .png
.github/workflows/release.yml     # NEW: tag-triggered release
.github/workflows/ci.yml          # unchanged
crates/app/
├── Cargo.toml                    # add [package.metadata.packager] (+ .deb, .nsis subtables)
├── icons/icon.png                # NEW: 512x512 placeholder icon (AppImage needs one)
└── src/main.rs                   # add windows_subsystem attribute for release builds
```

**Structure Decision**: Packager config lives in `crates/app/Cargo.toml` metadata so the packager
auto-fills version and the `academic-planner` binary from Cargo metadata; its relative paths
resolve from `crates/app/` (research R-1). `cargo-about` files sit at the workspace root, its default
config location. The notices file is generated, never committed, so it always matches `Cargo.lock`.

## Complexity Tracking

No violations.
