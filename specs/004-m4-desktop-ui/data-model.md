# Data Model: M4 Desktop UI

M4 adds **no stored entities** and no migration; the schema of `001_init.sql` and every §5 type are
used as they are. This file lists the in-memory shapes the app adds on top of them. Exact Rust
signatures are in [contracts/viewmodel.md](contracts/viewmodel.md); Slint shapes are in
[contracts/ui-bridge.md](contracts/ui-bridge.md).

## Theme (ui/theme.slint)

`export global Theme`, one `out property` per `DESIGN.md` token (research R-D13):

| Group | Properties |
| --- | --- |
| Colors (21) | `primary`, `primary-hover`, `on-primary`, `neutral`, `surface`, `surface-alt`, `surface-selected`, `border`, `on-surface`, `on-surface-muted`, `error`, `status-attended`, `status-attended-bg`, `status-absent`, `status-absent-bg`, `status-cancelled`, `status-cancelled-bg`, `status-pending`, `status-pending-bg`, `below-target`, `below-target-bg` |
| Typography | `font-heading` ("Georgia"), `font-body` ("" = platform default); for each of `title-lg`, `headline-md`, `body-md`, `body-sm`, `label-md`, `label-sm`: `<name>-size` (length) and `<name>-weight` (int); line heights as comments |
| Radii | `radius-none`, `radius-sm`, `radius-md`, `radius-full` |
| Spacing | `space-xs`, `space-sm`, `space-md`, `space-lg`, `space-xl`, `space-xxl` |
| Sizes | `nav-width`, `row-height`, `input-height`, `window-width`, `window-height`, `divider-width`, `border-width` |

## View-model rows (src/viewmodel.rs)

Display-ready, all strings already formatted. Built from core types; no database access.

| Row | Built from | Notes |
| --- | --- | --- |
| `SessionRow` | `SessionView` | R11–R12: `status_label` (core `display_status`, incl. "Rescheduled → {date}"), `chip`, `can_mark` = `is_past`, read-only `note` |
| `DayGroup` | `[SessionView]` of one week | One per date with sessions, title "Monday 2026-10-26" |
| `SummaryView` | `AttendanceSummary` | R13–R15: counts, `pct` or "—", `target` or "—", `below_target` |
| banner line (`String`) | `AttendanceSummary` | R14: "CSE101 — 66.7% (target 75%)" |
| `DashboardView` | `Dashboard` | R16–R20 rows + banner lines |
| `CourseRow` | `Course` | R3: `archived` flag; optional fields "—" |
| `SlotRow` | `Slot` + `[Course]` | weekday name, time range, course code, validity text |
| `AssessmentRow` | `Assessment` | R18–R19 lists; due "YYYY-MM-DD HH:MM", `completed` |
| `HolidayRow` | `Holiday` | label or "—" |
| `ChipKind`, `FilterChoice` | `Status`, filter index | enums |

## Form inputs (text → core input)

Each `parse_*_form` turns the form's text fields into the matching §5 input type or a
`FormErrors` (field → message). Format errors are the view model's; R2 rule errors come from core.

| Form | Produces | Fields validated in the view model |
| --- | --- | --- |
| Term | `TermInput` | start, end (`YYYY-MM-DD`) |
| Course | `CourseInput` | target (whole number) |
| Slot (add / edit) | `SlotInput` (+ effective date for edit, R7) | course, weekday, start, end (`HH:MM`), valid from, valid to (optional) |
| Makeup (reschedule R9 / add makeup R10) | `MakeupInput` | date, start, end |
| Assessment | `AssessmentInput` | course, kind, due date, due time, reminder minutes (optional) |
| Holiday | `Vec<Holiday>` | start, end (optional; `end ≥ start`) — inclusive expansion, one per day |
| Settings | `Settings` | reminder minutes, upcoming window days |

### Holiday list transitions (human decision §6)

- **Add** (single date or range): `new = merge_holidays(term.holidays, expand(start, end, label))`
  → `set_holidays(term.id, &new)` → sync (R6). A date already present takes the new label; each
  date appears once.
- **Remove**: `new = remove_holiday(term.holidays, date)` → `set_holidays` → sync.

## UI state (src/controller.rs)

`App` (one per process, `Rc`, UI thread):

| Field | Purpose |
| --- | --- |
| `planner: RefCell<Planner>` | the open database (R24 path) |
| `db_path: PathBuf` | shown in Settings (R24); parent folder holds `backups/` (R22) |
| `clock: fn() -> NaiveDateTime` | `main.rs::local_now`; the only clock read |
| `window: slint::Weak<AppWindow>` | refresh target |
| `state: RefCell<UiState>` | below |

`UiState`:

| Field | Initial | Changed by |
| --- | --- | --- |
| `last_sync: NaiveDate` | today at start | every sync; R6 rollover compares against it |
| `week_start: NaiveDate` | `week_start(today)` | prev / next / today |
| `show_archived: bool` | false | course list toggle (R3) |
| `detail_course: Option<i64>` | None | open / close course detail |
| `filter: FilterChoice` | Upcoming | assessments filter |
| `selected_session: Option<i64>` | None | week view selection |
| `course_ids: Vec<i64>` | from `list_courses(false)` | each refresh (combo index → id) |
| `pending_restore: Option<PathBuf>` | None | restore pick / cancel / confirm |

## Files written by the app (user action only)

| File | Trigger | Content |
| --- | --- | --- |
| user-chosen `.json` (default `academic-planner-backup-{YYYYMMDD-HHMM}.json`) | Backup | `export_json(now)` (R21) |
| `{folder}/attendance.csv`, `{folder}/assessments.csv` | Export CSV | R23 |
| `{db folder}/backups/pre-restore-{YYYYMMDD-HHMM}[-N].sqlite` | Restore confirm, before `restore_json` | full DB copy incl. WAL-committed data (R22, FR-025, Q2) |
