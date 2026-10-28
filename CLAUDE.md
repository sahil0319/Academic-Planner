# Agent rules
- Spec: docs/spec.md = source of truth. No change rules, schema, API without asking.
- Visuals: DESIGN.md = source of truth. UI use Theme.* only; no raw hex/px outside ui/theme.slint.
- Workflow: Spec Kit per milestone (docs/spec.md §12). No add requirements not in docs/spec.md.
- Skills: use exact §12.0 skill per step. No load design skills outside M0/M4.
- State: update .claude/state/progress.json after each task, checkpoint.md at session end.
- Talk caveman (chat only). Code, comments, tests, commits, docs, specs: normal English.
- Read only files current task need. No repo-wide exploration unless asked.
- One milestone at a time. Done = gate pass:
  cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
- Never claim done without run gate + paste summary.
- Write T-tests for rule before implement. Test names: t07_holiday_added_after_sync.
- Reference rule IDs in doc comments: /// R5: sync never deletes past or non-pending sessions.
- core crate: no slint, no system clock (clippy enforce), no file dialogs. Pass today/now in.
- Schema change = new numbered migration file, never edit applied one.
- UI: no logic in .slint beyond bindings; formatting in viewmodel.rs with unit tests.
- Tests use in-memory SQLite. Never touch real user DB.
- Small commits, one per coherent step, message prefix milestone: "M2: sync deletion rule R5".
