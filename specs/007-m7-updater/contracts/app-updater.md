# Contract: App updater (`crates/app/src/updater.rs`; R27–R30, T33)

All network code lives here (§1 Network). Core is not involved except that the relaunched app
opens the DB normally (R30).

## Constants

```rust
pub const REPO_URL: &str = "https://github.com/sahil0319/Academic-Planner";
/// R27: GitHub resolves `latest` to the newest published, non-draft, non-pre-release release.
pub const MANIFEST_URL: &str =
    "https://github.com/sahil0319/Academic-Planner/releases/latest/download/latest.json";
/// R28: minisign public key, embedded at build time (HD-1).
const PUBKEY: &str = include_str!("../updater.pub");   // used trimmed
pub const RUNNING_VERSION: &str = env!("CARGO_PKG_VERSION");
const CHECK_TIMEOUT: Duration = Duration::from_secs(15);
```

## Pure functions (unit-tested in this file)

```rust
/// R27, T33: offered only when newer by semver and not a pre-release.
pub fn is_offered(running: &Version, remote: &Version) -> bool;

pub enum InstallKind { SelfInstall, OpenReleasePage }
pub enum Os { MacOs, Windows, Linux }
/// R28: `.deb` (Linux without APPIMAGE) and unpackaged dev builds open the release page.
pub fn install_kind(os: Os, appimage: Option<&Path>, exe: &Path) -> InstallKind;

/// R28: `{REPO_URL}/releases/tag/v{version}`.
pub fn release_page_url(version: &str) -> String;
/// §13.1 UI: `{REPO_URL}/releases/download/v{version}/THIRD-PARTY-NOTICES.html`.
pub fn notices_url(version: &str) -> String;
```

Tests:

- `t33_newer_patch_is_offered` (0.1.9 vs 0.1.10 → true)
- `t33_same_version_not_offered` (0.1.9 vs 0.1.9 → false)
- `t33_prerelease_not_offered` (0.1.9 vs 0.2.0-beta → false)
- extra: older version not offered; `install_kind` cases (mac bundle path / mac dev path / linux
  with and without APPIMAGE / windows); URL builders.

## Effectful functions (no unit tests; never called by tests)

```rust
/// R27: blocking; call on a worker thread. Ok(None) = up to date.
pub fn check() -> Result<Option<Update>, cargo_packager_updater::Error>;
```

Builds `UpdaterBuilder::new(RUNNING_VERSION.parse()?, Config { endpoints: vec![MANIFEST_URL],
pubkey: PUBKEY.trim().into(), windows: Some(WindowsConfig { install_mode:
Some(WindowsUpdateInstallMode::Quiet), installer_args: None }) })`,
`.version_comparator(|cur, rel| is_offered(&cur, &rel.version))`, `.timeout(CHECK_TIMEOUT)`,
then `.build()?.check()`.

```rust
/// R28: blocking; worker thread. Downloads (progress callback), verifies the signature
/// (inside cargo-packager-updater), installs. Returns the relaunch command for macOS/AppImage;
/// on Windows the updater exits the process itself.
pub fn download_and_install(update: &Update, on_progress: impl Fn(u64, Option<u64>))
    -> Result<Option<std::process::Command>, cargo_packager_updater::Error>;

/// Opens a URL in the default browser (repo, notices, release page).
pub fn open_url(url: &str) -> anyhow::Result<()>;   // open::that
```

Relaunch command: macOS `open -n <update.extract_path>`; AppImage `<update.extract_path>`.

## Threading (research D8)

- Worker thread → UI thread only through `slint::invoke_from_event_loop` with `Send` data and a
  `slint::Weak<AppWindow>`.
- UI-thread state (phase, pending `Update`, banner, relaunch command) lives in a `thread_local`
  in `pages/about.rs`.
- `main.rs`: after `window.run()` returns, drop `_app` (closes the DB), then spawn the stored
  relaunch command if any.

## Startup (R29)

In `main.rs`, after the first `app.changed(true)`: if `settings().check_updates_on_start` is true,
start one background check flagged as `startup`. A startup result `Some(update)` sets the banner
and the About phase `Available`; `None` sets `UpToDate`; an error sets `Failed(Check)` silently
(no banner, no dialog). No check if the setting is off or the DB failed to open.

## Guarantees

- Never installs without the confirm click (R28, R29).
- Never touches the DB file, the backups folder, or the data folder (R30).
- No test performs a network request.
