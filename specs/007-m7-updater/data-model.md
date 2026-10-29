# Data Model: M7

No schema change. `SCHEMA_VERSION` stays 1; no migration file is added (constitution IV).

## Stored (core)

### Setting row `check_updates_on_start` (§3)

| Field | Value |
| --- | --- |
| `setting.key` | `check_updates_on_start` |
| `setting.value` | `"1"` (on) or `"0"` (off) |
| Missing row | reads as on (`true`) |
| Other value | `Error::Corrupt` on read; rejected on restore (`Error::Validation`, nothing written) |
| JSON backup | `settings.check_updates_on_start` (present once settings were saved); absent restores as on |

### `Settings` (core `model.rs`)

Adds `check_updates_on_start: bool` (default `true`). All other fields unchanged.

### Pre-migration backup file (R26)

| Item | Value |
| --- | --- |
| Folder | `{parent of the DB file}/backups/` (created if missing; shared with R22 pre-restore copies) |
| Name | `pre-migrate-v{from}-to-v{to}.sqlite`; if taken, `pre-migrate-v{from}-to-v{to}-{n}.sqlite`, n = 2, 3, … |
| `from` | the DB's `user_version` before opening (≥ 1) |
| `to` | the app's `SCHEMA_VERSION` (length of the migration list) |
| Content | a `VACUUM INTO` copy: same `user_version`, schema, and rows as the DB before the open |
| Written when | `1 ≤ from < to`; never for a fresh DB (0), a current DB, a newer DB (refused), or `open_in_memory` |

### `Error::MigrationFailed` (core `error.rs`)

| Field | Meaning |
| --- | --- |
| `version: i64` | the schema version the failed migration was upgrading to |
| `at: i64` | the version the DB was left at (last good) |
| `backup: Option<PathBuf>` | the pre-migration copy (always `Some` when `at ≥ 1` before the open) |
| `source: rusqlite::Error` | the SQLite failure |

Display names the backup path (R26).

## Not stored (app, UI thread)

### Update phase (`viewmodel::UpdatePhase`)

```text
Idle ──check──▶ Checking ──▶ UpToDate
                    │  └───▶ Failed(Check) ──retry──▶ Checking
                    ▼
               Available{version, notes}
                    │ Install → confirm panel (Cancel → Available)
                    ▼ confirm
         ┌── kind = OpenReleasePage → open browser → Available
         └── kind = SelfInstall → Downloading{received, total?}
                                      ├─▶ Failed(Install) ──retry──▶ Checking
                                      └─▶ Installing → relaunch (app exits)
```

- `Failed(Check)`: "Couldn't check for updates" + Retry (R27).
- `Failed(Install)`: download, signature, or install error; installed app unchanged (R28) + Retry.
- The check button is disabled in `Checking`, `Downloading`, `Installing`.

### Banner (R29)

| Field | Meaning |
| --- | --- |
| `available: Option<version>` | set by the startup check only |
| `dismissed: bool` | set by Dismiss; lasts for this launch |

### Pending update

The `cargo_packager_updater::Update` returned by the last successful check, kept on the UI thread
until Install or the next check.

### Install kind (`updater::InstallKind`)

`SelfInstall` (macOS `.app`, Windows NSIS, Linux AppImage) or `OpenReleasePage` (Linux `.deb`, or
an unpackaged dev build). Decided by `install_kind(os, APPIMAGE, current exe)`.

## Release artifacts (not app data)

### `latest.json` (updater manifest, one per release)

```json
{
  "version": "0.2.0",
  "notes": "<contents of docs/releases/v0.2.0.md>",
  "pub_date": "2026-11-01T12:00:00Z",
  "platforms": {
    "macos-aarch64":  { "url": "…/v0.2.0/Academic.Planner.app.tar.gz", "signature": "<.sig text>", "format": "app" },
    "windows-x86_64": { "url": "…/v0.2.0/<name>-setup.exe",           "signature": "<.sig text>", "format": "nsis" },
    "linux-x86_64":   { "url": "…/v0.2.0/<name>.AppImage",            "signature": "<.sig text>", "format": "appimage" }
  }
}
```

`…` = `https://github.com/sahil0319/Academic-Planner/releases/download`. File names are the
packaged names with spaces replaced by `.`.
