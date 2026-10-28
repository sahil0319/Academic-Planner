# Agent rules
- Spec: docs/spec.md is the source of truth. Do not change rules, schema, or API without asking.
- Visuals: DESIGN.md is the source of truth. UI uses Theme.* only; no raw hex/px outside ui/theme.slint.
- Workflow: Spec Kit per milestone (docs/spec.md §12). Do not add requirements that are not in docs/spec.md.
- Skills: use exactly the §12.0 skill per step. Do not load design skills outside M0/M4.
- State: update .claude/state/progress.json after each task and checkpoint.md at the end of each session.
- Talk in caveman mode (chat only). Code, comments, tests, commits, docs, specs: normal English.
- Read only the files the current task needs. No repo-wide exploration unless asked.
- Work one milestone at a time. Finish = the gate passes:
  cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
- Never claim done without running the gate and pasting its summary.
- Write the T-tests for a rule before implementing it. Test names: t07_holiday_added_after_sync.
- Reference rule IDs in doc comments: /// R5: sync never deletes past or non-pending sessions.
- core crate: no slint, no system clock (clippy enforces), no file dialogs. Pass today/now in.
- Schema changes = a new numbered migration file, never edit an applied one.
- UI: no logic in .slint beyond bindings; put formatting in viewmodel.rs with unit tests.
- Use in-memory SQLite in tests. Never touch the real user DB.
- Small commits, one per coherent step, message prefixed with the milestone: "M2: sync deletion rule R5".
