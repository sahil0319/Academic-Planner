//! Updater: checks GitHub Releases for a newer signed release and installs it on request
//! (docs/spec.md §13.1, R27–R30). All network code of the app lives here (§1 Network).
//! Contract: `specs/007-m7-updater/contracts/app-updater.md`.

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use cargo_packager_updater::semver::Version;
use cargo_packager_updater::{
    Config, Update, UpdaterBuilder, WindowsConfig, WindowsUpdateInstallMode,
};

#[cfg(test)]
mod tests;

/// §13.1 UI: the project repository, opened from the About page.
pub const REPO_URL: &str = "https://github.com/sahil0319/Academic-Planner";

/// R27: GitHub resolves `latest` to the newest published, non-draft, non-pre-release release.
pub const MANIFEST_URL: &str =
    "https://github.com/sahil0319/Academic-Planner/releases/latest/download/latest.json";

/// R28: minisign public key, embedded at build time; used trimmed.
const PUBKEY: &str = include_str!("../updater.pub");

/// R27: the version of this build, compared against the release manifest.
pub const RUNNING_VERSION: &str = env!("CARGO_PKG_VERSION");

/// R27: an update check gives up after this long and reports "Couldn't check for updates".
const CHECK_TIMEOUT: Duration = Duration::from_secs(15);

/// The operating system the app runs on, as far as installing an update is concerned.
// `MacOs` is the contract's name (contracts/app-updater.md); clippy flags its `Os` suffix.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    MacOs,
    Windows,
    Linux,
}

/// R28: how "Install" behaves for this copy of the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallKind {
    /// Download, verify, install, and relaunch in place.
    SelfInstall,
    /// `.deb` installs and unpackaged dev builds open the release page instead.
    OpenReleasePage,
}

/// R27, T33: an update is offered only when it is newer by semver and not a pre-release.
pub fn is_offered(running: &Version, remote: &Version) -> bool {
    remote.pre.is_empty() && remote > running
}

/// R28: `.deb` (Linux without `APPIMAGE`) and unpackaged dev builds open the release page.
/// A macOS copy updates itself only when it runs from inside an `.app` bundle
/// (`<name>.app/Contents/MacOS/<exe>`); Windows installs always update themselves.
pub fn install_kind(os: Os, appimage: Option<&Path>, exe: &Path) -> InstallKind {
    let self_install = match os {
        Os::Windows => true,
        Os::Linux => appimage.is_some(),
        Os::MacOs => in_app_bundle(exe),
    };
    if self_install {
        InstallKind::SelfInstall
    } else {
        InstallKind::OpenReleasePage
    }
}

/// R28: true when `exe` is `<something>.app/Contents/MacOS/<exe>`.
fn in_app_bundle(exe: &Path) -> bool {
    let name = |p: Option<&Path>| {
        p.and_then(Path::file_name)
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    };
    let macos = exe.parent();
    let contents = macos.and_then(Path::parent);
    let bundle = contents.and_then(Path::parent);
    name(macos) == "MacOS" && name(contents) == "Contents" && name(bundle).ends_with(".app")
}

/// R28: the install kind of the running copy (OS, `APPIMAGE`, current executable).
pub fn current_install_kind() -> InstallKind {
    let os = if cfg!(target_os = "macos") {
        Os::MacOs
    } else if cfg!(windows) {
        Os::Windows
    } else {
        Os::Linux
    };
    let appimage = std::env::var_os("APPIMAGE").map(PathBuf::from);
    match std::env::current_exe() {
        Ok(exe) => install_kind(os, appimage.as_deref(), &exe),
        // Without a known executable path nothing can be replaced in place.
        Err(_) => InstallKind::OpenReleasePage,
    }
}

/// R28: `{REPO_URL}/releases/tag/v{version}`.
pub fn release_page_url(version: &str) -> String {
    format!("{REPO_URL}/releases/tag/v{version}")
}

/// §13.1 UI: `{REPO_URL}/releases/download/v{version}/THIRD-PARTY-NOTICES.html`.
pub fn notices_url(version: &str) -> String {
    format!("{REPO_URL}/releases/download/v{version}/THIRD-PARTY-NOTICES.html")
}

/// R27, §1 Network: asks the release manifest whether a newer published release exists.
/// Blocking; call it on a worker thread. `Ok(None)` means this copy is up to date.
pub fn check() -> Result<Option<Update>, cargo_packager_updater::Error> {
    let config = Config {
        endpoints: vec![MANIFEST_URL.parse()?],
        pubkey: PUBKEY.trim().into(),
        windows: Some(WindowsConfig {
            install_mode: Some(WindowsUpdateInstallMode::Quiet),
            installer_args: None,
        }),
    };
    UpdaterBuilder::new(RUNNING_VERSION.parse()?, config)
        .version_comparator(|current, release| is_offered(&current, &release.version))
        .timeout(CHECK_TIMEOUT)
        .build()?
        .check()
}

/// R28, R30: downloads `update` (reporting bytes received so far and the total size, when
/// known), verifies its signature against [`PUBKEY`] inside `cargo-packager-updater`, and only
/// then installs it over the app. Any error returns before anything is installed. The
/// updater only replaces the app itself; it never touches the database, its backups, or the
/// data folder (R30). Blocking; call it on a worker thread.
///
/// Returns the command that starts the new version (macOS: `open -n <bundle>`; AppImage: the
/// AppImage itself). On Windows the quiet installer exits this process and relaunches the app
/// itself, so `None` is returned there.
pub fn download_and_install(
    update: &Update,
    on_progress: impl Fn(u64, Option<u64>),
) -> Result<Option<Command>, cargo_packager_updater::Error> {
    let received = Cell::new(0_u64);
    let bytes = update.download_extended(
        |chunk, total| {
            received.set(received.get().saturating_add(chunk as u64));
            on_progress(received.get(), total);
        },
        || {},
    )?;
    update.install(bytes)?;
    Ok(relaunch_command(&update.extract_path))
}

/// R28: how to start the freshly installed version at `extract_path`.
#[cfg(target_os = "macos")]
fn relaunch_command(extract_path: &Path) -> Option<Command> {
    let mut command = Command::new("open");
    command.arg("-n").arg(extract_path);
    Some(command)
}

/// R28: on Windows the installer relaunches the app itself.
#[cfg(windows)]
fn relaunch_command(_extract_path: &Path) -> Option<Command> {
    None
}

/// R28: the AppImage path is the program to start.
#[cfg(not(any(windows, target_os = "macos")))]
fn relaunch_command(extract_path: &Path) -> Option<Command> {
    Some(Command::new(extract_path))
}

/// §13.1 UI, R28: opens `url` in the default browser (repository, notices, release page).
pub fn open_url(url: &str) -> anyhow::Result<()> {
    open::that(url).map_err(|e| anyhow::anyhow!("could not open {url}: {e}"))
}
