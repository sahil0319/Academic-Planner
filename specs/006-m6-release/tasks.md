---

description: "Task list for M6 release packaging"
---

# Tasks: M6 Release Packaging

**Input**: Design documents from `specs/006-m6-release/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [quickstart.md](quickstart.md), [contracts/packager-config.md](contracts/packager-config.md), [contracts/notices.md](contracts/notices.md), [contracts/release-workflow.md](contracts/release-workflow.md), [contracts/readme.md](contracts/readme.md)

**Tests**: None. M6 is configuration and documentation and implements no rule, so there are no
T-tests (plan Constitution Check II). Verification tasks (local notices run, local macOS DMG,
workflow lint, app smoke launch, gate) replace tests. No tester role in M6.

**Start condition**: M5's gate is green and M5 is committed (one milestone at a time). M5 changes
`crates/app/Cargo.toml`, `crates/app/src/main.rs`, and `Cargo.lock`; M6 edits the first two after
M5 is done.

**Organization**: Phase 1 setup, Phase 2 the notices tooling that every installer needs, then one
phase per user story (US1 P1, US2 P1, US3 P2), then the gate and the human done-when steps.

## Format: `[ID] [P?] [Story] [owner] Description`

- **[P]**: can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: user story from spec.md (US1–US3)
- **[builder]** / **[human]**: who does the task. Human tasks are outward-facing (tag push) or need
  an unlocked screen (screenshots, install); agents do not do them.

## Path Conventions

Files created: `about.toml`, `about.hbs`, `README.md`, `.github/workflows/release.yml`,
`crates/app/icons/icon.png`, `docs/screenshots/*.png` (human). Files edited: `.gitignore`,
`NOTICE`, `crates/app/Cargo.toml` (append the packager tables only), `crates/app/src/main.rs` (one
attribute line). Not edited: `.github/workflows/ci.yml`, `Cargo.toml` (root), `Cargo.lock`, any
`crates/core` file, any `.slint` file, `.specify/feature.json`, `.claude/state/` (router only).
Generated and never committed: `THIRD-PARTY-NOTICES.html`, `target/tools/`.

---

## Phase 1: Setup

**Purpose**: Pinned tools available locally without touching the host's global cargo bin.

- [X] T001 [builder] Confirm the M5 gate is green on the current tree (`cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`); if not, stop and report to the router. Then install the pinned tools into the git-ignored `target/tools/` folder exactly as in [quickstart.md](quickstart.md) "Prerequisites": `cargo install cargo-packager --locked --version 0.11.8 --root target/tools` and `cargo install cargo-about --locked --version 0.9.3 --features cli --root target/tools` (the `cli` feature is required, research R-2). Use `PATH="$PWD/target/tools/bin:$PATH"` for every later local tool command.

---

## Phase 2: Foundational (notices tooling; blocks US1 packaging and US3)

**Purpose**: `THIRD-PARTY-NOTICES.html` must exist before any installer is built (contract: the
packager fails without it, SC-003).

- [X] T002 [P] [builder] Create `about.toml` at the repo root with exactly the accepted list (priority order `MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `BSL-1.0`, `Unicode-3.0`, `MPL-2.0`, `NCSA`, `GPL-3.0-only`, `GPL-3.0-or-later`), `ignore-dev-dependencies = true`, and the comment from [contracts/notices.md](contracts/notices.md).
- [X] T003 [P] [builder] Create `about.hbs` at the repo root: minimal HTML with no inline styles and the required content listed in [contracts/notices.md](contracts/notices.md) "`about.hbs`" (title and h1 "Academic Planner: third-party notices", intro sentence, overview list, per-license sections with "Used by" and `<pre>{{text}}</pre>`).
- [X] T004 [P] [builder] Add the line `/THIRD-PARTY-NOTICES.html` to `.gitignore` (generated file, research R-2).
- [X] T005 [builder] Verify notices: from the repo root run `cargo about generate --workspace --locked --fail -o THIRD-PARTY-NOTICES.html about.hbs`; expect exit 0, `git status --short THIRD-PARTY-NOTICES.html` prints nothing, and the HTML lists `slint` under "GNU General Public License v3.0 only". If a license is rejected: add it to `about.toml` only if it is GPL-3.0-compatible and say so in the task note; otherwise stop and ask the router. Needs T002, T003, T004

**Checkpoint**: notices generate locally from `Cargo.lock`.

---

## Phase 3: User Story 1 - Maintainer cuts a release with one tag (Priority: P1) MVP

**Goal**: a pushed `v*` tag builds `.dmg`, NSIS `.exe`, `.deb`, `.AppImage` and attaches them with
the notices to one draft release (FR-001, FR-002, FR-003).

**Independent Test**: locally, `cargo packager -p app --release --formats dmg` produces the DMG with
the notices inside, and the workflow lints clean; on GitHub, the human's tag build (T019) is green on
all three runners.

- [X] T006 [P] [US1] [builder] Create `crates/app/icons/icon.png`: a 512x512 PNG whose every pixel is DESIGN.md `colors.primary` `#B4532F` (research R-6). Generate it with a throwaway Python standard-library script (`zlib` + `struct`, run with `python3 -I`) written in a scratch directory outside the repo; commit only the PNG. Check with `file crates/app/icons/icon.png` (expect "PNG image data, 512 x 512").
- [X] T007 [US1] [builder] Append the `[package.metadata.packager]` and `[package.metadata.packager.deb]` tables to `crates/app/Cargo.toml` exactly as in [contracts/packager-config.md](contracts/packager-config.md) (product name "Academic Planner", identifier `io.github.sahil0319.academic-planner`, description, publisher, copyright, category `Education`, `license-file = "../../LICENSE"`, `before-packaging-command = "cargo build --release --locked -p app"`, `icons = ["icons/icon.png"]`, the three `{ src, target }` resources, deb `depends = ["libfontconfig1", "libxkbcommon0"]`). Do not set `version`, `binaries`, or `formats`. Do not change any existing line of the file. Needs T006
- [X] T008 [P] [US1] [builder] Add `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` to `crates/app/src/main.rs` right after the module doc comment (research R-7). No other change to the file.
- [X] T009 [P] [US1] [builder] Create `.github/workflows/release.yml` exactly as in [contracts/release-workflow.md](contracts/release-workflow.md): trigger `push.tags: ["v*"]` only; workflow `permissions: contents: read`; job `prepare` on `ubuntu-22.04` (checkout `@v7`, `dtolnay/rust-toolchain@stable`, tag-vs-version check that prints `::error::Tag <tag> does not match workspace version <version> (expected v<version>)` and exits 1, `taiki-e/install-action@86f7a0ffb25f40114606501a311ebcfaf882ea32 # v2.88.0` with `tool: cargo-about@0.9.3`, generate notices, upload artifact `third-party-notices`); job `package` (`needs: prepare`, `fail-fast: false`, matrix `macos-latest`/`dmg`, `windows-latest`/`nsis`, `ubuntu-22.04`/`deb,appimage`; download notices to `.`; Linux `apt-get install -y libfontconfig1-dev libxkbcommon-dev libfuse2`; toolchain; `Swatinem/rust-cache@v2`; `cargo install cargo-packager --locked --version 0.11.8`; `cargo packager -p app --release --formats ${{ matrix.formats }}`; upload `target/release` installers with `if-no-files-found: error`); job `release` (`needs: package`, `permissions: contents: write`, download all with `merge-multiple: true` into `dist`, `softprops/action-gh-release@efb35369e0ad2afab669f228072c1b0d510eae64 # v3.0.3` with `draft: true`, `fail_on_unmatched_files: true`, `files: dist/*`). Do not edit `.github/workflows/ci.yml`.
- [X] T010 [US1] [builder] Verify the macOS package per [quickstart.md](quickstart.md) step 2: with `THIRD-PARTY-NOTICES.html` present (T005), run `cargo packager -p app --release --formats dmg` from the repo root; expect exactly one `target/release/Academic Planner_0.1.0_aarch64.dmg` and `target/release/Academic Planner.app/Contents/Resources/` containing `THIRD-PARTY-NOTICES.html`, `LICENSE`, `NOTICE`, and an `.icns`. If the packager rejects a config key, fix only its spelling (contract "Guarantees"). Needs T005, T007, T008
  - Note (builder): `--formats dmg` alone deletes the intermediate `.app` after building the DMG, so the local check used `--formats app,dmg`. On a locked screen the DMG step's Finder AppleScript times out and leaves only `rw.*.dmg`; `CI=true` skips that step (as on runners). Result: `Academic Planner_0.1.0_aarch64.dmg`, 7.1 MiB, `hdiutil verify` VALID.
- [X] T011 [US1] [builder] Verify the workflow per [quickstart.md](quickstart.md) step 3: `actionlint .github/workflows/release.yml` if installed, else `ruby -ryaml -e 'YAML.load_file(".github/workflows/release.yml"); puts "ok"'`; `git diff --quiet HEAD -- .github/workflows/ci.yml` (ci.yml unchanged, FR-006). Also dry-run the version check locally: run the `prepare` check script with `GITHUB_REF_NAME=v0.1.0` (expect exit 0) and with `GITHUB_REF_NAME=v0.0.9` (expect exit 1 and the error line naming `v0.0.9` and `0.1.0`). Needs T009

**Checkpoint**: config and workflow ready; Windows and Linux packaging are proven only by the tag
build (T019).

---

## Phase 4: User Story 2 - Student installs the app (Priority: P1)

**Goal**: an installed app opens and keeps its data in the R24 per-user folder, never in the
install folder.

**Independent Test**: the packaged macOS app starts with a throwaway database; the human installs
one artifact and the app opens (T020).

- [X] T012 [US2] [builder] Smoke-launch the packaged app without touching the real database: run `ACADEMIC_PLANNER_DB=/tmp/ap-m6-smoke.sqlite "target/release/Academic Planner.app/Contents/MacOS/academic-planner" &`, wait 5 seconds, confirm the process is still running and `/tmp/ap-m6-smoke.sqlite` exists, then kill the process and delete the temp files. Confirm no `*.sqlite` file exists anywhere under `target/release/Academic Planner.app` (US2 scenario 4). If the process exits only because no GUI session is available (screen locked or headless), record the exit message in the task note and leave the launch check to the human's install (T021); the no-database-in-bundle check still applies. Needs T010

---

## Phase 5: User Story 3 - README and third-party notices (Priority: P2)

**Goal**: a README that lets a student install and a maintainer build; `NOTICE` points at the
generated notices (FR-004, FR-005).

**Independent Test**: README.md has every section of [contracts/readme.md](contracts/readme.md),
all non-screenshot links resolve to files in the repo, and `NOTICE` no longer says "generated at
release time (milestone M6)".

- [X] T013 [P] [US3] [builder] Edit `NOTICE`: replace only the last paragraph ("Third-party dependency notices are generated at release time (milestone M6).") with the paragraph in [contracts/notices.md](contracts/notices.md) "`NOTICE` change"; every other line stays the same.
- [X] T014 [P] [US3] [builder] Create `README.md` with the seven sections, in order, from [contracts/readme.md](contracts/readme.md): description; screenshots referencing `docs/screenshots/dashboard.png`, `docs/screenshots/timetable.png`, `docs/screenshots/courses.png`, `docs/screenshots/assessments.png`, `docs/screenshots/settings.png` (with alt text); download and install for macOS (Apple Silicon DMG, Gatekeeper "Open Anyway" / right-click Open / `xattr -dr com.apple.quarantine` steps, Intel builds from source), Windows (SmartScreen "More info" → "Run anyway"), Linux (`apt install ./….deb`, AppImage `chmod +x`); build from source (`cargo run -p app`, Linux apt packages, the gate command); where the data lives (paths from research R-9, Settings shows the path, `ACADEMIC_PLANNER_DB`); one backup/restore/CSV paragraph (R21–R23, using the button labels from `crates/app/ui/settings.slint`); license (GPL-3.0-or-later, Slint GPL-3.0-only, links to `LICENSE`, `NOTICE`, and `THIRD-PARTY-NOTICES.html` in releases). Promise nothing out of scope (§10).
- [X] T015 [US3] [builder] Verify README and NOTICE: every relative link in `README.md` except the five `docs/screenshots/*.png` points to an existing file; the README contains the strings "Open Anyway", "Run anyway", "chmod +x", "cargo run -p app", "ACADEMIC_PLANNER_DB", and "GPL-3.0"; `grep -n "milestone M6" NOTICE` prints nothing. Needs T013, T014
- [ ] T016 [US3] [human] Capture the five screenshots with real (or demo) data and save them as `docs/screenshots/dashboard.png`, `docs/screenshots/timetable.png`, `docs/screenshots/courses.png`, `docs/screenshots/assessments.png`, `docs/screenshots/settings.png` (agents cannot see an unlocked screen overnight). Use a throwaway `ACADEMIC_PLANNER_DB` if the real data is private. Needs T014

**Checkpoint**: README complete except the human's images.

---

## Phase 6: Gate and done-when

- [X] T017 [builder] Run `speckit-converge` against spec.md, plan.md, and tasks.md; append and do any leftover agent work. Needs T005, T010, T011, T012, T015
- [X] T018 [builder] Run the gate `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` and paste the summary (FR-006, SC-004). The local `THIRD-PARTY-NOTICES.html` may stay (git ignores it). Do not edit `.claude/state/` (the router does). Needs T017
- [ ] T019 [human] After the router's commits are pushed (`git push origin main`), push the release tag: `git tag v0.1.0 && git push origin v0.1.0`. Watch the "Release" workflow: `prepare`, the three `package` jobs, and `release` must all be green (US1, FR-007, SC-001). Needs T018
- [ ] T020 [human] Open the draft release for `v0.1.0` and confirm it holds the `.dmg`, the `-setup.exe`, the `.deb`, the `.AppImage`, and `THIRD-PARTY-NOTICES.html` (US1 scenario 2, FR-004, FR-007). Needs T019
- [ ] T021 [human] Install one artifact on your own OS by following README.md alone (on macOS, including the Gatekeeper steps) and open the app; it shows your data or an empty planner (US2, FR-007, SC-002). If you install on Windows or Linux, also check US2 scenario 2 (Start-menu entry and uninstaller) or 3 (deb installs, AppImage runs after `chmod +x`). Then publish the draft or keep it as a draft. Needs T020

---

## Dependencies & Execution Order

### Phase dependencies

- **Phase 1** (T001) before everything.
- **Phase 2**: T002, T003, T004 in parallel → T005.
- **Phase 3**: T006 → T007; T008 and T009 in parallel with T006–T007; T010 needs T005, T007, T008;
  T011 needs T009.
- **Phase 4**: T012 needs T010.
- **Phase 5**: T013 and T014 in parallel (may start after T001); T015 needs both; T016 (human) any
  time after T014.
- **Phase 6**: T017 → T018 → T019 → T020 → T021.

### Story dependencies

- US1 needs Phase 2 (notices must exist before packaging).
- US2 needs US1's packaged app.
- US3 is independent of US1 and US2 except that its README facts come from research.md.

### Parallel opportunities

| Group | Tasks | Owner |
| --- | --- | --- |
| Notices tooling | T002, T003, T004 | builder |
| Different files in US1 | T006, T008, T009 | builder |
| Docs | T013, T014 (beside Phase 3) | builder |
| Screenshots | T016 any time after T014 | human |

### Owner summary

- **builder**: T001–T015, T017, T018
- **human**: T016, T019, T020, T021

## Implementation Strategy

1. Setup and notices (T001–T005).
2. MVP: US1 config and workflow (T006–T011), proven locally by the DMG and the lint.
3. US2 smoke launch (T012), then US3 docs (T013–T015) and the human screenshots (T016).
4. Converge and gate (T017–T018). The router commits per coherent step ("M6: …").
5. Done-when (§7): the human's tag build produces the three OS artifacts (T019–T020) and the human
   installs on one OS (T021).
