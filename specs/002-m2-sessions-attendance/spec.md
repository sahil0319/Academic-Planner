# Feature Specification: M2 Sessions, Exceptions, Attendance, and Dashboard

**Feature Branch**: `002-m2-sessions-attendance` (spec directory; work happens on the current branch)

**Created**: 2026-10-29

**Status**: Draft

**Input**: User description: "Milestone M2 from docs/spec.md: rules R4-R20, tests T01-T19, T21-T23, T27-T29. Do not add requirements."

**Source of truth**: `docs/spec.md` §3 (data model), §4 (rules R4–R20, plus the session-generation
parts of R3 that T15/T28 verify), §5 (core API, `SessionView`), §7 (M2 row), §8 (T01–T19, T21–T23,
T27–T29). This spec restates that scope and adds nothing to it beyond the human decisions dated
2026-10-29 (exam periods as holidays; slot-change sync date; archived courses untouched by sync),
none of which changes a rule, the schema, or the API.

## User Scenarios & Testing *(mandatory)*

The "user" of this milestone is a student who tracks class attendance, and the app layer (M4) that
will show it. Nothing in M2 is visible on screen yet; every scenario is verified by core
integration tests against the §8 fixture: active term **2026-01-05 (Mon) → 2026-01-25 (Sun)**;
course `CSE101` with attendance target 75; slot A Monday 09:00–10:00; slot B Wednesday 11:00–12:00.

### User Story 1 - Weekly classes appear as dated sessions (Priority: P1)

A student has entered their term, holidays, courses, and weekly slots. The planner turns each
weekly slot into one dated class session for every matching weekday in the term, skipping
holidays. Running this again changes nothing. If a holiday is added later, future pending classes
on that day disappear, but classes that already happened stay.

**Why this priority**: Every other M2 feature (attendance, exceptions, dashboard) works on dated
sessions. Without generation there is nothing to mark.

**Independent Test**: Sync the fixture and check the exact set of sessions; sync again and check
nothing changed; add holidays before and after sync and check which sessions exist (T04–T07, T14).

**Acceptance Scenarios**:

1. **Given** the fixture, **When** sessions are synced with today = 2026-01-05, **Then** exactly 6
   regular sessions exist: Mondays 01-05, 01-12, 01-19 and Wednesdays 01-07, 01-14, 01-21, each
   copying start time, end time, and room from its slot (R4, T04).
2. **Given** a synced fixture, **When** sync runs again, **Then** the same 6 sessions exist with the
   same identities, and the sync reports nothing created and nothing deleted (R4, T05).
3. **Given** a holiday on 2026-01-12 set before the first sync, **When** sessions are synced,
   **Then** 5 sessions exist and none is on 01-12 (R4, T06).
4. **Given** a synced fixture, **When** holidays on 2026-01-19 and 2026-01-05 are added and sync
   runs with today = 2026-01-14, **Then** the pending 01-19 session is deleted and the 01-05
   session (in the past) is kept (R5, T07).
5. **Given** a slot whose valid-to date is the term end date and the term ends on that slot's
   weekday, **When** sessions are synced, **Then** a session is generated on the term end date
   (range is inclusive) (R4, T14).

---

### User Story 2 - Change or end a weekly slot without rewriting history (Priority: P1)

Partway through the term a class moves to another day or time, or stops meeting. The student
records the change with the date it takes effect. Classes before that date keep the day, time,
and room they actually had; future pending classes follow the new rule.

**Why this priority**: Timetables change mid-term in practice; rewriting past sessions would
corrupt attendance history.

**Independent Test**: Change slot A's weekday and time with an effective date, end slot A at a
date, and check which sessions exist and what times past sessions keep (T10, T11, T27).

**Acceptance Scenarios**:

1. **Given** a synced fixture and today = 2026-01-14, **When** slot A is changed to Tuesday
   effective 01-14, **Then** Monday 01-05 and 01-12 are kept, Monday 01-19 is deleted, Tuesday
   01-20 is created, and Tuesday 01-13 is not created (R7, R5, T10).
2. **Given** a synced fixture, **When** slot A's time is changed to 10:00–11:00 effective 01-14,
   **Then** the 01-05 and 01-12 sessions keep 09:00–10:00 (R7: sessions are snapshots, T11).
3. **Given** a synced fixture, **When** slot A is ended at 2026-01-11 and sync runs with today =
   2026-01-05, **Then** the pending Monday 01-12 and 01-19 sessions are deleted and 01-05 is kept
   (R5, T27).

---

### User Story 3 - Mark attendance and see the attendance percentage (Priority: P1)

After a class starts, the student marks it attended or absent. They can cancel a class or reset it
to pending at any time. For each course they see counts per status, their attendance percentage,
and whether they are below the course's target.

**Why this priority**: Attendance tracking against a target is the core value of the app.

**Independent Test**: Set statuses on fixture sessions at given moments and read the course
summary (T01–T03, T12, T21, T23).

**Acceptance Scenarios**:

1. **Given** a course with 3 attended, 1 absent, 2 cancelled, and 5 pending sessions, **When** its
   summary is read, **Then** the percentage is 75.0 and the counts are 3 / 1 / 2 / 5 (R13, R15,
   T01).
2. **Given** a course with only cancelled and pending sessions, **When** its summary is read,
   **Then** the percentage is "none" (shown as "—") (R13, T02).
3. **Given** target 75, **When** the percentage is 74.9 / 75.0 / there is no target, **Then**
   below-target is true / false / false (R14, T03).
4. **Given** a session that has not started yet, **When** it is marked attended, **Then** a
   validation error is returned and nothing changes; **When** it is cancelled instead, **Then**
   that succeeds (R12, T12).
5. **Given** a course with no slots, **When** its summary is read, **Then** all counts are 0 and
   the percentage is "none" (R13, R15, T21).
6. **Given** an attended session, **When** it is reset to pending, **Then** that is allowed and the
   summary counts update (R12, T23).

---

### User Story 4 - Cancel, reschedule, and add makeup classes (Priority: P2)

A class is cancelled, or moved to another date (a makeup that replaces it), or an extra one-off
makeup class is added. Cancelled classes stay cancelled when the timetable is re-synced. A moved
class shows where it moved to. Classes already attended or absent cannot be moved.

**Why this priority**: Exceptions are common but less frequent than routine marking; they build on
Stories 1 and 3.

**Independent Test**: Cancel, reschedule, and add makeups on fixture sessions, re-sync, and check
statuses, display labels, and summary counts (T08, T09, T19, T22).

**Acceptance Scenarios**:

1. **Given** a synced fixture, **When** the 01-12 session is cancelled and sync runs again,
   **Then** 01-12 is still cancelled and slot A is unchanged (R8, T08).
2. **Given** a synced fixture, **When** the 01-12 session is rescheduled to a makeup on 01-13 and
   the makeup is marked attended, **Then** the summary shows attended 1 and cancelled 1, and the
   original session's display status is "Rescheduled → 2026-01-13" (R9, T09).
3. **Given** an attended session, **When** it is rescheduled, **Then** an error is returned and
   nothing changes: no makeup is created and the original keeps its status (R9, T19).
4. **Given** a course, **When** a standalone makeup class is added and marked attended, **Then** it
   counts as attended in the summary (R10, T22).

---

### User Story 5 - Dashboard shows what matters now (Priority: P2)

When the student opens the app they see today's classes in time order, the next upcoming class,
assessments due soon, overdue assessments, classes that ended but still need attendance marked,
and courses below their attendance target.

**Why this priority**: The dashboard is the daily entry point, but it only reads data the earlier
stories produce.

**Independent Test**: Build the dashboard at fixed moments over the fixture plus assessments and a
second course (T13, T16–T18, T29).

**Acceptance Scenarios**:

1. **Given** the 01-05 09:00–10:00 session is pending, **When** the dashboard is built at 01-05
   09:30, **Then** it is not awaiting attendance (in progress); **When** built at 01-05 10:00,
   **Then** it is awaiting attendance (R20, T13).
2. **Given** an incomplete assessment due before now and a completed one due before now, **When**
   the dashboard is built, **Then** the first is overdue and the second is not (R19, T16).
3. **Given** an incomplete assessment due exactly now + 7 days (default upcoming window), **When**
   the dashboard is built, **Then** it is included in upcoming (R18, T17).
4. **Given** the earliest future session is cancelled, **When** the dashboard is built, **Then** the
   next class is the following pending session (R17, T18).
5. **Given** two courses with sessions on 01-05, **When** the dashboard is built on 01-05, **Then**
   today's classes contain both courses' sessions sorted by start time, each with its status (R16,
   T29).

---

### User Story 6 - Archived courses stop generating classes but keep history (Priority: P3)

A student archives a dropped course. It disappears from the dashboard and generates no new
sessions, but its history and summary remain. Unarchiving it resumes generation without
duplicates.

**Why this priority**: Less frequent; the R3 storage part shipped in M1, and M2 adds only its effect
on sessions and the dashboard.

**Independent Test**: Archive the fixture course, sync, build the dashboard and summary; unarchive,
sync, count sessions (T15, T28).

**Acceptance Scenarios**:

1. **Given** a synced fixture, **When** the course is archived and sync runs, **Then** the course is
   absent from the dashboard, its sessions are kept, its summary can still be computed, and sync
   creates no new sessions for it (R3, T15).
2. **Given** an archived course, **When** it is unarchived and sync runs, **Then** session
   generation resumes and no duplicate sessions exist (R3, T28).

---

### User Story 7 - Midterm and final exam periods have no classes (Priority: P2)

Each semester has a midterm and a final exam period. During a period there are no classes, only
exams, and some days in between are prep days with no exam. The student does not know the dates
when the term starts, so they enter each period later, when it is announced, as holidays covering
every date of the period (it may start and end mid-week and span weeks). Classes on those dates
disappear and need no attendance. The exams themselves are added as exam assessments on their own
dates.

**Why this priority**: Without this, exam weeks would show as missed classes awaiting attendance
and distort the attendance percentage. It reuses holiday behavior (R4, R5), so it adds no new rule
(human decision 2026-10-29).

**Independent Test**: Sync the fixture, then add holidays for a period that starts on a Wednesday
and ends on the Friday of the next week, sync again, and check which sessions exist; add exam
assessments inside the period and check they appear like any other assessment. Verified with the
same mechanics as T06/T07; no new T-ID is added to `docs/spec.md` §8.

**Acceptance Scenarios**:

1. **Given** a synced fixture and today = 2026-01-09, **When** every date from Wednesday 2026-01-14
   through Friday 2026-01-23 is added as a holiday labeled "Midterm exams" and sync runs, **Then**
   the pending sessions on 01-14, 01-19, and 01-21 are deleted, the 01-05, 01-07, and 01-12
   sessions are kept, and no session in the period appears in awaiting attendance (R4, R5,
   FR-006a).
2. **Given** that exam period, **When** exam assessments are added due on 01-15 and 01-20 (the
   days between are prep days), **Then** they appear in upcoming/overdue like any other assessment
   and no class session exists on the prep days (R18, R19, FR-006a).
3. **Given** an exam period entered as holidays, **When** one of its dates is removed from the
   holidays and sync runs, **Then** that date's regular sessions are generated again (R4).
4. **Given** a session inside a future exam period was already cancelled, **When** the period is
   entered as holidays and sync runs, **Then** the cancelled session is kept (R5).

---

### Edge Cases

- Generation range starts at the later of term start and slot valid-from, and ends at the earlier
  of term end and slot valid-to (open valid-to means term end); both ends inclusive (R4, T14).
- A holiday on a slot weekday produces no session that day (R4, T06).
- A session dated today counts as "today or later" for deletion, so a pending regular session today
  that is no longer produced is deleted (R5: `date >= today`).
- Sync never deletes: past sessions; attended, absent, or cancelled sessions; makeup sessions (R5).
- A cancelled regular session is never recreated or reset to pending by sync (R8, T08).
- Marking attended/absent at exactly the session start moment is allowed (R12: `now >= start`).
- Awaiting attendance includes a session at exactly its end moment and excludes one in progress
  (R20, T13).
- Next class requires start strictly after now; a session starting exactly now is not "next" (R17).
- Upcoming includes an assessment due exactly now and exactly now + window; overdue is strictly
  before now (R18, R19, T17).
- Percentage is rounded to 1 decimal; 74.9 against target 75 is below target, 75.0 is not (R13,
  R14, T03).
- Rescheduling a pending or cancelled session is allowed; rescheduling an attended or absent
  session is an error with no partial change (R9, T19).
- A course with no slots has a valid, all-zero summary (T21).
- Unarchiving and re-syncing never creates a second session for the same slot and date (R4
  uniqueness, T28).
- Sync neither creates nor deletes sessions of an archived course (FR-006).
- A slot change with an effective date in the past also replaces the old slot's pending sessions
  from that date on, because its sync uses the effective date as today (FR-007).
- An exam period entered after classes were already marked: attended, absent, or cancelled
  sessions inside the period are kept; only future pending ones are removed (R5, FR-006a).
- Prep days inside an exam period are holidays like exam days, so they have no classes either
  (FR-006a).
- A makeup class may still be added on an exam-period date; holidays only stop regular sessions
  (R4, R10).

## Requirements *(mandatory)*

### Functional Requirements

#### Session generation

- **FR-001** (R4, T04, T06, T14): Sync MUST create, for each slot of each non-archived course in
  the active term, exactly one regular session on every date in the inclusive range
  [later of term start and slot valid-from, earlier of term end and slot valid-to (term end if
  open)] whose weekday equals the slot's weekday and which is not a holiday. Each created session
  MUST copy start time, end time, and room from the slot at creation time; a slot without a room
  means "use the course room" (§3), so the course room is copied then (research R-D3).
- **FR-002** (R4, T05): Sync MUST be idempotent: repeated runs with no input changes MUST leave the
  same sessions with the same identities and report zero created and zero deleted.
- **FR-003** (R5, T07, T27): Sync MUST delete a session only if all hold: it is regular, it is
  pending, its date is on or after today, and FR-001 no longer produces it. It MUST NOT delete any
  other session.
- **FR-004** (§5): Sync MUST report how many sessions it created and deleted.
- **FR-005** (R6): Sync MUST be safe to run at any time (app start, after any term, holiday,
  course, or slot change, and on date rollover), because it is idempotent (FR-002) and only removes
  sessions per FR-003. Wiring those triggers into the app is M4 (§7).
- **FR-006** (R3, T15, T28): Archived courses MUST get no new sessions from sync, and sync MUST NOT
  delete any of their sessions: R3 ("its sessions are kept") takes precedence over R5, so FR-003
  applies only to sessions of non-archived courses (human decision 2026-10-29). After unarchiving,
  the next sync MUST resume generation without duplicates.
- **FR-006a** (R4, R5, human decision 2026-10-29): A midterm or final exam period MUST be
  representable as holidays: every date in the period is a holiday (labeled, e.g., "Midterm
  exams" or "Final exams"), including prep days inside the period that have no exam. A period may
  start and end on any weekday and span several weeks (e.g., Wednesday of one week through Friday
  of a later week). Holidays MAY be entered at any time, including after sessions already exist;
  the next sync then creates no class sessions on those dates and deletes the future pending ones
  already there (FR-001, FR-003), so no attendance is expected for them. If a period's dates
  change later, removing a holiday makes the next sync generate that date's classes again
  (FR-001). The exams themselves are assessments of kind "exam", whose due date-times can be set
  or edited later (M1 assessment operations).

#### Slot changes

- **FR-007** (R7, T10, T11): Changing a slot with an effective date MUST set the old slot's
  valid-to to the day before the effective date, create a new slot (with the new values) valid from
  the effective date, and then sync using the effective date as "today" (human decision
  2026-10-29; the §5 signature has no today parameter and stays unchanged). Existing sessions MUST
  keep their stored date, time, and room (sessions are snapshots).
- **FR-008** (§5, T27): Ending a slot MUST make the given date the slot's last valid date, and then
  sync using the day after that date as "today" (same decision as FR-007); its pending regular
  sessions after that date are removed per FR-003.

#### Exceptions

- **FR-009** (R8, T08): Cancelling a session MUST set its status to cancelled without changing its
  slot; sync MUST NOT recreate or reset it.
- **FR-010** (R9, T09, T19): Rescheduling MUST, as one all-or-nothing step, create a makeup
  session that references the original and set the original to cancelled. Rescheduling an
  attended or absent session MUST return an error and change nothing.
- **FR-011** (R10, T22): Adding a one-off makeup MUST create a makeup session that replaces no
  session; it MUST count in attendance like any other session.

#### Attendance

- **FR-012** (R11): Session status MUST be one of pending (default), attended, absent, cancelled.
  Status MUST never change automatically except as stated in FR-003, FR-009, and FR-010.
- **FR-013** (R12, T12, T23): Setting attended or absent MUST be allowed only when now is at or
  after the session's start (date + start time); otherwise it MUST return a validation error and
  change nothing. Setting cancelled or pending MUST be allowed at any time.
- **FR-014** (R13, T01, T02, T21): Attendance percentage MUST be attended / (attended + absent) ×
  100, rounded to 1 decimal, and MUST be "none" when attended + absent is 0.
- **FR-015** (R14, T03): Below-target MUST be true only when the course has a target, the
  percentage is not "none", and the percentage is less than the target.
- **FR-016** (R15, T01, T21): A course's summary MUST report counts of attended, absent, cancelled,
  and pending sessions (past and future), the percentage, the target, and below-target. A summary
  for every course MUST also be available.

#### Session views

- **FR-017** (§5): Session lists MUST be available for a date range (non-archived courses) and for
  one course, each item carrying course code and name, a display status, and an "is past" flag
  (now at or after the session start).
- **FR-018** (§5, T09): Display status MUST be "Pending", "Attended", "Absent", or "Cancelled",
  except that a cancelled session replaced by a makeup MUST display "Rescheduled → {makeup date}"
  (date as `YYYY-MM-DD`).

#### Dashboard (built for a given now)

- **FR-019** (R16, T29): Today's classes MUST be all sessions dated now's date for non-archived
  courses, sorted by start time, each with its status.
- **FR-020** (R17, T18): Next class MUST be the earliest pending session of a non-archived course
  whose start is strictly after now, or "none".
- **FR-021** (R18, T17): Upcoming assessments MUST be the incomplete ones with now ≤ due ≤ now +
  upcoming window days (setting), sorted by due date-time.
- **FR-022** (R19, T16): Overdue assessments MUST be the incomplete ones due strictly before now.
- **FR-023** (R20, T13): Awaiting attendance MUST be pending sessions of non-archived courses whose
  end (date + end time) is at or before now, oldest first; sessions in progress MUST NOT appear.
- **FR-024** (R14, §5 `Dashboard`, §6): The dashboard MUST list the non-archived courses that are
  below target (FR-015), for the below-target banner.
- **FR-025** (R3, T15): Archived courses MUST NOT appear anywhere in the dashboard.

#### Constraints

- **FR-026** (§1, §5): No M2 operation may read the system clock; today/now are passed in. All
  operation signatures MUST stay exactly as frozen in §5.

### Key Entities *(include if feature involves data)*

- **Session**: one dated class occurrence of a course: kind (regular, from a slot; or makeup),
  date, start and end time, room, status (pending, attended, absent, cancelled), optional
  reference to the session it replaces (makeups only), optional note. At most one regular session
  per slot per date.
- **Slot**: recurring weekly rule (from M1); M2 adds changing it from an effective date and ending
  it at a last date.
- **Sync report**: counts of sessions created and deleted by one sync.
- **Session view**: a session plus its course code and name, display status, and "is past" flag.
- **Attendance summary**: per course: attended, absent, cancelled, pending counts; percentage or
  "none"; target; below-target flag.
- **Dashboard**: today's classes, next class, upcoming assessments, overdue assessments, awaiting
  attendance, below-target courses.
- **Holiday**: from M1; a class-free date with an optional label. Also used for every date of a
  midterm or final exam period, including prep days (FR-006a).
- **Term, Course, Assessment, Settings**: from M1; read by M2 (upcoming window days comes from
  Settings; exams are assessments of kind "exam").

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All 25 listed tests pass: T01–T19, T21–T23, T27–T29, each named per the pattern
  `t07_holiday_added_after_sync`.
- **SC-002**: The milestone gate (format check, lint with warnings as errors, full test suite)
  passes with zero failures and zero warnings, and M1 tests (T20, T24, T26) still pass.
- **SC-003**: Running sync twice in a row on any test data set produces zero creations and zero
  deletions on the second run (T05).
- **SC-004**: In every test, no past session and no non-pending session is ever deleted by sync
  (T07, T08, T10, T27).
- **SC-005**: No §5 operation signature changes, so M4 UI work built against the M1 stubs needs no
  edits.
- **SC-006**: In the exam-period scenarios (User Story 7), 0 class sessions exist inside a period
  after sync, and 0 of them appear in awaiting attendance or count toward the percentage.

## Assumptions

- Scope is exactly the §7 M2 row plus the R3 session/dashboard effects that T15/T28 verify; no
  requirement beyond `docs/spec.md` is added.
- R6 trigger points (app start, after changes, 60-second date-rollover check) are app behavior
  wired in M4 (§7 M4 row: "sync triggers per R6"). M2 delivers a sync that is safe to call at
  those points.
- "Summary for every course" (all summaries) covers the non-archived courses of the active term,
  matching R3 ("hidden from default lists"); a single archived course's summary is still available
  on request (T15).
- The upcoming window uses the stored `upcoming_window_days` setting (default 7, from M1).
- "Now + N days" means exactly N × 24 hours later in naive local time (no time zones, §1).
- A makeup created by reschedule belongs to the course given in its input; the spec does not
  require it to match the original's course, and M2 adds no such check.
- Rescheduling an already-cancelled session is allowed (R9 forbids only attended/absent).
- Tests use the §8 fixture with an in-memory store; no test touches the real user data file.
- Exam periods (human decision 2026-10-29) are modeled as holidays, not as a new concept: no
  schema, API, or rule change. Entering a whole date range in one step is a UI convenience for M4;
  `docs/spec.md` §6 currently lists holidays as add/remove one date + label, so a range input
  would need a §6 update first. In M2, the caller passes every date of the period to the existing
  set-holidays operation.
- Human decisions 2026-10-29: the internal sync of a slot change uses the effective date as today
  (FR-007, FR-008); R3 takes precedence over R5 for archived courses (FR-006).
