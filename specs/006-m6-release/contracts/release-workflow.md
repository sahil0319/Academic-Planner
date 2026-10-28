# Contract: release workflow (FR-002, FR-003, FR-004, FR-006)

File: `.github/workflows/release.yml` (new). `.github/workflows/ci.yml` is not edited (FR-006).
Pins and their sources: research R-3, R-4, R-5.

```yaml
name: Release

on:
  push:
    tags: ["v*"]

permissions:
  contents: read

env:
  CARGO_TERM_COLOR: always

jobs:
  prepare:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v7
      - uses: dtolnay/rust-toolchain@stable
      - name: Check tag matches workspace version
        shell: bash
        run: |
          version=$(cargo metadata --no-deps --format-version 1 --locked \
            | jq -r '.packages[] | select(.name == "app") | .version')
          if [ "$GITHUB_REF_NAME" != "v$version" ]; then
            echo "::error::Tag $GITHUB_REF_NAME does not match workspace version $version (expected v$version)"
            exit 1
          fi
      - uses: taiki-e/install-action@86f7a0ffb25f40114606501a311ebcfaf882ea32 # v2.88.0
        with:
          tool: cargo-about@0.9.3
      - name: Generate third-party notices
        run: cargo about generate --workspace --locked --fail -o THIRD-PARTY-NOTICES.html about.hbs
      - uses: actions/upload-artifact@v7
        with:
          name: third-party-notices
          path: THIRD-PARTY-NOTICES.html
          if-no-files-found: error

  package:
    needs: prepare
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: macos-latest
            formats: dmg
            files: target/release/*.dmg
          - os: windows-latest
            formats: nsis
            files: target/release/*-setup.exe
          - os: ubuntu-22.04
            formats: deb,appimage
            files: |
              target/release/*.deb
              target/release/*.AppImage
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v7
      - uses: actions/download-artifact@v8
        with:
          name: third-party-notices
          path: .
      - name: Install system libraries (Linux)
        if: runner.os == 'Linux'
        run: sudo apt-get update && sudo apt-get install -y libfontconfig1-dev libxkbcommon-dev libfuse2
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Install cargo-packager
        run: cargo install cargo-packager --locked --version 0.11.8
      - name: Package
        run: cargo packager -p app --release --formats ${{ matrix.formats }}
      - uses: actions/upload-artifact@v7
        with:
          name: installer-${{ matrix.os }}
          path: ${{ matrix.files }}
          if-no-files-found: error

  release:
    needs: package
    runs-on: ubuntu-22.04
    permissions:
      contents: write
    steps:
      - uses: actions/download-artifact@v8
        with:
          path: dist
          merge-multiple: true
      - uses: softprops/action-gh-release@efb35369e0ad2afab669f228072c1b0d510eae64 # v3.0.3
        with:
          draft: true
          fail_on_unmatched_files: true
          files: dist/*
```

## Behavior guarantees

| Spec item | How the workflow meets it |
| --- | --- |
| US1-1 | `on.push.tags: v*`; three-OS matrix; `cargo packager --release` in each job |
| US1-2 | one `release` job attaches `.dmg`, `-setup.exe`, `.deb`, `.AppImage`, and the notices to one draft release; the release name defaults to the tag |
| US1-3 | no `branches`, `pull_request`, or `workflow_dispatch` trigger; `ci.yml` untouched |
| US1-4 / FR-003 | `prepare` fails before packaging and names both versions |
| FR-004 / SC-003 | notices generated once in `prepare`; each `package` job downloads them before the packager bundles them; `release` attaches them (`merge-multiple` puts them in `dist/`) |
| Edge: license policy | `cargo about generate --fail` fails `prepare`, so nothing is packaged |
| Edge: AppImage portability | Linux job on `ubuntu-22.04` |

## Allowed builder adjustments

The builder may change only mechanics that a lint or a local run proves wrong (for example YAML
quoting, a step name, or the upload glob if the packager writes to a different folder). Triggers,
the job split, `draft: true`, permissions, and pinned versions stay as written unless the router
approves.
