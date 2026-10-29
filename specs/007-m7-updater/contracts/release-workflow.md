# Contract: Release workflow with signed updater artifacts (§13.1 note, HD-1, HD-3)

File: `.github/workflows/release.yml` (edit), `scripts/updater-manifest.sh` (new).

## Secrets (created by the human, never by the agent)

| Secret | Content |
| --- | --- |
| `CARGO_PACKAGER_SIGN_PRIVATE_KEY` | full text of the generated private key file |
| `CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD` | the password chosen during generation |

## `package` job changes

1. New first step after checkout: fail with `::error::` if either secret is empty (passed through
   `env`, never echoed).
2. `Package` step gets both secrets as env vars with the same names; `cargo packager` then writes
   `<file>.sig` next to every output.
3. Matrix:

| OS | formats | files uploaded |
| --- | --- | --- |
| macos-latest | `app,dmg` | `*.dmg`, `*.app.tar.gz`, `*.app.tar.gz.sig` |
| windows-latest | `nsis` | `*-setup.exe`, `*-setup.exe.sig` |
| ubuntu-22.04 | `deb,appimage` | `*.deb`, `*.AppImage`, `*.AppImage.sig` |

`if-no-files-found: error` stays, so a missing signature fails the build.

## `release` job changes

1. After `download-artifact`: rename every file in `dist/` replacing spaces with `.`.
2. `bash scripts/updater-manifest.sh "$GITHUB_REF_NAME" dist "docs/releases/$GITHUB_REF_NAME.md" > dist/latest.json`.
3. `softprops/action-gh-release`: unchanged, `draft: true`, `files: dist/*` (now includes
   `latest.json` and the three `.sig` files).

## `scripts/updater-manifest.sh <tag> <dir> <notes-file>`

- `version` = tag without the leading `v`; `notes` = notes file text; `pub_date` = UTC now
  (RFC 3339; the script runs in CI, not in core).
- For each platform, exactly one artifact and its `.sig` must exist in `<dir>`, else exit 1 with a
  message:
  - `macos-aarch64`: `*.app.tar.gz`, format `app`
  - `windows-x86_64`: `*-setup.exe`, format `nsis`
  - `linux-x86_64`: `*.AppImage`, format `appimage`
- `url` = `https://github.com/sahil0319/Academic-Planner/releases/download/<tag>/<file name>`;
  `signature` = `.sig` file text.
- Writes JSON with `jq -n` (safe quoting). `set -euo pipefail`.
- Local check: run against a folder of dummy files; output parses with `jq` and has 3 platforms.

## Unchanged

Tag/version check, notices generation, `ci.yml`, `draft: true`, release notes requirement.
