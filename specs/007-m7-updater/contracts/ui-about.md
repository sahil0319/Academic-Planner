# Contract: About page and update banner (§13.1 UI, §6, R27–R29)

Visual rules: DESIGN.md and `Theme.*` only; no hex or px outside `theme.slint`; reuse existing
components (`PageTitle`, `SectionCard`, `PrimaryButton`, `SecondaryButton`, `Banner`, std
`Switch`, std `ProgressIndicator`). No design skill is loaded (§12.0).

## Navigation

- `app-window.slint`: sixth `NavItem` "About", page index 5, after Settings;
  `if AppState.current-page == 5: AboutPage { }`.

## `AboutState` global (`ui/about.slint`)

| Property / callback | Dir | Source |
| --- | --- | --- |
| `app-name` | in | "Academic Planner" |
| `version-text` | in | `vm::version_text(RUNNING_VERSION)` → "Version 0.1.1" |
| `license-text` | in | `vm::license_text()` → "Free software under the GNU GPL v3 or later." |
| `db-path` | in | `vm::db_path_text` (existing) |
| `status-text` | in | `vm::update_view(phase).status` |
| `action-label` | in | `vm::update_view(phase).action` ("Check for updates" / "Retry" / "Install") |
| `action-enabled` | in | false while checking / downloading / installing |
| `downloading` | in | phase is Downloading |
| `progress` | in | 0.0–1.0, or −1 when total is unknown |
| `progress-text` | in | "Downloading… 42%" / "Downloading…" |
| `is-error` | in | phase is Failed (status shown in `Theme.error`, always with text) |
| `confirm-open` | in | confirm panel visible |
| `confirm-title` | in | "Install version 0.2.0?" |
| `confirm-notes` | in | release notes (plain text, wrapped) |
| `confirm-label` | in | "Install and restart" / "Open release page" (by `InstallKind`) |
| `check-on-start` | in-out | `Settings.check_updates_on_start` |
| `action()` | cb | Check / Retry → start check; Install → open confirm panel |
| `confirm()` / `cancel()` | cb | confirm panel buttons |
| `check-on-start-toggled()` | cb | saves the setting at once (§1 Saving) |
| `open-repo()` / `open-notices()` | cb | `updater::open_url` |

## Banner (`types.slint` `AppState`, `app-window.slint`)

| Property / callback | Dir | Meaning |
| --- | --- | --- |
| `update-banner` | in | text, "" = hidden; `vm::banner_text(version)` → "Version 0.2.0 is available." |
| `update-banner-view()` | cb | go to page 5 (About) |
| `update-banner-dismiss()` | cb | hide for this launch |

Shown above the page content in the content column, below the title row area, on every page.

## View-model functions (`viewmodel.rs`, tests in `viewmodel/tests.rs`)

```rust
pub enum UpdatePhase { Idle, Checking, UpToDate, Available { version: String },
    Downloading { received: u64, total: Option<u64> }, Installing, CheckFailed, InstallFailed }
pub struct UpdateView { pub status: String, pub action: String, pub action_enabled: bool,
    pub progress: f32, pub progress_text: String, pub is_error: bool }
pub fn update_view(phase: &UpdatePhase) -> UpdateView;
pub fn version_text(version: &str) -> String;
pub fn license_text() -> String;
pub fn banner_text(version: &str) -> String;
pub fn confirm_title(version: &str) -> String;
pub fn confirm_label(kind: InstallKind) -> String;
pub fn progress_fraction(received: u64, total: Option<u64>) -> f32;   // clamp 0..=1, -1 unknown
```

Exact status strings:

| Phase | status | action |
| --- | --- | --- |
| Idle | "" | Check for updates |
| Checking | "Checking for updates…" | Check for updates (disabled) |
| UpToDate | "You're up to date." | Check for updates |
| Available | "Version {v} is available." | Install |
| Downloading | "Downloading…" (+ progress text) | Install (disabled) |
| Installing | "Installing…" | Install (disabled) |
| CheckFailed | "Couldn't check for updates" (R27) | Retry |
| InstallFailed | "The update couldn't be installed. Nothing was changed." | Retry |

## Wiring (`pages/about.rs`)

- `refresh(app)`: version, license, db path, setting switch, current phase view, banner.
- `wire(app)`: callbacks above. `App::refresh_all` calls `about::refresh`.
- On `confirm()` with `OpenReleasePage`: `open_url(release_page_url(version))`, close panel.
- On `confirm()` with `SelfInstall`: phase Downloading, start worker; progress → phase updates;
  success → Installing, store relaunch command, `slint::quit_event_loop()`; error → InstallFailed.
