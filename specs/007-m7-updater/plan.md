# Implementation Plan: M7 About Page, Updater, Pre-Migration Backup

**Branch**: `007-m7-updater` (spec directory; work happens on the current branch) | **Date**: 2026-10-29 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/007-m7-updater/spec.md`

## Summary

Three parts, in this order:

1. **Core, pre-migration backup (R26, R25, R30; T31, T32).** Move the migration loop out of
   `Planner::init` into a private module `crates/core/src/migrate.rs` that takes the migration
   list as a parameter. Before the first pending migration of an existing database
   (`1 ≤ user_version < len`), it writes a `VACUUM INTO` copy to
   `{db_dir}/backups/pre-migrate-v{from}-to-v{to}[-N].sqlite`. A failed migration returns a new
   `Error::MigrationFailed` whose message names the backup path. The test-only migration list
   lives in `#[cfg(test)]` unit tests inside `migrate.rs`, so no new public item is needed for it
   (research D1). Add the `check_updates_on_start` setting to `Settings` (default on, missing row
   reads as on, kept in backups, validated on restore like `theme`). No new migration;
   `SCHEMA_VERSION` stays 1.
2. **App, updater and About page (R27–R30; T33).** A new `crates/app/src/updater.rs` wraps
   `cargo-packager-updater` 0.2.3: it reads `latest.json` from the latest published GitHub
   release, decides "offered" with a pure, unit-tested function (T33: newer by semver and not a
   pre-release), downloads with progress on a worker thread, verifies the minisign signature
   against the embedded public key (`crates/app/updater.pub`), installs per format, and relaunches.
   A `.deb` install (Linux, no `APPIMAGE`) and an unpackaged dev build open the release page
   instead. A sixth page, About (`ui/about.slint`, `pages/about.rs`), shows the §13.1 items, the
   update states, a confirm panel with version and notes, and the startup switch. A dismissible
   banner above every page appears when the startup check finds an update. All text comes from
   `viewmodel.rs` functions with unit tests.
3. **Release workflow (§13.1 note, HD-1, HD-3).** `release.yml` passes the two signing secrets to
   `cargo packager`, builds macOS with `--formats app,dmg` so the `.app.tar.gz` is produced and
   signed, collects the three updater artifacts and their `.sig` files, writes `latest.json` with
   a script, and still creates a **draft** release. The human generates the key pair and creates
   the two secrets (one task gives the exact commands).

`docs/spec.md` §1–§5 and §13.1 are the plan: nothing in §1 is re-decided (Network row: only the
updater uses the network; Updates row: `cargo-packager-updater` + minisign; Clock row: core reads
no clock). No rule, schema, or §5 method changes.

## Technical Context

**Language/Version**: Rust stable, edition 2021, `rust-version = "1.90"`, resolver 3, workspace
version `0.1.1`, license `GPL-3.0-or-later`.

**Primary Dependencies**:

- core: unchanged runtime dependencies; new dev-dependency `tempfile = "3"` (T31/T32 need a
  database file, research D2).
- app: `cargo-packager-updater = "0.2.3"` (default feature `rustls-tls`; re-exports `semver`,
  `url`, `reqwest`, so no separate `semver` dependency), `open = "5"` to open links and the
  release page in the browser (research D9).
- tools: `cargo-packager` 0.11.8 (already used by `release.yml`; its `signer generate` makes the
  key pair; `CARGO_PACKAGER_SIGN_PRIVATE_KEY` / `CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD` make it
  sign every output), `jq` (preinstalled on GitHub runners) for the manifest.

**Storage**: SQLite, schema v1 unchanged. New `setting` row `check_updates_on_start` (`1`/`0`).
Backups folder `{db_dir}/backups/` (already used by R22 pre-restore copies).

**Testing**: core unit tests `t31_*`, `t32_*` in `crates/core/src/migrate.rs` (temporary
directory, never the user DB); core integration tests for the setting in
`crates/core/tests/settings_updates.rs` and the real-list no-copy cases in
`crates/core/tests/migrations.rs`; app unit tests `t33_*` in `crates/app/src/updater.rs` and
About/update view-model tests in `crates/app/src/viewmodel/tests.rs`. No test touches the network.
Manual: the §7 macOS update run ([quickstart.md](quickstart.md)).

**Target Platform**: macOS Apple Silicon (`.app`), Windows x64 (NSIS, per-user install), Linux
x86_64 (AppImage self-installs; `.deb` opens the release page).

**Project Type**: desktop app (Rust workspace: `planner-core` library + `app` binary).

**Performance Goals**: the startup check never delays the first frame (worker thread); the check
request has a 15-second timeout (research D6).

**Constraints**: core has no network, Slint, clock, or dialogs; network code only in the app
crate; `.slint` uses `Theme.*` only and holds no logic; formatting in `viewmodel.rs` with tests;
no new migration; the private key never enters the repo or agent context; releases stay drafts.

**Scale/Scope**: about 1 new core module, 1 error variant, 1 settings field; 2 new app modules,
1 new `.slint` page; 1 workflow edit plus 1 manifest script.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| I. Spec is source of truth | PASS | Scope is §13.1 + T31–T33. The human decisions HD-1 to HD-5 are recorded in spec.md. `Error::MigrationFailed` and `Settings::check_updates_on_start` are the minimum the §13.1 text requires (the error must name the backup path; the switch is a saved setting); no §5 signature changes. Planner decisions are in research.md. |
| II. Test-first, traceable | PASS (justified exception) | T31/T32/T33 are written before the code (tasks order). T31/T32 must use a database **file** because R26 writes a copy next to the file; they use a fresh temporary directory, never the real user DB. See Complexity Tracking. |
| III. Pure, clock-free core | PASS | Backup names use `-2`, `-3` suffixes, not time. No network in core. The updater, threads, and browser opening are in the app. The `.slint` page has bindings only; all strings come from `viewmodel.rs`. |
| IV. Append-only migrations | PASS | No migration added or edited; `001_init.sql` untouched. The setting is a row. |
| V. Gate before done | PASS | Gate runs at the end; §7 adds the manual macOS update run. |

Post-design re-check (after Phase 1): unchanged, PASS.

## Project Structure

### Documentation (this feature)

```text
specs/007-m7-updater/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── core-migrations.md
│   ├── core-settings.md
│   ├── app-updater.md
│   ├── ui-about.md
│   └── release-workflow.md
├── checklists/requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/core/
├── Cargo.toml                 # + [dev-dependencies] tempfile
├── src/
│   ├── lib.rs                 # + mod migrate
│   ├── migrate.rs             # NEW: run(conn, path, migrations), backup naming, t31/t32 unit tests
│   ├── db.rs                  # init() delegates to migrate; settings read/save new key
│   ├── error.rs               # + Error::MigrationFailed
│   ├── model.rs               # Settings + check_updates_on_start
│   └── backup.rs              # restore validates check_updates_on_start
└── tests/
    ├── migrations.rs          # + real list: fresh/current DB file makes no backup
    ├── settings_updates.rs    # NEW: setting default, save, export, restore without key
    └── (existing Settings literals and settings-map fixtures updated)

crates/app/
├── Cargo.toml                 # + cargo-packager-updater, open
├── updater.pub                # NEW: minisign public key (base64 text), embedded with include_str!
├── src/
│   ├── main.rs                # wire About, startup check, relaunch after the event loop ends
│   ├── updater.rs             # NEW: manifest URL, is_offered (T33), install_kind, check/download/install
│   ├── pages/about.rs         # NEW: refresh + wire About page and banner
│   ├── pages/mod.rs           # + about
│   ├── pages/settings.rs      # save keeps check_updates_on_start
│   ├── controller.rs          # select_theme keeps check_updates_on_start (uses ..current); refresh_all includes about
│   ├── viewmodel.rs           # + update/About text functions
│   └── viewmodel/tests.rs     # + tests
└── ui/
    ├── about.slint            # NEW: AboutPage + AboutState global
    ├── app-window.slint       # 6th NavItem "About", update banner
    └── types.slint            # AppState: banner properties/callbacks

.github/workflows/release.yml  # signing env, app,dmg, updater files, manifest, still draft
scripts/updater-manifest.sh    # NEW: writes latest.json from the downloaded artifacts
README.md                      # short "Updates" note (where updates come from, .deb behavior)
```

**Structure Decision**: existing two-crate workspace. Core changes stay private except one error
variant and one settings field. All network code is in `crates/app/src/updater.rs`.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
| --- | --- | --- |
| T31/T32 use a database file in a temp dir, not in-memory SQLite | R26 writes `{db_dir}/backups/…` next to the database file; an in-memory DB has no directory and `VACUUM INTO` must produce a file to compare | Shared-cache in-memory URIs have no real parent directory, so the R26 path could not be checked |
| T31/T32 are unit tests in `src/migrate.rs`, not files in `crates/core/tests/` | The test-only migration list must reach the private runner; integration tests can only call the public API | A `#[doc(hidden)] pub` or feature-gated `open_with_migrations` would put a test hook in the public API (HD-5 asks to avoid that); a self dev-dependency with a feature is fragile and would let `cargo test` silently skip the tests if the feature were off |
