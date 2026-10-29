# Feature Specification: M5 Reminders

**Feature Branch**: `005-m5-reminders` (spec directory; work happens on the current branch)

**Created**: 2026-10-29

**Status**: Draft

**Input**: User description: "Milestone M5 from docs/spec.md: reminders with a 60-second timer in
the app, `due_reminders(now)`, and `notify-rust` notifications that fire only while the app is
running. Done when the gate passes, reminder dedupe tests pass, and one manual notification is
observed. Do not add requirements."

**Source of truth**: `docs/spec.md` §1 (Notifications row: `notify-rust`; Clock row), §3
(`assessment.remind_minutes_before`, the `setting` defaults `class_reminder_minutes=15` and
`notifications_enabled=1`, and the `reminder_log` table), §5 (`due_reminders(&mut self, now) ->
Result<Vec<Reminder>>`, "logs to reminder_log, never returns one twice"), §7 (M5 row), and §10
(no notifications while the app is closed). `docs/spec.md` gives no rule IDs for reminders; the
behavior details below are router decisions dated 2026-10-29, made under the human's overnight
delegation ("take the recommended option"), and change no rule, schema, or §5 API.

## User Scenarios & Testing *(mandatory)*

The user is a student who keeps the app open during the day and wants a desktop notification a
few minutes before each class and before each assessment they asked to be reminded about.
Behavior is verified by core integration tests on the §8 fixture (active term 2026-01-05 →
2026-01-25; `CSE101`; slot A Monday 09:00–10:00; slot B Wednesday 11:00–12:00) and by one manual
notification check.

### User Story 1 - Class reminder before each class (Priority: P1)

While the app is open, the student gets one notification `class_reminder_minutes` before each
upcoming class they have not cancelled.

**Why this priority**: Classes are the daily event; it is the main reason for reminders.

**Independent Test**: Sync the fixture; call `due_reminders` at 08:45 on 2026-01-05 with the
default 15 minutes; one session reminder for the 09:00 class is returned; a second call at 08:46
returns nothing.

**Acceptance Scenarios**:

1. **Given** a pending session at 2026-01-05 09:00 and `class_reminder_minutes = 15`, **When**
   `due_reminders(2026-01-05 08:45)` runs, **Then** it returns exactly one `Session` reminder for
   that session with `fire_at = 2026-01-05 08:45` and a title naming the course code, name, and
   start time.
2. **Given** the same, **When** `due_reminders` runs at 08:44, **Then** nothing is returned
   (not yet due).
3. **Given** the reminder was returned once, **When** `due_reminders` runs again at 08:46 or
   08:59, **Then** it is not returned again (dedupe through `reminder_log`).
4. **Given** the session has started (now 09:00 or later), **When** `due_reminders` runs and the
   reminder was never returned (app was closed), **Then** it is not returned: a missed reminder
   is skipped, not fired late.
5. **Given** a cancelled session, or a session of an archived course, **Then** no reminder is
   returned for it.
6. **Given** a makeup session, **Then** it is reminded like any other pending session.

---

### User Story 2 - Assessment reminder (Priority: P1)

For each incomplete assessment with `remind_minutes_before` set, the student gets one notification
that many minutes before it is due.

**Why this priority**: Deadlines are the other half of the planner; the field already exists.

**Independent Test**: Create an assessment due 2026-01-07 23:59 with `remind_minutes_before =
60`; `due_reminders(2026-01-07 22:59)` returns it once; later calls do not.

**Acceptance Scenarios**:

1. **Given** an incomplete assessment due at D with `remind_minutes_before = m`, **When**
   `due_reminders(now)` runs with `D - m <= now < D`, **Then** it returns one `Assessment`
   reminder with `fire_at = D - m` and a title naming the course code, kind, and assessment title.
2. **Given** `remind_minutes_before` is absent, or the assessment is completed, or its course is
   archived, **Then** no reminder is returned.
3. **Given** the reminder was returned, **When** the assessment's due time or reminder minutes
   change, **Then** a new reminder for the new `fire_at` can be returned once (the log key
   includes `fire_at`).
4. **Given** the due time has passed without the reminder being returned, **Then** it is not
   returned (missed reminders are skipped).

---

### User Story 3 - Notifications while the app runs (Priority: P1)

The app checks for due reminders every 60 seconds and shows each one as a desktop notification,
only while it is running and only when notifications are switched on in Settings.

**Why this priority**: Without the app side, reminders are never seen.

**Independent Test**: With notifications on and a class starting in 15 minutes, keep the app open;
one desktop notification appears (the manual check of §7).

**Acceptance Scenarios**:

1. **Given** the app is running and notifications are on, **When** the 60-second timer fires,
   **Then** the app calls `due_reminders(now)` and shows one desktop notification per returned
   reminder, using the reminder's title as the notification text.
2. **Given** notifications are off in Settings, **When** the timer fires, **Then** the app does not
   call `due_reminders`, so nothing is logged and reminders still in their window fire once the
   switch is turned back on.
3. **Given** showing a notification fails (no notification service), **Then** the app keeps
   running, does not retry that reminder, and shows no error dialog (the reminder is already
   logged).
4. **Given** the app is closed, **Then** no notification is shown (§10).

---

### Edge Cases

- **Lead time 0** (`class_reminder_minutes = 0` or `remind_minutes_before = 0`): the reminder is
  due during the event's start (or due) minute, `T <= now < T + 1 minute`, so the 60-second timer
  can still catch it once.
- **Several due at once** (app opened just before two classes): all due reminders are returned in
  one call, ordered by `fire_at`, then kind (session first), then id.
- **Time edits**: a session's stored start is a snapshot (R7); rescheduling cancels the original
  (no reminder) and the makeup gets its own reminder.
- **No active term**: session reminders cover only the active term's sessions (R1); assessment
  reminders cover assessments of non-archived courses of the active term.
- **Restore**: `reminder_log` is untouched by restore (M3 decision), so reminders already fired are
  not fired again after restoring the same data.
- **Clock**: core never reads the clock; `now` is passed in (§1).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001** (§5): `due_reminders(now)` MUST return the reminders that are due at `now` and MUST
  record each returned reminder in `reminder_log` as `(kind, ref_id, fire_at)` in the same
  transaction, so no reminder is ever returned twice.
- **FR-002** (§3, router decision): A session reminder exists for each `pending` session of a
  non-archived course in the active term, with `fire_at = session start - class_reminder_minutes`.
- **FR-003** (§3, router decision): An assessment reminder exists for each incomplete assessment of
  a non-archived course in the active term whose `remind_minutes_before` is set, with
  `fire_at = due_at - remind_minutes_before`.
- **FR-004** (router decision): A reminder is due when `fire_at <= now < T`, where `T` is the
  session start or assessment due time; when the lead time is 0, when `T <= now < T + 1 minute`.
  Reminders outside that window are never returned and never logged.
- **FR-005** (router decision): Results are ordered by `fire_at`, then session before assessment,
  then `ref_id`. Titles: session "`{code} {name} at {HH:MM}`" plus " in {room}" when a room is
  known; assessment "`{code} {kind}: {title} due {Month D, YYYY, HH:MM}`" (long date per the 2026-10-29 display-format change).
- **FR-006** (§7): The app MUST run a 60-second timer (the existing R6 timer may be reused) that,
  when `notifications_enabled` is on, calls `due_reminders(local now)` and shows each reminder as a
  desktop notification through `notify-rust`, from the app's `notify.rs` module (§2).
- **FR-007** (§7, §10): Notifications are shown only while the app is running; there is no
  background service or autostart.
- **FR-008**: A failure to show a notification or a core error during the reminder check MUST NOT
  crash the app or block the UI; it is ignored for that reminder (core errors may be shown in the
  current page's message).
- **FR-009** (§7): Reminder dedupe tests (core integration tests, in-memory SQLite) MUST cover
  FR-001–FR-005, and the gate MUST pass with every earlier test still green.
- **FR-010** (§7): One manual notification MUST be observed by the human (or recorded as pending
  for the human if the run is unattended).

### Key Entities

- **Reminder** (§5 type): kind (session or assessment), the referenced id, the `fire_at` moment,
  and a display title.
- **Reminder log** (§3 table): one row per reminder already returned; key `(kind, ref_id,
  fire_at)`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every due reminder is returned exactly once across any number of checks (0 duplicates
  in tests).
- **SC-002**: A reminder whose event has already started or passed is never shown.
- **SC-003**: With the app open and notifications on, a class reminder appears within 60 seconds of
  its fire time.
- **SC-004**: The gate (format, lint with warnings as errors, full test suite) passes, including the
  new reminder tests.

## Assumptions

- "Before each class" uses the session's stored start (sessions are snapshots, R7).
- Reminders use naive local time like the rest of the app (§1); no time-zone handling.
- Turning notifications off pauses checks; it does not mark reminders as fired.
- `notify-rust` is added to the app crate only; core stays free of notification code.
- The notification shows the app name "Academic Planner" as its summary and the reminder title as
  its body.
