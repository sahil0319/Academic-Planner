# Tasks: M15 Mid exam and final exam periods

Spec: `docs/spec.md` §13.10 (R71-R75, T71-T74); toasts §13.13 (R88, R89, R89a). Plan and frozen
interfaces: [plan.md](plan.md). Decisions: [decisions.md](decisions.md). Look: `DESIGN.md`
"Calendar days off", "Mini calendar", "Day panel lines", "Exam period cards (Settings)".

Rules for every task:

- One task = one commit, message prefixed `M15:` (e.g. `M15: day off kind migration and model`).
  Tick the box in this file in the same commit.
- Tests are written first by the tester; the builder never edits tests. Names follow
  `t71_exam_days_skip_classes`; doc comments cite the R-id.
- One new migration file only; never edit 001 or 002. No change to `docs/spec.md`, README, or
  other docs. `DESIGN.md` and `ui/theme.slint` already hold `calendar-holiday`,
  `calendar-midterm`, `calendar-final`; no token is added.
- `.claude/state/` is updated by the router, not in these commits.

Stories (for labels):

- **US1** Core: day-off kinds, R71 check, sync skips every kind, migration (R71, R72; T71, T72).
- **US2** Core: backup carries kinds, older backups restore as holiday (R75; T73).
- **US3** Settings: Mid exams and Final exams cards (R73).
- **US4** Dashboard: per-kind fill, legend, day panel line (R74; T74).

## Phase 1: Tests first (tester)

- [x] T001 [P] [US1] [US2] Write core tests in new `crates/core/tests/m15_exam_periods.rs`
  (`mod common;`, in-memory SQLite; the migration test uses `tempfile::TempDir` as
  `m9_migration.rs` does) exactly as plan.md "Test plan" core list:
  `t71_exam_days_skip_classes` (midterm 2026-01-12, final 2026-01-19 on the fixture's Monday slot:
  no session on those dates, other slot days kept, loaded kinds Midterm and Final),
  `t71_holiday_midterm_final_skip_alike` (one day of each kind: no session; dropping the final day
  and syncing recreates its session),
  `t72_date_with_other_kind_is_rejected` (stored holiday 01-12; `set_holidays([holiday 01-12,
  midterm 01-12])` → `Error::Validation("January 12, 2026 is already a holiday")`; holidays and
  sessions unchanged),
  `t72_date_listed_twice_names_first_kind` (`[final 01-19, midterm 01-19]`, nothing stored →
  `Validation("January 19, 2026 is already a final exam day")`, nothing stored),
  `t72_kinds_stored_and_loaded` (three kinds, unsorted input → loaded sorted with labels and
  kinds),
  `t72_existing_rows_stay_holiday_after_migration` (version 2 file DB built from
  `include_str!("../src/migrations/001_init.sql")`, `002_grading.sql`, `grading_seed.sql`,
  `user_version` 2, a term and a holiday row; `Planner::open` → `user_version == SCHEMA_VERSION`,
  row kind `DayOffKind::Holiday`),
  `t73_backup_keeps_kinds` (export has `"kind"` `"holiday"`/`"midterm"`/`"final"`; restore into a
  fresh planner keeps them; re-export equal except `exported_at`),
  `t73_older_backup_restores_every_day_off_as_holiday` (`schema_version` 2, D24; with `kind`
  keys removed and with them kept → all `Holiday`),
  `t73_current_backup_kind_missing_or_invalid_is_rejected` (missing `kind` → Validation containing
  "missing field `kind`"; `"exam"` → Validation containing "holiday kind"; DB unchanged).
  Add to `crates/core/tests/api_surface.rs` under `// M15 (§13.10)` the two lines in plan.md.
  Update existing tests (D26): `kind: DayOffKind::Holiday` in every `Holiday { .. }` literal in
  `crates/core/tests/common/mod.rs` (`holidays`, `holiday_range`), `terms_courses.rs`,
  `m3_backup_restore.rs`, `m3_backup_export.rs`; `m3_backup_export.rs`
  `r21_export_rows_keep_ids_and_value_types` expects
  `{"term_id": .., "date": "2026-01-19", "label": "Break", "kind": "holiday"}`;
  `migrations.rs` `assert_eq!(SCHEMA_VERSION, 3)`; `m9_migration.rs` expects
  `user_version == SCHEMA_VERSION` and `backups/pre-migrate-v1-to-v{SCHEMA_VERSION}.sqlite`;
  `m9_backup.rs` `doc["schema_version"] == SCHEMA_VERSION`; any other assertion the full run
  shows that only encodes the old version or holiday shape. Commit: `M15: core tests T71-T73`.
- [x] T002 [P] [US3] [US4] Write app unit tests in `crates/app/src/viewmodel/tests.rs` against
  plan.md "App view model": helper `holiday()` gains `kind: DayOffKind::Holiday`; new helper
  `day_off(date, label, kind)`;
  `t74_month_grid_marks_each_day_off_kind` (October 2026: holiday 10-06, midterm 10-14, final
  10-22, plus a weekend midterm 10-17 → `day_off` Some(..) per kind, `day_off_key` `"holiday"`,
  `"midterm"`, `"final"`; others `None`/`""`; 10-17 keeps `is_weekend`),
  `t74_legend_lists_holiday_mid_exam_final_exam` (`day_off_legend()` texts
  `["Holiday", "Mid exam", "Final exam"]`, keys `["holiday", "midterm", "final"]`),
  `r74_day_panel_day_off_text_per_kind` (None `""`; `"Holiday: Break"`; `"Mid exam: Week 7"`;
  unlabeled final `"Final exam: Final exam"`), `r74_holiday_text_unlabeled_uses_kind_name`
  (`"Mid exam"`, `"Holiday"`), `r73_as_day_off_sets_kind_and_default_labels`,
  `r73_day_off_rows_filter_by_kind`, `r71_merge_keeps_other_kind_on_same_date`,
  `r89a_day_off_toast_texts` (`"Holidays saved."`, `"Holiday removed."`, `"Mid exams saved."`,
  `"Mid exam removed."`, `"Final exams saved."`, `"Final exam removed."`). Existing holiday and
  calendar tests stay as they are apart from the `kind` field. Commit: `M15: viewmodel tests T74`.

## Phase 2: Core (builder) — US1, US2

- [ ] T003 [US1] Add `crates/core/src/migrations/003_day_off_kind.sql` with exactly
  `ALTER TABLE holiday ADD COLUMN kind TEXT NOT NULL DEFAULT 'holiday' CHECK (kind IN
  ('holiday','midterm','final'));` and a header comment (R71, R72; spec migration 004
  renumbered, D1); append it to `MIGRATIONS` in `crates/core/src/db.rs`. In
  `crates/core/src/model.rs` add `DayOffKind { #[default] Holiday, Midterm, Final }` (derives
  `Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize`, doc `/// R71`) and
  `Holiday.kind: DayOffKind`; update the `Holiday` and `Term.holidays` docs. In `db.rs`:
  `DayOffKind::as_db`, `from_db`, `noun` (crate-private, plan.md); `set_holidays` R71 check before
  any write (D9, D10, exact text `"{%B %-d, %Y} is already a {noun}"`) and insert `kind`;
  `load_term` selects `kind`. Fix `Holiday { .. }` literals in `crates/core/src` if any. T71,
  T72 tests and `migrations.rs` pass. Commit: `M15: day off kind migration and model`.
- [ ] T004 [US1] [US2] Update the `sync_in` doc in `crates/core/src/schedule.rs` to cite R72 (no
  logic change). In `crates/core/src/backup.rs`: `const DAY_OFF_KIND_VERSION: i64 = 3;` (doc:
  renumber with the migration); `HolidayRow.kind: String` after `label` (serde default for old
  documents); export selects `kind`; `restore_json` requires a `kind` key on every holiday object
  when `version >= DAY_OFF_KIND_VERSION` (``"backup has the wrong shape: missing field `kind`"``)
  and sets every row to `"holiday"` below it; `check_texts` rejects an unknown kind via
  `bad("holiday kind", ..)`; `apply` inserts `kind`. `export.rs` unchanged. T73 and all
  `m3_*`/`m9_*` tests pass. Commit: `M15: backup carries day off kind`.

## Phase 3: View model (builder) — US3, US4

- [ ] T005 [US3] [US4] In `crates/app/src/viewmodel.rs` add exactly the plan.md "App view model"
  items: `day_off_kind_name`, `day_off_key`, `LegendItem`, `day_off_legend`, `as_day_off`,
  `day_off_rows`, `day_off_saved_text`, `day_off_removed_text` (Holiday delegates to
  `holidays_saved_text` / `holiday_removed_text`), `day_panel_day_off_text` (Holiday delegates to
  `day_panel_holiday_text`), `day_off_on`; `CalendarDay.day_off: Option<DayOffKind>` set in
  `month_grid`; `expand_holiday_range` sets `DayOffKind::Holiday`; `merge_holidays` keeps an
  other-kind entry on the same date (D11, D12); `holiday_text` falls back to the kind name. Doc
  comments cite R71-R74, R89a. T002 tests pass. Commit: `M15: day off view model`.

## Phase 4: Settings cards (builder) — US3

- [ ] T006 [US3] [US4] `crates/app/ui/types.slint`: `CalendarDay.day-off: string` (comment: fill
  key, D20). `crates/app/src/bridge.rs`: `calendar_day` sets
  `day_off: vm::day_off_key(c.day_off).into()`. `crates/app/ui/components.slint` `SectionCard`:
  optional swatch (`has-swatch`, `swatch`) before the title, `Theme.swatch-size`,
  `Theme.radius-none`, `Theme.space-sm` gap; unchanged look without it.
  Commit: `M15: calendar day off key and card swatch`.
- [ ] T007 [US3] Settings cards (R73) in `crates/app/ui/settings.slint` and
  `crates/app/src/pages/settings.rs`, as plan.md "Slint" and "Page wiring": `DayOffCard`
  component; `SettingsState` additions (`midterms`, `finals`, `mid-*`, `fin-*` fields and errors,
  `add-midterm`, `add-final`); Holidays card rebuilt on `DayOffCard` with today's texts and no
  swatch; Mid exams (swatch `Theme.calendar-midterm`, "Add mid exam") and Final exams (swatch
  `Theme.calendar-final`, "Add final exam") after it, empty texts per D19. Rust: `refresh` fills
  the three row lists via `vm::day_off_rows`; `add_day_off(app, kind)` replaces `add_holiday`
  (parse → `as_day_off` → `merge_holidays` → `set_holidays`; error inline in that card's
  `*-form-error`, fields kept; success clears that card and calls
  `changed_then_toast(app, true, &vm::day_off_saved_text(kind))`); `remove_holiday` toasts
  `vm::day_off_removed_text(removed.kind)`. Commit: `M15: settings mid and final exam cards`.

## Phase 5: Dashboard (builder) — US4

- [ ] T008 [US4] Dashboard calendar (R74) in `crates/app/ui/dashboard.slint` and
  `crates/app/src/pages/dashboard.rs`: `DayCell` fill by `day-off` key (today, then
  `Theme.calendar-holiday` / `Theme.calendar-midterm` / `Theme.calendar-final`, then weekend),
  comment updated; `export struct LegendItem { key: string, text: string }` in
  `crates/app/ui/types.slint` (exported through `app-window.slint` like the other structs) and
  `pub fn legend_item(i: &vm::LegendItem) -> LegendItem` in `crates/app/src/bridge.rs` (A14);
  `DashboardState.day-off-legend: [LegendItem]`; legend line 1 repeats it with
  a `Theme.swatch-size` square of the key's fill, line 2 keeps Weekend and Assessment due (D21;
  remove "Holiday / exam"); `fill_calendar` sets `panel-holiday` from
  `vm::day_panel_day_off_text(vm::day_off_on(&holidays, selected))` and `day-off-legend` from
  `vm::day_off_legend()` via `bridge::legend_item`. Commit: `M15: calendar fills, legend, day panel`.

## Phase 6: Gate (builder)

- [ ] T009 Run `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo
  test --workspace` and paste the summary; run the theme scan
  `grep -nE '#[0-9a-fA-F]{3,8}\b|[0-9]+(\.[0-9]+)?px' crates/app/ui/*.slint | grep -v '^crates/app/ui/theme.slint'`
  (must print nothing; comments count). Confirm `SCHEMA_VERSION`, `DAY_OFF_KIND_VERSION`, the
  migration file number, and `migrations.rs` agree (D1). Commit: `M15: gate` (only if files
  changed).

## Dependencies

- T001, T002 first (parallel; different crates).
- T003 → T004 (backup needs `DayOffKind` and the column).
- T005 needs T003 (`DayOffKind`, `Holiday.kind`) and T002.
- T006 needs T005 (`vm::LegendItem`, `CalendarDay.day_off`); T007 needs T006 (swatch, rows) and
  T005; T008 needs T006 and T005.
- T009 last.

## Parallel examples

- Tester: T001 and T002 together.
- Builder: T007 and T008 touch different files and can be built side by side after T006,
  committed separately.

## Implementation strategy

MVP = US1 (T001, T003): kinds stored, R71 enforced, every kind skips classes. Then US2 (T004,
backup), US3 (Settings cards), US4 (dashboard). The tests-first commits may not compile until
T003 and T005 land, as in earlier milestones.
