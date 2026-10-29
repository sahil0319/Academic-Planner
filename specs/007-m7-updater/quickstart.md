# Quickstart: validating M7

## 1. Gate and tests (agent)

```bash
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
cargo test -p planner-core t31_ && cargo test -p planner-core t32_ && cargo test -p app t33_
! grep -rnE '#[0-9a-fA-F]{6}|[0-9]+px' crates/app/ui --include=*.slint | grep -v theme.slint
```

Expected: all pass; T31/T32/T33 listed; theme grep prints nothing. No test makes a network request.

## 2. Signing keys (human, once; the agent never sees the private key)

```bash
cargo install cargo-packager --locked --version 0.11.8
mkdir -p ~/.academic-planner-signing
cargo packager signer generate --path ~/.academic-planner-signing/updater.key
# prompts for a password; writes updater.key (private) and updater.key.pub (public)
gh secret set CARGO_PACKAGER_SIGN_PRIVATE_KEY -R sahil0319/Academic-Planner < ~/.academic-planner-signing/updater.key
gh secret set CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD -R sahil0319/Academic-Planner   # paste the password
cat ~/.academic-planner-signing/updater.key.pub   # public: give this one line to the agent
```

The agent puts the public line into `crates/app/updater.pub` and commits it. Keep the private key
and password backed up outside the repo: losing them means installed copies can no longer update.

## 3. Manifest script (agent, local)

Make a temp folder with dummy `Academic.Planner.app.tar.gz(.sig)`, `x-setup.exe(.sig)`,
`x.AppImage(.sig)`; run `bash scripts/updater-manifest.sh v9.9.9 <dir> docs/releases/v0.1.1.md | jq .`
→ three platforms, URLs under `/releases/download/v9.9.9/`. Remove one `.sig` → exit 1.

## 4. Manual app checks (agent or human, macOS dev build)

- `cargo run -p app`: About is the sixth page; it shows name, version, license line, repo and
  notices links (open in the browser), DB path, check button, startup switch.
- Turn the switch off, restart: no check (no banner). Turn it on: one check at start.
- With Wi-Fi off, press Check: "Couldn't check for updates" + Retry; every page still works.
- Dev build with an update available: Install opens the release page (never replaces
  `target/debug`).

## 5. §7 Done-when: update an installed copy on macOS (human)

1. Bump the workspace version (e.g. `0.2.0`), add `docs/releases/v0.2.0.md`, tag, push. Check the
   draft has `latest.json`, the three updater artifacts, and their `.sig` files. Publish it.
2. Install v0.2.0 from its DMG (first-launch steps as in the README).
3. Bump again (e.g. `0.2.1`), release notes, tag, push; while it is a **draft**, the installed
   app shows "You're up to date." Publish it.
4. Restart the installed app: banner "Version 0.2.1 is available." → View → Install → confirm
   panel shows 0.2.1 and its notes → Install and restart → progress → the app relaunches; About
   shows Version 0.2.1; all data is present.
