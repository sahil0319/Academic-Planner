# Quickstart: validate M17 Carbon Restyle

Never use the real database. Every run below sets `ACADEMIC_PLANNER_DB` to a scratch file.

## 1. Mechanical checks (every task that touches `.slint` files)

```sh
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
! grep -rnE '#[0-9a-fA-F]{6}|[0-9]+px' crates/app/ui --include=*.slint | grep -v theme.slint
```

Expected: gate green with the same test count as before M17; the grep prints nothing.

## 2. End-of-milestone checks (task 11)

```sh
! grep -rnE 'radius-sm|radius-md|calendar-holiday-highlight' crates/app/ui
npx -y @google/design.md@0.4.0 lint DESIGN.md
```

Expected: no output from the first; lint 0 errors.

Fonts: `ls crates/app/ui/fonts/` shows `IBMPlexSans-Regular.ttf`, `IBMPlexSans-SemiBold.ttf`,
`OFL.txt` and nothing else; `grep -c "SIL Open Font License" THIRD-PARTY-NOTICES.html about.hbs`
is at least 1 for each file.

## 3. Scratch data for the screenshots

```sh
mkdir -p target/scratch
rm -f target/scratch/m17.sqlite target/scratch/m17-empty.sqlite
ACADEMIC_PLANNER_DB="$PWD/target/scratch/m17.sqlite" cargo run -p app
```

In the app: Courses → Import… → Copy template, paste, Preview, Apply (effective date = term start).
Then on the Dashboard mark one class Attended and one Absent so a course is below target (banner
visible); add a holiday range in Settings that falls in the visible month; add two assessments
(one due within the upcoming window, one overdue) and complete a third. Run once more with
`m17-empty.sqlite` for the empty-state screenshot.

## 4. Screenshots (task 11, router)

At the default 1100 × 720 window, for Light and then Dark (appearance switch), capture into
`specs/017-m17-carbon-restyle/screenshots/` as `NN-<screen>-<light|dark>.png`:

| NN | Screen | How to reach it |
|---|---|---|
| 01 | dashboard | Dashboard, a non-today day selected in the current month |
| 02 | dashboard-empty | Dashboard on `m17-empty.sqlite` |
| 03 | timetable-week | Timetable, Week tab, a session selected (session panel open) |
| 04 | timetable-rules | Timetable, Rules tab |
| 05 | courses | Courses list |
| 06 | course-detail | Courses, open one course |
| 07 | courses-delete-confirm | tick two courses, click the delete button (confirm visible) |
| 08 | import | Courses → Import…, after Preview of the template |
| 09 | assessments | Assessments, All filter |
| 10 | rules | Rules, one rule set open in the form |
| 11 | settings-restore-confirm | Settings, pick a backup for Restore (confirm visible; do not confirm) |
| 12 | about | About |

Expected: 24 files.

## 5. Critique and approval (task 12)

The router runs `impeccable` on the 24 screenshots against DESIGN.md (allowed in M17, R86) and
writes the findings with severities into
`specs/017-m17-carbon-restyle/checklists/design-review.md`. The builder fixes each high-severity
item in `theme.slint` or the layout only; the router retakes the affected screenshots. The human
looks at the screenshots (and, if they want, the running app) and approves; the router records the
approval in that file.
