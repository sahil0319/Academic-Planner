# Tasks: M20 export courses and routine

Spec: `docs/spec.md` section 13.15 (R96-R100, T92-T96). Light flow like M11: no plan, checklist, or analyze. One commit per step, prefix `M20:`.

- [x] T001 Tests first: `crates/core/tests/m20_export_courses.rs` (T92-T95), app tests T96 in `crates/app/src/viewmodel/tests.rs`, the §5 signature in `api_surface.rs`, the new handlers in the T91 list.
- [ ] T002 Core: `export_courses_json` (R96, R97) in `crates/core/src/import/export.rs`.
- [ ] T003 App: `viewmodel.rs` export texts and `ExportNote` (R99).
- [ ] T004 App: Courses page "Copy JSON" and "Save JSON..." buttons, helper line, handlers in `pages/courses.rs` (R99).
- [ ] T005 Docs: `docs/import/` and Import panel text (R100).
- [ ] T006 Gate, theme grep, rebase onto `main`, gate again.
