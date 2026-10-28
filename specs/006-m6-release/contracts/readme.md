# Contract: README.md (FR-005, US3 scenario 1)

File: `README.md` at the repo root (new). Plain Markdown, normal English, no badges that need
external services. Sections in this order:

1. **Title and description**: `# Academic Planner` and one paragraph: a local-first, open-source
   desktop app for university courses, weekly timetables, assessments, and manually logged
   attendance; native UI (Slint, no WebView); no account, no server; data stays on your computer
   (docs/spec.md Goal line).
2. **Screenshots**: five images, each with alt text, in this order:
   `docs/screenshots/dashboard.png`, `docs/screenshots/timetable.png`,
   `docs/screenshots/courses.png`, `docs/screenshots/assessments.png`,
   `docs/screenshots/settings.png`. The files are captured by the human (spec edge case
   "Screenshots"); the README references them before they exist.
3. **Download and install**: link to the repository's GitHub Releases page.
   - macOS (Apple Silicon; research R-10): open the `.dmg`, drag "Academic Planner" to
     Applications. Gatekeeper (unsigned app, §10; research R-11): first launch is blocked; open
     System Settings → Privacy & Security → "Open Anyway" (macOS 15 and later), or right-click the
     app → Open (older macOS); terminal fallback
     `xattr -dr com.apple.quarantine "/Applications/Academic Planner.app"`. Intel Macs: build from
     source.
   - Windows: run `academic-planner_<version>_x64-setup.exe`; if SmartScreen warns, "More info" →
     "Run anyway". Start-menu entry and uninstaller are created (US2 scenario 2).
   - Linux: `sudo apt install ./academic-planner_<version>_amd64.deb`, or
     `chmod +x academic-planner_<version>_x86_64.AppImage` and run it.
4. **Build from source**: Rust 1.90 or later (`rustup`); on Debian/Ubuntu first
   `sudo apt-get install libfontconfig1-dev libxkbcommon-dev`; then `cargo run -p app` (debug) or
   `cargo build --release -p app`; run checks with
   `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
5. **Where your data lives** (R24; research R-9): one SQLite file `planner.sqlite` in the per-user
   data folder, never in the install folder: macOS `~/Library/Application Support/Academic-Planner/`,
   Windows `%APPDATA%\Academic Planner\data\`, Linux `~/.local/share/academicplanner/` (or
   `$XDG_DATA_HOME/academicplanner/`). Settings shows the exact path. Set `ACADEMIC_PLANNER_DB` to
   a file path to use a different database (for development and testing).
6. **Backup, restore, and export** (one paragraph; R21–R23, §6 Settings): Settings → Backup saves
   all data as JSON; Restore replaces all data from a backup after a confirmation, and first copies
   the current database to `backups/pre-restore-YYYYMMDD-HHMM.sqlite` in the data folder; Export CSV
   writes attendance and assessments as CSV files for a spreadsheet app.
7. **License**: GPL-3.0-or-later (link `LICENSE`). Uses Slint under GPL-3.0-only (link `NOTICE`).
   Third-party crate licenses: `THIRD-PARTY-NOTICES.html`, bundled in every installer and attached
   to each release; regenerate with the command in `NOTICE`.

## Rules

- State only facts from docs/spec.md, this feature's spec, and research.md; do not promise features
  (for example notifications while the app is closed, signing, or auto-update are out of scope, §10).
- If a statement depends on behavior the builder cannot confirm from code or docs (for example the
  exact Settings label), use the label from `crates/app/ui/settings.slint`.
