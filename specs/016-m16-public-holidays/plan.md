# Implementation Plan: M16 Public holidays

**Spec directory**: `specs/016-m16-public-holidays` (work happens on the current branch) | **Date**: 2026-10-31
**Spec**: `docs/spec.md` §13.11 (R76-R81, T75-T79). Related: R6 (sync), R27 (network only in the app
crate), R71 (one day off per date), R74 (day off fills), R21/R22 (backup), R23 (CSV, unchanged),
R25 (migrations), §13.13 R88, R89, R89a (toasts). Visuals: `DESIGN.md` "Mini calendar", "Day panel
lines", "Public holidays card (Settings)"; the `calendar-public-holiday` token already exists in
`DESIGN.md` and `crates/app/ui/theme.slint`.
**Decisions**: [decisions.md](decisions.md) (D1-D16 fixed by the router; D17 onward by the planner).

## Summary

The dashboard shows the government public holidays of the user's country. They never change
classes until the human adds them as holidays. Five parts:

1. **Core cache (R77, R80).** Migration `006_public_holiday.sql` (spec: "next free number") creates
   `public_holiday`. `PublicHoliday`, `set_public_holidays`, `public_holidays_between`, and two small
   methods for the `public_holiday_country` setting. Backups and CSV do not carry the table; the
   setting rides in the backup `settings` map as every other setting does.
2. **App holidays module (R76, R77, R81).** `crates/app/src/holidays/`: a pure iCalendar parser, a
   pure locale-to-country mapping, the built-in calendar table (`crates/app/data/holiday-calendars.json`),
   a `HolidayFetcher` trait with a `reqwest` implementation, and a pure service layer that takes the
   fetcher. Tests inject a fake fetcher; nothing in tests or the build touches the network.
3. **View model (R76-R79; T78, T79).** Status text and Retry flag, country choices, rows, the
   add-selected plan and its toast text, the public holiday day panel line, cell fill key, and legend.
4. **Settings UI (R76, R77, R79).** A **Public holidays** card: country choice, status line with
   Retry, checkbox list, **Add selected as holidays**. Background fetch on launch, on country change,
   and on Retry.
5. **Dashboard UI (R78).** Teal fill, a legend entry, and a "Public holiday: {name}" day panel line.

## Technical Context

**Language/Version**: Rust (workspace edition 2021, rust-version 1.90), Slint UI
**Primary Dependencies**: core: none new. App: adds direct dependencies on `reqwest` (blocking,
rustls, native roots), `sys-locale`, and `serde_json`. All three are already in `Cargo.lock`
(D20), so no new package enters the lock file.
**Storage**: SQLite; one new migration with the exact §13.11 SQL.
**Testing**: `cargo test --workspace`; core integration tests on in-memory SQLite; app unit tests
(parser on saved feeds, locale mapping, service with a fake fetcher, view model).
**Target Platform**: desktop (macOS, Windows, Linux).
**Project Type**: Cargo workspace: `crates/core` (library), `crates/app` (Slint desktop app).
**Constraints**: core reads no clock, has no Slint and no network; `.slint` bindings only; `Theme.*`
only, no raw hex or px outside `ui/theme.slint` (comments count); square corners; nothing blocks the
UI (R77).
**Scale/Scope**: tens of public holidays per country and year; one request per launch.

No NEEDS CLARIFICATION remains; every interpretation is in decisions.md.

## Constitution Check

| Principle | Status |
|---|---|
| I. Spec is source of truth | Pass. Schema, rules, and API exactly as §13.11. The two extra country-setting methods and the new view-model functions add no rule (D17). Wording not fixed by the spec is in decisions.md. |
| II. Test-first, traceable | Pass. T75-T79 and supporting tests are tasks T001-T002, before any code; names `t75_…`; doc comments cite R76-R81. |
| III. Pure, clock-free core | Pass. No clock or network in core; parser, locale mapping, and service are pure functions in the app crate; every UI string comes from `viewmodel.rs`; `.slint` maps a fill key to a `Theme` token (a binding, as for the day-off fills). |
| IV. Append-only migrations | Pass. New file `006_public_holiday.sql`; 001-005 untouched. |
| V. Gate before done | Pass. T012 runs the gate, the theme grep, and the lock-file check. |

Re-check after design: still pass.

## Every place touched

| Place | Today | M16 change |
|---|---|---|
| `crates/core/src/migrations/` | 001-005 | new `006_public_holiday.sql` (T003) |
| `crates/core/src/db.rs` `MIGRATIONS`, `SCHEMA_VERSION` | 5 entries | 6 entries; `SCHEMA_VERSION` follows (T003) |
| `crates/core/src/model.rs` | — | `PublicHoliday` (T003) |
| `crates/core/src/db.rs` | — | `set_public_holidays`, `public_holidays_between`, `public_holiday_country`, `set_public_holiday_country`, `SETTING_PUBLIC_HOLIDAY_COUNTRY` (T003) |
| `crates/core/src/lib.rs` | exports | + `PublicHoliday` (T003) |
| `crates/core/src/backup.rs` | all `setting` rows exported and restored; tables list has no `public_holiday` | doc comment citing R80 only; no logic change (T004) |
| `crates/core/src/export.rs` CSV | no public holidays | unchanged |
| `crates/core/src/schedule.rs` sync | skips `term.holidays` | unchanged (public holidays never skip classes, R79) |
| `crates/app/Cargo.toml` | no HTTP or locale dependency | + `reqwest`, `sys-locale`, `serde_json` (T005) |
| `crates/app/data/holiday-calendars.json` | — | new (T007) |
| `crates/app/src/holidays/{mod,ics,locale,calendars,fetch,service,tests}.rs`, `testdata/` | — | new (T005-T008, tests T002) |
| `crates/app/src/main.rs` | `mod` list; wires pages; startup update check | `mod holidays;`; `pages::public_holidays::wire/start` (T010) |
| `crates/app/src/pages/{mod,public_holidays}.rs` | — | new page glue (T010) |
| `crates/app/src/pages/settings.rs` `refresh` | day off cards | calls `public_holidays::refresh` (T010) |
| `crates/app/src/pages/dashboard.rs` `fill_calendar` | day off line, legend | cached public holidays: fills, day panel line, legend (T011) |
| `crates/app/src/viewmodel.rs` | `CalendarDay`, `day_off_legend`, `day_panel_day_off_text` | per "App view model" below (T009) |
| `crates/app/src/bridge.rs` | `calendar_day`, `legend_item` | fill key from `vm::cell_fill_key`; `public_holiday_row` (T009, T010) |
| `crates/app/ui/types.slint` | `CalendarDay`, `LegendItem` | + `PublicHolidayRow`; `day-off` comment (T010) |
| `crates/app/ui/settings.slint` | day off cards | new Public holidays card, `SettingsState` additions (T010) |
| `crates/app/ui/dashboard.slint` | `DayCell`, `DayOffSwatch`, legend, panel | public-holiday fill, legend entry, panel line (T011) |
| Tests with `SCHEMA_VERSION` | symbolic (`SCHEMA_VERSION`, `+ 1`) | no literal to change (D19); builder greps again after the final rebase |

Not touched: import, grading, reminders. No `Holiday { .. }` literal changes (`Holiday` is unchanged).

## Frozen interfaces

Tester and builder code against these. A change needs the router.

### Migration `crates/core/src/migrations/006_public_holiday.sql`

Exactly the §13.11 SQL:

```sql
CREATE TABLE public_holiday (country TEXT NOT NULL, date TEXT NOT NULL, name TEXT NOT NULL, PRIMARY KEY (country, date, name));
```

Header comment (normal SQL comment) names R77, R80 and says the number is the next free one
(D1). `db.rs` `MIGRATIONS` gains `include_str!("migrations/006_public_holiday.sql")`;
`SCHEMA_VERSION` becomes 6 automatically. No foreign key (the cache is independent of terms).

### Core model and API (`crates/core/src/model.rs`, `db.rs`, `lib.rs`)

```rust
/// R77: one government public holiday of a country (a cache row, R80).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicHoliday { pub date: NaiveDate, pub name: String }

impl Planner {
    /// R77: replaces every cached row of `country` with `days`, in one transaction. Other
    /// countries are untouched. The country text is stored as given (D18).
    pub fn set_public_holidays(&mut self, country: &str, days: &[PublicHoliday]) -> Result<()>;
    /// R77, R79: cached rows of `country` with `from <= date <= to`, ordered by date, then name.
    pub fn public_holidays_between(&self, country: &str, from: NaiveDate, to: NaiveDate)
        -> Result<Vec<PublicHoliday>>;
    /// R76: the stored `public_holiday_country` setting text, `None` when absent (D17).
    pub fn public_holiday_country(&self) -> Result<Option<String>>;
    /// R76: stores the setting text (`"off"` or an ISO code, D8); included in backups.
    pub fn set_public_holiday_country(&mut self, value: &str) -> Result<()>;
}
```

- `set_public_holidays`: `DELETE FROM public_holiday WHERE country = ?1`, then
  `INSERT OR IGNORE` each day (a repeated `(date, name)` does not fail, D18); dates stored with the
  crate's `fmt_date` (`YYYY-MM-DD`); an empty slice just clears the country.
- `public_holidays_between`: `WHERE country = ?1 AND date BETWEEN ?2 AND ?3 ORDER BY date, name`.
- `pub(crate) const SETTING_PUBLIC_HOLIDAY_COUNTRY: &str = "public_holiday_country";` beside the other
  `SETTING_*` constants. `Settings` is not extended (it is `Copy` and built in many places, D17).

`crates/core/tests/api_surface.rs` gains, under `// M16 (§13.11)`:

```rust
let _ = PublicHoliday { date: NaiveDate::default(), name: String::new() };
let _: fn(&mut Planner, &str, &[PublicHoliday]) -> Result<()> = Planner::set_public_holidays;
```

### Backup (R80)

No logic change. The export already writes every `setting` row, so `public_holiday_country` is in
the backup `settings` map; the restore already re-inserts every `settings` entry. `public_holiday`
is not in the export document and not in the restore `DELETE` list, so a restore leaves cached rows
alone. `schema_version` in a new backup is `SCHEMA_VERSION`; every older backup (version <=
`SCHEMA_VERSION`) still restores because the migration adds a table outside the document (D19).
No new version constant (unlike `DAY_OFF_KIND_VERSION`); `DEFAULT_BANDS_MIGRATION` is unrelated.

### App holidays module (`crates/app/src/holidays/`)

`calendars.rs`:

```rust
/// R76: one built-in country: ISO code, display name, Google calendar id.
pub struct Calendar { pub code: String, pub name: String, pub id: String }
/// Parsed once from `include_str!("../../data/holiday-calendars.json")`; empty if the text is invalid
/// (a unit test guarantees it is valid).
pub fn calendars() -> &'static [Calendar];
pub fn calendar_for(code: &str) -> Option<&'static Calendar>;
/// The table sorted by display name (the order of the Settings choice after "Off").
pub fn calendars_by_name() -> Vec<&'static Calendar>;
```

`holiday-calendars.json` shape (D21): an array of objects
`{ "code": "BD", "name": "Bangladesh", "id": "en.bd#holiday@group.v.calendar.google.com" }`, at least
BD, IN, PK, NP, LK, MY, SG, ID, AE, SA, GB, IE, US, CA, AU, NZ, DE, FR. Parsed with
`serde_json::Value` (no derive needed).

`locale.rs`:

```rust
/// R76: a country in the table, or Off.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Country { Off, Code(String) }
/// R76, T76: `en-BD` -> BD; `bn_BD.UTF-8` -> BD; `fr` (no region) -> Off; `xx-ZZ` -> Off.
pub fn country_from_locale(tag: &str) -> Country;
/// R76, D8: setting absent -> the OS region; `"off"` -> Off; an ISO code in the table -> it; any
/// other stored text -> Off.
pub fn resolve_country(setting: Option<&str>, os_locale: Option<&str>) -> Country;
/// The text stored in `public_holiday_country`: `"off"` or the code.
pub fn setting_value(country: &Country) -> String;
/// Thin wrapper over `sys_locale::get_locale()`; not unit tested.
pub fn os_locale() -> Option<String>;
```

`ics.rs` (R81, T75):

```rust
/// R81: public holidays in an iCalendar feed. Pure; no I/O. Rules in D22, D23.
pub fn parse_public_holidays(ics: &str) -> Vec<PublicHoliday>;
```

`fetch.rs`:

```rust
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(10);
/// R77: `https://calendar.google.com/calendar/ical/{url-encoded id}/public/basic.ics`.
pub fn feed_url(calendar_id: &str) -> String;
pub trait HolidayFetcher { fn fetch(&self, url: &str) -> Result<String, String>; }
/// The real fetcher: one blocking GET with `FETCH_TIMEOUT`; a non-2xx status is an error.
pub struct HttpFetcher;
```

`service.rs` (pure apart from the injected fetcher):

```rust
/// R77: the years of the active term, start year to end year inclusive.
pub fn term_years(start: NaiveDate, end: NaiveDate) -> RangeInclusive<i32>;
/// R77: one fetch of the country's feed; events dated in `years`, sorted by date then name.
/// Err when the code is not in the table, the fetch fails, or the body is not a calendar
/// (no `BEGIN:VCALENDAR`, D26).
pub fn fetch_events(fetcher: &dyn HolidayFetcher, code: &str, years: RangeInclusive<i32>)
    -> Result<Vec<PublicHoliday>, String>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchOutcome { Updated, Failed }
/// R77: success replaces the country's rows (`set_public_holidays`); a failure, or a write error,
/// leaves the cached rows as they are and returns `Failed`.
pub fn apply_fetch(planner: &mut Planner, code: &str, result: Result<Vec<PublicHoliday>, String>)
    -> FetchOutcome;
```

### App view model (`crates/app/src/viewmodel.rs`)

```rust
/// R77, D29: where the public holiday refresh stands (UI state, not stored).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HolidayFetch { Idle, Loading, Updated(NaiveDate), Failed }
/// R77: "" | "Loading public holidays…" | "Updated today" / "Updated {date}" | "Couldn't load public holidays".
pub fn public_holiday_status_text(state: HolidayFetch, today: NaiveDate) -> String;
/// R77: Retry is shown only for `Failed`.
pub fn public_holiday_retry_visible(state: HolidayFetch) -> bool;

/// R76: ["Off", then the table names sorted] and the index mapping both ways.
pub fn country_options() -> Vec<String>;
pub fn country_index(country: &Country) -> i32;
pub fn country_from_index(index: i32) -> Country;

/// R79: one row of the Settings list: key `"{YYYY-MM-DD}|{name}"`, date text, name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicHolidayRow { pub key: String, pub date: String, pub name: String }
pub fn public_holiday_rows(days: &[PublicHoliday]) -> Vec<PublicHolidayRow>;

/// R79, D34: result of "Add selected as holidays".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddPlan { pub merged: Vec<Holiday>, pub added: usize, pub skipped: usize }
/// R71, R79: each ticked public holiday whose date is not already a day off (of any kind, also one
/// added earlier in this call) becomes a `Holiday` with the public name as label; other ticks are
/// skipped. `merged` = `existing` plus the additions, sorted by date.
pub fn plan_add_public_holidays(existing: &[Holiday], ticked: &[PublicHoliday]) -> AddPlan;
/// R79, R89a: "Added 1 holiday." | "Added 3 holidays." | "Added 2 holidays. 1 skipped, already a day off."
pub fn public_holidays_added_text(added: usize, skipped: usize) -> String;
/// R79: inline error when nothing is ticked: "Tick at least one public holiday."
pub fn public_holiday_tick_text() -> String;

/// R78: "" for none; "Public holiday: {name}"; several names joined with "; ".
pub fn day_panel_public_holiday_text(days: &[PublicHoliday]) -> String;
/// R78, D30: `CalendarDay.public_holiday` (names joined) and the label addition, for each cell
/// whose date has a public holiday.
pub fn mark_public_holidays(cells: &mut [CalendarDay], days: &[PublicHoliday]);
/// R74, R78, D30: the fill key of a cell: the day off kind key, else "public-holiday" when
/// `public_holiday` is not empty, else "". (Today and weekend are decided in Slint, D30.)
pub fn cell_fill_key(cell: &CalendarDay) -> &'static str;
/// R78, D31: `day_off_legend()` plus `LegendItem { key: "public-holiday", text: "Public holiday" }`.
pub fn calendar_legend() -> Vec<LegendItem>;
```

`CalendarDay` gains `pub public_holiday: String` (empty by default; `month_grid` sets `""`).
`month_grid`, `day_off_legend`, `day_panel_day_off_text` keep their signatures and results.

### Slint (`crates/app/ui`)

`types.slint`: `export struct PublicHolidayRow { key: string, date: string, name: string, checked: bool }`;
the `CalendarDay.day-off` comment says it holds the fill key (day off kind or `public-holiday`).

`settings.slint`: `SettingsState` additions: `in property <[string]> ph-countries;`
`in-out property <int> ph-country-index;` `in property <string> ph-status;`
`in property <bool> ph-can-retry;` `in property <[PublicHolidayRow]> ph-rows;`
`in property <string> ph-empty-text;` `in property <string> ph-error;`
`callback ph-country-selected();` `callback ph-retry();` `callback ph-toggle(string, bool);`
`callback ph-add-selected();`. New `SectionCard` titled "Public holidays" after the Final exams
card: `ComboField` (label "Country", `selected` -> `ph-country-selected`), a `MutedText` status line
with a `LinkButton` "Retry" when `ph-can-retry`, the rows (`CheckBox` with date and name, `checked`
from the row, `toggled` -> `ph-toggle(key, checked)`), the empty text when there are no rows, a
`FormError` for `ph-error`, and a `SecondaryButton` "Add selected as holidays". Reuse the existing
`SectionCard`, `ComboField`, `LinkButton`, `SecondaryButton`, `MutedText`, `FormError`.

`dashboard.slint`: `DayCell` and `DayOffSwatch` fill: `"public-holiday"` ->
`Theme.calendar-public-holiday`, after the three day off kinds and before the weekend fill (today
stays first); `DashboardState.panel-public-holiday: string` shown as a second panel line under the
day off line; the legend binds the four-entry `day-off-legend` (D31).

### Page glue (`crates/app/src/pages/public_holidays.rs`)

- `wire(app: &Rc<App>)`: stores a weak `App` handle in a UI-thread `thread_local` (the
  `invoke_from_event_loop` closure is `Send` and cannot hold an `Rc`, D28) and connects
  `ph-country-selected`, `ph-retry`, `ph-toggle`, `ph-add-selected`.
- `start(app: &Rc<App>)`: once per launch, called from `main.rs` after the first `app.changed(true)`
  (D36): `request_fetch`.
- `refresh(app: &App)`: fills the card (options and index, status, retry, rows with their ticks,
  empty text). Called from `settings::refresh`. Keeps ticks across refreshes (the tick set lives in
  the `thread_local`, keyed by row key; keys no longer listed are dropped).
- `request_fetch(app)`: no active term, country Off, unknown code, or a fetch already in flight -> returns
  without a request. Else sets status `Loading`, spawns `std::thread` that runs
  `service::fetch_events(&HttpFetcher, code, term_years(..))` and posts the result with
  `slint::invoke_from_event_loop` (the pattern of `pages/about.rs`). On the UI thread the result
  is dropped when the current country is no longer `code`; else `service::apply_fetch` (UI-thread
  planner), status `Updated(today)` or `Failed`, then `app.refresh_all()` (no sync, no toast, D28).
- Country change: saves `set_public_holiday_country(setting_value(..))` (error inline in `ph-error`),
  clears the ticks, status `Idle`, `refresh_all`, then `request_fetch`.
- Add selected: no ticks -> `ph-error` = `vm::public_holiday_tick_text()`. Else
  `vm::plan_add_public_holidays(&term.holidays, &ticked)`; `added > 0` -> `set_holidays(term.id,
  &plan.merged)` (error inline), clear ticks, `changed_then_toast(.., true, text)` (R6 sync);
  `added == 0` -> clear ticks and `app.toast(text)` only. Text = `vm::public_holidays_added_text`.
  `changed_then_toast` is the helper in `settings.rs` (no toast when the page `message` holds an
  error); it becomes `pub(super)` and the glue calls it (D36).
- `cached_between(app, from, to) -> Result<Vec<PublicHoliday>, String>`: resolved country from the
  setting and `os_locale()`; Off or unknown -> empty; else `public_holidays_between`. Used by
  `dashboard.rs`.

## Project Structure

```text
specs/016-m16-public-holidays/
├── plan.md        # this file (analysis report at the end)
├── tasks.md
├── decisions.md
└── quickstart.md  # manual checks for the human (D15)

crates/core/src/
├── migrations/006_public_holiday.sql   # new
├── db.rs          # MIGRATIONS, public holiday methods, SETTING_PUBLIC_HOLIDAY_COUNTRY
├── model.rs, lib.rs   # PublicHoliday
└── backup.rs      # doc comment only (R80)
crates/core/tests/
├── m16_public_holidays.rs   # new: T77, T78 + support
└── api_surface.rs           # + M16 lines
crates/app/
├── Cargo.toml               # reqwest, sys-locale, serde_json
├── data/holiday-calendars.json
└── src/
    ├── holidays/{mod,ics,locale,calendars,fetch,service,tests}.rs, testdata/*.ics
    ├── pages/public_holidays.rs, pages/{mod,settings,dashboard}.rs
    ├── viewmodel.rs, viewmodel/tests.rs, bridge.rs, main.rs
    └── ../ui/{types,settings,dashboard}.slint
```

No research.md or data-model.md: the frozen interfaces above are the contract and decisions.md
replaces research (router instruction).

## Test plan (written first)

Core, new `crates/core/tests/m16_public_holidays.rs` (`mod common;`, in-memory SQLite unless noted):

- `t77_set_public_holidays_replaces_one_country`: BD with 3 rows, then BD with 2 different rows,
  IN with 1 row; `public_holidays_between` BD returns only the second set; IN untouched.
- `t77_between_is_inclusive_and_ordered`: rows out of order, two names on one date; bounds
  inclusive; ordered by date then name; another country never appears.
- `t77_empty_set_clears_the_country`; `t77_duplicate_date_and_name_does_not_fail` (D18).
- `t77_backup_has_no_public_holidays`: after `set_public_holidays`, the `export_json` document has
  no `public_holiday` key at the top level and its text holds none of the cached names; CSV export
  unchanged (existing `m3_csv.rs` stays green).
- `t77_backup_keeps_country_setting`: `set_public_holiday_country("BD")`; export `settings` has it;
  restore into a fresh planner returns `Some("BD")`; absent setting returns `None` (R76).
- `t77_restore_leaves_cached_rows_alone`: cache BD rows, restore a backup made before (no rows,
  other settings): rows still returned; country setting follows the backup.
- `t77_older_backup_still_restores`: export, set `schema_version` to 5 (D24 style: an older version
  literal, not `SCHEMA_VERSION - 1`), restore succeeds.
- `t77_country_setting_roundtrip`: absent -> `None`; `"off"`, `"BD"` read back.
- `t78_added_public_holiday_is_a_holiday_and_sync_deletes_sessions`: fixture term and Monday slot,
  midterm on 2026-01-12 (a day off already), public holidays 2026-01-12 ("Day A") and 2026-01-19
  ("Day B"); stored holidays built with the pure shape of R79 (kind `Holiday`, label = public name)
  for 01-19 only, midterm kept. Start from the synced fixture (sessions on 01-05, 01-07, 01-12,
  01-14, 01-19, 01-21 as in `m15_exam_periods.rs`), with the cache rows set first: syncing with
  `today` 2026-01-05 changes no session (public holidays never skip classes). Then `set_holidays`
  (midterm 01-12 plus the new holiday 01-19) and `sync_sessions(2026-01-05)`: the 01-12 and 01-19
  sessions are gone (R5: pending, `date >= today`), 01-14 and 01-21 stay, and the active term
  holds the midterm and a `Holiday` 01-19 labelled "Day B".
- Migration counts: `migrations.rs` and `m9_migration.rs` already compare with `SCHEMA_VERSION`; the
  tester confirms no literal needs a change (D19).

App, `crates/app/src/holidays/tests.rs` (sample feeds in `crates/app/src/holidays/testdata/`):

- `t75_public_holiday_kept_observance_dropped_folded_name_joined`: `sample.ics` has a public holiday,
  an observance, a two-day all-day event (`DTEND` exclusive), and a folded SUMMARY (CRLF + space and
  CRLF + tab): the observance is dropped; the two-day event gives two dates; the folded name is
  joined; an escaped comma and `\n` in text are unescaped; result sorted.
- `t75_date_time_event_uses_its_date`, `t75_lf_only_line_endings`, `t75_event_without_summary_or_start_is_skipped`,
  `t75_same_date_and_name_twice_is_listed_once`, `t75_description_check_is_case_sensitive`.
- `t76_locales_map_to_country_or_off`: `en-BD` BD, `bn-BD` BD, `en-US` US, `fr` Off, `xx-ZZ` Off;
  plus `bn_BD.UTF-8` BD, `en-Latn-BD` BD, `` Off, `en-bd` BD (region uppercased).
- `t76_resolve_country_setting_wins`: setting `off` -> Off; `GB` -> GB; absent + `en-BD` -> BD;
  absent + `fr` -> Off; stored `ZZ` -> Off; stored `bd` (lowercase) -> Off (stored codes are upper-case).
- `t76_calendar_table_is_valid`: JSON parses; the 18 required codes are present; codes unique;
  each id ends with `#holiday@group.v.calendar.google.com`; names not empty.
- `r77_feed_url_encodes_the_calendar_id`: `en.bd#holiday@group.v.calendar.google.com` ->
  `https://calendar.google.com/calendar/ical/en.bd%23holiday%40group.v.calendar.google.com/public/basic.ics`.
- `t79_failed_fetch_keeps_cached_rows`: planner with cached BD rows; a fake fetcher returning `Err`;
  `fetch_events` is `Err`; `apply_fetch` returns `Failed`; `public_holidays_between` still returns the
  rows; `vm::public_holiday_status_text(Failed, ..)` is "Couldn't load public holidays" and
  `vm::public_holiday_retry_visible(Failed)` is true.
- `r77_successful_fetch_replaces_country_rows`: fake fetcher returns the sample feed; `Updated`;
  BD rows replaced; another country untouched.
- `r77_only_term_years_are_kept`: sample feed with events in 2025, 2026, 2027; term 2026-01-05..2026-12-20
  keeps 2026 only; a term spanning 2026-09-01..2027-01-31 keeps 2026 and 2027.
- `r77_unknown_code_and_non_calendar_body_are_errors`: code `ZZ` -> `Err` and the fetcher is not
  called; body `<html>` -> `Err`; cached rows stay.
- `r77_off_and_no_term_fetch_nothing`: the request decision is the page glue; the pure part is
  `resolve_country` (Off) and `term_years`; no test calls a real fetcher.

App, `crates/app/src/viewmodel/tests.rs`:

- `t78_plan_add_skips_existing_day_off`: existing midterm on 10-14; ticked 10-14 and 10-20 ("Day B")
  -> `added == 1`, `skipped == 1`, merged has a `Holiday` 10-20 labelled "Day B" and the midterm kept.
- `t78_plan_add_counts_a_repeated_date_once`: two ticked rows on one date -> first added, second skipped.
- `r79_added_text_singular_plural_and_skipped`: the three texts of D11.
- `r79_tick_hint_text`: "Tick at least one public holiday."
- `r77_status_text_per_state`: Idle "", Loading, Updated same day "Updated today", Updated earlier
  day "Updated {format_date}", Failed text; retry only for Failed.
- `r76_country_options_start_with_off_and_round_trip`: options[0] "Off"; `country_index` and
  `country_from_index` invert each other for Off and for a code; out-of-range index -> Off.
- `r79_public_holiday_rows_format_date_and_key`.
- `r78_day_panel_public_holiday_text`: none "", one "Public holiday: Eid", two joined with "; ".
- `r78_mark_public_holidays_sets_fill_key_and_label`: October 2026 grid, a public holiday 10-06
  -> cell has `public_holiday` and `cell_fill_key` "public-holiday"; label gains the public
  holiday text; unmarked cells keep `""`.
- `r78_term_day_off_wins_over_public_holiday_fill`: 10-14 is a midterm and a public holiday ->
  `cell_fill_key` "midterm", `public_holiday` still set (the panel shows both lines).
- `r78_calendar_legend_has_public_holiday_last`: `calendar_legend()` texts are Holiday, Mid exam,
  Final exam, Public holiday (keys `holiday`, `midterm`, `final`, `public-holiday`);
  `day_off_legend()` still has three (the M15 test stays).

Existing tests stay green unchanged apart from the new `CalendarDay.public_holiday` field in any
literal the tester finds.

## Complexity Tracking

None. No constitution violation.

## Analysis report (speckit-analyze, read-only)

Artifacts compared: `docs/spec.md` §13.11 (+ R5, R6, R27, R71, R74, R88-R89a), this plan, tasks.md,
decisions.md, quickstart.md, constitution, DESIGN.md ("Mini calendar", "Day panel lines", "Public
holidays card"). Code facts re-checked: `Settings` is `Copy`; `backup.rs` exports and restores every
`setting` row and its restore `DELETE` list has no `public_holiday`; no test hard-codes the schema
version count (D19); `LinkButton`, `ComboField`, `SectionCard`, `CheckBox` exist; the
`calendar-public-holiday` token exists in `theme.slint` and `DESIGN.md`.

| ID | Category | Severity | Location | Summary | Resolution |
|---|---|---|---|---|---|
| A1 | Inconsistency | LOW | §13.11 "next free number" vs repo | 001-005 exist, so 006 | 006 now; renumber after the final rebase, keep `SCHEMA_VERSION` and migration tests consistent (D1, D19). T012 checks. |
| A2 | Underspecification | MEDIUM | R76 "saved in setting" vs `Settings` | `Settings` is `Copy`; a string field breaks every literal | Two Planner methods and a `SETTING_*` constant; no `Settings` change (D17). |
| A3 | Underspecification | MEDIUM | R80 + D2 backup version | A new migration could break older-backup restores or need a version constant | No logic change; version <= `SCHEMA_VERSION` still restores; test `t77_older_backup_still_restores` (D19, D24). |
| A4 | Underspecification | LOW | R78 "both lines" | Order and multiple names on one date | Day off line first, then the public holiday line; names joined with "; " (D32). |
| A5 | Underspecification | LOW | R79 "how many were added and skipped" | Wording, zero-added case, repeated date | D11 texts; `added == 0` writes nothing and still toasts; a repeated ticked date counts as skipped (D34). |
| A6 | Underspecification | MEDIUM | R77 "failure keeps cached rows" | A 200 answer that is not a calendar would wipe the cache | `fetch_events` requires `BEGIN:VCALENDAR` (D26); test `r77_unknown_code_and_non_calendar_body_are_errors`. |
| A7 | Coverage | LOW | T79 (app) | Status and Retry are Slint bindings, no headless UI test | Text and flag come from `vm::public_holiday_status_text` / `public_holiday_retry_visible` and are tested; Slint only binds (D29). |
| A8 | Coverage | LOW | "Updated today", "once per launch" | Launch and Retry triggers live in page glue, not unit-testable without the UI | The decision inputs are pure (`resolve_country`, `term_years`, `fetch_events`); `quickstart.md` covers the triggers by hand (D41). |
| A9 | Inconsistency | LOW | plan.md test list | A draft line read "five texts? four" and a backup test described its assertion awkwardly | Remediated: legend test lists the four texts and keys; backup test checks no top-level `public_holiday` key (D43). |
| A10 | Inconsistency | MEDIUM | plan.md page glue vs D36 | `changed_then_toast` was described two ways (made `pub(super)`, or re-implemented in the glue) | Remediated: one way, `pub(super)` and reuse (D36, D43). |
| A11 | Inconsistency | MEDIUM | plan.md T78 test text | The session set and the `today` value that make R5 delete the 01-19 session were not stated | Remediated: synced fixture, `today` 2026-01-05, sessions listed (D43). |
| A12 | Inconsistency | MEDIUM | tasks.md T005 | `mod.rs` declaring every `pub mod` before the files exist would not compile | Remediated: T005 declares only the test module; each of T006-T008 adds its own `pub mod` (D43). |
| A13 | Constitution | LOW | `.slint` fill-key ternary and legend `if i < 3` | Binding or logic? | Same pattern as the M15 day off fills: key strings come from Rust, `.slint` maps key to token; no violation. |
| A14 | Risk | LOW | D40 calendar ids | Ids are unverified guesses (no network) | Listed in D40 and quickstart step 8; a wrong id only shows the failure status. The builder records verified ids. |
| A15 | Risk | LOW | dashboard refresh reads the setting and the OS locale every 60 s | Small repeated work | Accepted: one SQLite read and one locale call per refresh. |
| A16 | Coverage | LOW | R80 | No code beyond a doc comment | `t77_backup_has_no_public_holidays` and `t77_restore_leaves_cached_rows_alone` guard it. |

Coverage summary:

| Requirement | Has task? | Task IDs | Notes |
|---|---|---|---|
| Migration / table | yes | T001, T003, T012 | `t77_*`, `migrations.rs` |
| R76 country from OS region, setting, choice | yes | T002, T003, T007, T009, T010 | `t76_*`, `t77_country_setting_roundtrip` |
| R77 fetch once per launch, on country change, replace, keep on failure, nothing with no term or Off | yes | T001, T002, T003, T008, T010 | `t77_*`, `t79_*`, `r77_*` |
| R78 fill, legend, day panel, both lines | yes | T002, T009, T011 | `r78_*` |
| R79 list, checkboxes, Add selected, added/skipped result, no skipping before add | yes | T001, T002, T009, T010 | `t78_*`, `r79_*` |
| R80 cache not in backup or CSV, restore leaves it | yes | T001, T004 | `t77_backup_*`, `t77_restore_*` |
| R81 pure parser with saved sample feeds | yes | T002, T006 | `t75_*` |
| R88/R89/R89a toast, inline errors | yes | T002, T009, T010 | `r79_added_text_*`, `r79_tick_hint_text` |
| API (`PublicHoliday`, two Planner methods) | yes | T001, T003 | `api_surface.rs` |

Constitution alignment issues: none. Unmapped tasks: none (T012 = gate).

Metrics: 6 rules (R76-R81) and 5 tests (T75-T79); 12 tasks; coverage 100%; ambiguities 5 (A2-A6,
resolved in decisions.md); duplications 0; critical 0; high 0; remediated 4 (A9-A12).
Open questions for the human: none blocking. The two things to check by hand are the unverified
calendar ids (D40, quickstart step 8) and the real network fetch (D15).
Next action: implementation (tester T001-T002, then builder T003-T012).
