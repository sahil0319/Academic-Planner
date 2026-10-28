# Checkpoint

- Milestone: M4 (specs/004-m4-desktop-ui). spec.md committed 65b7975; docs/spec.md §6 updated (holiday ranges, reschedule note, makeup form).
- Last green gate: c271454 (M4 T014; 227 tests pass, theme grep clean).
- Done: planning (54456d2), T001–T014 (deps, theme, CI grep, viewmodel 103 tests green, shell, controller, bridge, main; last c271454/8fb904b).
- In progress: five page builders in worktrees (T015–T019, T020–T022 with settings). Then router merges, T023 timer, T024–T027 converge + gate, T028 critique, T029 human smoke.
- Router decisions (per human's overnight "take the recommended option" instruction):
  - Q1 = B: no file-DB test for the pre-restore copy (constitution II); checked manually in quickstart §6.
  - Q2: same-minute pre-restore name clash -> add suffix -2, -3, ...
- Plan decisions: one local_now() in main.rs with scoped #[allow(clippy::disallowed_methods)]; app deps chrono clock, rfd, rusqlite; VACUUM INTO for the pre-restore copy; one Slint global + src/pages/<page>.rs per page; inline panels, no modal dialogs.
- Open: the impeccable skill is not installed (needed for the end-of-M4 screenshot critique).
- Notes: rustc 1.90; Slint 1.16; rusqlite 0.41; packages planner-core / app.
