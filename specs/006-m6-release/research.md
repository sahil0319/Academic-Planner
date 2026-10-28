# Research: M6 Release Packaging

All entries were checked on 2026-10-29 against crates.io, the tools' tagged sources, and the GitHub
API. Nothing here changes a rule, the schema, or the §5 API. "Planner decision" marks a naive choice
made under the human's overnight delegation.

## R-1 cargo-packager version and config location

- **Decision**: `cargo-packager` **0.11.8** (2025-11-27, latest). Install with
  `cargo install cargo-packager --locked --version 0.11.8` (no prebuilt binaries are published and
  `taiki-e/install-action` has no manifest for it). Verified: installs and builds on Rust 1.90.0.
  Config goes in `crates/app/Cargo.toml` under `[package.metadata.packager]`.
- **Rationale**: In Cargo-metadata mode the packager fills `version` (0.1.0 from the workspace), the
  binary list (`academic-planner`, the app crate's only bin, marked main), `out-dir` and
  `binaries-dir` (`target/release` with `--release`). It then changes directory to `crates/app/`, so
  every relative path in the config (resources, icons, license file) is relative to `crates/app/`,
  and `before-packaging-command` runs there too.
- **Formats are passed on the command line, not in the config**: the packager does not filter
  formats by host OS (`nsis` has no OS guard), so a shared `formats` list would try to build NSIS on
  macOS. Each matrix entry passes `--formats dmg`, `--formats nsis`, or `--formats deb,appimage`.
- **Outputs** (in `target/release/`, from the 0.11.8 sources): DMG `{product-name}_{version}_{arch}.dmg`
  (`Academic Planner_0.1.0_aarch64.dmg`); NSIS `{bin}_{version}_{arch}-setup.exe`
  (`academic-planner_0.1.0_x64-setup.exe`); `{bin}_{version}_{arch}.deb`;
  `{bin}_{version}_{arch}.AppImage`. Planner decision: keep the packager's default file names.
- **CI quirks handled by the packager**: when `CI` is set, the DMG step passes `--skip-jenkins` (no
  Finder AppleScript on runners). NSIS and linuxdeploy are downloaded by the packager on first use.
- **Alternatives**: `Packager.toml` at the root (rejected: loses Cargo metadata auto-fill);
  `cargo-bundle` (rejected: no NSIS, less maintained); listing `formats` in config (rejected above).

## R-2 cargo-about version, policy, and template

- **Decision**: `cargo-about` **0.9.3** (2026-10-26, MSRV 1.88). CI installs the prebuilt binary
  with `taiki-e/install-action` (manifest covers 0.9.3 for x86_64 Linux musl). Local install:
  `cargo install cargo-about --locked --version 0.9.3 --features cli` (the binary needs the `cli`
  feature; verified on Rust 1.90.0).
- **Command**: `cargo about generate --workspace --locked --fail -o THIRD-PARTY-NOTICES.html about.hbs`
  from the repo root (`about.toml` is read from the workspace root by default). `--fail` and the
  default "unsatisfied license is an error" behavior make the job fail and name the crate (spec edge
  case "License not allowed by policy").
- **Accepted list** (priority order), derived from a trial run on the current `Cargo.lock` (including
  M5's `notify-rust`), which succeeded with exactly these:
  `MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `BSL-1.0`, `Unicode-3.0`,
  `MPL-2.0`, `NCSA`, `GPL-3.0-only`, `GPL-3.0-or-later`. Plus `ignore-dev-dependencies = true`.
  Trial result: 12 license groups, about 600 crate entries, a 450 KB HTML file.
- **Slint**: its expression `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR
  LicenseRef-Slint-Software-3.0` resolves to `GPL-3.0-only`, matching `NOTICE` (§1 License row).
- **Workspace crates**: `app` and `planner-core` (`GPL-3.0-or-later`) are included with
  `--workspace`; accepting `GPL-3.0-or-later` is simpler than marking them `publish = false` and
  setting `private.ignore`. Planner decision.
- **No `targets` filter**: notices are generated once on Linux and cover the crates of all three
  OSes, so one file is correct for every installer (SC-003).
- **Template**: Planner decision: HTML (`about.hbs`), minimal markup and no inline styles, adapted
  from cargo-about's example. Output name `THIRD-PARTY-NOTICES.html`; generated, not committed
  (added to `.gitignore`) so it never drifts from `Cargo.lock`.
- **Alternatives**: plain-text output (rejected: Handlebars HTML-escapes `{{text}}`, so text output
  needs triple braces and loses the overview links); committing the generated file (rejected: drift).

## R-3 Release workflow shape

- **Decision**: three jobs in `.github/workflows/release.yml`, `on: push: tags: ['v*']` only.
  1. `prepare` (`ubuntu-22.04`): version check (FR-003), generate notices, upload them as artifact
     `third-party-notices`.
  2. `package` (`needs: prepare`, matrix `macos-latest` / `windows-latest` / `ubuntu-22.04`,
     `fail-fast: false`): download notices to the repo root, Linux apt deps, toolchain, cache,
     install `cargo-packager`, `cargo packager -p app --release --formats <matrix.formats>`, upload
     the installers as artifacts with `if-no-files-found: error`.
  3. `release` (`needs: package`, `ubuntu-22.04`, `permissions: contents: write`): download all
     artifacts merged into `dist/`, create one draft release with `softprops/action-gh-release`
     (`draft: true`, `fail_on_unmatched_files: true`).
- **Rationale**: a single release job avoids the known race where parallel matrix jobs each create
  their own draft (drafts are not found by tag lookup). Generating notices once guarantees identical
  notices in every installer. Workflow-level `permissions: contents: read`; only `release` writes.
- **Version check**: `cargo metadata --no-deps --format-version 1` + `jq` reads the `app` package
  version; if `$GITHUB_REF_NAME != "v$version"` it prints
  `::error::Tag <tag> does not match workspace version <version> (expected v<version>)` and exits 1.
  It runs in `prepare`, so nothing is packaged on mismatch.
- **Alternatives**: one matrix job with per-job uploads to the release (rejected: duplicate drafts);
  `cargo install cargo-about` in every job (rejected: slow, prebuilt exists).

## R-4 Action and runner pins

| Item | Pin | Note |
| --- | --- | --- |
| `actions/checkout` | `@v7` | current major, Node 24 (ci.yml keeps its own `@v4`, unchanged by FR-006) |
| `actions/upload-artifact` | `@v7` | current major |
| `actions/download-artifact` | `@v8` | current major; `merge-multiple: true` in `release` |
| `dtolnay/rust-toolchain` | `@stable` | same as ci.yml |
| `Swatinem/rust-cache` | `@v2` | same as ci.yml |
| `taiki-e/install-action` | `@86f7a0ffb25f40114606501a311ebcfaf882ea32  # v2.88.0` | `tool: cargo-about@0.9.3` |
| `softprops/action-gh-release` | `@efb35369e0ad2afab669f228072c1b0d510eae64  # v3.0.3` | Node 24; inputs `draft`, `files`, `fail_on_unmatched_files` |
| Linux runner | `ubuntu-22.04` | oldest current runner; AppImage glibc baseline |

Planner decision: third-party actions that write or install are pinned by commit SHA with the tag in
a comment; GitHub-owned actions and the two already used by ci.yml use major tags.

## R-5 Linux build dependencies

- **Decision**: `sudo apt-get install -y libfontconfig1-dev libxkbcommon-dev libfuse2`.
- **Rationale**: ci.yml already compiles the whole workspace (Slint, `rfd` 0.17 with its default
  `xdg-portal` + `wayland` features, both pure Rust over `zbus`) with only the first two packages.
  `notify-rust` 4.18 defaults to feature `z` (pure-Rust `zbus`), so no `libdbus`. `libfuse2` lets
  the packager run the downloaded `linuxdeploy` AppImage on `ubuntu-22.04`.
- **`.deb` runtime dependencies**: planner decision: `depends = ["libfontconfig1", "libxkbcommon0"]`
  in `[package.metadata.packager.deb]`, the runtime counterparts of the build packages.

## R-6 Icon (AppImage requires one)

- **Finding**: `cargo-packager` has no default icon. Its AppImage step fails with `AppImageSquareIcon`
  when no square PNG icon is configured; `.app`/`.dmg`, NSIS, and `.deb` work without one. This
  contradicts the spec Assumption "the packager's default icon is used", which spec.md now corrects.
- **Decision** (planner, flagged to the router): add `crates/app/icons/icon.png`, a 512x512 PNG of a
  plain square filled with DESIGN.md `colors.primary` (`#B4532F`), generated once with a throwaway
  Python standard-library script (`zlib` + `struct`) in a scratch directory; only the PNG is
  committed. Config: `icons = ["icons/icon.png"]`. It is used for the AppImage, the `.deb` desktop
  entry, and the macOS `.icns` (converted by the packager). The human can replace it later with no
  config change.
- **Alternatives**: drop the AppImage (rejected: changes §7 scope); design a real icon (rejected:
  no design skill outside M0/M4, and DESIGN.md defines no icon).

## R-7 Windows console window

- **Decision** (planner): add `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
  at the top of `crates/app/src/main.rs`.
- **Rationale**: without it, the release `.exe` opens an empty console window next to the app on
  Windows. Debug builds (`cargo run -p app`) keep the console for logs. No behavior change on macOS
  or Linux; no rule involved.

## R-8 Packager metadata values

Planner decisions (naive):

| Key | Value |
| --- | --- |
| `product-name` | `Academic Planner` |
| `identifier` | `io.github.sahil0319.academic-planner` (from the `origin` remote) |
| `description` | `Local-first planner for university courses, weekly timetables, assessments, and attendance.` (docs/spec.md "Goal" line) |
| `publisher` | `Academic Planner contributors` (NOTICE copyright holder) |
| `copyright` | `Copyright (C) 2026 Academic Planner contributors` |
| `category` | `Education` |
| `license-file` | `../../LICENSE` (NSIS shows it as the license page) |
| `before-packaging-command` | `cargo build --release --locked -p app` |
| `resources` | `THIRD-PARTY-NOTICES.html`, `LICENSE`, `NOTICE`, each as `{ src = "../../<file>", target = "<file>" }` |

The identifier does not affect the data folder: `core` uses
`ProjectDirs::from("", "", "Academic Planner")` (R24), not the bundle id.

## R-9 Data folder per OS (for the README)

From `directories` 6.0.0 with `ProjectDirs::from("", "", "Academic Planner")`, file `planner.sqlite`:

- macOS: `~/Library/Application Support/Academic-Planner/`
- Windows: `%APPDATA%\Academic Planner\data\`
- Linux: `$XDG_DATA_HOME/academicplanner/` (default `~/.local/share/academicplanner/`)

The Settings page shows the exact path (§6), and `ACADEMIC_PLANNER_DB` overrides it (R24).

## R-10 Platform coverage of the DMG

- **Decision** (planner): `macos-latest` is Apple Silicon, so the `.dmg` is `aarch64` only. The
  README says the DMG is for Apple Silicon Macs and Intel Mac users build from source. A universal
  binary would add a second target and a `lipo` step that §7 does not ask for.

## R-11 Gatekeeper and SmartScreen steps (README, §10)

- macOS 15 and later no longer offer right-click → Open for unsigned apps; the README gives
  System Settings → Privacy & Security → "Open Anyway" first, right-click → Open for older macOS,
  and the terminal fallback `xattr -dr com.apple.quarantine "/Applications/Academic Planner.app"`.
- Windows: SmartScreen → "More info" → "Run anyway".
