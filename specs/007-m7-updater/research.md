# Research: M7 About Page, Updater, Pre-Migration Backup

Sources checked on 2026-10-29: `cargo-packager-updater` 0.2.3 source (crates.io, latest),
`cargo-packager` 0.11.8 source (the version `release.yml` installs), the current
`.github/workflows/release.yml`, `crates/core/src/db.rs`, `crates/app/src/restore.rs`.

## Facts about the tools (verified in source)

- `cargo_packager_updater::UpdaterBuilder::new(Version, Config)` with `Config { endpoints,
  pubkey, windows }`; `.version_comparator(Fn(Version, RemoteRelease) -> bool)`;
  `.timeout(Duration)`; `.build()?.check()? -> Option<Update>`. Without a comparator it offers
  any `remote > current`, which would offer `0.2.0-beta` over `0.1.9` (semver orders
  `0.2.0-beta > 0.1.9`), so T33 needs our comparator.
- The manifest JSON is either single-platform or `{"version","notes","pub_date","platforms":
  {"<os>-<arch>": {"url","signature","format"}}}`. The platform key uses **`macos`**, `linux`,
  `windows` (code: `get_updater_target`), not `darwin` as the README example shows. Formats:
  `app`, `appimage`, `nsis`, `wix`. A missing platform key is an error (`TargetNotFound`).
- `Update` has public `version`, `body` (notes), `download_url`, `format`, `extract_path`.
  `download_extended(on_chunk(len, total), on_finish)` downloads and **verifies the minisign
  signature** before returning bytes; `install(bytes)` installs. A bad or missing signature is an
  error from `download*`, before any file is touched.
- Install per OS: macOS extracts `.app.tar.gz` and swaps the bundle at `extract_path` (asks for
  admin with AppleScript if needed), then returns; Linux replaces the AppImage at `$APPIMAGE`
  (any other format is `UnsupportedUpdateFormat`), then returns; Windows NSIS starts the installer
  with the install-mode flags and calls `std::process::exit(0)`. `Quiet` mode passes `/S /R`
  (silent, relaunch). The updater does not relaunch on macOS or Linux.
- `cargo packager` signs **every** output when `CARGO_PACKAGER_SIGN_PRIVATE_KEY` (key text or
  path) and `CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD` are set, writing `<file>.sig`. A directory
  output (the `.app`) is first packed into `<name>.app.tar.gz`. With `--formats dmg` alone the
  `.app` is deleted after the DMG is made, so macOS must use `--formats app,dmg`.
- `cargo packager signer generate --path <file>` writes `<file>` (encrypted private key, base64)
  and `<file>.pub` (public key, base64) and prompts for the password. The `.pub` text is exactly
  what `Config.pubkey` expects.
- NSIS default install mode in cargo-packager is `CurrentUser` (no admin), so the updater's
  `Quiet` mode works.

## Decisions

### D1. Test-only migration list without widening the public API (HD-5)

- **Decision**: new private module `crates/core/src/migrate.rs` with
  `pub(crate) fn run(conn: &mut Connection, db_path: Option<&Path>, migrations: &[&str]) -> Result<()>`.
  `Planner::init` calls it with the real `MIGRATIONS`. T31/T32 are `#[cfg(test)]` unit tests in
  the same file that pass a 2-entry list (`[001_init.sql, test SQL]`). Nothing new is `pub`.
- **Rationale**: integration tests can only see public items; any public hook (even
  `#[doc(hidden)]` or behind a feature) is a test seam in the shipped API. A cargo feature also
  risks `cargo test --workspace` silently skipping the tests.
- **Alternatives**: `#[doc(hidden)] pub fn open_with_migrations` (leaks); `test-support` feature
  with a self dev-dependency (fragile, can be skipped); a global mutable migration list (unsafe,
  races between tests).

### D2. Files in tests

- **Decision**: T31/T32 and the real-list file tests use `tempfile::TempDir`; each test gets its
  own folder, removed at the end. Core gets `tempfile = "3"` as a dev-dependency only.
- **Rationale**: R26's copy lives next to the file. The constitution's "in-memory SQLite" rule
  exists to protect the real user DB; a fresh temp folder meets that intent. Recorded in plan
  Complexity Tracking.

### D3. Backup mechanics (R26)

- **Decision**: order inside `run`: (1) read `user_version`; if `> len` return `SchemaTooNew`
  (R25, no copy); (2) if `1 ≤ found < len` and `db_path` is a real file path, create
  `{parent}/backups`, pick the free name, `VACUUM INTO` it; any error here aborts the open before
  any migration (DB unchanged); (3) run each pending migration in its own transaction with its
  `user_version` bump (unchanged from today); on failure return `Error::MigrationFailed`.
  `open_in_memory` passes `db_path = None` (always fresh anyway).
- **Name choice**: pure `fn backup_file_name(from, to, existing: &[String]) -> String`: base
  `pre-migrate-v{from}-to-v{to}.sqlite`, else first free `pre-migrate-v{from}-to-v{to}-{n}.sqlite`
  for n = 2, 3, …. Unit-tested. Same shape as the app's `free_pre_restore_name` (R22) but a
  separate function, because core cannot depend on the app.
- **"to"** is the target `SCHEMA_VERSION` (the list length), so one copy per open, not one per
  step.
- **"equals the old DB"** in T31 means: same `user_version`, same `sqlite_master` rows, and same
  rows in every table. `VACUUM INTO` rewrites pages, so byte equality is not expected.
- **WAL**: `VACUUM INTO` on the open connection includes WAL content (as R22 already relies on).

### D4. Error that names the backup (R26)

- **Decision**: add
  `Error::MigrationFailed { version: i64, at: i64, backup: Option<PathBuf>, source: rusqlite::Error }`
  with message `"upgrading the database to schema version {version} failed; it was left at version
  {at}. A copy from before the upgrade is at {backup}"` (the last sentence is omitted when there
  is no backup, i.e. a fresh DB). The app shows it on the existing startup error screen through
  `viewmodel::error_text` (Display), so the path reaches the user.
- **Rationale**: R26 requires the error to name the path. A new variant keeps `Error::Db` meaning
  "any other SQLite error". This is the only public core addition besides the settings field.

### D5. `check_updates_on_start` setting (§3, §13.1 UI)

- **Decision**: `Settings.check_updates_on_start: bool`, default `true`. Stored as `"1"`/`"0"`
  like `notifications_enabled`; a missing row reads as `true`; another stored value is
  `Error::Corrupt`. `save_settings` writes it. JSON export carries it because export dumps all
  `setting` rows. Restore validates it like `theme`: if present it must be `"0"` or `"1"`, else
  the backup is rejected (R22: nothing written). A backup without the key restores with no row, so
  it reads as on.
- Callers that save settings keep the current value: the Settings page form (like `theme` today)
  and `select_theme` (already uses `..settings`).

### D6. Update source and offer decision (R27, HD-3, T33)

- **Decision**: one endpoint,
  `https://github.com/sahil0319/Academic-Planner/releases/latest/download/latest.json`.
  GitHub's `releases/latest` resolves only to the newest **published, non-draft, non-pre-release**
  release, which is exactly R27. Comparator: `is_offered(running, remote) = remote.pre.is_empty()
  && remote > running` (semver). Timeout 15 s.
- **T33**: `t33_*` unit tests in `updater.rs`: 0.1.9 vs 0.1.10 → offered; vs 0.1.9 → not; vs
  0.2.0-beta → not. Pure function, no network.
- Any error from `check()` (network, HTTP status, JSON, missing platform) maps to one state,
  "Couldn't check for updates", with Retry.
- **Alternatives**: GitHub REST API `/releases/latest` + asset lookup (more code, rate limits, a
  second JSON format); a self-hosted endpoint (no server, §0 goal).

### D7. Install kind and relaunch (R28, HD-4)

- **Decision**: pure `fn install_kind(os, appimage: Option<&Path>, exe: &Path) -> InstallKind`
  with `InstallKind::{SelfInstall, OpenReleasePage}`, unit-tested:
  - macOS: `SelfInstall` if the executable is inside `*.app/Contents/MacOS/`, else
    `OpenReleasePage` (dev build: `cargo run` must never replace `target/debug`).
  - Linux: `SelfInstall` if `APPIMAGE` is set, else `OpenReleasePage` (`.deb`).
  - Windows: `SelfInstall` (NSIS is the only Windows package).
- Release page: `https://github.com/sahil0319/Academic-Planner/releases/tag/v{version}`.
- **Relaunch**: macOS and AppImage: after `install` returns `Ok`, the UI thread records a relaunch
  command (macOS `open -n <extract_path>`; AppImage: run `<extract_path>`), quits the event loop;
  `main` drops the `App` (closing the database) and then spawns the command and exits. Windows:
  the updater starts the installer with `/S /R` (`WindowsUpdateInstallMode::Quiet`) and exits the
  process; the installer relaunches the app. All mutations are already committed (§1 Saving), so
  the exit loses nothing.
- **Note**: treating a dev build like `.deb` is not a user-facing rule; it only stops a developer
  build from overwriting its own build folder.

### D8. Threads (R27 "never blocks", R29 "background")

- **Decision**: `check` and `download+install` run on `std::thread::spawn`. Results and progress
  return to the UI thread with `slint::invoke_from_event_loop`, carrying only `Send` data (the
  `Update` value, numbers, strings) and a `slint::Weak<AppWindow>`. The pending `Update` and the
  update phase live in a UI-thread `thread_local` in `pages/about.rs` (same pattern as
  `bridge.rs`'s remembered theme), because `App` is `Rc` and not `Send`. Progress posts only when
  the whole percent changes.

### D9. Opening links

- **Decision**: `open = "5"` (`open::that`) for the repository link, the notices link, and the
  `.deb`/dev release page. MIT-licensed; uses `open`/`xdg-open`/`ShellExecute`.
- **Notices link**: `https://github.com/sahil0319/Academic-Planner/releases/download/v{version}/THIRD-PARTY-NOTICES.html`
  (the release job already attaches it). The bundled copy's location differs per format and is
  not resolvable for a dev build.
- **Alternatives**: hand-written `Command` per OS (Windows `start` quoting is error-prone).

### D10. Public key embedding (HD-1)

- **Decision**: commit the public key as `crates/app/updater.pub` (the `.pub` file's one line)
  and embed it with `include_str!` (trimmed). It is not secret. Until the human has generated the
  key pair, the file holds a clearly marked placeholder; with the placeholder every download fails
  verification, so nothing can be installed (R28: abort, nothing changed).
- **Alternatives**: `option_env!` at build time from a CI variable (dev builds would differ from
  release builds; easy to ship without a key).

### D11. Release workflow (§13.1 note)

- **Decision**:
  - A step at the start of `package` fails with a clear error if either secret is empty
    (no unsigned release; spec US5 scenario 2).
  - `Package` step env: `CARGO_PACKAGER_SIGN_PRIVATE_KEY: ${{ secrets.CARGO_PACKAGER_SIGN_PRIVATE_KEY }}`,
    `CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD: ${{ secrets.CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD }}`.
  - macOS `formats: app,dmg`; upload `*.dmg`, `*.app.tar.gz`, `*.app.tar.gz.sig`. Windows: add
    `*-setup.exe.sig`. Linux: add `*.AppImage.sig`. (`.dmg.sig`, `.deb.sig` are made but not
    uploaded.)
  - Release job: replace spaces in downloaded file names with `.` (GitHub does this to asset
    names anyway; doing it first keeps manifest URLs exact), then run
    `scripts/updater-manifest.sh <tag> dist docs/releases/<tag>.md > dist/latest.json`, which
    fails if any of the three artifacts or signatures is missing. Platforms: `macos-aarch64`
    (`app`), `windows-x86_64` (`nsis`), `linux-x86_64` (`appimage`). URLs:
    `https://github.com/sahil0319/Academic-Planner/releases/download/<tag>/<file>`.
  - `draft: true` unchanged (HD-3).
- **Secret names**: `CARGO_PACKAGER_SIGN_PRIVATE_KEY` and `CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD`
  (the env var names `cargo packager` reads, so no mapping is needed).

### D12. License notices

- **Decision**: after adding the dependencies, run `cargo about generate` locally. If a new
  license expression is not covered by `about.toml`'s `accepted` list, stop and ask the human
  (license policy is a human decision); do not edit `accepted` silently.

### D13. Update banner (R29)

- **Decision**: shown above the page content on every page when the startup check found an
  update and the user has not dismissed it this launch. Text "Version {v} is available." with
  "View" (opens About) and "Dismiss". Not stored. A manual check on About does not re-show a
  dismissed banner.

### D14. Versions for the §7 test

- The §7 check needs two published releases: the first one with the updater, installed by hand
  on macOS, then a newer one that the installed copy updates to. The version numbers are a release
  decision for the router/human at that time (suggestion: `0.2.0` then `0.2.1`); they do not
  change any rule.
