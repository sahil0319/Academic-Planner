# Feature Specification: M8 12-Hour Times

**Feature Branch**: `008-m8-12-hour-times` (spec directory; work happens on the current branch)

**Created**: 2026-10-29

**Status**: Draft

**Input**: User description: "Milestone M8 from docs/spec.md §13.2: rules R31–R32, tests T34–T35.
Do not add requirements."

**Source of truth**: `docs/spec.md` §6 ("Date/time inputs" paragraph), §7 (M8 row), §13.2 (R31,
R32), and §13.5 (T34, T35). Everything in §1–§12 still applies. This feature adds no rule, no
schema change, no migration, and no §5 method. It changes only how the desktop app shows and reads
times; the core crate is not touched.

## User Scenarios & Testing *(mandatory)*

The user is a student who thinks in 12-hour times ("9:30 AM", "1:30 PM"). Today the app shows
and asks for 24-hour `HH:MM` times. After M8, every time on screen reads the 12-hour way and every
time field takes a few digits plus an AM/PM choice.

### User Story 1 - See every time in 12-hour form (Priority: P1)

Wherever the app shows a time (class times on the Dashboard, the Timetable week view, the
Timetable Rules tab, the course detail session list, assessment due date-times on the Assessments
page and the Dashboard), it reads like "9:30 AM" or "1:30 PM".

**Why this priority**: Reading times is far more frequent than entering them, and the display
change alone already removes the 24-hour friction on every page.

**Independent Test**: T35 (app unit test of the formatting function) plus a look at each page.

**Acceptance Scenarios**:

1. **Given** the stored times 00:00, 09:05, 12:00, 13:30, **When** they are formatted for
   display, **Then** they read "12:00 AM", "9:05 AM", "12:00 PM", "1:30 PM" (T35).
2. **Given** a class from 09:00 to 10:30, **When** it appears in any session or slot list,
   **Then** its time reads "9:00 AM–10:30 AM".
3. **Given** an assessment due 2026-10-30 23:59, **When** it appears in the Assessments list,
   **Then** the due text reads "October 30, 2026, 11:59 PM"; in the Dashboard day panel the due
   time reads "11:59 PM".
4. **Given** any time shown in the UI, **When** the user reads it, **Then** it never appears in
   24-hour form (R31).

---

### User Story 2 - Enter a time with digits and an AM/PM toggle (Priority: P1)

Every time field (Timetable reschedule start/end, Timetable Rules slot start/end, course detail
"Add makeup class" start/end, assessment due time) is a short digits field next to an AM/PM
toggle. The user types `930` and picks AM to mean 9:30 in the morning.

**Why this priority**: Input must match display; a 12-hour display with 24-hour entry would be
inconsistent and error-prone.

**Independent Test**: T34 (app unit test of the parsing function) plus one save on each form.

**Acceptance Scenarios**:

1. **Given** the toggle on AM, **When** the user types `9`, `930`, `0930`, `1215`, or `12:15`,
   **Then** the stored times are 09:00, 09:30, 09:30, 00:15, 00:15 (T34).
2. **Given** the toggle on PM, **When** the user types `1215` or `130`, **Then** the stored times
   are 12:15 and 13:30 (T34).
3. **Given** either toggle state, **When** the user types `1260`, `1300`, `0`, or leaves the field
   empty, **Then** the field shows an inline error and nothing is saved (T34).
4. **Given** a form with several bad fields, **When** the user saves, **Then** every bad field
   shows its own inline error at once, as forms already do (§6 "Date/time inputs").

---

### User Story 3 - Edit an existing time (Priority: P2)

When the user opens an existing slot, a session to reschedule, or an assessment to edit, each
time field is pre-filled with the stored time as `hmm` digits and the toggle is set to match.

**Why this priority**: Editing is less frequent than viewing or creating, but a wrong pre-fill
would silently change a stored time on save.

**Independent Test**: unit test of the pre-fill function, plus a round trip (pre-fill, then parse
with the same toggle state) that returns the original time for every minute of the day.

**Acceptance Scenarios**:

1. **Given** a stored time 09:30, **When** its form opens, **Then** the field shows `930` and the
   toggle shows AM.
2. **Given** a stored time 13:30, **When** its form opens, **Then** the field shows `130` and the
   toggle shows PM.
3. **Given** stored times 00:15 and 12:05, **When** their forms open, **Then** the fields show
   `1215` with AM and `1205` with PM.
4. **Given** any pre-filled field, **When** the user saves without changes, **Then** the stored
   time is unchanged.

---

### Edge Cases

- `12` with AM is midnight (00:00); `12` with PM is noon (12:00) (R32).
- Hour `0` or `00` (for example `0`, `030`, `0030`) is an error: hours run 1–12 (R32, T34 `0`).
- Hour above 12 (for example `1300`, `13`) is an error even with PM; the user must pick PM and
  type `100` (R32, T34 `1300`).
- Minutes above 59 (for example `1260`, `960`) are an error (R32, T34 `1260`).
- A typed `:` anywhere is ignored, so `12:15` equals `1215` and `9:30` equals `930` (R32).
- Five or more digits, or any character other than digits and `:`, is an error (R32: 1–4 digits).
- Leading and trailing spaces are ignored, as in the existing date fields (§6 "Date/time
  inputs").
- An empty field is an error (T34); every time field in the app is required today.
- A slot whose end is not after its start is still rejected by core (R2); the message the user
  sees shows both times in 12-hour form (R31).
- A new form (nothing to pre-fill) starts with an empty field and the toggle on AM.
- Storage, JSON backup, CSV export, and (later) the M10 import keep 24-hour `HH:MM`; a backup
  taken before and after M8 has the same time text (R31, §1, R21, R23).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001 (R31)**: Every time shown in the UI MUST use 12-hour form `h:mm AM` / `h:mm PM`: hour
  1–12 without a leading zero, two-digit minutes, one space, then `AM` or `PM`. Midnight is
  "12:00 AM" and noon is "12:00 PM" (T35).
- **FR-002 (R31)**: FR-001 MUST apply on every page that shows a time: session rows (Dashboard
  sections and day panel, Timetable week view, course detail session list), slot rows (Timetable
  Rules tab), assessment due date-times (Assessments list, Dashboard), the Dashboard day panel's
  due times, and validation messages that quote a time.
- **FR-003 (R31)**: A time range MUST show both ends in 12-hour form joined by the existing en
  dash, for example "9:00 AM–10:30 AM". A date-time MUST keep the long date form followed by the
  12-hour time, for example "October 30, 2026, 11:59 PM".
- **FR-004 (R31)**: Storage, JSON backup, CSV export, and the M10 import MUST keep 24-hour
  `HH:MM`. M8 MUST NOT change the core crate, the schema, the backup format, or the CSV format.
- **FR-005 (R32)**: Every time input MUST be a digits field plus an AM/PM toggle: Timetable
  reschedule start and end, Timetable Rules slot start and end, course detail "Add makeup class"
  start and end, and the assessment due time.
- **FR-006 (R32)**: The digits field MUST be read as follows after trimming spaces and removing
  every `:`: 1–2 digits are the hour with minutes 00 (`9` is 9:00); 3–4 digits are the hour
  followed by two minute digits (`930`, `0930`, `1215`). Anything else is an error.
- **FR-007 (R32)**: The hour MUST be 1–12 and the minutes 00–59, otherwise the field shows an
  inline error. 12 AM maps to 00:mm, 12 PM to 12:mm, other PM hours to hour + 12, other AM hours
  unchanged (T34).
- **FR-008 (R32, §6)**: A bad or empty time field MUST show its error inline under that field,
  alongside the errors of the other fields in the same form, and the form MUST NOT save.
- **FR-009 (R32)**: Editing an existing time MUST pre-fill the field with `hmm` (12-hour hour
  without a leading zero, then two-digit minutes) and set the toggle to match the stored time.
  Pre-filling then parsing with the same toggle MUST return the stored time.
- **FR-010 (R32)**: Parsing (FR-006, FR-007), pre-fill (FR-009), and display formatting (FR-001,
  FR-003) MUST be pure functions in `crates/app/src/viewmodel.rs` with unit tests, including T34
  and T35. `.slint` files hold only bindings and callbacks (constitution III).
- **FR-011 (§6, §12.3)**: The AM/PM toggle MUST be built from existing components and `Theme.*`
  tokens only. Its two choices carry the text labels "AM" and "PM" and are reachable by keyboard,
  like the existing appearance switch. The theme grep check
  (`! grep -rnE '#[0-9a-fA-F]{6}|[0-9]+px' crates/app/ui --include=*.slint | grep -v theme.slint`)
  MUST stay clean (§7 M8 "Done when").
- **FR-012 (§6)**: Field labels MUST stop saying "(HH:MM)"; the inline error text MUST describe
  the new digits form.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: T34 and T35 pass (§7 M8 row).
- **SC-002**: The gate passes: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D
  warnings && cargo test --workspace` (§7).
- **SC-003**: The theme grep check passes (§7 M8 row).
- **SC-004**: On a manual pass through all pages, 0 times appear in 24-hour form and each of the 7
  time fields accepts `930` + PM and saves 21:30.
- **SC-005**: For all 1,440 minutes of the day, pre-fill followed by parse returns the original
  time (FR-009).
- **SC-006**: A JSON backup and a CSV export taken after M8 contain the same `HH:MM` time text as
  before M8 for the same data (FR-004).

## Assumptions

- Decided (naive UI choice): a new form starts with an empty time field and the toggle on AM.
- Decided (naive): surrounding spaces are trimmed, matching the existing date and time parsers.
- Decided (naive): the inline error text is one message for every bad time, for example "Enter the
  time as digits, e.g. 930 (hour 1–12, minutes 00–59)". Wording is free; tests check that an
  error is returned, not its exact text.
- Decided (follows R31 and the existing T09 precedent where the app rewrites core's ISO date in
  `display_status` to the long form): core's R2 slot message ("slot end time 09:00 is not after
  its start time 10:30") is rewritten in the app so its times read in 12-hour form. Core text is
  unchanged.
- The existing 24-hour `parse_time` and its form callers are replaced by the 12-hour parser;
  existing view-model tests that assert 24-hour display or input are updated to the new forms.
- Settings, About, Courses list, and Settings holiday fields show no times and are unaffected.

## Open Questions (for the human)

- **OQ-1 Desktop notification text.** Reminder notifications (M5) show text that core builds with
  24-hour times ("CSE 4711 Networks at 09:00 in C3", "… due October 30, 2026, 23:59"). R31 says
  "every time shown in the UI"; a desktop notification is shown by the operating system, so it is
  unclear whether it counts.
  - A: Leave notifications 24-hour in M8 (strict reading: notifications are not the app UI).
  - B (recommended): Rewrite the times in the notification text in the app, with the same pure
    function used for core's validation messages. No core change; one small task.
  - C: Change core's reminder titles to 12-hour. Core change; updates M5 tests; widens M8 beyond
    the app crate.
  - Until answered, the task list carries B as one conditional task that the router keeps or
    drops.

## Router decision (2026-10-29)

OQ-1 resolved as option B: the human asked for 12-hour times system wide, so desktop notification text is rewritten in the app (notify.rs, `times_to_12h`). T023 is no longer conditional. No core change.
