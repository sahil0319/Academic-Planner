# M15 decisions

D1-D8 were fixed by the router before planning. D9 onward were taken by the planner with the recommended option. One line each; the human may revisit.

- D1 (router) Migration number = next free number: `003_day_off_kind.sql` (spec says 004; only 001 and 002 exist here). M12 and M19 also add 003 on parallel branches, so the builder renumbers after the final rebase and keeps `SCHEMA_VERSION`, `DAY_OFF_KIND_VERSION`, and the migration test counts consistent.
- D2 (router) Every kind skips classes (R72); rows that exist before the migration stay `holiday` (column default).
- D3 (router) An older backup restores every day off as `holiday` (R75).
- D4 (router) `set_holidays(term_id, days)` keeps replacing every day off of the term, all kinds together (§13.10 API).
- D5 (router) Confirmations are toasts through `App::toast` (M18, R88/R89/R89a); errors stay inline.
- D6 (router) UI uses `Theme.*` only, including `calendar-holiday`, `calendar-midterm`, `calendar-final` (all three already exist in `ui/theme.slint` and DESIGN.md, so no token is added); square corners; no raw hex or px outside `theme.slint`, comments included.
- D7 (router) Core: no Slint, no clock; tests on in-memory SQLite (the migration test uses a temp file DB as `m9_migration.rs` does); test names `t71_…` to `t74_…`.
- D8 (router) Task order: tests first (tester), then core migration + model + sync + backup, then view model, then Settings cards, then dashboard calendar, legend, and day panel, then the gate.
- D9 R71 check lives in core `set_holidays`: any date listed twice in `days` is `Error::Validation`, checked before anything is written. The kind it names is the kind stored for that date, else the kind of the date's first entry in `days`. A date listed once with a new kind is allowed (set_holidays replaces all); the UI never does that (remove, then add).
- D10 R71 message text: `"{date} is already a {kind}"`, date as `"%B %-d, %Y"` (M14 D2 precedent for user-facing core text), kind as `holiday` / `mid exam day` / `final exam day`, no trailing period. Example: `January 12, 2026 is already a holiday`.
- D11 The app never pre-checks R71: `merge_holidays` keeps an existing day off of another kind next to the added one, so core rejects the add and its text shows in that card's inline form error (one source of truth).
- D12 `merge_holidays` keeps its signature; for a date in both lists with the same kind the added entry wins (label update, as today); with different kinds both are kept.
- D13 Blank label on the Mid exams / Final exams cards stores the default label `"Mid exam"` / `"Final exam"` (R73 "default labels"); the Holidays card keeps storing `None`. Any stored day off without a label displays its kind name.
- D14 R73 "a list of ranges, and remove per day": each card lists one row per day with Remove, exactly like the Holidays card (DESIGN.md "same layout as the Holidays card"); consecutive days are not grouped.
- D15 The three cards are one `DayOffCard` component in `settings.slint` (title, optional swatch, rows, fields, errors, empty text, button text, callbacks); order Holidays, Mid exams, Final exams.
- D16 DESIGN.md puts the 12px swatch before the Mid exams and Final exams titles only; the Holidays card title stays as it is. `SectionCard` gains an optional swatch (`has-swatch`, `swatch`), default off.
- D17 Removal reuses the one `remove-holiday(key)` callback (one day off per date); the removed row's kind picks the toast.
- D18 Toast texts: `"Holidays saved."` / `"Holiday removed."` (unchanged), `"Mid exams saved."` / `"Mid exam removed."`, `"Final exams saved."` / `"Final exam removed."`.
- D19 Empty states: `"No mid exams yet. Add a date or a range below with Add mid exam; no classes are scheduled on exam days."` and the same with `final exams` / `Add final exam`; buttons `"Add mid exam"`, `"Add final exam"`; field labels as on the Holidays card.
- D20 Calendar cell: the view model carries `day_off: Option<DayOffKind>`; Slint gets the key `"holiday"` / `"midterm"` / `"final"` / `""` (`vm::day_off_key`) and maps it to the `Theme.calendar-*` fill. Fill priority stays today, then day off, then weekend.
- D21 Legend: the three day-off entries come from `vm::day_off_legend()` (key + name, order Holiday, Mid exam, Final exam) on the first line; Weekend and Assessment due stay as static entries on a second line (Slint has no flow layout; five entries do not fit the 252px grid width).
- D22 The cell's accessible label keeps adding the day-off text (label, else kind name) as today; it does not add the kind prefix (the legend and day panel name the kind).
- D23 Backup: `HolidayRow` gains `kind` (text `holiday` / `midterm` / `final`) after `label`. For `schema_version >= DAY_OFF_KIND_VERSION` every holiday row must carry `kind` (else "backup has the wrong shape: missing field `kind`"), and `check_texts` rejects an unknown kind; for older versions any `kind` is ignored and set to `holiday`. CSV unchanged.
- D24 Older-backup tests use `schema_version` 2 (the last version before any of the parallel migrations), not `SCHEMA_VERSION - 1`, so they survive the renumbering.
- D25 The existing `day_panel_holiday_text(&str)` stays (Holiday line, tested); the new `day_panel_day_off_text(Option<&Holiday>)` uses it for holidays and adds "Mid exam: …" / "Final exam: …".
- D27 Analysis remediation applied: plan.md task references aligned with the nine tasks of tasks.md (gate = T009, A13); `LegendItem` and `bridge::legend_item` moved from T006 to T008 so each commit compiles (A14).
- D26 Adding the `kind` field breaks every `Holiday { .. }` literal in tests; the tester updates them in Phase 1 (list in tasks.md T001/T002), the builder never edits tests.
- D29 (builder) `migrations.rs` `t26_newer_schema_version_refused` hard-coded `found: 3, supported: 2`; it now compares with `SCHEMA_VERSION + 1` / `SCHEMA_VERSION` so it survives the renumbering (old schema version only, no rule change).
- D28 (router) After rebase onto main (M12 003, M19 004): migration is 005_day_off_kind.sql, SCHEMA_VERSION 5, DAY_OFF_KIND_VERSION 5; backups with schema_version < 5 restore every day off as holiday. Plan/task text that says 003 means 005.
- D30 (builder) `DayOffCard` properties are `first-day`, `last-day`, `first-error`, `last-error` (not `start`/`end`): a property named `start` shadows the `start` alignment keyword inside the component. The remove-row message reads "This day off no longer exists" (was "This holiday ...") because the row can be any kind.
