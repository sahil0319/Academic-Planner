# Data Model: M6 Release Packaging

M6 adds no database tables, columns, or migrations (§3 unchanged). The only "entities" are the
files a tag build produces.

## Release (per tag)

| Field | Value |
| --- | --- |
| Tag | `v` + workspace version, e.g. `v0.1.0` (FR-003) |
| GitHub release | one draft, named after the tag (FR-002) |
| Assets | the five files below |

## Release artifacts

| Artifact | Built on | Packager format | Example name (0.1.0) | Contains notices |
| --- | --- | --- | --- | --- |
| macOS disk image | `macos-latest` (Apple Silicon) | `dmg` | `Academic Planner_0.1.0_aarch64.dmg` | yes, `Academic Planner.app/Contents/Resources/` |
| Windows installer | `windows-latest` | `nsis` | `academic-planner_0.1.0_x64-setup.exe` | yes, next to `academic-planner.exe` |
| Debian package | `ubuntu-22.04` | `deb` | `academic-planner_0.1.0_amd64.deb` | yes, under `usr/lib/` |
| AppImage | `ubuntu-22.04` | `appimage` | `academic-planner_0.1.0_x86_64.AppImage` | yes, inside the image |
| Notices | `ubuntu-22.04` (`prepare` job) | n/a (`cargo about`) | `THIRD-PARTY-NOTICES.html` | is the notices file |

Example names follow the packager's default pattern; the exact casing is whatever
`cargo-packager` 0.11.8 emits (research R-1) and is not renamed.

Each installer also carries `LICENSE` and `NOTICE` as resources next to the notices (research R-8).

## Validation rules

- Tag ≠ `v{version}` → the `prepare` job fails before any packaging and names both versions.
- A dependency license outside the `about.toml` accepted list → `cargo about generate` fails and
  names the crate; no artifacts are built.
- Any matrix job finds no installer to upload → that job fails (`if-no-files-found: error`), and the
  `release` job does not run.

## Installed app data (unchanged)

The database stays at the R24 location (research R-9) or `ACADEMIC_PLANNER_DB`; the installers never
write a database into the install folder (US2 scenario 4).
