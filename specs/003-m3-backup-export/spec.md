# Feature Specification: M3 Backup, Restore, and CSV Export

**Feature Branch**: `003-m3-backup-export` (spec directory; work happens on the current branch)

**Created**: 2026-10-29

**Status**: Draft

**Input**: User description: "Milestone M3 from docs/spec.md: rules R21, R22, R23; tests T25, T30 plus backup tests. Do not add requirements."

**Source of truth**: `docs/spec.md` §3 (data model), §4 (rules R21–R23), §5 (core API:
`export_json`, `restore_json`, `export_attendance_csv`, `export_assessments_csv`), §7 (M3 row),
§8 (T25, T30). This spec restates that scope and adds nothing to it beyond the human decision dated
2026-10-29 (the R22 pre-restore file copy is built in the app layer in M4), which changes no rule,
schema, or API.

## User Scenarios & Testing *(mandatory)*

The "user" of this milestone is a student who wants a safe copy of their planner data and a way to
open their attendance and assessment history in a spreadsheet, plus the app layer (M4) that will
offer these as buttons. Nothing in M3 is visible on screen yet; every scenario is verified by core
integration tests against the §8 fixture: active term **2026-01-05 (Mon) → 2026-01-25 (Sun)**;
course `CSE101` with attendance target 75; slot A Monday 09:00–10:00; slot B Wednesday 11:00–12:00.

### User Story 1 - Back up everything to one file (Priority: P1)

A student exports all planner data (terms, holidays, courses, slots, sessions, assessments,
settings) as one versioned JSON document that they can keep somewhere safe. Every record keeps its
identity, so links between records (a makeup that replaces a session, a session's slot) survive.

**Why this priority**: A backup is the only protection against losing a term of attendance
history. Restore (Story 2) is useless without it.

**Independent Test**: Build the fixture with sessions, an assessment, and changed settings; export;
parse the document and check its top-level shape, the version, the timestamp, and that every row
appears with its original identity (R21, T30 first half).

**Acceptance Scenarios**:

1. **Given** the fixture synced and populated, **When** the student exports a backup at a given
   moment, **Then** the document has exactly the keys `format`, `schema_version`, `exported_at`,
   `terms`, `holidays`, `courses`, `slots`, `sessions`, `assessments`, `settings`, with
   `format = "academic-planner-backup"`, `schema_version` equal to the app's current schema
   version, and `exported_at` equal to the given moment (R21).
2. **Given** a populated planner, **When** a backup is exported, **Then** every stored term,
   holiday, course, slot, session, and assessment appears once with its original identity and
   field values, and every setting appears as a key → value entry (R21).
3. **Given** an empty planner (fresh database), **When** a backup is exported, **Then** every
   collection is empty and `settings` holds the default settings (R21).

---

### User Story 2 - Restore a backup, all or nothing (Priority: P1)

A student loads a backup file. The planner checks that it is a planner backup and not from a newer
app version, then replaces all current data with the backup's data in one step. If anything is
wrong with the file, the planner keeps exactly the data it had before.

**Why this priority**: Restore is the other half of backup, and a half-applied restore would
corrupt the student's history, so the all-or-nothing guarantee is the core of this milestone.

**Independent Test**: Export a populated planner, restore into an empty planner, export again,
and compare; then feed bad documents into a populated planner and check nothing changed (R22,
T30).

**Acceptance Scenarios**:

1. **Given** a backup exported from a populated planner, **When** it is restored into an empty
   planner and that planner is exported again, **Then** the two documents are equal except for
   `exported_at` (R21, R22, T30).
2. **Given** a populated planner and a backup of different data, **When** the backup is restored,
   **Then** the planner holds only the backup's data: no row that existed before the restore and
   is absent from the backup remains (R22).
3. **Given** a populated planner, **When** a document whose `format` is not
   `"academic-planner-backup"` (or is missing) is restored, **Then** restore fails with an error
   and the planner's data is unchanged (R22, T30).
4. **Given** a populated planner, **When** a backup whose `schema_version` is greater than the
   app's is restored, **Then** restore fails with an error and the planner's data is unchanged
   (R22, T30).
5. **Given** a populated planner, **When** a document that passes the format and version checks
   but cannot be fully applied (for example, a session that points to a course not in the backup,
   or text that is not valid JSON) is restored, **Then** restore fails with an error and the
   planner's data is unchanged (R22).

---

### User Story 3 - Open attendance and assessments in a spreadsheet (Priority: P2)

A student exports two CSV files: one row per class session with its date, times, course, kind,
status, the original date it replaced (for makeups), and note; and one row per assessment with
its course, kind, title, due time, completion, and notes. Both have a header row and are sorted by
date so they read top to bottom in time order.

**Why this priority**: Useful for reporting attendance to an instructor or keeping records, but
the student's data is already safe after Stories 1–2.

**Independent Test**: Build the fixture with a cancelled session, a rescheduled one with its
makeup, a note, and two assessments; export both CSVs and compare them byte for byte with the
expected text (R23, T25).

**Acceptance Scenarios**:

1. **Given** any planner, **When** attendance is exported, **Then** the first line is exactly
   `date,start,end,course_code,course_name,kind,status,replaces_date,note` (R23, T25).
2. **Given** any planner, **When** assessments are exported, **Then** the first line is exactly
   `course_code,kind,title,due_at,completed,notes` (R23, T25).
3. **Given** the fixture synced, with the 01-12 session rescheduled to a makeup on 01-13 that is
   marked attended, **When** attendance is exported, **Then** there is one row per session, rows
   are in date order (then start time), the 01-12 row has status `cancelled` and an empty
   `replaces_date`, and the 01-13 makeup row has kind `makeup` and `replaces_date = 2026-01-12`
   (R23, T25).
4. **Given** two assessments with different due times entered in reverse order, **When**
   assessments are exported, **Then** rows are in due-time order (R23, T25).
5. **Given** a note, title, or name containing a comma, a double quote, or a line break, **When**
   either CSV is exported, **Then** that field is quoted so a spreadsheet reads it as one cell
   with the original text (R23).

### Edge Cases

- Empty planner: each CSV contains only its header row (R23); the backup has empty collections
  and default settings (R21).
- Restore into a planner that already has data: all existing rows are replaced, not merged (R22).
- Restore of a backup with `schema_version` equal to or less than the app's is accepted; only a
  greater version is refused (R22).
- Restore preserves identities, so a makeup's link to the session it replaces, a session's link to
  its slot, and a course's link to its term are intact after restore (R21, R22).
- Archived courses, inactive terms, and sessions of every status (including pending and
  cancelled) are included in the backup and in attendance.csv; nothing is filtered (R21, R23).
- A field with no value (no note, no notes, no replaced session) is an empty CSV cell (R23).
- Two sessions on the same date and start time, or two assessments with the same due time, are
  ordered by course code, then by identity, so output is stable across runs (R23).
- The reminder log (M5) is not part of the backup (R21 lists the exported collections) and is left
  as it is by restore.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001** (R21): Exporting a backup MUST produce one JSON document with exactly the top-level
  keys `format`, `schema_version`, `exported_at`, `terms`, `holidays`, `courses`, `slots`,
  `sessions`, `assessments`, `settings`.
- **FR-002** (R21): `format` MUST be `"academic-planner-backup"`; `schema_version` MUST be the
  app's current schema version; `exported_at` MUST be the moment passed in by the caller (the
  core never reads the clock).
- **FR-003** (R21): Each collection MUST contain every stored row of its table, one object per row
  with every column of §3, and identities MUST be preserved. `settings` MUST be an object mapping
  each setting key to its stored value.
- **FR-004** (R21): Collections MUST be emitted in a stable order (by identity, or by the table's
  key for holidays and settings) so two exports of the same data are identical apart from
  `exported_at`.
- **FR-005** (R22): Restore MUST reject a document whose `format` is missing or is not
  `"academic-planner-backup"`, with an error.
- **FR-006** (R22): Restore MUST reject a document whose `schema_version` is missing or greater
  than the app's current schema version, with an error.
- **FR-007** (R22): Restore MUST replace all data in the exported tables (terms, holidays,
  courses, slots, sessions, assessments, settings) with the document's data in one transaction.
- **FR-008** (R22): On any error (invalid JSON, failed checks, missing or ill-typed collections,
  rows that violate the §3 constraints or references), restore MUST leave the database exactly as
  it was before the call.
- **FR-009** (R21, R22, T30): Export → restore into an empty planner → export MUST yield a
  document equal to the first except for `exported_at`.
- **FR-010** (R23, T25): Attendance CSV export MUST write the header
  `date,start,end,course_code,course_name,kind,status,replaces_date,note`, then one row per
  session (all courses, all terms, all statuses), sorted by date, then start time, then course
  code, then session identity.
- **FR-011** (R23, T25): In attendance rows, `replaces_date` MUST be the date of the session that a
  makeup replaces, and empty otherwise; `kind` and `status` MUST be the stored values (`regular`
  / `makeup`; `pending` / `attended` / `absent` / `cancelled`); a missing note MUST be empty.
- **FR-012** (R23, T25): Assessment CSV export MUST write the header
  `course_code,kind,title,due_at,completed,notes`, then one row per assessment, sorted by due time,
  then course code, then assessment identity.
- **FR-013** (R23): CSV fields containing a comma, double quote, or line break MUST be quoted with
  embedded double quotes doubled, so the field reads back as the original text.
- **FR-014** (R22, human decision 2026-10-29): The pre-restore copy of the database file to
  `{data_dir}/backups/pre-restore-{YYYYMMDD-HHMM}.sqlite` is out of M3 scope; it is built and
  verified in the M4 app restore flow, where the database path and current time are known.

### Key Entities *(include if feature involves data)*

- **Backup document**: a versioned JSON snapshot of all planner data: format marker, schema
  version, export moment, and one collection per exported table plus settings. Identities inside
  it match the database.
- **Attendance CSV**: a flat, date-sorted listing of every class session with its course code and
  name and, for makeups, the date of the session replaced.
- **Assessments CSV**: a flat, due-time-sorted listing of every assessment with its course code.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A backup round trip (export → restore into empty → export) loses nothing: 100% of
  rows and settings match, and the two documents differ only in the export moment (T30).
- **SC-002**: 100% of rejected restores (bad format, newer version, broken data) leave the
  student's data exactly as before, verified by comparing an export taken before and after the
  failed restore.
- **SC-003**: Both CSV files open in a spreadsheet with the exact column headers from R23, one row
  per session or assessment, in time order, with every text field intact (T25).
- **SC-004**: The milestone gate (format check, lint with warnings as errors, full test suite)
  passes with T25, T30, and the backup tests green, and all earlier milestone tests still green.

## Assumptions

- Timestamps in the backup and CSVs use the formats the database already stores (dates
  `YYYY-MM-DD`, times `HH:MM`, date-times `YYYY-MM-DDTHH:MM`); `exported_at` uses the same
  date-time format. The plan fixes the exact JSON value types and the CSV encoding of
  `completed` and line endings.
- "Backup tests" in the §7 M3 row means tests of R21 and R22 beyond T30 (document shape, identity
  preservation, replace-not-merge, all-or-nothing on broken data); they add no new behavior.
- Restore does not sync sessions or touch the clock; the restored sessions are exactly those in
  the backup.
- Only schema version 1 exists today, so "older version" restores are not exercised beyond
  accepting a version equal to the app's.
- The pre-restore database file copy (R22) belongs to M4 per the human decision dated 2026-10-29;
  M3 adds no core API for it, and the §5 API stays unchanged.
- The reminder log is not exported (R21 does not list it) and restore leaves it untouched.
