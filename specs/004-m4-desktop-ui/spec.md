# Feature Specification: M4 Desktop UI

**Feature Branch**: `004-m4-desktop-ui` (spec directory; work happens on the current branch)

**Created**: 2026-10-29

**Status**: Draft

**Input**: User description: "Milestone M4 from docs/spec.md: ui/theme.slint from DESIGN.md first,
then the full UI (§6) wired to the core API (§5); view-model formatting functions unit-tested; sync
triggers per R6; R22 pre-restore DB file copy in the app restore flow; holidays as a single date
or an inclusive date range; theme grep check in CI; §9 smoke checklist. Do not add requirements."

**Source of truth**: `docs/spec.md` §1 (fixed decisions), §2 (layout and the formatting rule),
§4 (rules R1–R25 as surfaced in the UI; R6 and the R22 file copy are implemented here), §5 (core
API, frozen), §6 (UI), §7 (M4 row), §9 (smoke checklist), §12.3 (DESIGN.md, theme grep check,
screenshot critique), and `DESIGN.md` (visual tokens and component rules). This spec restates that
scope and adds nothing to it beyond these human decisions, all dated 2026-10-29 and already written
into `docs/spec.md` §6 or recorded in earlier milestone specs:

- The R22 pre-restore database file copy is built in the app restore flow (M3 FR-014).
- Holidays can be added as a single date or an inclusive date range with one label, expanded into
  one holiday per day; removal is per day (exam periods are entered as holidays, M2 FR-006a).
- A session note is entered only in the reschedule form and stored on the makeup session; stored
  notes are shown read-only. The §5 API has no separate note setter and none is added.
- Course detail offers an "Add makeup class" form for R10 (one-off makeup, no replaced session).

None of these changes a rule, the schema, or the §5 API.

## User Scenarios & Testing *(mandatory)*

The user is a student who opens the app between classes to mark attendance, check what is due,
and keep their timetable current. Logic and formatting are verified by unit tests on pure view-model
functions; the screens themselves are verified by the human §9 smoke checklist and a screenshot
critique against `DESIGN.md`. Every scenario below uses the core's existing behavior; the UI only
calls the §5 API, passing the current date and time in.

### User Story 1 - Set up a term, courses, and a weekly timetable (Priority: P1)

A student creates the active term (name, start, end), adds courses (code, name, optional
instructor, room, notes, attendance target), and adds the recurring weekly class slots for each
course. The app generates the term's class sessions immediately.

**Why this priority**: Nothing else in the app has data until a term, courses, and slots exist. It
is the first half of the §9 smoke checklist.

**Independent Test**: On a fresh database, create a term in Settings, two courses in Courses, and
three slots in Timetable → Rules; the Timetable week view for the current week shows the generated
sessions grouped by weekday.

**Acceptance Scenarios**:

1. **Given** a fresh database, **When** the student opens the app, **Then** every page shows an
   empty state, and the Settings page lets them create a term (R1).
2. **Given** a term form with end before start, or a date not in `DD-MM-YYYY`, **When** the student
   saves, **Then** an inline error appears under the offending field and nothing is saved (R2).
3. **Given** an active term, **When** the student creates a course with an empty code or name, a
   duplicate code, or a target outside 0–100, **Then** an inline error is shown and nothing is
   saved (R2).
4. **Given** a course, **When** the student adds a slot (weekday, start, end, optional room, valid
   from, optional valid to) with end not after start, or a time not in `HH:MM`, **Then** an inline
   error is shown (R2); **when** the slot is valid, **Then** it is saved, sessions are synced, and
   the Timetable week view shows them (R4, R6).
5. **Given** an existing slot, **When** the student edits it, **Then** the form asks for an
   effective date defaulting to today, and past sessions keep their stored date, time, and room
   (R7).
6. **Given** an existing slot, **When** the student ends it on a date, **Then** later pending
   sessions disappear on the next sync and past ones stay (R5).
7. **Given** courses, **When** the student archives one, **Then** it disappears from the dashboard,
   timetable, and default course list, reappears with "show archived" on, and can be unarchived
   (R3).

---

### User Story 2 - Mark attendance and see the percentage (Priority: P1)

From the dashboard, the week view, or a course's detail page, the student marks a past session
Attended or Absent, cancels a session, or resets it to Pending. The course's attendance summary and
percentage update at once.

**Why this priority**: Attendance tracking is the app's core purpose and the center of the §9
smoke checklist ("course shows 50.0%").

**Independent Test**: With one past session marked Attended and one Absent for a course, the
course detail shows 50.0%; view-model unit tests cover percentage formatting and button
enablement.

**Acceptance Scenarios**:

1. **Given** a pending session whose start is in the past, **When** the student clicks Attended or
   Absent, **Then** the status changes and the summary counts and percentage update (R11, R13).
2. **Given** a session that has not started yet, **Then** its Attended and Absent buttons are
   disabled; Cancelled and resetting to Pending remain available (R12).
3. **Given** a course with no attended or absent sessions, **Then** its percentage shows "—", never
   "0%" (R13).
4. **Given** a course with a target whose percentage is below it, **Then** the course shows a
   "Below target" label and the dashboard banner lists it, e.g. "CSE101 — 66.7% (target 75%)"
   (R14).
5. **Given** a course detail page, **Then** it shows counts of attended, absent, cancelled, and
   pending, the percentage, and the full session list with status buttons (R15).
6. **Given** an attended session, **When** the student resets it to Pending, **Then** the counts
   update (R12, T23 behavior).

---

### User Story 3 - See today at a glance on the dashboard (Priority: P1)

On opening the app, the dashboard shows today's classes, the next class, upcoming assessments,
overdue assessments, sessions awaiting attendance with one-click Attended / Absent / Cancelled
buttons, and a banner listing courses below target.

**Why this priority**: It is the landing page and the fastest path to the app's main job.

**Independent Test**: With the §9 data, the dashboard shows an assessment due tomorrow under
upcoming and past unmarked sessions under awaiting attendance; view-model tests cover row
formatting for each section.

**Acceptance Scenarios**:

1. **Given** sessions today for two courses, **Then** "Today's classes" lists them sorted by start
   time with their status labels (R16).
2. **Given** pending future sessions, **Then** "Next class" shows the earliest pending one that has
   not started (R17).
3. **Given** incomplete assessments due within the upcoming window, **Then** they are listed under
   upcoming, sorted by due time; incomplete ones past due are listed under overdue (R18, R19).
4. **Given** pending sessions that have ended, **Then** each is listed oldest first under awaiting
   attendance with Attended / Absent / Cancelled buttons; a session in progress is not listed
   (R20).
5. **Given** any dashboard section with nothing to show, **Then** it shows its empty-state sentence.

---

### User Story 4 - Handle exceptions in the week view (Priority: P2)

The Timetable page shows one week (previous / next / today buttons) with sessions grouped by
weekday. Clicking a session lets the student mark its status, cancel it, or reschedule it to a new
date, time, and room with an optional note. Rescheduled originals read "Rescheduled → {date}". A
course's detail page also lets the student add a one-off makeup class.

**Why this priority**: Cancellations and reschedules happen every term but less often than marking
attendance.

**Independent Test**: Cancel one session and reschedule another in the week view; the original
shows "Rescheduled → 2026-…", the makeup appears on its new date, and a makeup added from course
detail appears in the week view and counts in the summary.

**Acceptance Scenarios**:

1. **Given** the week view, **When** the student clicks previous, next, or today, **Then** the view
   shows that week's sessions (Monday to Sunday) grouped by weekday, each day with its date.
2. **Given** a session, **When** the student cancels it, **Then** it shows "Cancelled" and stays
   cancelled after any later sync (R8).
3. **Given** a pending or cancelled session, **When** the student reschedules it with a valid date,
   start, end, room, and optional note, **Then** a makeup session appears on the new date, the
   original shows "Rescheduled → {date}", and the note is shown read-only on the makeup (R9).
4. **Given** an attended or absent session, **When** the student tries to reschedule it, **Then**
   the core's error is shown and nothing changes (R9).
5. **Given** a course detail page, **When** the student adds a makeup class with a valid date,
   start, end, room, and note, **Then** it appears in the week view and counts like any other
   session (R10).
6. **Given** a week with no sessions, **Then** the view shows its empty-state sentence.

---

### User Story 5 - Track assessments (Priority: P2)

The Assessments page lists assessments with filters upcoming / overdue / completed / all, lets the
student create, edit, and delete them, and has a checkbox to mark one complete.

**Why this priority**: Deadlines matter, but the dashboard already surfaces the urgent ones.

**Independent Test**: Add an assessment due tomorrow; it appears under the "upcoming" filter and on
the dashboard; ticking complete moves it to "completed".

**Acceptance Scenarios**:

1. **Given** the form, **When** the student saves with an empty title or a due date-time not in
   `DD-MM-YYYY` and `HH:MM`, **Then** an inline error is shown and nothing is saved (R2).
2. **Given** a valid form (course, kind quiz / exam / assignment / other, title, due date and time,
   optional notes, optional reminder minutes), **When** saved, **Then** it appears in the matching
   filters.
3. **Given** an assessment, **When** the student ticks complete, **Then** it leaves upcoming and
   overdue and appears under completed; unticking reverses this.
4. **Given** an assessment, **When** the student deletes it, **Then** it is gone from every list.

---

### User Story 6 - Settings, holidays, backup, restore, and export (Priority: P2)

The Settings page edits the term, holidays, reminder minutes, the upcoming window, and the
notifications switch; saves a JSON backup; restores one after a confirm dialog (first copying the
current database file aside); exports both CSV files; and shows the database file location.

**Why this priority**: Holidays and exam periods keep attendance correct; backup and restore are the
second half of the §9 smoke checklist.

**Independent Test**: Add a three-day exam range; the timetable shows no sessions on those days.
Back up, delete data, restore; the data is back and a `pre-restore-*.sqlite` file exists in the
backups folder. Export CSVs and open them in a spreadsheet.

**Acceptance Scenarios**:

1. **Given** the holiday form, **When** the student enters one date and a label, **Then** that date
   becomes a holiday and pending future sessions on it disappear after sync (R4, R5, R6).
2. **Given** the holiday form, **When** the student enters a start date, an end date, and a label,
   **Then** every date from start to end inclusive becomes a holiday with that label (human
   decision, §6).
3. **Given** a range whose end is before its start, or a date not in `DD-MM-YYYY`, **Then** an
   inline error is shown and no holiday is added.
4. **Given** the holiday list, **When** the student removes one date, **Then** only that date is
   removed and sessions are synced again (R6).
5. **Given** reminder minutes, upcoming window days, and the notifications switch, **When** saved,
   **Then** the values persist across restarts, and the dashboard's upcoming section uses the new
   window (R18).
6. **Given** a populated planner, **When** the student clicks Backup and picks a file, **Then** the
   JSON backup is written there (R21).
7. **Given** a backup file, **When** the student clicks Restore and picks it, **Then** a confirm
   dialog warns that all current data will be replaced; on confirm, the current database file is
   copied to `{data_dir}/backups/pre-restore-{YYYYMMDD-HHMM}.sqlite`, then the backup is restored
   and every page shows the restored data (R22).
8. **Given** an invalid or newer-version backup, **When** restore is attempted, **Then** an error
   message is shown and the data is unchanged (R22).
9. **Given** Export CSV, **When** the student picks a folder, **Then** `attendance.csv` and
   `assessments.csv` are written there (R23).
10. **Given** the Settings page, **Then** it shows the database file path as text (R24).

---

### Edge Cases

- **No active term**: pages other than Settings show their empty states; the Settings term form is
  the way in (R1). Syncing with no active term does nothing.
- **Database newer than the app**: the app shows a clear error naming the problem and does not
  open the database (R25).
- **Day rollover while the app is open**: within about a minute of midnight the app syncs sessions
  for the new date and refreshes the visible page (R6).
- **Holiday range overlapping existing holidays**: each date in the range ends up as exactly one
  holiday carrying the range's label.
- **Holiday range outside the term**: dates are stored as given; only dates inside the term affect
  sessions (R4).
- **Restore cancelled at the confirm dialog or file picker**: nothing is copied and nothing changes.
- **Pre-restore copy fails** (e.g. disk full, folder not creatable): the restore is not attempted
  and an error is shown.
- **Any core error** (validation or storage): shown as a message in the relevant panel; the app
  never panics or closes.
- **Long lists** (a full term of sessions on course detail): the list scrolls.
- **Optional fields left empty** (instructor, room, notes, target, reminder minutes, slot valid to):
  stored as absent, shown as "—" or omitted, never as an empty-looking error.

## Requirements *(mandatory)*

### Functional Requirements

**Theme and visual rules**

- **FR-001** (§12.3): Before any page is built, every token in `DESIGN.md` (colors, typography,
  spacing, radii, row, field, and nav sizes, window size) MUST be defined in `ui/theme.slint` as
  the single `Theme` global.
- **FR-002** (§6, §12.3): Every color, font size, spacing, and radius in `.slint` files MUST
  reference `Theme.*`; the §12.3 grep check (no hex colors or `px` values in `.slint` files other
  than `theme.slint`) MUST pass and MUST run in CI.
- **FR-003** (DESIGN.md): Status colors MUST always appear with their text label (Attended, Absent,
  Cancelled, Pending, Below target); a missing percentage MUST show "—".

**Structure and wiring**

- **FR-004** (§6): One window with a left navigation of five pages: Dashboard, Timetable, Courses,
  Assessments, Settings. Forms are inline panels or dialogs. Every list MUST have an empty state.
- **FR-005** (§2, §6): `.slint` files MUST only bind properties and invoke callbacks. All
  formatting shown in the UI (percentages, "—", status labels, dates and times, row text, summary
  lines, banner text, button enablement) MUST come from pure functions in the app's view-model
  module, each covered by unit tests.
- **FR-006** (§5): The UI MUST use only the frozen §5 core API, passing the current date and time
  in; every mutation is saved immediately (§1, no save-all button).
- **FR-007** (§6): Date and time inputs MUST be text fields validated as `DD-MM-YYYY` and `HH:MM` (human decision 2026-10-29),
  with the error message directly under the field stating the expected format. Core validation
  errors (R2) MUST be shown inline in the form that caused them.
- **FR-008** (R24, R25): On start the app MUST open the database at the R24 path (honouring the
  override variable); if opening fails, including a database newer than the app, it MUST show a
  clear error instead of crashing.

**Sync triggers**

- **FR-009** (R6): The app MUST sync sessions with today's date on start, after every term,
  holiday, course (including archive / unarchive), or slot change, and after a restore, then
  refresh the visible page.
- **FR-010** (R6): A timer MUST run every 60 seconds; when the date has changed since the last sync,
  it MUST sync and refresh.

**Dashboard**

- **FR-011** (R16–R20): The dashboard MUST show today's classes, the next class, upcoming
  assessments, overdue assessments, and awaiting attendance, each with an empty state.
- **FR-012** (R20, R12): Each awaiting-attendance row MUST have Attended / Absent / Cancelled
  buttons that set the status and refresh the dashboard.
- **FR-013** (R14): A banner MUST list every course below target as "{code} — {pct}% (target
  {target}%)"; it is hidden when no course is below target.

**Timetable**

- **FR-014** (§6): The week view MUST show Monday–Sunday of the selected week, grouped by weekday,
  with previous / next / today buttons; each row shows time, course code and name, room, and the
  status label (including "Rescheduled → {date}").
- **FR-015** (R8, R9, R11, R12): Selecting a session MUST offer set status (Attended, Absent,
  Pending), Cancel, and Reschedule (date, start, end, room, optional note); stored notes MUST be
  shown read-only. Attended and Absent MUST be disabled for sessions that have not started.
- **FR-016** (R7, §6): The Rules tab MUST list the slots valid on the reference date with add, edit
  (asking for an effective date, default today), and end (asking for the last date) actions.

**Courses**

- **FR-017** (R3, §6): The course list MUST have a show-archived toggle and create, edit, archive,
  and unarchive actions.
- **FR-018** (R13–R15, §6): Course detail MUST show the attendance summary (counts, percentage or
  "—", target, below-target label) and the course's full session list with status buttons.
- **FR-019** (R10, §6): Course detail MUST offer an "Add makeup class" form (date, start, end, room,
  note).

**Assessments**

- **FR-020** (R18, R19, §6): The assessments page MUST list assessments with the filters upcoming /
  overdue / completed / all, a complete checkbox per row, and create / edit / delete actions.

**Settings, backup, restore, export**

- **FR-021** (R1, R2, §6): Settings MUST let the student create or edit the active term (name,
  start, end).
- **FR-022** (§6, human decision): Settings MUST list holidays (date and label) and let the student
  add either one date or an inclusive start–end range with one label, expanded into one holiday per
  day, and remove a single date.
- **FR-023** (§6): Settings MUST edit class reminder minutes, upcoming window days, and the
  notifications switch, saved through the core settings call.
- **FR-024** (R21, §6): Backup MUST open a save-file dialog and write `export_json` output to the
  chosen file.
- **FR-025** (R22, human decision): Restore MUST open a file dialog, then a confirm dialog; on
  confirm it MUST copy the current database file to
  `{data_dir}/backups/pre-restore-{YYYYMMDD-HHMM}.sqlite` (creating the folder if missing, with the
  copy containing all committed data), and only if the copy succeeded call `restore_json`. On
  restore error the message is shown and the data is unchanged.
- **FR-026** (R23, §6): Export CSV MUST open a folder dialog and write `attendance.csv` and
  `assessments.csv` there.
- **FR-027** (R24, §6): Settings MUST show the database file path as text.

**Verification**

- **FR-028** (§7): The milestone gate MUST pass, the theme grep check MUST pass, and
  `DESIGN.md` lint MUST pass.
- **FR-029** (§7, §12.3): One screenshot per page plus one empty-state screenshot MUST be critiqued
  against `DESIGN.md`, and no high-severity item may remain open; fixes go in `theme.slint` or
  layout, never hardcoded values.
- **FR-030** (§9): The human runs the §9 smoke checklist and it passes.

### Key Entities *(include if feature involves data)*

- **Theme**: the set of design tokens from `DESIGN.md` (colors, type, spacing, radii, sizes), the
  only source of visual values.
- **View models**: display-ready rows built from core data (session row, assessment row, course
  row, summary line, banner line, holiday row, slot row) plus the parsing and validation of form
  text, all as pure, unit-tested functions.
- **Pre-restore copy**: a timestamped copy of the database file kept in the data folder's
  `backups` subfolder before every restore.

No new stored entities; the schema is unchanged.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A student completes the full §9 smoke checklist (create term → … → open CSVs) in
  about 5 minutes without errors or crashes.
- **SC-002**: Marking a past session's attendance from the dashboard takes at most two clicks.
- **SC-003**: Every formatting and parsing function shown in the UI has unit tests, and the gate
  (format check, lint with warnings as errors, full test suite) passes with all earlier milestone
  tests still green.
- **SC-004**: The theme grep check finds zero hex colors or pixel values outside `theme.slint`.
- **SC-005**: The screenshot critique against `DESIGN.md` has zero open high-severity items.
- **SC-006**: Entering a five-day exam period as holidays takes one form submission.
- **SC-007**: After every restore, a pre-restore copy of the previous data exists in the backups
  folder.

## Assumptions

- `DESIGN.md` is already approved (M0); M4 translates it by hand and does not re-decide tokens.
  DESIGN.md says headings use Georgia with the platform default as fallback.
- The "reference date" for the Rules tab list is today; slots that ended before today are not
  listed (the §5 API lists slots valid on a date).
- Setting a session back to Pending is offered as a status action because R12 allows it at any
  time and T23 covers it.
- The dashboard and other "now"-dependent lists are recomputed whenever a page is shown, after each
  mutation, and on each 60-second tick, using the current time.
- The pre-restore copy is a copy of the database file contents including any committed data still
  in the write-ahead log; the plan chooses how.
- Default file names for backup (e.g. a timestamped `.json`) and the folder dialog start location
  are naive choices left to the plan.
- After a successful restore the app syncs (R6: the restore changes terms, holidays, courses, and
  slots).
- Notifications are not sent in M4; the switch and reminder minutes are only stored (M5 uses them).
- Screenshot critique uses the `impeccable` skill per §12.0; if it is not installed in this
  environment, the router raises it with the human at the end of M4.
- Out of scope per §10: visual time grid, date pickers, multiple simultaneous terms in the UI.

## Change 2026-10-29: date input and display format (human request)

- Date inputs are typed as `DD-MM-YYYY` (dash separator; day and month may have 1 or 2 digits;
  4-digit year). Invalid input shows "Use DD-MM-YYYY, e.g. 29-10-2026". Pre-filled date fields
  (edit forms, defaults) show zero-padded `DD-MM-YYYY`.
- Dates shown as text read like "October 26, 2026" wherever space allows: with a weekday
  "Monday, October 26, 2026"; with a time "October 30, 2026, 23:59" (times stay 24-hour `HH:MM`);
  week title "October 5 – 11, 2026" within one month, "October 26 – November 1, 2026" when the month changes, and "December 28, 2026 – January 3, 2027" when the year changes.
- The core `display_status` text "Rescheduled → 2026-01-13" (§5, T09) is unchanged in core; the
  app reformats its date part to "Rescheduled → January 13, 2026".
- Unchanged and still ISO: database storage (§1), JSON backup (R21), CSV exports (R23), the
  backup and pre-restore file names, and the database path.
