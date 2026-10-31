# Tasks: M16 Public holidays

Spec: `docs/spec.md` §13.11 (R76-R81, T75-T79); toasts §13.13 (R88, R89, R89a). Plan and frozen
interfaces: [plan.md](plan.md). Decisions: [decisions.md](decisions.md). Manual checks:
[quickstart.md](quickstart.md). Look: `DESIGN.md` "Mini calendar", "Day panel lines", "Public
holidays card (Settings)".

Rules for every task:

- One task = one commit, message prefixed `M16:` (e.g. `M16: public holiday table and core API`).
  Tick the box in this file in the same commit.
- Tests are written first by the tester; the builder never edits tests. Names follow
  `t77_set_public_holidays_replaces_one_country`; doc comments cite the R-id.
- One new migration file only; never edit 001-005. No change to `docs/spec.md`, README, or other
  docs. `DESIGN.md` and `ui/theme.slint` already hold `calendar-public-holiday`; no token is added.
- Nothing in tests or the build touches the network: tests use a fake `HolidayFetcher`; the real
  `HttpFetcher` is only built into the app (D4).
- `.claude/state/` is updated by the router, not in these commits.

Stories (for labels):

- **US1** Core cache: table, API, country setting, backups leave the cache out (R77, R80; T77, T78).
- **US2** App fetch: parser, locale mapping, calendar table, fetcher, service (R76, R77, R81; T75, T76, T79).
- **US3** Settings: Public holidays card, background fetch, Retry, Add selected (R76, R77, R79).
- **US4** Dashboard: teal fill, legend entry, day panel line (R78).

## Phase 1: Tests first (tester)

- [ ] T001 [P] [US1] Write core tests in new `crates/core/tests/m16_public_holidays.rs` (`mod common;`,
  in-memory SQLite) exactly as plan.md "Test plan" core list:
  `t77_set_public_holidays_replaces_one_country` (BD twice, IN once: BD replaced, IN untouched),
  `t77_between_is_inclusive_and_ordered`, `t77_empty_set_clears_the_country`,
  `t77_duplicate_date_and_name_does_not_fail`, `t77_backup_has_no_public_holidays`,
  `t77_backup_keeps_country_setting`, `t77_restore_leaves_cached_rows_alone`,
  `t77_older_backup_still_restores` (`schema_version` 5, D24), `t77_country_setting_roundtrip`
  (absent -> `None`, `"off"`, `"BD"`),
  `t78_added_public_holiday_is_a_holiday_and_sync_deletes_sessions` (fixture term and Monday slot;
  midterm 2026-01-12 already a day off; the Holiday for 2026-01-19 labelled with the public name;
  `set_holidays` + `sync_sessions`: the 01-19 session is gone, 01-12 stays a midterm; public holiday
  rows alone change no session). Add to `crates/core/tests/api_surface.rs` under `// M16 (§13.11)`
  the two lines in plan.md. Confirm `migrations.rs`, `m9_migration.rs`, `m12_biweekly.rs`,
  `m15_exam_periods.rs`, `m19_default_bands.rs` need no literal change (D19). Commit:
  `M16: core tests T77-T78`.
- [ ] T002 [P] [US2] [US3] [US4] Write app tests against plan.md "Test plan" app lists. New
  `crates/app/src/holidays/tests.rs` and sample feeds `crates/app/src/holidays/testdata/sample.ics`
  (public holiday, observance, two-day event with exclusive `DTEND`, folded SUMMARY with a space
  and a tab continuation, an escaped comma and `\n`), `datetime.ics`, `years.ics` (events in 2025,
  2026, 2027), included with `include_str!`: `t75_public_holiday_kept_observance_dropped_folded_name_joined`,
  `t75_date_time_event_uses_its_date`, `t75_lf_only_line_endings`,
  `t75_event_without_summary_or_start_is_skipped`, `t75_same_date_and_name_twice_is_listed_once`,
  `t75_description_check_is_case_sensitive`, `t76_locales_map_to_country_or_off` (`en-BD` BD,
  `bn-BD` BD, `en-US` US, `fr` Off, `xx-ZZ` Off, plus `bn_BD.UTF-8`, `en-Latn-BD`, empty, `en-bd`),
  `t76_resolve_country_setting_wins`, `t76_calendar_table_is_valid`,
  `r77_feed_url_encodes_the_calendar_id`, `t79_failed_fetch_keeps_cached_rows`,
  `r77_successful_fetch_replaces_country_rows`, `r77_only_term_years_are_kept`,
  `r77_unknown_code_and_non_calendar_body_are_errors`; the fake fetcher is a struct in the test
  file implementing `HolidayFetcher` (counts calls; returns a fixed `Result`). In
  `crates/app/src/viewmodel/tests.rs`: `t78_plan_add_skips_existing_day_off`,
  `t78_plan_add_counts_a_repeated_date_once`, `r79_added_text_singular_plural_and_skipped`,
  `r79_tick_hint_text`, `r77_status_text_per_state`,
  `r76_country_options_start_with_off_and_round_trip`, `r79_public_holiday_rows_format_date_and_key`,
  `r78_day_panel_public_holiday_text`, `r78_mark_public_holidays_sets_fill_key_and_label`,
  `r78_term_day_off_wins_over_public_holiday_fill`, `r78_calendar_legend_has_public_holiday_last`;
  add `public_holiday: String::new()` to any existing `CalendarDay` literal. The test commit may
  not compile until T005-T009 land (as in earlier milestones). Commit: `M16: app tests T75-T79`.

## Phase 2: Core cache (builder) — US1

- [x] T003 [US1] Add `crates/core/src/migrations/006_public_holiday.sql` with exactly the §13.11 SQL
  `CREATE TABLE public_holiday (country TEXT NOT NULL, date TEXT NOT NULL, name TEXT NOT NULL,
  PRIMARY KEY (country, date, name));` and a header comment (R77, R80; next free number, D1); append
  it to `MIGRATIONS` in `crates/core/src/db.rs`. In `model.rs` add `PublicHoliday { date, name }`
  (derives `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`, doc `/// R77`) and export it from
  `lib.rs`. In `db.rs` add `set_public_holidays` (delete the country's rows, then `INSERT OR
  IGNORE` each day, one transaction, dates through `fmt_date`), `public_holidays_between`
  (`WHERE country = ?1 AND date BETWEEN ?2 AND ?3 ORDER BY date, name`),
  `public_holiday_country` and `set_public_holiday_country`, and `pub(crate) const
  SETTING_PUBLIC_HOLIDAY_COUNTRY: &str = "public_holiday_country"` beside the other `SETTING_*`
  constants (D17, D18). Doc comments cite R76, R77. T77, T78, `api_surface.rs`, and `migrations.rs`
  pass. Commit: `M16: public holiday table and core API`.
- [x] T004 [US1] `crates/core/src/backup.rs`: doc comment only (R80): the export writes every
  `setting` row so `public_holiday_country` is included, `public_holiday` is neither exported nor in
  the restore `DELETE` list. No logic change; if a test fails the builder stops and tells the router.
  Confirm `SCHEMA_VERSION` is 6 and `DAY_OFF_KIND_VERSION` still 5 (D19). All `m3_*`, `m9_*`, `m12_*`,
  `m15_*`, `m19_*` tests pass. Commit: `M16: backup leaves public holidays out`.

## Phase 3: App holidays module (builder) — US2

- [x] T005 [US2] `crates/app/Cargo.toml`: `reqwest = { version = "0.12", default-features = false,
  features = ["blocking", "rustls-tls-native-roots"] }`, `sys-locale = "0.3"`, `serde_json.workspace
  = true` (comment: M16, R77; same features the updater already builds, D3, D20). Check `git diff
  Cargo.lock` shows only the `app` package's dependency list changing and no new `[[package]]`
  (if a new package appears: stop, record in decisions.md, choose the smallest). Create
  `crates/app/src/holidays/mod.rs` (module doc citing R76, R77, R81, "Only the app crate uses the
  network (R27)") and add `mod holidays;` to `main.rs`; `mod.rs` declares only `#[cfg(test)] mod
  tests;` now, and each later task (T006-T008) adds the `pub mod` line for the file it creates, so
  every commit compiles. Commit: `M16: holidays module and dependencies`.
- [x] T006 [US2] `crates/app/src/holidays/ics.rs`: `parse_public_holidays(&str) -> Vec<PublicHoliday>`
  exactly per D22, D23 and plan.md; doc comments cite R81. T75 tests pass. Commit:
  `M16: iCalendar public holiday parser`.
- [x] T007 [US2] `crates/app/data/holiday-calendars.json` (D21, D40: the 18 required codes, names,
  ids; the builder may verify each id with one manual request and records verified and unverified
  ids in decisions.md as a new D-line), `calendars.rs` (`Calendar`, `calendars`, `calendar_for`,
  `calendars_by_name`; `OnceLock`, `include_str!("../../data/holiday-calendars.json")`, parse with
  `serde_json::Value`), `locale.rs` (`Country`, `country_from_locale`, `resolve_country`,
  `setting_value`, `os_locale` via `sys_locale::get_locale()`, untested). T76 tests pass. Commit:
  `M16: country table and locale mapping`.
- [x] T008 [US2] `crates/app/src/holidays/fetch.rs` (`FETCH_TIMEOUT` 10 s, `feed_url` with percent
  encoding of everything but unreserved characters, `HolidayFetcher`, `HttpFetcher` using
  `reqwest::blocking::Client` with the timeout and a non-2xx status as `Err`) and
  `service.rs` (`term_years`, `fetch_events`, `FetchOutcome`, `apply_fetch`; D26). T079 and the
  `r77_*` service tests pass. Commit: `M16: public holiday fetch service`.

## Phase 4: View model (builder) — US3, US4

- [ ] T009 [US3] [US4] In `crates/app/src/viewmodel.rs` add exactly the plan.md "App view model"
  items: `HolidayFetch`, `public_holiday_status_text`, `public_holiday_retry_visible`,
  `country_options`, `country_index`, `country_from_index`, `PublicHolidayRow`,
  `public_holiday_rows`, `AddPlan`, `plan_add_public_holidays`, `public_holidays_added_text`,
  `public_holiday_tick_text`, `day_panel_public_holiday_text`, `mark_public_holidays`,
  `cell_fill_key`, `calendar_legend`; `CalendarDay.public_holiday` (set to empty in `month_grid`).
  In `bridge.rs`: `calendar_day` sets `day_off` from `vm::cell_fill_key(c)`. Doc comments cite
  R76-R79, R89a. T002 view-model tests pass. Commit: `M16: public holiday view model`.

## Phase 5: Settings card (builder) — US3

- [ ] T010 [US3] Settings card (R76, R77, R79) and fetch wiring, as plan.md "Slint" and "Page glue":
  `ui/types.slint` (`PublicHolidayRow`, `day-off` comment), `ui/settings.slint` (`SettingsState`
  additions and the Public holidays card after the Final exams card; `Theme.*` only),
  `bridge.rs` (`public_holiday_row`), new `crates/app/src/pages/public_holidays.rs` (`wire`,
  `start`, `refresh`, `request_fetch`, country change, Retry, tick toggle, Add selected,
  `cached_between`; thread and `invoke_from_event_loop` pattern of `pages/about.rs`; UI-thread
  `thread_local` state, D28), `pages/mod.rs`, `pages/settings.rs` (`refresh` calls
  `public_holidays::refresh`; `changed_then_toast` becomes `pub(super)`, D36), `main.rs`
  (`pages::public_holidays::wire(&app)` with the other wires; `pages::public_holidays::start(&app)`
  after `app.changed(true)`). Add selected: none ticked -> `ph-error`; else `plan_add_public_holidays`,
  `set_holidays` when `added > 0`, `changed_then_toast(.., true, text)` or `app.toast(text)`. No
  network call in tests. Commit: `M16: settings public holidays card`.

## Phase 6: Dashboard (builder) — US4

- [ ] T011 [US4] Dashboard (R78) in `crates/app/ui/dashboard.slint` and
  `crates/app/src/pages/dashboard.rs`: `DayCell` and `DayOffSwatch` fill `"public-holiday"` ->
  `Theme.calendar-public-holiday` (after the three day off kinds, before the weekend; today first);
  update the comments; legend binds the four-entry `day-off-legend` (D31; Public holiday, Weekend,
  Assessment due fit the width or wrap to a further line); `DashboardState.panel-public-holiday` as a
  line under the day off line; `fill_calendar` reads `public_holidays::cached_between(app, first,
  last)`, calls `vm::mark_public_holidays` on the cells, sets `panel-public-holiday` from
  `vm::day_panel_public_holiday_text` (rows of the selected date) and `day-off-legend` from
  `vm::calendar_legend()` via `bridge::legend_item`. A read error shows in the page message like the
  other dashboard reads. Commit: `M16: calendar public holiday fill, legend, day panel`.

## Phase 7: Gate (builder)

- [ ] T012 Run `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo
  test --workspace` and paste the summary; run the theme scan
  `grep -nE '#[0-9a-fA-F]{3,8}\b|[0-9]+(\.[0-9]+)?px' crates/app/ui/*.slint | grep -v '^crates/app/ui/theme.slint'`
  (must print nothing; comments count). Confirm: `SCHEMA_VERSION` equals the number of migration
  files and the migration file number; `git diff Cargo.lock` adds no `[[package]]`; no test or
  `build.rs` opens a socket (`grep -rn "reqwest" crates/app/src` shows only `fetch.rs`); the verified
  and unverified calendar ids are in decisions.md. Commit: `M16: gate` (only if files changed).

## Dependencies

- T001, T002 first (parallel; different crates).
- T003 -> T004 (the backup check needs the table and the setting constant).
- T005 needs T003 (`PublicHoliday`); T006, T007, T008 follow T005 in order (T008's service uses T006
  and T007). They touch different files but share `mod.rs`; keep them sequential.
- T009 needs T003 and T007 (`Country`, `calendars_by_name`) and T002.
- T010 needs T008 and T009; T011 needs T009 and T010 (`cached_between`).
- T012 last.

## Parallel examples

- Tester: T001 and T002 together.
- No builder task is marked [P]: the module files share `mod.rs`, and the UI tasks share `bridge.rs`.

## Implementation strategy

MVP = US1 (T001, T003, T004): the cache exists and the backup rules hold. Then US2 (parser, locale,
fetcher, service: all pure and tested with a fake fetcher), US3 (Settings card and the background
fetch), US4 (dashboard). The tests-first commits may not compile until T003 and T005-T009 land, as
in earlier milestones. The real fetch is verified by the human (quickstart.md).
