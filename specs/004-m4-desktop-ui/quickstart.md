# Quickstart: validating M4

Run from the repository root. Never point the app at your real planner while testing: set
`ACADEMIC_PLANNER_DB` to a scratch file (R24).

## 1. Gate (FR-028, SC-003)

```sh
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Expected: all green. View-model tests live in `crates/app/src/viewmodel/tests.rs`; the pre-restore
copy has no automated test (manual checks in §6); all earlier core tests still pass.

## 2. Theme grep check (FR-002, SC-004)

```sh
! grep -rnE '#[0-9a-fA-F]{6}|[0-9]+px' crates/app/ui --include=*.slint | grep -v theme.slint
```

Expected: exit code 0, no output. The same command runs in CI (`.github/workflows/ci.yml`).

## 3. DESIGN.md lint (FR-028)

```sh
npx -y @google/design.md@0.4.0 lint DESIGN.md
```

## 4. Clock read is single and scoped (§1, research R-D1)

```sh
grep -rn "Local::now\|Utc::now\|SystemTime::now" crates/
```

Expected: exactly one hit, in `crates/app/src/main.rs`, directly under
`#[allow(clippy::disallowed_methods)]`.

## 5. Run the app on a scratch database

```sh
ACADEMIC_PLANNER_DB="$PWD/target/scratch/planner.sqlite" cargo run -p app
```

Expected: one window, left nav with Dashboard, Timetable, Courses, Assessments, Settings; every page
shows its empty state; Settings shows the scratch DB path.

## 6. §9 smoke checklist (human, ~5 minutes; FR-030, SC-001)

Create term → add 2 courses → add 3 slots → the timetable shows this week → mark one past session
attended and one absent → course shows 50.0% → cancel a session → reschedule one → add an
assessment due tomorrow → dashboard shows it → back up → delete a course's data / restore → data is
back (§6 has no course delete: archive a course and delete an assessment instead, then restore) → export CSVs and open them in a spreadsheet app.

Extra checks during the same run:

- Add a holiday range of three days inside the term: the week view shows no sessions on those days
  (SC-006); remove one day: its sessions return.
- After the restore, `target/scratch/backups/pre-restore-*.sqlite` exists (SC-007) and opens with
  the pre-restore data.
- Restore a second time in the same minute: a `-2` copy appears, the first is unchanged (Q2).
- Enter `2026-1-5` in any date field and `9:30` in any time field: the inline error names the
  expected format.
- A future session's Attended / Absent buttons are disabled; Cancelled and Pending work (R12).

## 7. Rollover (R6, FR-010)

Leave the app open across midnight (or, for a quick check, run with a scratch DB late in the day):
within about a minute after 00:00 the dashboard's "Today's classes" shows the new day. Unit test
`date_rolled_over` covers the decision logic.

## 8. Newer database refused (R25, FR-008)

```sh
sqlite3 target/scratch/newer.sqlite "PRAGMA user_version = 99;"
ACADEMIC_PLANNER_DB="$PWD/target/scratch/newer.sqlite" cargo run -p app
```

Expected: the window shows a clear error naming both schema versions; no crash.

## 9. Screenshot critique (FR-029, SC-005)

One screenshot per page plus one empty-state screenshot, critiqued with `impeccable` against
`DESIGN.md` (§12.3). Zero open high-severity items; fixes go in `theme.slint` or layout only.
