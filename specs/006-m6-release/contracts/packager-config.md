# Contract: cargo-packager configuration (FR-001)

Tool: `cargo-packager` 0.11.8 (research R-1). Location: `crates/app/Cargo.toml`, appended after
`[build-dependencies]`. Relative paths resolve from `crates/app/`.

```toml
# Release packaging (M6). `cargo packager -p app --release --formats <fmt>` from the repo root;
# formats per OS are chosen on the command line (dmg / nsis / deb,appimage).
[package.metadata.packager]
product-name = "Academic Planner"
identifier = "io.github.sahil0319.academic-planner"
description = "Local-first planner for university courses, weekly timetables, assessments, and attendance."
publisher = "Academic Planner contributors"
copyright = "Copyright (C) 2026 Academic Planner contributors"
category = "Education"
license-file = "../../LICENSE"
before-packaging-command = "cargo build --release --locked -p app"
icons = ["icons/icon.png"]
resources = [
    { src = "../../THIRD-PARTY-NOTICES.html", target = "THIRD-PARTY-NOTICES.html" },
    { src = "../../LICENSE", target = "LICENSE" },
    { src = "../../NOTICE", target = "NOTICE" },
]

[package.metadata.packager.deb]
depends = ["libfontconfig1", "libxkbcommon0"]
```

## Guarantees

- `version` is not set in the table; the packager reads `0.1.0` from the workspace via Cargo
  metadata, so the tag check (FR-003) and the installers always agree.
- `binaries` is not set; the packager finds the single bin target `academic-planner` and marks it
  main.
- `formats` is not set (research R-1); `cargo packager --release` with no `--formats` on macOS builds
  the platform default (`.app` + `.dmg`).
- `THIRD-PARTY-NOTICES.html` must exist at the repo root before packaging; if it is missing the
  packager fails, so an installer can never ship without notices (SC-003).
- No signing keys, no notarization, no updater (§10).
- The exact key spelling above follows the 0.11.8 config struct (kebab-case aliases; `deny_unknown_fields`
  rejects typos). If the builder's local run rejects a key, fix the spelling only, never the value
  set.

## Related code change (research R-7)

`crates/app/src/main.rs`, first line after the module doc comment:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

## Icon (research R-6)

`crates/app/icons/icon.png`: 512x512 RGB(A) PNG, every pixel DESIGN.md `colors.primary`
(`#B4532F`). Generated once by a throwaway script outside the repo; only the PNG is committed.
