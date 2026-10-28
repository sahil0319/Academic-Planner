# Feature Specification: M1 Core Storage and Setup Data

**Feature Branch**: `001-m1-core-storage` (spec directory; work happens on the current branch)

**Created**: 2026-10-28

**Status**: Draft

**Input**: User description: "Milestone M1 from docs/spec.md: rules R1-R3, R24-R25, tests T20, T24, T26. Do not add requirements."

**Source of truth**: `docs/spec.md` §3 (data model), §4 (rules R1–R3, R24–R25), §5 (core API), §7 (M1 row), §8 (T20, T24, T26). This spec restates that scope and adds nothing to it.

## User Scenarios & Testing *(mandatory)*

The "user" of this milestone is a student whose planner data must be stored safely on their own
computer, and the app layer that will be built on top of the core in later milestones (M2–M4).
Nothing in M1 is visible on screen yet; every scenario is verified by core integration tests.

### User Story 1 - Planner data file opens safely every time (Priority: P1)

A student starts the app. Their planner data file is found (or created) in their personal data
location, brought up to the current format, and opened. Starting the app again later changes
nothing. If the file was written by a newer version of the app, the app refuses to open it and
says why, instead of damaging it.

**Why this priority**: Every other feature reads and writes this file. If opening is unsafe, all
data is at risk.

**Independent Test**: Open a brand-new data file, check its format version, reopen it and confirm
nothing changed; then open a file marked with a newer format version and confirm a clear refusal
(T20, T26).

**Acceptance Scenarios**:

1. **Given** no data file exists, **When** the planner is opened, **Then** the file is created
   with the complete current structure and its format version equals the latest version (T20).
2. **Given** a data file already at the latest version, **When** it is opened again, **Then** no
   changes are applied and its contents and version are unchanged (T20).
3. **Given** a data file whose format version is newer than the app supports, **When** it is
   opened, **Then** opening fails with a clear error that names the problem, and the file is left
   untouched (T26).
4. **Given** the environment override for the data file location is set, **When** the app resolves
   where the data file lives, **Then** it uses the override path; otherwise it uses
   `planner.sqlite` inside the per-user application data directory, creating that directory if it
   is missing (R24).
5. **Given** a test run, **When** a planner is opened, **Then** it uses an in-memory store and
   never touches the real user data file.

---

### User Story 2 - Set up a term, its holidays, and courses (Priority: P1)

A student enters the current term (name, start date, end date) and its holidays, then adds the
courses they are taking (code, name, optional instructor, room, notes, and attendance target).
Later they edit a course, archive one they dropped, and unarchive it if needed.

**Why this priority**: Terms and courses are the base records that timetable, attendance, and
dashboard features in M2–M4 depend on.

**Independent Test**: Save a term, set holidays, create/update/list/archive courses, and confirm
the stored records and the active-term behavior.

**Acceptance Scenarios**:

1. **Given** no term exists, **When** a term is saved, **Then** it becomes the active term and is
   returned as the active term (R1).
2. **Given** an active term, **When** another term is saved, **Then** the newly saved term is the
   active term and at most one term is active at any time (R1).
3. **Given** no term exists, **When** the active term is requested, **Then** the answer is "none"
   (R1: zero or one active term).
4. **Given** an active term, **When** holidays are set for it, **Then** exactly the given holiday
   dates (with optional labels) are stored for that term.
5. **Given** an active term, **When** a course is created, updated, and listed, **Then** the list
   shows the course with its current values, scoped to the active term (R1).
6. **Given** a course, **When** it is archived, **Then** it no longer appears in the default course
   list but appears when archived courses are included, and its data is kept; **When** it is
   unarchived, **Then** it appears in the default list again (R3).

---

### User Story 3 - Invalid input is rejected with a clear validation error (Priority: P1)

When the student enters something invalid, the planner rejects it with a validation error that
the app can show inline, never a crash, and nothing invalid is stored.

**Why this priority**: Bad data stored now would corrupt session generation and attendance math
in M2.

**Independent Test**: Submit each invalid input listed in R2 and confirm each returns a validation
error and stores nothing (T24).

**Acceptance Scenarios** (each returns a validation error; nothing is stored) (R2, T24):

1. **Given** a term whose end date is before its start date, **When** it is saved, **Then** a
   validation error is returned.
2. **Given** a timetable slot whose end time is equal to or before its start time, **When** it is
   added, **Then** a validation error is returned.
3. **Given** a course whose attendance target is below 0 or above 100, **When** it is created or
   updated, **Then** a validation error is returned.
4. **Given** a course with an empty code or an empty name, **When** it is created or updated,
   **Then** a validation error is returned.
5. **Given** an assessment with an empty title, **When** it is created or updated, **Then** a
   validation error is returned. (Resolved 2026-10-28 by the human: M1 implements assessment create, update, complete,
   delete, and list as plain storage operations, so this case is covered in M1.)
6. **Given** a course code that already exists in the same term, **When** another course with that
   code is created, or an existing course is updated to that code, **Then** a validation error is
   returned.

---

### User Story 4 - Weekly timetable slots and settings are stored (Priority: P2)

A student adds the recurring weekly class slots for each course (weekday, start/end time, optional
room, valid-from date, optional valid-to date) and can list the slots that apply on a given date.
They can read and change their settings (class reminder minutes, upcoming window days,
notifications on/off), which start with sensible defaults.

**Why this priority**: Slots and settings are inputs to session generation (M2) and reminders
(M5), but nothing consumes them until then.

**Independent Test**: Add slots and list them for dates inside and outside their validity range;
read default settings, save new values, read them back.

**Acceptance Scenarios**:

1. **Given** a course, **When** a slot is added, **Then** it is stored and returned with its
   values.
2. **Given** slots with different validity ranges, **When** slots are listed for a date, **Then**
   only slots valid on that date (inclusive of both ends; open-ended valid-to means "until term
   end") are returned.
3. **Given** a fresh data file, **When** settings are read, **Then** they are: class reminder
   minutes = 15, upcoming window days = 7, notifications enabled = on.
4. **Given** settings, **When** new values are saved and settings are read again, **Then** the new
   values are returned.

---

### User Story 5 - The full core API exists so UI work can start (Priority: P2)

The app layer can be written against the complete set of core operations and data types from
`docs/spec.md` §5, even though operations outside M1 scope are placeholders until M2/M3/M5.

**Why this priority**: §7 parallelism: after M1 the API is frozen, so UI (M4) and core (M2–M3)
work can proceed in parallel.

**Independent Test**: The workspace builds with every §5 operation and type present; M1 operations
work; out-of-scope operations are explicit placeholders.

**Acceptance Scenarios**:

1. **Given** the core library, **When** the workspace is built, **Then** every operation and type
   listed in §5 exists with the exact signature given there.
2. **Given** an operation that belongs to a later milestone, **When** it is inspected, **Then** it
   is a clearly marked placeholder (not partial logic).

---

### Edge Cases

- A data file at the current version is reopened repeatedly: no changes each time (R25, T20).
- A data file with a newer format version: refused, untouched (R25, T26).
- The data directory does not exist yet: it is created (R24).
- Saving a second term while one is active: exactly one active term afterwards (R1).
- Term with end date equal to start date: valid (only end < start is rejected, R2).
- Slot with end time equal to start time: rejected (R2: end ≤ start).
- Attendance target exactly 0 or exactly 100: valid; no target at all: valid (R2 range 0–100).
- Same course code in two different terms: allowed (uniqueness is per term, R2).
- Updating a course without changing its own code: not a duplicate (R2).
- Archived course: still listed when archived courses are requested; data kept (R3).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001** (§3, R25): Opening a data file MUST apply all pending format migrations so the stored
  structure exactly matches `docs/spec.md` §3 (migration 001), including referential integrity
  enforcement, and record the latest format version.
- **FR-002** (R25, T20): Opening a data file already at the latest format version MUST be a no-op.
- **FR-003** (R25, T26): Opening a data file whose format version is newer than the app's MUST fail
  with a clear error and MUST NOT modify the file.
- **FR-004** (R24): The app's data file location MUST be `planner.sqlite` in the per-user
  application data directory unless the `ACADEMIC_PLANNER_DB` environment variable is set, in which
  case that path MUST be used. The directory MUST be created if missing. The data file MUST never
  be placed in the install directory or the repository.
- **FR-005** (§5): The core MUST support opening a file-backed planner and an in-memory planner;
  tests MUST use the in-memory planner.
- **FR-006** (R1): There MUST be zero or one active term. Saving a term (create or update) MUST
  make it the active term. The active term MUST be retrievable, returning "none" when there is no
  term.
- **FR-007** (§5): Holidays for a term MUST be settable as a set of dates with optional labels.
- **FR-008** (§5, R1): Courses MUST support create, update, archive/unarchive, and list (with or
  without archived courses), operating on the active term.
- **FR-009** (R3): Archived courses MUST be excluded from the default course list, MUST be included
  when archived courses are requested, and MUST keep all their data.
- **FR-010** (§5): Timetable slots MUST support add, and list-valid-on-a-date.
- **FR-011** (§3, §5): Settings MUST be readable and savable, with defaults class reminder minutes
  = 15, upcoming window days = 7, notifications enabled = on.
- **FR-012** (R2, T24): The following MUST return a validation error (never a crash) and store
  nothing: term end before start; slot end time at or before start time; attendance target outside
  0–100; empty course code; empty course name; empty assessment title; duplicate course code within
  a term.
- **FR-013a** (§5, R2): Assessments MUST support create, update, mark completed/not completed,
  delete, and list (human decision 2026-10-28, so that T24 covers the empty-title case of R2 in M1).
  Dashboard rules that read assessments (R18, R19) stay in M2.
- **FR-013** (§5, §7): Every operation and type in `docs/spec.md` §5 MUST exist with its exact
  signature. Operations outside M1 scope MUST be explicit placeholders.
- **FR-014** (§1): Core operations MUST NOT read the system clock; any date/time they need is
  passed in by the caller.

### Key Entities *(include if feature involves data)*

- **Term**: a teaching period with a name, start date, end date, and an active flag; at most one is
  active.
- **Holiday**: a date (with optional label) within a term on which no classes occur.
- **Course**: belongs to a term; code (unique within the term), name, optional instructor, room,
  notes, optional attendance target (0–100), archived flag.
- **Slot**: a recurring weekly class rule for a course: weekday (1 = Monday … 7 = Sunday), start and
  end time, optional room override, valid-from date, optional valid-to date.
- **Settings**: class reminder minutes, upcoming window days, notifications enabled.
- **Assessment**: belongs to a course; kind (quiz, exam, assignment, other), title (non-empty), due
  date-time, optional notes, completed flag, optional reminder lead time in minutes.
- **Session, Reminder log**: part of the stored structure from migration 001; their behavior
  belongs to M2 and M5.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: T20 passes: a fresh data file reports the latest format version, and reopening it
  changes nothing.
- **SC-002**: T24 passes: 100% of the R2 invalid-input cases return a validation error and store
  nothing.
- **SC-003**: T26 passes: a data file from a newer app version is refused with a clear error and
  left unchanged.
- **SC-004**: The milestone gate (format check, lint with warnings as errors, full test suite)
  passes with zero failures and zero warnings.
- **SC-005**: 100% of the operations and types in `docs/spec.md` §5 exist, so later milestones can
  be built without changing any signature.

## Assumptions

- Spec scope is exactly the §7 M1 row; no requirement beyond `docs/spec.md` is added.
- Slot changes and slot ending (`change_slot`, `end_slot`) are rule R7 / test T10, T11, T27, which
  belong to M2; in M1 they are placeholders. "Slots CRUD" in M1 therefore means add and list.
- Assessment storage operations are pulled into M1 by human decision (2026-10-28) only to make T24
  complete; the assessment list filter's upcoming/overdue meaning follows R18/R19 and is verified
  by T16/T17 in M2.
- The parts of R3 that concern the dashboard, timetable, and session generation are verified in M2
  (T15, T28), because those features are M2 scope. In M1, R3 is verified through the course list.
- "Setting holidays" replaces the term's holiday set with the given dates.
- The data-file location rule (R24) lives where the app can resolve the per-user data directory;
  no T-test is listed for it, so it is verified by a unit test that does not touch the real user
  data file.
