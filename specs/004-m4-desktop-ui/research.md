# Research: M4 Desktop UI

All decisions below fill details that `docs/spec.md` §1–§6, `DESIGN.md`, and spec.md leave open.
None changes a rule, the schema, or the §5 API. Two questions went to the human on 2026-10-29 and
are answered: Q1 (no file-database test; the pre-restore copy's file behavior is checked manually, router
decision under the human's delegation, chosen over a constitution amendment) and Q2 (suffix on a
same-minute name clash).

## R-D1 Clock read in the app

- **Decision**: The app reads the clock in exactly one function, `fn local_now() -> NaiveDateTime`
  in `crates/app/src/main.rs`, implemented with `chrono::Local::now().naive_local()` (minute
  precision is applied by callers where needed) and annotated
  `#[allow(clippy::disallowed_methods)] // §1: the app is the only place that reads the clock`.
  Everything else receives `now`/`today` from this function (the controller stores it as a
  `fn() -> NaiveDateTime`).
- **Rationale**: `clippy.toml` is at the workspace root, so its `disallowed-methods` apply to every
  crate, including `app`; its own comment says the app's single clock read is allowed at its call
  site. The workspace `chrono` has `default-features = false` and no `clock` feature, so
  `crates/app/Cargo.toml` uses `chrono = { workspace = true, features = ["clock"] }`. Feature
  unification also enables `clock` for core's build, but the clippy ban still covers core.
- **Alternatives considered**: a crate-level `clippy.toml` in `crates/app` (clippy does not merge
  config files; it would have to repeat or drop the ban for the whole app, which is wider than one
  call site); `std::time::SystemTime::now` (gives UTC; local naive time still needs a time-zone
  source).

## R-D2 File dialogs (`rfd`)

- **Decision**: Add `rfd` (latest release that builds on Rust 1.90; the builder records the exact
  version from `cargo add`) with default features. Use the synchronous `rfd::FileDialog` from Slint
  callbacks (they run on the main thread, which macOS requires). Backup: `save_file()` with a
  `json` filter and default name from `viewmodel::backup_file_name(now)`. Restore: `pick_file()`
  with a `json` filter. Export CSV: `pick_folder()`. No start directory is set. Cancel (`None`)
  does nothing.
- **Rationale**: §1 fixes `rfd`. Its default Linux backend is the XDG desktop portal (pure Rust), so
  the Linux CI job needs no new system packages. If the build in CI fails for a missing system
  library, add it to the existing `apt-get install` line.
- **Alternatives considered**: the GTK3 backend (needs `libgtk-3-dev` in CI); async dialogs (no
  benefit for a single-window app).

## R-D3 R22 pre-restore copy mechanism

- **Decision**: `restore::pre_restore_copy(db_path, now) -> anyhow::Result<PathBuf>` creates
  `{db_path.parent()}/backups/` if missing, then opens a second `rusqlite::Connection` to `db_path`
  and runs `VACUUM INTO '{target}'`. The target name is
  `viewmodel::free_pre_restore_name(now, existing)`, normally `pre-restore-{YYYYMMDD-HHMM}.sqlite`
  (R-D4). Only after it
  returns `Ok` does the flow call `Planner::restore_json`.
- **Rationale**: `VACUUM INTO` reads the database through SQLite, so it includes every committed
  transaction still in the `-wal` file, and writes one self-contained file (FR-025). A second
  connection is fine in WAL mode while the `Planner` connection is idle between calls. It needs
  `rusqlite` in the app (workspace dependency, `bundled` SQLite has `VACUUM INTO`); no core API is
  added.
- **Alternatives considered**: `std::fs::copy` of the main file (misses WAL-only commits); dropping
  the `Planner` to force a checkpoint, copying, then reopening (relies on a clean checkpoint on
  close and briefly leaves the app without a database); rusqlite's online backup API (needs an extra
  feature flag; same effect).

## R-D4 Backups folder and name collisions

- **Decision**: `{data_dir}` for R22 is the folder that holds the database file (`db_path.parent()`).
  With the default R24 path that is the `ProjectDirs` data dir; with `ACADEMIC_PLANNER_DB` set it is
  the override file's folder.
- **Decision (human, Q2 = A, 2026-10-29)**: `VACUUM INTO` fails when the target exists, which
  happens for two restores in the same minute. On a clash the name gets a suffix `-2`, `-3`, …
  before `.sqlite` (`pre-restore-20261029-1430-2.sqlite`), so no earlier copy is lost. The pure
  function `viewmodel::free_pre_restore_name(now, existing)` picks the first free name and is
  unit-tested; `restore.rs` passes it the file names already in the backups folder.
- **Alternatives considered**: overwrite the earlier copy (loses the pre-first-restore data);
  refuse the restore until the next minute.

## R-D5 State ownership and threading

- **Decision**: One `Rc<App>` on the UI thread holds `RefCell<Planner>`, the DB path, the clock
  function, `Weak<AppWindow>`, and a `RefCell<UiState>` (last sync date, selected week start,
  show-archived flag, open course detail id, assessment filter, edit ids, pending restore path,
  index→id lists for combo boxes). Every core call is synchronous.
- **Rationale**: Data is small; `rusqlite::Connection` is not `Sync`; Slint callbacks run on the UI
  thread.
- **Alternatives considered**: worker thread with `invoke_from_event_loop` (needless complexity).

## R-D6 Slint structure: one global per page

- **Decision**: Each page file exports a page component and a global (`DashboardState`,
  `TimetableState`, `CoursesState`, `AssessmentsState`, `SettingsState`) holding its `in`
  properties (rows, error texts), `in-out` form text properties, and callbacks. `types.slint` holds
  shared structs/enums and `global AppState` (current page, startup error). `app-window.slint`
  imports every page and re-exports all globals so Rust reaches them with
  `window.global::<DashboardState>()`. Rust side: `src/pages/<page>.rs` has `refresh(&App)` and
  `wire(&Rc<App>)`.
- **Rationale**: Page tasks then touch disjoint files (one `.slint` + one `.rs`) and can run in
  parallel; `app-window.slint` and `controller.rs` are written once with stub globals.
- **Alternatives considered**: all properties on `AppWindow` (every page task edits the same file).

## R-D7 Widgets and style

- **Decision**: Build NavItem, PrimaryButton, SecondaryButton, Field (label + `TextInput` in a
  themed `Rectangle` + error text), StatusChip, SectionCard, ListRow, EmptyState, Banner, and
  PageTitle in `components.slint` from base elements so every color, size, and radius comes from
  `Theme.*`. Use `std-widgets` only for `ScrollView`/`ListView`, `ComboBox`, `CheckBox`, and
  `Switch`. `build.rs` compiles with style `fluent-light` so standard widgets render in a light
  theme on every OS.
- **Rationale**: DESIGN.md prescribes exact field, button, and chip visuals that standard widgets
  cannot take; light theme only. Any mismatch in the four standard widgets is left to the
  end-of-M4 critique.
- **Alternatives considered**: all std-widgets (can't apply tokens); all custom (combo box and
  scrolling are costly to rebuild).

## R-D8 Static text vs. formatted text

- **Decision**: Fixed copy (page titles, section headings, button labels, field labels,
  empty-state sentences, the restore warning) is written in the `.slint` files. Anything computed
  from data (percentages, "—", status labels, dates, times, row text, summary and banner lines,
  week titles, enablement flags, error messages) comes from `viewmodel.rs`.
- **Rationale**: §2 puts formatting in the view model; fixed labels are bindings, not formatting.

## R-D9 Date and time parsing

- **Decision**: Input is trimmed. A date must match exactly four digits, `-`, two digits, `-`, two
  digits and be a real calendar date; error text: `Use YYYY-MM-DD, e.g. 2026-10-29`. A time must
  match exactly two digits, `:`, two digits, 00:00–23:59; error text: `Use HH:MM, e.g. 09:30`.
  An empty required date or time field gets the same format message (FR-007: the error states the
  expected format). Optional fields (slot valid-to, holiday end date) accept empty
  as `None`.
- **Rationale**: §6 "validated as `YYYY-MM-DD` / `HH:MM`"; DESIGN.md "the message states the
  expected format". Strict shape avoids chrono accepting unpadded values.

## R-D10 Number fields

- **Decision**: Attendance target: empty → `None`; otherwise must parse as a whole number
  (`Enter a whole number`), range 0–100 is left to core R2 so its message is shown. Assessment
  reminder minutes: empty → `None`; otherwise a non-negative whole number. Settings reminder
  minutes and upcoming window days: required non-negative whole numbers.
- **Rationale**: R2 owns the range rule; the view model only turns text into the input type.

## R-D11 Error display

- **Decision**: `viewmodel::error_text(&planner_core::Error)`: `Validation(msg)` → `msg` with its
  first letter capitalized; every other variant → its `Display` text. Parse errors go to the field's
  error property; core errors go to the form's `form-error` property (or the page's `message` for
  row actions). Nothing panics; `anyhow` errors from files/dialogs use their `Display` text.

## R-D12 Inline panels instead of modal dialogs

- **Decision**: All forms, the slot edit/end prompts, the reschedule form, and the restore confirm
  are inline panels (§6 allows "inline panels or dialogs").
- **Rationale**: DESIGN.md gives no backdrop color or shadow token for dialogs; inline panels need no
  new token and keep `theme.slint` a pure translation.

## R-D13 Theme translation

- **Decision**: `export global Theme` with `out property` per token, names kebab-case as in
  DESIGN.md: every color (`primary`, `primary-hover`, …, `below-target-bg`); typography as
  `<name>-size` (length), `<name>-weight` (int), and two families `font-heading: "Georgia"` and
  `font-body: ""` (empty = platform default, which also is the Georgia fallback); radii
  `radius-none/sm/md/full`; spacing `space-xs…space-xxl`; `nav-width`, `row-height`,
  `input-height`, `window-width`, `window-height`; `divider-width: 1px`; `border-width: 1px`.
  Line heights are recorded as comments (Slint `Text` has no line-height property). The existing
  `neutral`, `window-width`, `window-height` names are kept. CI: add a step to the `check` job
  running the exact §12.3 command.
- **Rationale**: §12.3 hand translation; one source for every value.

## R-D14 Refresh strategy and sync triggers

- **Decision**: `App::refresh_all()` reloads every page global from core with a fresh `now`. It runs
  after every mutation, on page switch, and on every 60-second tick. `App::changed(sync: bool)`
  runs `sync_sessions(today)` first when `sync` is true. Sync triggers (R6): app start, save term,
  add/remove holiday, create/update course, archive/unarchive, add/change/end slot, successful
  restore, date rollover. No sync after status changes, reschedule, makeup, assessments, or
  settings.
- **Rationale**: FR-009 asks to refresh the visible page; reloading all pages includes it, costs
  little, and keeps cross-page data consistent.
- **Sync failure**: the mutation before it is already committed (§1) and stands; the sync error is
  shown in the current page's `message` and the next trigger retries.

## R-D15 Timer (R6)

- **Decision**: A `slint::Timer` in `TimerMode::Repeated` every 60 s calls `App::tick()`:
  `now = clock()`; if `viewmodel::date_rolled_over(last_sync, now)` then sync with `now.date()` and
  store it; then `refresh_all()`. The timer is kept alive for the window's lifetime in `main.rs`.

## R-D16 Defaults in forms

- **Decision**: Slot valid-from pre-fills with the active term's start date; slot edit's effective
  date and the end-slot last date pre-fill with today (R7 default for the effective date); reschedule
  pre-fills the original's date, times, and room; the holiday end date starts empty (single day).
  Week view starts at the week of today. Backup default name:
  `academic-planner-backup-{YYYYMMDD-HHMM}.json`.

## R-D17 Week view grouping

- **Decision**: Only weekdays that have sessions are listed, Monday first, each headed
  "Monday 2026-10-26". A week with no sessions shows its empty-state sentence. Title:
  "2026-10-26 – 2026-11-01".

## R-D18 IDs in Slint

- **Decision**: Slint `int` is `i32`. `bridge.rs` converts `i64` ids with `i32::try_from`; a failure
  is reported as an error message (not reachable with realistic data). Combo boxes map selected
  index → id through lists kept in `UiState`.

## Test strategy

- View model: every public function in [contracts/viewmodel.md](contracts/viewmodel.md) has at
  least one test in `src/viewmodel/tests.rs`; tests construct core types directly (no database).
- Pre-restore copy: the file name and same-minute suffix (Q2 = A) are pure view-model functions
  with unit tests. `pre_restore_copy` itself is a thin file-system wrapper with no automated test
  (Principle II: tests use in-memory SQLite only, Q1 = B); quickstart §6 checks by hand that the
  copy holds WAL-committed data and that a same-minute second copy gets `-2`. All app tests use no
  database or in-memory SQLite only.
- No automated UI tests: screens are checked by §9 smoke and the §12.3 screenshot critique.
