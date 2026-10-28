# Checkpoint

- Milestone: M4 (specs/004-m4-desktop-ui). T001–T027 done; converge clean; code-level DESIGN.md pre-review done (checklists/design-review.md, 3d2c2ad).
- Last green gate: 3d2c2ad (fmt ok, clippy -D warnings ok, 234 tests pass, theme grep clean, DESIGN.md lint 0 errors/0 warnings, app launches on scratch DB).
- Next: T028 screenshot critique — human takes one screenshot per page + one empty state (screen was locked overnight; computer use unavailable in VS Code); router critiques against DESIGN.md (impeccable skill not installed). Then T029 human §9 smoke (quickstart.md §6).
- Demo data: ACADEMIC_PLANNER_DB="$PWD/target/scratch/demo.sqlite" cargo run -p app (3 courses, 72 sessions, MAT201 below target).
- Router decisions (human delegated overnight 2026-10-29):
  - Q1 = B: no file-DB test for the pre-restore copy (constitution II); checked manually (builder temp test, not committed).
  - Q2: same-minute pre-restore name clash -> suffix -2, -3, ...
  - Timer tick without rollover refreshes all pages except Settings (keeps typed form text).
  - Page `message` colors left as is (no message-is-error bridge property) until T028 screenshots.
- Open for T028: fluent CheckBox/Switch/ComboBox blue accent; two-line course/assessment rows > 34px; low items in design-review.md.
- §9 note: §6 has no course delete; smoke uses archive + assessment delete before restore.
- Notes: rustc 1.90; Slint 1.16 (fluent-light); rusqlite 0.41; rfd 0.17.2.
