# Contract: Slint ↔ Rust bridge

How the `.slint` files and the Rust app talk. Property and callback names are the contract between a
page's `.slint` file and its `src/pages/*.rs` module; the page task may add properties it needs, as
long as both files change together and no other page's files are touched. Static copy (titles,
labels, empty-state sentences) lives in `.slint`; every data-derived string comes from
[viewmodel.md](viewmodel.md).

## Shared: `ui/types.slint`

```slint
export enum Chip { attended, absent, cancelled, pending }
export enum StatusAction { attended, absent, cancelled, pending }

export struct SessionRow { id: int, date: string, weekday: string, time: string, course: string,
    room: string, status-label: string, chip: Chip, can-mark: bool, note: string }
export struct DayGroup { title: string, rows: [SessionRow] }
export struct AssessmentRow { id: int, course-code: string, kind: string, title: string,
    due: string, completed: bool, notes: string }
export struct CourseRow { id: int, code: string, name: string, instructor: string, room: string,
    target: string, archived: bool }
export struct SlotRow { id: int, weekday: string, time: string, course: string, room: string,
    valid: string }
export struct HolidayRow { date: string, label: string }
export struct SummaryView { attended: string, absent: string, cancelled: string, pending: string,
    pct: string, target: string, below-target: bool }

export global AppState {
    in-out property <int> current-page;      // 0 Dashboard, 1 Timetable, 2 Courses, 3 Assessments, 4 Settings
    in property <string> startup-error;      // FR-008: non-empty → only the error screen is shown
    callback page-selected(int);             // Rust refreshes all pages (fresh `now`)
}
```

`bridge.rs` converts `viewmodel::SessionRow` etc. into these structs (field copies, `i64 → i32`
ids, `ChipKind → Chip`) and `StatusAction → planner_core::Status`. No formatting happens there.

## Shared: `ui/components.slint`

Built from base elements, every value from `Theme.*`: `NavItem`, `PrimaryButton`,
`SecondaryButton` (both with `enabled`), `Field` (label, `in-out text`, `error` string shown under
the field in the error color, error border when non-empty), `StatusChip` (`chip`, `text`; text is
always shown), `SectionCard` (headline title), `ListRow`, `EmptyState` (`text`), `Banner`
(`lines: [string]`), `PageTitle`.

## `ui/app-window.slint`

`AppWindow` imports `theme.slint`, `types.slint`, and every page; re-exports `AppState` and the five
page globals; shows the nav column (`Theme.nav-width`) with five `NavItem`s and the current page, or
only the startup error text when `AppState.startup-error` is non-empty.

## Page globals

Each page file exports `<Name>Page` and `global <Name>State`. Every page global has
`in property <string> message` for row-action errors and success notes.

### DashboardState (`ui/dashboard.slint`, `src/pages/dashboard.rs`)

- In: `banner: [string]`, `today: [SessionRow]`, `has-next: bool`, `next-class: SessionRow`,
  `upcoming: [AssessmentRow]`, `overdue: [AssessmentRow]`, `awaiting: [SessionRow]`.
- Callbacks: `set-status(int, StatusAction)` (awaiting rows: Attended / Absent disabled unless
  `can-mark`; Cancelled always enabled).
- Rust: `refresh` calls `dashboard(now)` → `dashboard_view`; `set-status` calls
  `set_status(id, status, now)` then `refresh_all` (no sync).

### TimetableState (`ui/timetable.slint`, `src/pages/timetable.rs`)

- In: `tab: int` (in-out; 0 Week, 1 Rules), `week-title: string`, `days: [DayGroup]`,
  `selected: SessionRow`, `has-selected: bool`, `slots: [SlotRow]`, `course-options: [string]`.
- In-out form text: reschedule (`rs-date`, `rs-start`, `rs-end`, `rs-room`, `rs-note`), slot form
  (`slot-course`, `slot-weekday` indexes, `slot-start`, `slot-end`, `slot-room`, `slot-from`,
  `slot-to`, `slot-effective`, `slot-editing-id`), end form (`end-slot-id`, `end-date`).
- In: error properties per field (`rs-date-error`, …, `slot-form-error`, `end-date-error`).
- Callbacks: `prev-week()`, `next-week()`, `this-week()`, `select-session(int)`,
  `set-status(int, StatusAction)`, `reschedule(int)`, `begin-add-slot()`, `begin-edit-slot(int)`,
  `save-slot()`, `begin-end-slot(int)`, `end-slot()`.
- Rust: week from `UiState.week_start`; `sessions_between(start, end, now)` → `group_week`;
  `list_slots(today)` + `list_courses(false)` → `slot_row`. Status: `set_status` (no sync); the
  selected session offers Attended, Absent (disabled unless `can-mark`), Pending, and Cancel (R8,
  R11, R12). Reschedule: `parse_makeup_form` with the selected session's course id → `reschedule`
  (no sync; an attended/absent original returns the core error, shown in `message`, R9). Save slot:
  `parse_slot_form` → `add_slot` or, when editing, `change_slot(id, input, effective)` with
  `effective` parsed by `parse_date` (default today, R7) → sync. End: `end_slot(id, last_date)` →
  sync. The selected session shows its stored note read-only.

### CoursesState (`ui/courses.slint`, `src/pages/courses.rs`)

- In: `show-archived: bool` (in-out), `courses: [CourseRow]`, `detail-open: bool`,
  `detail-title: string`, `summary: SummaryView`, `sessions: [SessionRow]`.
- In-out form text: course form (`code`, `name`, `instructor`, `room`, `notes`, `target`,
  `editing-id`), makeup form (`mk-date`, `mk-start`, `mk-end`, `mk-room`, `mk-note`).
- In: `target-error`, `course-form-error`, `mk-date-error`, `mk-start-error`, `mk-end-error`,
  `mk-form-error`.
- Callbacks: `toggle-archived(bool)`, `begin-create()`, `begin-edit(int)`, `save-course()`,
  `set-archived(int, bool)`, `open-detail(int)`, `close-detail()`, `set-status(int, StatusAction)`,
  `add-makeup()`.
- Rust: `list_courses(show_archived)`; detail: `attendance_summary(id)` + `sessions_for_course(id,
  now)`. Create/update/archive/unarchive → sync (R6). Status → no sync. Add makeup:
  `parse_makeup_form` → `add_makeup` (R10, no sync).

### AssessmentsState (`ui/assessments.slint`, `src/pages/assessments.rs`)

- In: `filter: int` (in-out), `list-title: string`, `empty-text: string`, `rows: [AssessmentRow]`, `course-options: [string]`.
- In-out form text: `course`, `kind` (indexes), `title`, `due-date`, `due-time`, `notes`, `remind`,
  `editing-id`.
- In: `course-error`, `kind-error`, `due-date-error`, `due-time-error`, `remind-error`, `form-error`.
- Callbacks: `set-filter(int)`, `begin-create()`, `begin-edit(int)`, `save()`, `set-completed(int,
  bool)`, `delete(int)`.
- Rust: `list_assessments(assessment_filter(choice, now))`; course options from
  `list_courses(false)`; no sync.

### SettingsState (`ui/settings.slint`, `src/pages/settings.rs`)

- In-out form text: term (`term-name`, `term-start`, `term-end`), holiday (`hol-start`, `hol-end`,
  `hol-label`), settings (`reminder`, `window`, `notifications: bool`).
- In: `has-term: bool`, `holidays: [HolidayRow]`, `db-path: string`, `confirm-restore: bool`,
  errors (`term-start-error`, `term-end-error`, `term-form-error`, `hol-start-error`,
  `hol-end-error`, `hol-form-error`, `reminder-error`, `window-error`, `settings-form-error`).
- Callbacks: `save-term()`, `add-holiday()`, `remove-holiday(string)`, `save-settings()`,
  `backup()`, `restore-pick()`, `restore-confirm()`, `restore-cancel()`, `export-csv()`.
- Rust: term → `save_term` (id = active term id when editing) → sync. Holidays:
  `parse_holiday_form` → `merge_holidays` with the active term's holidays →
  `set_holidays(term_id, …)` → sync; remove → `remove_holiday` → `set_holidays` → sync. Holiday add
  without an active term shows `error_text(&Error::NoActiveTerm)`. Settings → `save_settings` (no
  sync). Backup → `rfd` save dialog (default name `backup_file_name(now)`) → `export_json(now)` →
  write file (R21). Export CSV → folder dialog → `attendance.csv`, `assessments.csv` (R23). DB path
  via `db_path_text` (R24). Restore → see below.

## Restore flow (`src/restore.rs`, wired from `settings.rs`)

1. `restore-pick`: `rfd` open dialog (json). Cancel → nothing. Else store the path in
   `UiState.pending_restore` and set `confirm-restore = true` (inline panel with the warning that
   all current data will be replaced).
2. `restore-cancel`: clear the pending path, hide the panel. Nothing copied.
3. `restore-confirm`: read the file to a string (error → message, stop). Call
   `restore::pre_restore_copy(db_path, now)` (error → message, stop, no restore). Call
   `restore_json(&json)` (error → `error_text`, data unchanged). On success: sync (R6),
   `refresh_all`, message `restored_message(copy_path)`.

`pub fn pre_restore_copy(db_path: &Path, now: NaiveDateTime) -> anyhow::Result<PathBuf>`: creates
`{db_path.parent()}/backups`, lists the file names already in that folder, picks the target name
with `viewmodel::free_pre_restore_name(now, &existing)` (Q2 = A), runs `VACUUM INTO` on a second
`rusqlite::Connection` to `db_path`, and returns the copy's path.

## Controller (`src/controller.rs`)

`pub struct App` (see data-model.md `UiState`). Public methods used by page modules:
`now()`, `today()`, `planner()` / `planner_mut()` (RefCell borrows), `window()`, `state()`,
`changed(sync: bool)` (sync with today if `sync`, then `refresh_all`), `refresh_all()` (calls
every `pages::*::refresh`), `tick()` (R6 rollover check, then `refresh_all`). Sync errors are shown
in the visible page's `message`.
