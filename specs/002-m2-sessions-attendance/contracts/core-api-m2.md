# Contract: M2 behavior of the stubbed `planner_core` methods

The §5 API is frozen (`crates/core/tests/api_surface.rs`); signatures are not repeated or changed
here. This file states only the M2 behavior of the methods M1 left as `todo!()`. Decisions behind
each entry are in [../research.md](../research.md). "Active scope" = non-archived courses of the
active term. Every mutation commits before returning; multi-step ones are all-or-nothing.

## Timetable and sync

| Method | Behavior | Errors | Rules |
|---|---|---|---|
| `sync_sessions(today)` | Creates every missing regular session R4 produces for the active scope; deletes regular pending sessions of the active scope dated `>= today` that R4 no longer produces. Returns `{created, deleted}`. Second run with no input change returns `{0, 0}`. No active term: `{0, 0}`. | `Db` only | R3, R4, R5, FR-006; T04–T07, T14, T15, T28 |
| `change_slot(id, input, effective_from)` | Old slot `valid_to = effective_from - 1`; new slot from `input` with `valid_from = effective_from`; then sync with today = `effective_from`. Returns the new slot. Existing sessions keep their stored values. | `Validation` (R2 times), `NotFound` (slot) | R7, FR-007; T10, T11 |
| `end_slot(id, last_date)` | Slot `valid_to = last_date`; then sync with today = `last_date + 1`. | `NotFound` (slot) | R7, FR-008; T27 |

## Sessions

| Method | Behavior | Errors | Rules |
|---|---|---|---|
| `sessions_between(from, to, now)` | Sessions dated `from..=to` in the active scope, ordered by date, start, id. | — | R3, §5 |
| `sessions_for_course(course_id, now)` | All sessions of the course (also when archived), ordered by date, start, id. | `NotFound` (course) | R3, R15, §5 |
| `set_status(id, status, now)` | Writes `status`. `Attended`/`Absent` require `now >= start`. | `Validation` (before start, nothing written), `NotFound` | R11, R12; T12, T23 |
| `reschedule(id, makeup)` | One transaction: inserts a pending makeup with `replaces_session_id = id`, sets original to `Cancelled`. Returns the makeup. | `Validation` (original attended/absent, or makeup end ≤ start; nothing written), `NotFound` (session or course) | R9; T09, T19 |
| `add_makeup(makeup)` | Inserts a pending makeup with `replaces_session_id = NULL`. Returns it. | `Validation` (end ≤ start), `NotFound` (course) | R10; T22 |

Every `SessionView` has `course_code`, `course_name`, `display_status` ("Pending" / "Attended" /
"Absent" / "Cancelled" / "Rescheduled → YYYY-MM-DD"), and `is_past = now >= start`.

## Attendance and dashboard

| Method | Behavior | Errors | Rules |
|---|---|---|---|
| `attendance_summary(course_id)` | Counts of all sessions (regular + makeup, past + future) by status; `pct` 1 decimal or `None`; `target`; `below_target`. Works for archived courses. | `NotFound` (course) | R13–R15; T01–T03, T15, T21–T23 |
| `all_summaries()` | `attendance_summary` of each course in the active scope, ordered by code. | — | R3, R15 |
| `dashboard(now)` | `today` (R16, by start), `next_class` (R17, pending, start > now), `upcoming` (R18), `overdue` (R19), `awaiting` (R20, pending, end ≤ now, oldest first), `below_target` (R14). Active scope only; no active term = empty. | — | R3, R14, R16–R20; T13, T15–T18, T29 |
| `list_assessments(Upcoming { now })` | Incomplete, `now <= due_at <= now + upcoming_window_days`, by due. | — | R18; T17 |
| `list_assessments(Overdue { now })` | Incomplete, `due_at < now`, by due. | — | R19; T16 |

Time comparisons use minute precision (research R-D12). No method reads the clock.
