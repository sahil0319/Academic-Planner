# Feature Specification: M10 Courses and Timetable Import

**Feature Branch**: `010-m10-course-import` (spec directory; work happens on the current branch)

**Created**: 2026-10-29

**Status**: Draft

**Input**: User description: "Milestone M10 from docs/spec.md §13.4: rules R42–R50, tests T44–T50.
Do not add requirements."

**Source of truth**: `docs/spec.md` §7 (M10 row), §13.4 (format, R42–R50, API additions, UI),
§13.5 (T44–T50), and from §13.3 only the fields M10 depends on: course `credits` and
`rule_set_id`, and rule set names (`rule_set.name`). R7 (slot change semantics), R2 (validation),
R4–R6 (sync), and R1 (active term) still apply. This feature adds no schema change and no core
method beyond the two in §13.4.

## Dependencies

- **M9 (planned concurrently in `specs/009-m9-grading-gpa/`)**: migration 002 adds
  `course.credits`, `course.rule_set_id`, and the `rule_set` table with the seeded rule sets
  "Theory" and "Lab"; `CourseInput` gains `credits: Option<f64>` and `rule_set_id: Option<i64>`;
  `list_rule_sets()` returns the rule sets. M10 refers to these by the names in §13.3.
  Implementation of M10 starts after M9's core is merged.
- **M7**: R26 pre-migration backup is unchanged; M10 adds no migration, so it triggers none.
- **M8**: the import format keeps 24-hour `HH:MM` (R31); M8's 12-hour display applies to how the
  preview shows times in the app.

## Router Notes (2026-10-29, recorded; not new requirements)

- Robustness is the priority: idempotence (T45), the slot-set equality check, archived matches,
  null versus absent fields, unknown keys as warnings, and day-name variants must each be covered
  by acceptance scenarios and tests.

## User Scenarios & Testing *(mandatory)*

The user is a student who has their semester timetable as a table (often a photo or a grid with
merged period cells). They want to get all courses and weekly slots into the app at once, and to
re-import a corrected table later without losing attendance history or creating duplicates.

### User Story 1 - First import into an empty planner (Priority: P1)

The user pastes a JSON document describing the term and courses with slots, previews it, picks the
effective date (for a fresh term, the term start), and applies it. The term, courses, and slots are
created, and the class sessions appear from the effective date on.

**Why this priority**: This is the main reason for the feature: one paste instead of dozens of
form entries.

**Independent Test**: T44 and T50 (core tests on an in-memory database with no term).

**Acceptance Scenarios**:

1. **Given** a database with no active term and a valid file with `term` and two courses with
   slots, **When** the file is applied with an effective date, **Then** the term is created and
   active, both courses exist with the file's fields, their slots start at the effective date, and
   regular sessions exist from the effective date to the term end (T44, R47, R45).
2. **Given** a database with no active term and a file without `term`, **When** it is previewed,
   **Then** the preview has the error `term: required when there is no active term` and apply
   writes nothing (R47, R42).
3. **Given** the shipped `docs/import/template.json` and a fresh database with the seeded rule
   sets, **When** it is previewed, **Then** it has no errors and no warnings (R50).
4. **Given** a valid file, **When** previewed, **Then** nothing is written; **When** then applied,
   **Then** the applied changes equal the previewed plan (T50, R48).

---

### User Story 2 - Re-import safely (Priority: P1)

The user imports the same or a corrected file again during the term. Nothing is duplicated;
unchanged courses stay untouched; changed slots take effect from the chosen date while past
sessions keep their recorded time.

**Why this priority**: Timetables change during a term; a re-import that duplicates courses or
rewrites history would destroy trust in the data.

**Independent Test**: T45, T46, T47, T49 (core tests).

**Acceptance Scenarios**:

1. **Given** a file was applied, **When** the same file with the same effective date is previewed
   again, **Then** the plan has no term to create, no courses to add, no courses to update, and no
   slot changes; applying it changes nothing (T45, R48).
2. **Given** a course whose slot is Mon 10:30–11:45 and a file with the same course at Mon
   11:00–12:15, **When** applied with effective date 01-14, **Then** sessions before 01-14 keep
   10:30–11:45 and sessions from 01-14 on are at 11:00–12:15 (T46, R45, R7).
3. **Given** the database has course `CSE 4711` and the file has `cse4711`, **When** applied,
   **Then** that course is matched and updated, its code stays `CSE 4711`, and no second course is
   created (T47, R43).
4. **Given** a course in the database that is absent from the file, **When** the file is applied,
   **Then** that course, its slots, and its sessions are unchanged (T49, R46).
5. **Given** the file's slots for a course equal (day, start, end, room) the course's slots valid
   on the effective date, listed in any order, **When** previewed, **Then** no slot change is
   planned for that course (R45).
6. **Given** a course in the file with `"slots": []`, **When** applied, **Then** every slot of
   that course valid on the effective date ends the day before it (R45).
7. **Given** a course in the file without a `slots` key, **When** applied, **Then** its slots are
   untouched (R45).
8. **Given** an existing course and a file that omits `instructor`, **When** applied, **Then** the
   instructor is kept; with `"instructor": null` it is cleared (R44).
9. **Given** the matched course is archived, **When** previewed, **Then** the preview warns that it
   is archived, and after apply it is still archived with the file's changes applied (R44).
10. **Given** an active term exists and the file's `term` differs from it, **When** previewed,
    **Then** the term is not changed and the preview warns that the file's term is ignored (R47).

---

### User Story 3 - See every problem before anything is written (Priority: P1)

The user pastes a file with several mistakes. The preview lists all of them, each with the place
in the file, and nothing is written until the file is fixed.

**Why this priority**: AI-converted timetables often contain small mistakes; showing all of them
at once with their location makes them quick to fix.

**Independent Test**: T48 plus the parse robustness tests (core tests).

**Acceptance Scenarios**:

1. **Given** a file with a bad day name, a slot whose end is not after its start, a duplicate
   course code, and an unknown rule set, **When** previewed or applied, **Then** all four errors
   are returned, each naming its JSON path (e.g. `courses[2].slots[0].end: must be after start`),
   and the database is unchanged (T48, R42).
2. **Given** a file with a key the format does not define (e.g. `courses[0].teacher`), **When**
   previewed, **Then** it is listed as a warning with its path and ignored; it is not an error
   (R42).
3. **Given** slot days written as `Mon`, `monday`, `MONDAY`, `1`, or the number `1`, **When**
   previewed, **Then** each is read as Monday; `Mo`, `0`, or `8` is an error at that slot's `day`
   path (§13.4 format).

---

### User Story 4 - Import panel on the Courses page (Priority: P2)

On the Courses page, "Import…" opens a panel with a paste box, Copy template, Copy AI prompt,
Preview, an effective date, and Apply. The preview shows the plan (courses to add, field changes,
slot changes, warnings) or the errors with their paths.

**Why this priority**: The core rules (Stories 1–3) carry the value; the panel exposes them.

**Independent Test**: View-model formatting is unit tested; the panel is checked by running the
app.

**Acceptance Scenarios**:

1. **Given** the Import panel, **Then** the effective date defaults to today and can be changed
   (R49).
2. **Given** the user presses Copy template or Copy AI prompt, **Then** the content of
   `docs/import/template.json` or `docs/import/ai-prompt.md` is on the clipboard (R50).
3. **Given** the user presses Preview, **Then** the panel shows the plan or the errors with paths,
   and the warnings (§13.4 UI, R48).
4. **Given** a preview with no errors and at least one change, **When** the user presses Apply,
   **Then** the plan is applied, the panel reports what was done, and the Courses, Timetable, and
   Dashboard pages show the new data.

---

### User Story 5 - Convert a timetable with an AI assistant (Priority: P3)

The user copies the AI prompt, pastes it with their timetable into an AI assistant, and gets back
a file in the import format, with merged period cells turned into one slot spanning those periods.

**Why this priority**: It saves typing, but the import works without it.

**Independent Test**: Read `docs/import/ai-prompt.md`; it states the format, 24-hour times, and
the period → time rule for merged cells, and includes the user's 7th-semester table as a worked
example.

**Acceptance Scenarios**:

1. **Given** the AI prompt, **Then** it contains the format, the template, and the rule that a
   class spanning merged period cells starts at the first period's start and ends at the last
   period's end (R50).
2. **Given** the AI prompt, **Then** it includes the user's 7th-semester table as a worked example
   with 24-hour times (router instruction for the R50 content).

---

### Edge Cases

- The pasted text is not valid JSON: one error at path `document` with the line and column; nothing
  else is checked.
- The root is not an object, `courses` is not a list, or a course is not an object: an error at
  that path; the rest of the document is still checked where possible.
- `format` is not `academic-planner-courses` or `version` is not `1`: an error at that path.
- Two courses in the file whose codes match ignoring case and whitespace (`CSE 4711`,
  `cse4711`): a duplicate-code error at the second one's `code` path (R42, R43).
- `code` or `name` is `null`, or empty after trimming: an error (R2). `name` is required for a
  course that will be created; for a matched course it may be absent (keep).
- `"slots": null` is an error (use `[]` to end all slots); a missing `slots` key leaves slots
  untouched (R45).
- A slot `room` that is absent or `null` means "use the course room" (no room on the slot).
- Duplicate identical slots in the file are compared and created as written (the comparison counts
  each slot); no error is added.
- `credits: 0`, a negative value, or a non-number: error; `credits: null` clears it (course becomes
  ungraded per R33).
- `min_attendance_pct` outside 0–100 or not a whole number: error; `null` clears it.
- `rule_set: null` clears it; a name that is not an existing rule set is an error listing the
  available names.
- An active term exists and the file's `term` is malformed (e.g. a bad date): it is still an error,
  because the whole document is validated (R42).
- The effective date is before today (back-dated re-import): R7 semantics apply as in
  `change_slot`, so pending regular sessions of ended slots from the effective date on are
  removed and the new slots generate from the effective date; attended, absent, and cancelled
  sessions are never removed (R5, R7).
- A slot valid on the effective date that itself started on the effective date is ended the day
  before it (it produces no sessions), as `change_slot` does (R7).
- A file with `courses: []` and a `term` into a database without an active term: only the term is
  created.
- An empty plan (nothing to change): Apply writes nothing.

## Requirements *(mandatory)*

### Functional Requirements

**Parse and validate (R42)**

- **FR-001** (R42): The whole document MUST be parsed and validated before anything is written. Any
  error MUST mean nothing is written.
- **FR-002** (R42): Errors MUST be collected (not only the first), and each MUST name its JSON path
  in the form `courses[2].slots[0].end: must be after start`.
- **FR-003** (R42): Checks MUST cover: `format` equals `academic-planner-courses` and `version`
  equals 1; required fields; dates (`YYYY-MM-DD`); times (24-hour `HH:MM`); day names; slot end
  after start; term end not before start (R2); credits > 0; `min_attendance_pct` 0–100; `rule_set`
  names an existing rule set; duplicate course codes within the file (compared as in R43).
- **FR-004** (§13.4 format): `day` MUST accept `Mon`/`Monday`/`1` in any case (ISO numbering,
  1 = Monday … 7 = Sunday), as a string or a JSON number.
- **FR-005** (R42): Keys the format does not define MUST be ignored and reported as warnings with
  their path.

**Matching and merging (R43, R44, R46)**

- **FR-006** (R43): A file course MUST match an active-term course (archived or not) whose code is
  equal ignoring case and whitespace. A code with no match MUST create a new course with the code
  as written. A matched course's stored code is not changed.
- **FR-007** (R44): For a matched course, only fields present in the file MUST be updated; an
  absent field keeps its value; an explicit `null` clears it. Only fields whose value differs are
  listed as changes.
- **FR-008** (R44): An archived match MUST stay archived, and the preview MUST warn.
- **FR-009** (R46): Courses not in the file MUST never be changed or deleted.

**Slots (R45)**

- **FR-010** (R45): For each file course with a `slots` key, if the file's slots (day, start, end,
  room) equal, as an unordered collection, the course's slots valid on the effective date, nothing
  changes.
- **FR-011** (R45, R7): Otherwise every slot of that course valid on the effective date MUST be
  ended at effective − 1 day and the file's slots added from the effective date; past sessions keep
  their stored values.
- **FR-012** (R45): `"slots": []` MUST end all of the course's current slots; a missing `slots`
  key MUST leave them untouched. A new course's slots start at the effective date.

**Term (R47)**

- **FR-013** (R47): With no active term, `term` MUST be present and is created as the active term.
- **FR-014** (R47): With an active term, the file's `term` MUST be ignored and the preview MUST
  warn when its name, start, or end differs from the active term.

**Preview and apply (R48, R49)**

- **FR-015** (R48): `preview_import(json, effective_from)` MUST write nothing and MUST return the
  plan: the term to create (if any), courses to add, courses to update with field-by-field
  changes, slot changes per course, warnings, and errors.
- **FR-016** (R48): `apply_import(json, effective_from, today)` MUST apply exactly the plan that
  `preview_import` returns for the same input and database, in one transaction, then sync (R6). If
  the plan has errors it MUST return a validation error listing them all with paths and write
  nothing.
- **FR-017** (R48, T45): Importing the same file twice with the same effective date MUST produce
  an empty second plan, and applying it MUST change nothing.
- **FR-018** (R49): In the UI the effective date MUST default to today and be changeable in the
  preview.

**Template, AI prompt, and UI (R50, §13.4 UI)**

- **FR-019** (R50): `docs/import/template.json` MUST be a valid import file that previews with no
  errors and no warnings against a fresh database with the seeded rule sets.
- **FR-020** (R50): `docs/import/ai-prompt.md` MUST ask the assistant to convert a timetable to
  this format, state the period → time rule for merged cells, require 24-hour times, and include
  the user's 7th-semester table as a worked example.
- **FR-021** (§13.4 UI): The Courses page MUST have "Import…" opening a panel with a paste box,
  Copy template, Copy AI prompt, Preview (plan and errors with paths), the effective date, and
  Apply. Copy buttons put the shipped file's text on the clipboard.
- **FR-022** (§6, constitution III): All text shown in the panel MUST be produced by view-model
  functions with unit tests; `.slint` files contain only bindings and use `Theme.*` only.

### Key Entities

- **Import file**: a JSON document in the §13.4 format: `format`, `version`, optional `term`
  (name, start, end), and `courses`, each with `code` and optional `name`, `credits`, `rule_set`,
  `instructor`, `room`, `min_attendance_pct`, `slots` (day, start, end, optional room).
- **Import plan**: what an import would do: term to create, courses to add (with their slots),
  courses to update (field changes old → new, slots to end and slots to add, archived flag),
  warnings, and errors; each warning and error has a JSON path and a message.
- **Import report**: the plan that was applied plus the sync result.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The gate passes and T44–T50 pass (§7).
- **SC-002**: Importing the same file twice never creates a duplicate course or slot and never
  changes a session.
- **SC-003**: A file with N independent mistakes reports all N, each with its location, and leaves
  the data unchanged.
- **SC-004**: After a re-import with a changed slot, every session before the effective date shows
  the time it had before.
- **SC-005**: The user can go from a pasted file to a populated term with courses and a weekly
  timetable in one preview and one apply.

## Assumptions

- Matching considers only courses of the active term (course codes are unique per term, §3).
- Only the keys shown in the §13.4 format are known; any other key, including `notes` or
  `archived`, is an unknown key (warning, ignored).
- Text values are stored as written; only course-code matching normalizes case and whitespace.
- The effective date is not checked against the term range (none of R42–R50 requires it); R4
  clamps generation to the term.
- Values are compared after parsing: `3` and `3.0` credits are equal; times are compared as times.
- The template's contents are the §13.4 example.

## Open Questions (for the router; defaults used until answered)

- **OQ-1** (R43 gap): the active term may already hold two courses whose codes differ only in case
  or spacing (e.g. `CSE 4711` and `CSE4711`, allowed by `UNIQUE(term_id, code)`). Options: (A)
  error at the file course's `code` path, "matches more than one course"; (B) match the lowest id.
  Recommended: A (nothing written, user resolves it). The plan uses A.
- **OQ-2** (R50 content): the user's 7th-semester table is not in the repository. The human must
  provide it (periods with times and the course grid) for the worked example in
  `docs/import/ai-prompt.md`. A (human) task covers it.
- **OQ-3** (R45 wording): which slots "every current slot" covers. Options: (A) every slot of the
  course not ended before the effective date, including a slot that starts after it (left by an
  earlier import with a later effective date); (B) only slots valid on the effective date.
  Recommended: A (B can leave a future-dated slot running beside the new ones). The plan uses A.

## Router decisions (2026-10-29)

- OQ-1 (two DB courses with the same normalized code): the import fails with an error naming both courses; nothing is written.
- OQ-2 (the 7th-semester table): supplied by the human, saved as `worked-example-source.md` in this folder. T001 is done; T027 uses it.
- OQ-3 (R45 "current slots"): this clarifies R45 and does not change it. Every slot that has not ended by the day before the effective date, including slots that start after it, is ended, so replacing a timetable never leaves duplicate sessions.
