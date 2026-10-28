# Checkpoint

- Milestone: M4 (specs/004-m4-desktop-ui). spec.md committed 65b7975; docs/spec.md §6 updated (holiday ranges, reschedule note, makeup form).
- Last green gate: 2dfaf5c (M3).
- Planner wrote plan.md, research.md, contracts/viewmodel.md (uncommitted). Stopped: write of contracts/ui-bridge.md was denied by the permission system ("STOP and wait for user").
- Next: human checks why the ui-bridge.md write was denied; then resume planner for ui-bridge.md, data-model.md, quickstart.md, checklist, tasks, analyze.
- Router decisions (per human's overnight "take the recommended option" instruction):
  - Q1: the R22 pre-restore copy test may use a file DB in a temp dir (never the user DB); all other tests stay in-memory.
  - Q2: same-minute pre-restore name clash -> add suffix -2, -3, ...
- Plan decisions: one local_now() in main.rs with scoped #[allow(clippy::disallowed_methods)]; app deps chrono clock, rfd, rusqlite; VACUUM INTO for the pre-restore copy; one Slint global + src/pages/<page>.rs per page; inline panels, no modal dialogs.
- Open: the impeccable skill is not installed (needed for the end-of-M4 screenshot critique).
- Notes: rustc 1.90; Slint 1.16; rusqlite 0.41; packages planner-core / app.
