# Quickstart: validate M6 Release Packaging

## Prerequisites

- macOS host with Rust 1.90 or later, repository root as the working directory, M5 gate green.
- Tools installed into the ignored `target/tools/` folder so the host's global cargo bin is not
  changed:

```sh
cargo install cargo-packager --locked --version 0.11.8 --root target/tools
cargo install cargo-about --locked --version 0.9.3 --features cli --root target/tools
export PATH="$PWD/target/tools/bin:$PATH"
```

## 1. Notices (FR-004)

```sh
cargo about generate --workspace --locked --fail -o THIRD-PARTY-NOTICES.html about.hbs
git status --short THIRD-PARTY-NOTICES.html     # expect: no output (ignored)
```

Expected: exit 0; the HTML lists the license overview and includes `slint` under
"GNU General Public License v3.0 only". Contracts: [contracts/notices.md](contracts/notices.md).

## 2. macOS package (FR-001)

```sh
cargo packager -p app --release --formats dmg
ls target/release/*.dmg
ls "target/release/Academic Planner.app/Contents/Resources/"
```

Expected: one `Academic Planner_0.1.0_aarch64.dmg`; the `.app` resources hold
`THIRD-PARTY-NOTICES.html`, `LICENSE`, `NOTICE`, and the converted icon. Optional: open the `.app`
with `ACADEMIC_PLANNER_DB=/tmp/ap-m6-check.db` to avoid touching the real database. Contract:
[contracts/packager-config.md](contracts/packager-config.md).

## 3. Workflow lint (FR-002)

```sh
actionlint .github/workflows/release.yml
# if actionlint is not installed:
ruby -ryaml -e 'YAML.load_file(".github/workflows/release.yml"); puts "ok"'
git diff --quiet HEAD -- .github/workflows/ci.yml && echo "ci.yml unchanged"
```

Contract: [contracts/release-workflow.md](contracts/release-workflow.md).

## 4. Gate (FR-006, SC-004)

```sh
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

## 5. Human steps (FR-007)

1. Capture the five screenshots into `docs/screenshots/` (see [contracts/readme.md](contracts/readme.md)).
2. `git push origin main`, then `git tag v0.1.0 && git push origin v0.1.0`.
3. Watch the "Release" workflow: `prepare`, three `package` jobs, `release` all green.
4. Open the draft release: confirm `.dmg`, `-setup.exe`, `.deb`, `.AppImage`, and
   `THIRD-PARTY-NOTICES.html` are attached (SC-001).
5. Install one artifact by following README.md alone and open the app (SC-002). Then publish or keep
   the draft.
