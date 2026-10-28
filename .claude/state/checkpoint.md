# Checkpoint

- M4 (specs/004-m4-desktop-ui): T001–T028 done. T028 impeccable critique 22/40, three P1 fixed (dashboard order, keyboard focus, empty states); P2 follow-ups in checklists/design-review.md. Waiting only on T029 human §9 smoke.
- M5 (specs/005-m5-reminders): spec bde3dcc, plan/tasks ee86772. In progress: tester T001–T004 (core m5_* tests), builder T008–T011 (notify-rust + tick hook). Next: core T005–T007, converge T013, gate T014. T012 manual notification = human.
- M6 (specs/006-m6-release): spec 28838a6; planner running plan/tasks/analyze (does not touch .specify/feature.json, which stays on M5 until M5 converge).
- Last green gate: 4912f26 (M4 after P1 fixes; app 113 tests, theme grep clean, launches on demo and empty DB).
- Morning human tasks: M4 T029 §9 smoke; M5 T012 see one notification; M6 README screenshots, push tag v0.1.0, confirm 3 OS artifacts, install on one OS. Origin is 50+ commits behind; nothing pushed.
- Demo data: ACADEMIC_PLANNER_DB="$PWD/target/scratch/demo.sqlite" cargo run -p app
- Router decisions (human delegated overnight 2026-10-29):
  - M4: Q1 = B (no file-DB test for pre-restore copy); same-minute suffix -2, -3; timer tick without rollover skips Settings refresh; page message colors unchanged; only P1 critique items fixed (§12.3).
  - M5: due window fire_at <= now < max(T, fire_at + 1 min); missed reminders skipped; pending sessions + incomplete assessments of non-archived courses in the active term; notifications off = app skips the check; titles built in core.
  - M6: draft GitHub release on tag v*, tag/version check, default icon, ubuntu-22.04 for AppImage, screenshots as human task.
- Notes: rustc 1.90; Slint 1.16 (fluent-light); rusqlite 0.41; rfd 0.17.2; impeccable skill in .claude/skills/impeccable.
