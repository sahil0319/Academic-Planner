# Tasks: M19 default attendance bands

Spec: `docs/spec.md` section 13.14 (R92-R95, T86-T90). Light flow like M11: no plan, checklist, or analyze. One commit per step, prefix `M19:`.

- [x] T001 Tests first: `crates/core/tests/m19_default_bands.rs` (T86-T89), app tests T90 in `crates/app/src/viewmodel/tests.rs`, amended old-band expectations (R95) and schema-version literals.
- [x] T002 Core: `default_attendance_bands()` (R92), exported from `planner_core`.
- [x] T003 Core: data-only migration `003_default_bands.sql` (R93), appended to `MIGRATIONS`; v1 backup restore runs it after the grading seed.
- [x] T004 App: `vm::bands_with_defaults` and the Rules page wiring (R94).
- [ ] T005 Gate, theme grep, rebase onto `main`, renumber the migration if needed, gate again.
