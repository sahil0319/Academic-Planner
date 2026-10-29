---

description: "Task list for M7 About page, updater, pre-migration backup"
---

# Tasks: M7 About Page, Updater, Pre-Migration Backup

**Input**: Design documents from `specs/007-m7-updater/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [quickstart.md](quickstart.md), [contracts/core-migrations.md](contracts/core-migrations.md), [contracts/core-settings.md](contracts/core-settings.md), [contracts/app-updater.md](contracts/app-updater.md), [contracts/ui-about.md](contracts/ui-about.md), [contracts/release-workflow.md](contracts/release-workflow.md)

**Tests**: Required. T31, T32 (core) and T33 (app) come from `docs/spec.md` §13.5; the setting,
backup-name, install-kind, and view-model tests come from the contracts. Test tasks are done by the
**tester** before the matching implementation task (constitution II). No test makes a network
request or touches the real user database.

**Start condition**: M6 is committed (it is: `19c2153`). One milestone at a time.

**Organization**: Phase 1 setup, Phase 2 foundations (stubs, error variant, the core setting), then
one phase per user story (US1–US3 P1, US4–US5 P2), then the gate and the human Done-when steps.

## Format: `[ID] [P?] [Story] (owner) Description`

- **[P]**: can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: user story from spec.md (US1–US5)
- **(tester)** / **(builder)** / **(human)**: who does the task. Human tasks handle the private
  key, secrets, tags, publishing, and installing; agents never do them.

## Path Conventions

New: `crates/core/src/migrate.rs`, `crates/core/src/migrate/tests.rs`,
`crates/core/tests/settings_updates.rs`, `crates/app/updater.pub`, `crates/app/src/updater.rs`,
`crates/app/src/updater/tests.rs`, `crates/app/src/pages/about.rs`, `crates/app/ui/about.slint`,
`scripts/updater-manifest.sh`. Edited: `crates/core/{Cargo.toml, src/lib.rs, src/db.rs,
src/error.rs, src/model.rs, src/backup.rs}`, existing core tests with `Settings` literals or
settings fixtures, `crates/app/{Cargo.toml, src/main.rs, src/controller.rs, src/viewmodel.rs,
src/viewmodel/tests.rs, src/pages/mod.rs, src/pages/settings.rs, ui/app-window.slint,
ui/types.slint}`, `.github/workflows/release.yml`, `README.md`, `Cargo.lock`. Not edited:
`crates/core/src/migrations/001_init.sql` (no new migration), `ci.yml`, `docs/spec.md`,
`DESIGN.md`, `.claude/state/` (router only).

---

## Phase 1: Setup

- [x] T001 (builder) Add dependencies: in crates/core/Cargo.toml a `[dev-dependencies]` table with `tempfile = "3"`; in crates/app/Cargo.toml `cargo-packager-updater = "0.2.3"` (default features, `rustls-tls`) and `open = "5"`. Run `cargo build --workspace` so Cargo.lock updates, then `cargo build --workspace --locked`. If a dependency needs a Rust newer than `rust-version = "1.90"`, stop and report (do not raise `rust-version`).
- [x] T002 (builder) License check (research D12): run `cargo about generate --workspace --locked --fail -o target/notices-check.html about.hbs` (install `cargo-about@0.9.3 --features cli` under `target/tools` if missing, as in M6). If it fails on a license not in about.toml `accepted`, stop and report the crate and license to the router; do not edit about.toml. Needs T001
- [x] T003 [P] (builder) Create crates/app/updater.pub containing exactly one line: `PLACEHOLDER-run-cargo-packager-signer-generate-see-specs-007-m7-updater-quickstart` (research D10: with the placeholder every download fails verification, so nothing installs).
- [ ] T004 [P] (human) Generate the signing key pair and secrets exactly as in quickstart.md §2: `cargo install cargo-packager --locked --version 0.11.8`; `mkdir -p ~/.academic-planner-signing`; `cargo packager signer generate --path ~/.academic-planner-signing/updater.key` (choose a password at the prompt); `gh secret set CARGO_PACKAGER_SIGN_PRIVATE_KEY -R sahil0319/Academic-Planner < ~/.academic-planner-signing/updater.key`; `gh secret set CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD -R sahil0319/Academic-Planner` (paste the password); then give the router the one line from `cat ~/.academic-planner-signing/updater.key.pub` (public, safe to share). Back up the private key and password outside the repo. Blocks T030 and US5's tag build only.

---

## Phase 2: Foundational (blocking)

- [ ] T005 (builder) In crates/core/src/error.rs add `Error::MigrationFailed { version: i64, at: i64, backup: Option<PathBuf>, #[source] source: rusqlite::Error }` with doc comment `/// R26: a migration failed; the database stays at version \`at\`.` and Display `"upgrading the database to schema version {version} failed; it was left at version {at}"` followed by `". A copy from before the upgrade is at {path}"` when `backup` is `Some` (implement `Display` by hand or with a helper so the optional sentence works).
- [ ] T006 (builder) Create crates/core/src/migrate.rs per contracts/core-migrations.md: move the migration loop out of `Planner::init` in crates/core/src/db.rs into `pub(crate) fn run(conn: &mut Connection, db_path: Option<&Path>, migrations: &[&str]) -> Result<()>` with behavior unchanged for now (newer-schema refusal still before any write, including the WAL switch; one transaction per migration with its `user_version` bump). `Planner::open` passes `Some(path)`, `open_in_memory` passes `None`; `SCHEMA_VERSION` still equals `MIGRATIONS.len()`. Add the stub `fn backup_file_name(from: i64, to: i64, existing: &[String]) -> String { todo!("M7: R26") }` (allow dead_code until T013), `#[cfg(test)] mod tests;`, an empty crates/core/src/migrate/tests.rs, and `mod migrate;` in crates/core/src/lib.rs. All existing core tests stay green. Needs T005
- [ ] T007 (builder) Add `pub check_updates_on_start: bool` (doc `/// §3, R29: check for an update once per launch. Default on.`) to `Settings` in crates/core/src/model.rs with default `true`. Make every full `Settings { … }` literal compile: crates/core/tests/{m3_backup_restore.rs, settings_theme.rs, m3_backup_export.rs, slots_settings.rs, m5_reminders.rs}, crates/core/src/db.rs (`settings()` returns the default for now), crates/app/src/viewmodel.rs (`parse_settings_form`: default value), crates/app/src/viewmodel/tests.rs. Do not change storage yet. Gate-relevant tests stay green.
- [ ] T008 (tester) Create crates/core/tests/settings_updates.rs per contracts/core-settings.md: `check_updates_fresh_db_is_on`; `check_updates_saved_off_reads_back_and_keeps_other_settings` (save all fields with `check_updates_on_start: false`, read back equal); `check_updates_export_contains_key` (after saving off, `export_json` has `settings.check_updates_on_start == "0"`); `check_updates_backup_without_key_restores_as_on` (save off, export, remove the key from the JSON, restore, read `true`); `check_updates_restore_rejects_bad_value` (values `"yes"` and `"true"` → `Error::Validation`, and the data before the restore is unchanged). Doc comments cite §3, §13.1 UI, R21, R22. Run `cargo test -p planner-core --test settings_updates` and note which fail (expected: saved-off, export, reject). Needs T007
- [ ] T009 (builder) Implement the setting per contracts/core-settings.md: `pub(crate) const SETTING_CHECK_UPDATES: &str = "check_updates_on_start";` in crates/core/src/db.rs; `settings()` reads `"1"` → true, `"0"` → false, missing → `true`, other → `Error::Corrupt`; `save_settings()` upserts `"1"`/`"0"` in its transaction; crates/core/src/backup.rs restore validation: if present the value must be exactly `"0"` or `"1"`, else the same rejection path as an invalid `theme`. Update expected `settings` objects in crates/core/tests/m3_backup_export.rs (lines near 285 and 475) to include `"check_updates_on_start"` with the saved value. T008 and all core tests pass. Needs T008
- [ ] T010 (builder) Keep the setting when other settings are saved: crates/app/src/pages/settings.rs `save_settings` keeps `current.theme` **and** `current.check_updates_on_start`; confirm crates/app/src/controller.rs `select_theme` builds from the current settings (`..settings`). Needs T009
- [ ] T011 (builder) App stubs so tests can compile: create crates/app/src/updater.rs with the constants and pure-function signatures of contracts/app-updater.md (`REPO_URL`, `MANIFEST_URL`, `PUBKEY` via `include_str!("../updater.pub")`, `RUNNING_VERSION`, `CHECK_TIMEOUT`, `is_offered`, `Os`, `InstallKind`, `install_kind`, `release_page_url`, `notices_url`), bodies `todo!("M7")`, `#[cfg(test)] mod tests;`, empty crates/app/src/updater/tests.rs, `mod updater;` in crates/app/src/main.rs. In crates/app/src/viewmodel.rs add `UpdatePhase`, `UpdateView`, and stubs for `update_view`, `version_text`, `license_text`, `banner_text`, `confirm_title`, `confirm_label`, `progress_fraction` (contracts/ui-about.md). Use `#[allow(dead_code)]` only on the new stub items until they are wired. Needs T001, T003

**Checkpoint**: core and app compile; existing tests green; the setting works in core.

---

## Phase 3: User Story 1 - Database upgrades leave a backup (Priority: P1)

**Goal**: R26 copy before migrating an existing DB; atomic migrations; error names the copy.

**Independent Test**: `cargo test -p planner-core t31_ t32_` passes.

- [ ] T012 [US1] (tester) Write crates/core/src/migrate/tests.rs per contracts/core-migrations.md "Tests": a helper that snapshots a DB file (`user_version`, all `sqlite_master` rows, every table's rows ordered by rowid), a `TEST_V2` SQL constant (e.g. `CREATE TABLE t31_probe (x INTEGER);`), and the tests `t31_upgrade_writes_pre_migrate_copy_equal_to_old_db`, `t31_reopen_current_db_makes_no_new_copy`, `t31_fresh_db_makes_no_copy`, `t31_existing_copy_name_gets_suffix`, `t32_failed_second_migration_leaves_db_at_v1` (second entry `"CREATE TABLE t32_partial (x); CREATE TABLE term (x);"`; asserts `MigrationFailed { version: 2, at: 1, backup: Some(p), .. }`, `user_version` 1, snapshot unchanged, no `t32_partial`, `err.to_string()` contains `p.display().to_string()`), and `backup_file_name` cases (base free; base taken → `-2`; base and `-2` taken → `-3`; unrelated names ignored). Each test uses its own `tempfile::TempDir`; the v1 DB is created through `run` with the 1-entry real list and gets a few rows (term, course, setting). Doc comments cite R25, R26, T31, T32. Run `cargo test -p planner-core migrate::` and confirm the new tests fail (todo or assertion) and nothing else breaks. Needs T006
- [ ] T013 [P] [US1] (tester) Append to crates/core/tests/migrations.rs: `t31_fresh_file_db_makes_no_backup` and `t31_current_file_db_reopen_makes_no_backup` (real list, `Planner::open` on a file in a `TempDir`, open twice; the `backups` folder is absent or empty). Confirm `t26_newer_schema_version_refused` still asserts no schema was written. Needs T006
- [ ] T014 [US1] (builder) Implement R26 in crates/core/src/migrate.rs per contracts/core-migrations.md "Behavior": `backup_file_name`; when `1 <= found < len` and `db_path` is `Some`, create `{parent}/backups`, list names, `VACUUM INTO` the chosen path (error → return before any migration); on a failed migration return `Error::MigrationFailed { version, at: version - 1, backup, source }`. Doc comments `/// R26: …` and `/// R25: …`. No clock. T012 and T013 pass; all core tests pass. Needs T012, T013
- [ ] T015 [US1] (tester) Add `error_text_names_pre_migrate_backup_path` to crates/app/src/viewmodel/tests.rs: `viewmodel::error_text(&Error::MigrationFailed { version: 2, at: 1, backup: Some("/x/backups/pre-migrate-v1-to-v2.sqlite".into()), source: <any rusqlite error> })` contains that path (it is shown on the existing startup error screen). Needs T014
- [ ] T016 [US1] (builder) Make T015 pass if it does not already (expected: `error_text` uses Display). `cargo clippy -p planner-core --all-targets -- -D warnings` clean. Needs T015

**Checkpoint**: US1 complete and independently tested.

---

## Phase 4: User Story 2 - Find out that an update exists (Priority: P1)

**Goal**: startup check (if on) + About "Check for updates" with all check states; banner.

**Independent Test**: `cargo test -p app t33_` and the update view-model tests pass; manual quickstart §4 (offline → "Couldn't check for updates").

- [ ] T017 [P] [US2] (tester) Write crates/app/src/updater/tests.rs per contracts/app-updater.md: `t33_newer_patch_is_offered` (0.1.9 vs 0.1.10 → true), `t33_same_version_not_offered` (0.1.9 vs 0.1.9 → false), `t33_prerelease_not_offered` (0.1.9 vs 0.2.0-beta → false), `older_version_not_offered`; `install_kind` cases: macOS `/Applications/Academic Planner.app/Contents/MacOS/academic-planner` → SelfInstall, macOS `/Users/x/repo/target/debug/academic-planner` → OpenReleasePage, Linux with `APPIMAGE` → SelfInstall, Linux without → OpenReleasePage, Windows → SelfInstall; `release_page_url("0.2.0")` = `https://github.com/sahil0319/Academic-Planner/releases/tag/v0.2.0`; `notices_url("0.2.0")` = `https://github.com/sahil0319/Academic-Planner/releases/download/v0.2.0/THIRD-PARTY-NOTICES.html`. Doc comments cite R27, R28, T33. Versions via `cargo_packager_updater::semver::Version::parse`. Confirm they fail on `todo!`. Needs T011
- [ ] T018 [P] [US2] (tester) Append view-model tests to crates/app/src/viewmodel/tests.rs for the exact strings in contracts/ui-about.md: `update_view` for every `UpdatePhase` (status, action label, `action_enabled` false for Checking/Downloading/Installing, `is_error` only for CheckFailed/InstallFailed, CheckFailed status exactly `"Couldn't check for updates"` and action `"Retry"`), `progress_fraction` (0 of 100 → 0.0; 42 of 100 → 0.42; 150 of 100 → 1.0; unknown total → -1.0), Downloading progress text (`"Downloading… 42%"`, unknown total `"Downloading…"`), `version_text("0.1.1")` = `"Version 0.1.1"`, `license_text()` = `"Free software under the GNU GPL v3 or later."`, `banner_text("0.2.0")` = `"Version 0.2.0 is available."`, `confirm_title("0.2.0")` = `"Install version 0.2.0?"`, `confirm_label(SelfInstall)` = `"Install and restart"`, `confirm_label(OpenReleasePage)` = `"Open release page"`. Confirm they fail on `todo!`. Needs T011
- [ ] T019 [US2] (builder) Implement in crates/app/src/updater.rs: `is_offered` (`remote.pre.is_empty() && remote > running`, doc `/// R27, T33`), `install_kind`, `release_page_url`, `notices_url`, and `check()` exactly as contracts/app-updater.md (one endpoint `MANIFEST_URL`, `pubkey: PUBKEY.trim()`, `WindowsConfig { install_mode: Some(WindowsUpdateInstallMode::Quiet), installer_args: None }`, `.version_comparator(|cur, rel| is_offered(&cur, &rel.version))`, `.timeout(CHECK_TIMEOUT)`). T017 passes. Needs T017
- [ ] T020 [US2] (builder) Implement the view-model functions and `UpdatePhase`/`UpdateView` in crates/app/src/viewmodel.rs exactly per the contracts/ui-about.md table. T018 passes. Needs T018
- [ ] T021 [US2] (builder) UI: create crates/app/ui/about.slint with `export global AboutState` (all properties and callbacks of contracts/ui-about.md) and `export component AboutPage` using `PageTitle "About"`, `SectionCard`, `PrimaryButton`/`SecondaryButton`, std `ProgressIndicator`, std `Switch`; `Theme.*` only, bindings only. In crates/app/ui/app-window.slint import/export them, add the sixth `NavItem "About"` (index 5) and `if AppState.current-page == 5: AboutPage { }`. In crates/app/ui/types.slint add to `AppState`: `in property <string> update-banner`, `callback update-banner-view()`, `callback update-banner-dismiss()`; show the existing `Banner` component with View and Dismiss buttons above the page content when `update-banner != ""`. Run the theme grep from quickstart §1. Needs T020
- [ ] T022 [US2] (builder) Create crates/app/src/pages/about.rs (and `pub mod about;` in crates/app/src/pages/mod.rs): a UI-thread `thread_local` holding phase, pending `Update`, banner version, dismissed flag (research D8); `refresh(app)` sets `AboutState` from `vm::update_view` and `AppState.update-banner` from `vm::banner_text` unless dismissed; `wire(app)`: `action()` in Idle/UpToDate/CheckFailed/InstallFailed starts a check on `std::thread::spawn` calling `updater::check()`, and posts the result back with `slint::invoke_from_event_loop` (only `Send` data + `slint::Weak<AppWindow>`): `Ok(Some(u))` → Available, `Ok(None)` → UpToDate, `Err(_)` → CheckFailed; banner View → page 5, Dismiss → hidden for this launch. Add `about::refresh` to `App::refresh_all` in crates/app/src/controller.rs. Needs T019, T021
- [ ] T023 [US2] (builder) Startup check (R29) in crates/app/src/main.rs: wire `pages::about::wire(&app)`; after the first `app.changed(true)`, if `app.planner().settings()` reads `check_updates_on_start == true`, start one background check flagged `startup` (a startup `Some` sets the banner; a startup error sets CheckFailed silently, no dialog). No check when the setting is off or the DB failed to open. Doc comments cite R27, R29, §1 Network. Needs T022

**Checkpoint**: About page checks for updates; startup banner works; offline shows Retry.

---

## Phase 5: User Story 3 - Install an update with one click (Priority: P1)

**Goal**: confirm panel → download with progress → signature verified → install → relaunch; `.deb`/dev opens the release page.

**Independent Test**: quickstart §5 on macOS (human). Agent-side: gate green; dev build Install opens the release page (quickstart §4).

- [ ] T024 [US3] (builder) In crates/app/src/updater.rs implement `download_and_install(update, on_progress) -> Result<Option<Command>, cargo_packager_updater::Error>` per contracts/app-updater.md: `update.download_extended(|chunk, total| …accumulate…, || {})` (signature verified inside; any error returns before install, R28), then `update.install(bytes)`; return `Some(open -n <extract_path>)` on macOS, `Some(<extract_path>)` for an AppImage, `None` on Windows (the updater exits the process; `/S /R` relaunches). Also `open_url(url) -> anyhow::Result<()>` via `open::that`. Doc comments cite R28, R30. Needs T019
- [ ] T025 [US3] (builder) Install flow in crates/app/src/pages/about.rs: `action()` in Available opens the confirm panel (`confirm-title` = `vm::confirm_title`, `confirm-notes` = `update.body` or empty, `confirm-label` = `vm::confirm_label(install_kind(current os, APPIMAGE env, current exe))`); `cancel()` closes it; `confirm()` with OpenReleasePage → `updater::open_url(release_page_url(version))`, close panel; with SelfInstall → phase Downloading, run `download_and_install` on a worker thread, post progress via `invoke_from_event_loop` only when the whole percent changes; `Ok(cmd)` → phase Installing, store `cmd` in the thread-local, `slint::quit_event_loop()`; `Err` → InstallFailed (Retry restarts with a check). The check/Install button is disabled while Checking/Downloading/Installing. Needs T022, T024
- [ ] T026 [US3] (builder) Relaunch in crates/app/src/main.rs: after `window.run()` returns, drop `_app` (closes the database) and the timer, then take the stored relaunch command from `pages::about` and spawn it; ignore a spawn error after logging it to stderr. Doc comment cites R28, R30 (the updater never touches user data; the new version's `open` applies R25/R26). Needs T025

**Checkpoint**: on a dev build, Install opens the release page; the self-install path compiles on all three OS targets (CI Linux + local macOS).

---

## Phase 6: User Story 4 - About page (Priority: P2)

**Goal**: all §13.1 About items and the startup switch.

**Independent Test**: quickstart §4 first two bullets.

- [ ] T027 [US4] (builder) Fill the static About items in crates/app/src/pages/about.rs `refresh`: `app-name` "Academic Planner", `version-text` = `vm::version_text(RUNNING_VERSION)`, `license-text` = `vm::license_text()`, `db-path` = `vm::db_path_text(app.db_path())`; `open-repo()` → `open_url(REPO_URL)`, `open-notices()` → `open_url(notices_url(RUNNING_VERSION))`; a failed open shows the error text through `app.show_message` (existing). Needs T022, T024
- [ ] T028 [US4] (builder) Startup switch: `check-on-start` shows `settings().check_updates_on_start`; `check-on-start-toggled()` saves at once with the current settings and only this field changed (§1 Saving), then refreshes; on error show the message and reload the switch. Needs T027, T010

**Checkpoint**: About page shows every §13.1 item; switch persists across restarts and is in backups.

---

## Phase 7: User Story 5 - Release builds publish signed updates (Priority: P2)

**Goal**: tag build signs the updater artifacts and attaches `latest.json`; release stays a draft.

**Independent Test**: quickstart §3 locally; quickstart §5 step 1 (human).

- [ ] T029 [P] [US5] (builder) Create scripts/updater-manifest.sh per contracts/release-workflow.md (`set -euo pipefail`; args `<tag> <dir> <notes-file>`; exactly one `*.app.tar.gz`, `*-setup.exe`, `*.AppImage` and each `.sig` required, else exit 1 with a message; JSON via `jq -n`; platforms `macos-aarch64`/`app`, `windows-x86_64`/`nsis`, `linux-x86_64`/`appimage`; URLs `https://github.com/sahil0319/Academic-Planner/releases/download/<tag>/<file>`; `pub_date` from `date -u +%Y-%m-%dT%H:%M:%SZ`). Validate as in quickstart §3 using dummy files in the scratchpad (3 platforms; a missing `.sig` exits 1). Make it executable.
- [ ] T030 [US5] (builder) Replace the placeholder in crates/app/updater.pub with the public key line from T004 (public only; if the router has not received it yet, leave this task open and continue). Needs T004
- [ ] T031 [US5] (builder) Edit .github/workflows/release.yml per contracts/release-workflow.md: a "Check signing secrets" step in `package` that fails with `::error::` if `CARGO_PACKAGER_SIGN_PRIVATE_KEY` or `CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD` is empty (values passed via `env`, never echoed); both secrets as env on the `Package` step; macOS `formats: app,dmg`, files `target/release/*.dmg`, `target/release/*.app.tar.gz`, `target/release/*.app.tar.gz.sig`; Windows add `target/release/*-setup.exe.sig`; Linux add `target/release/*.AppImage.sig`; in `release`, after download, rename files in `dist/` replacing spaces with `.`, then `bash scripts/updater-manifest.sh "$GITHUB_REF_NAME" dist "docs/releases/${GITHUB_REF_NAME}.md" > dist/latest.json`; keep `draft: true` and `files: dist/*`. Lint with `actionlint` if available, else a YAML parse. Needs T029
- [ ] T032 [P] [US5] (builder) README.md: a short "Updates" section: the app checks GitHub Releases for a newer published version (on startup if the About switch is on, or with "Check for updates"); nothing installs without a click; updates are signature-checked; `.deb` installs open the release page instead; this is the only network use (§1). Needs T031

**Checkpoint**: workflow and script ready; a tag build is a human step.

---

## Phase 8: Polish, gate, Done-when

- [ ] T033 (builder) Run the gate `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`, the theme grep, and `cargo test -p planner-core t31_`, `t32_`, `cargo test -p app t33_`; remove any leftover `#[allow(dead_code)]` from T006/T011; paste the summary. Needs T014, T016, T023, T026, T028, T031
- [ ] T034 (builder) Manual dev-build checks from quickstart §4 on macOS (About items and links, switch persists across restart, offline → "Couldn't check for updates" + Retry, Install on a dev build opens the release page); report results. Needs T033
- [ ] T035 (human) §7 Done-when, quickstart §5: release the first updater version (version number and release notes chosen at release time), install it on macOS by hand, release and publish a newer version, update the installed copy in-app, confirm the new version and that all data is present. Needs T004, T030, T033

---

## Dependencies & Execution Order

- Phase 1 → Phase 2 → stories. T004 (human) runs any time; it blocks only T030 and T035.
- US1 (T012–T016) needs only T005–T006; it is independent of the app.
- US2 (T017–T023) needs T009–T011. US3 needs US2's page and `check()`. US4 needs US2's page and
  T024's `open_url`. US5 is independent of the app code except T030 (public key).
- Within a story: tester task → run it failing → builder task.

## Parallel Examples

- After T011: T012 (core tests), T017 (updater tests), T018 (view-model tests) in parallel.
- T029 (manifest script) any time after Phase 1, in parallel with US1–US4.
- T013 alongside T012 (different files).

## Implementation Strategy

1. MVP = US1 (safe migrations) + US2 (know about updates): ship value even before signing exists.
2. Then US3 (install), US4 (About details), US5 (release workflow).
3. Gate (T033), manual checks (T034), human release run (T035).

## Owners

- **tester**: T008, T012, T013, T015, T017, T018
- **builder**: T001–T003, T005–T007, T009–T011, T014, T016, T019–T034
- **human**: T004, T035
