# Tasks: M26 extra classes from the Dashboard and the Next class row layout

Spec: `docs/spec.md` section 13.21 (R139-R143, T129-T135). Light flow like M20: no plan, checklist, or analyze. No schema change, no core API change. One commit per step, prefix `M26:`.

- [x] T001 Tests first: `crates/core/tests/m26_extra_class.rs` (T130), app tests T129 and T131-T134 in `crates/app/src/viewmodel/tests.rs`, `add_extra_class` in the T91 handler list.
- [x] T002 `viewmodel.rs`: `extra_class_defaults`, `extra_class_date_text`, `parse_extra_class_form`, `extra_class_added_text`, `calendar_focus_after_add` (R139-R142).
- [x] T003 Dashboard: "Add extra class" button and inline form in the day panel, handlers `open_extra_class`, `add_extra_class`, `cancel_extra_class` in `pages/dashboard.rs` (R139-R142).
- [x] T004 Next class row layout: `SessionLine` grows with the second line and wraps its cells below the wrap width; new tokens in `DESIGN.md` and `theme.slint` (R143).
- [x] T005 Audit table of `specs/018-m18-toasts/tasks.md` lists the new handler.
- [x] T006 Gate and theme grep.
- [ ] T007 (T135, manual, human only) Next class card with a bi-weekly class at the smallest window size, light and dark; add an extra class from the Dashboard on a day in another month.
