//! Updater: checks GitHub Releases for a newer signed release and installs it on request
//! (docs/spec.md §13.1, R27–R30). All network code of the app lives here (§1 Network).
//! Contract: `specs/007-m7-updater/contracts/app-updater.md`.

use std::path::Path;
use std::time::Duration;

use cargo_packager_updater::semver::Version;

#[cfg(test)]
mod tests;

/// §13.1 UI: the project repository, opened from the About page.
#[allow(dead_code)]
pub const REPO_URL: &str = "https://github.com/sahil0319/Academic-Planner";

/// R27: GitHub resolves `latest` to the newest published, non-draft, non-pre-release release.
#[allow(dead_code)]
pub const MANIFEST_URL: &str =
    "https://github.com/sahil0319/Academic-Planner/releases/latest/download/latest.json";

/// R28: minisign public key, embedded at build time; used trimmed.
#[allow(dead_code)]
const PUBKEY: &str = include_str!("../updater.pub");

/// R27: the version of this build, compared against the release manifest.
#[allow(dead_code)]
pub const RUNNING_VERSION: &str = env!("CARGO_PKG_VERSION");

/// R27: an update check gives up after this long and reports "Couldn't check for updates".
#[allow(dead_code)]
const CHECK_TIMEOUT: Duration = Duration::from_secs(15);

/// The operating system the app runs on, as far as installing an update is concerned.
// `MacOs` is the contract's name (contracts/app-updater.md); clippy flags its `Os` suffix.
#[allow(dead_code, clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    MacOs,
    Windows,
    Linux,
}

/// R28: how "Install" behaves for this copy of the app.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallKind {
    /// Download, verify, install, and relaunch in place.
    SelfInstall,
    /// `.deb` installs and unpackaged dev builds open the release page instead.
    OpenReleasePage,
}

/// R27, T33: an update is offered only when it is newer by semver and not a pre-release.
#[allow(dead_code)]
pub fn is_offered(running: &Version, remote: &Version) -> bool {
    let _ = (running, remote);
    todo!("M7")
}

/// R28: `.deb` (Linux without `APPIMAGE`) and unpackaged dev builds open the release page.
#[allow(dead_code)]
pub fn install_kind(os: Os, appimage: Option<&Path>, exe: &Path) -> InstallKind {
    let _ = (os, appimage, exe);
    todo!("M7")
}

/// R28: `{REPO_URL}/releases/tag/v{version}`.
#[allow(dead_code)]
pub fn release_page_url(version: &str) -> String {
    let _ = version;
    todo!("M7")
}

/// §13.1 UI: `{REPO_URL}/releases/download/v{version}/THIRD-PARTY-NOTICES.html`.
#[allow(dead_code)]
pub fn notices_url(version: &str) -> String {
    let _ = version;
    todo!("M7")
}
