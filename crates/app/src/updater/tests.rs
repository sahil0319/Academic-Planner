//! Unit tests for the pure updater functions (contracts/app-updater.md). No test makes a
//! network request.

use std::path::Path;

use super::*;

fn v(text: &str) -> Version {
    Version::parse(text).unwrap()
}

const MAC_BUNDLE_EXE: &str = "/Applications/Academic Planner.app/Contents/MacOS/academic-planner";
const MAC_DEV_EXE: &str = "/Users/x/repo/target/debug/academic-planner";

/// R27, T33: a newer patch version is offered (0.1.10 is newer than 0.1.9, not older by text).
#[test]
fn t33_newer_patch_is_offered() {
    assert!(is_offered(&v("0.1.9"), &v("0.1.10")));
}

/// R27, T33: the same version is not offered.
#[test]
fn t33_same_version_not_offered() {
    assert!(!is_offered(&v("0.1.9"), &v("0.1.9")));
}

/// R27, T33: a newer pre-release is not offered.
#[test]
fn t33_prerelease_not_offered() {
    assert!(!is_offered(&v("0.1.9"), &v("0.2.0-beta")));
}

/// R27, T33: an older version is not offered.
#[test]
fn older_version_not_offered() {
    assert!(!is_offered(&v("0.1.9"), &v("0.1.8")));
}

/// R28: an app installed as a macOS bundle updates itself.
#[test]
fn install_kind_macos_bundle_is_self_install() {
    assert_eq!(
        install_kind(Os::MacOs, None, Path::new(MAC_BUNDLE_EXE)),
        InstallKind::SelfInstall
    );
}

/// R28: an unpackaged macOS dev build opens the release page.
#[test]
fn install_kind_macos_dev_build_opens_release_page() {
    assert_eq!(
        install_kind(Os::MacOs, None, Path::new(MAC_DEV_EXE)),
        InstallKind::OpenReleasePage
    );
}

/// R28: a Linux AppImage (the `APPIMAGE` variable is set) updates itself.
#[test]
fn install_kind_linux_appimage_is_self_install() {
    assert_eq!(
        install_kind(
            Os::Linux,
            Some(Path::new("/home/x/Academic.Planner.AppImage")),
            Path::new("/tmp/.mount_Academ/usr/bin/academic-planner")
        ),
        InstallKind::SelfInstall
    );
}

/// R28: a Linux install without `APPIMAGE` (a `.deb`) opens the release page.
#[test]
fn install_kind_linux_without_appimage_opens_release_page() {
    assert_eq!(
        install_kind(Os::Linux, None, Path::new("/usr/bin/academic-planner")),
        InstallKind::OpenReleasePage
    );
}

/// R28: a Windows install updates itself.
#[test]
fn install_kind_windows_is_self_install() {
    assert_eq!(
        install_kind(
            Os::Windows,
            None,
            Path::new(r"C:\Program Files\Academic Planner\academic-planner.exe")
        ),
        InstallKind::SelfInstall
    );
}

/// R28: the release page for a version is `{REPO_URL}/releases/tag/v{version}`.
#[test]
fn release_page_url_points_at_the_version_tag() {
    assert_eq!(
        release_page_url("0.2.0"),
        "https://github.com/sahil0319/Academic-Planner/releases/tag/v0.2.0"
    );
}

/// §13.1 UI: the notices link for a version is the release asset `THIRD-PARTY-NOTICES.html`.
#[test]
fn notices_url_points_at_the_release_asset() {
    assert_eq!(
        notices_url("0.2.0"),
        "https://github.com/sahil0319/Academic-Planner/releases/download/v0.2.0/THIRD-PARTY-NOTICES.html"
    );
}
