# Research: M2 Sessions, Exceptions, Attendance, and Dashboard

All technology choices are fixed by `docs/spec.md` §1, and the rules by §4. This file records only
the M2 design decisions that §3–§5 and spec.md leave open. No item re-decides §1 or changes a rule,
the schema, or the API.

## R-D1. Sync algorithm (R4, R5, R3 / FR-006)

- **Decision**: `sync_sessions(today)` opens one transaction and calls a crate-private
  `sync_in(conn, today) -> Result<SyncReport>`. It (1) loads the active term, its holidays, and the
  slots of its **non-archived** courses; (2) builds the expected set of `(slot_id, date)` by walking
  each slot's inclusive range `[max(term.start, valid_from), min(term.end, valid_to ?? term.end)]`
  and keeping dates whose weekday matches and that are not holidays; (3) inserts each missing pair
  with `INSERT OR IGNORE` (the `UNIQUE(slot_id, date)` constraint keeps existing rows, including
  cancelled ones, untouched — R8) and counts inserted rows as `created`; (4) deletes sessions with
  `kind='regular' AND status='pending' AND date >= today`, belonging to non-archived courses of the
  active term, whose `(slot_id, date)` is not in the expected set, counting `deleted`.
- **Rationale**: a full rebuild of the expected set is simple and idempotent (T05) at this scale.
  Restricting both steps to non-archived courses implements FR-006 (R3 over R5). Sessions of
  inactive terms are never touched (R1).
- **Alternatives**: incremental per-change sync — rejected, more code and more failure modes.

## R-D2. No active term

- **Decision**: `sync_sessions` returns `SyncReport { created: 0, deleted: 0 }`; `dashboard`
  returns an empty `Dashboard`; `all_summaries` and `sessions_between` return empty lists.
- **Rationale**: R6 runs sync on app start, possibly before any term exists; M1 list methods
  already return empty results without an active term. `Error::NoActiveTerm` stays for methods
  that create data.

## R-D3. Session room snapshot (router decision 2026-10-29)

- **Decision**: a generated session stores `coalesce(slot.room, course.room)` at creation time:
  the slot's room, or the course room when the slot room is `NULL`.
- **Rationale**: §3 says a `NULL` slot room means "use course.room", and R7 says sessions are
  snapshots, so the room is resolved once at creation and a later course-room edit does not change
  existing sessions. T04 (`t04_session_copies_slot_room`) covers the explicit slot-room case.
- **Alternatives**: store `slot.room` verbatim (`NULL` resolved at display time) — rejected, a
  course-room edit would then change past sessions' room.

## R-D4. `change_slot` (R7, FR-007)

- **Decision**: validate `input` (R2, same as `add_slot`); `NotFound` if the slot does not exist;
  `Error::Validation` if `effective_from` has no previous day. In one transaction: set old
  `valid_to = effective_from - 1 day`; insert the new slot with all fields from `input` except
  `valid_from = effective_from`; then `sync_in(tx, effective_from)`; commit; return the new slot.
- **Rationale**: R7 text plus the human decision (sync uses the effective date as today). One
  transaction keeps the slot pair and the sessions consistent. `input.valid_from` is overridden
  because R7 fixes it (router decision 2026-10-29); no extra course check is added (spec adds
  none).

## R-D5. `end_slot` (FR-008, T27)

- **Decision**: `NotFound` if the slot does not exist. In one transaction: set
  `valid_to = last_date`; then `sync_in(tx, last_date + 1 day)`; commit.
- **Rationale**: human decision 2026-10-29. Pending regular sessions after `last_date` are removed
  by R5; sessions on or before it are kept.

## R-D6. Status marking (R11, R12)

- **Decision**: `set_status` loads the session (`NotFound` if missing). For `Attended`/`Absent`,
  if `now < date.and_time(start_time)` it returns `Error::Validation` and writes nothing.
  `Pending` and `Cancelled` are always written. No other check (e.g. archived course) is added.
- **Rationale**: R12 verbatim; FR-013 boundary `now >= start` is allowed. `Error::Validation`
  per router decision 2026-10-29.

## R-D7. Reschedule and makeups (R9, R10)

- **Decision**: `reschedule(id, makeup)`, one transaction: `NotFound` if the original or
  `makeup.course_id` is missing; `Error::Validation` if the original is attended or absent, or
  if `makeup.end_time <= makeup.start_time`; otherwise insert a `makeup` session (status pending,
  `slot_id NULL`, `replaces_session_id = id`) and set the original to `cancelled`; return the
  makeup. Rescheduling a pending or cancelled original is allowed. `add_makeup` does the same
  insert with `replaces_session_id NULL` and no original.
- **Rationale**: R9 "error" is reported as `Error::Validation` (router decision 2026-10-29, same
  variant as R12; user input, never a panic). The
  time check surfaces the schema `CHECK (end_time > start_time)` as a validation error instead of a
  raw `Error::Db`; it adds no rule.

## R-D8. Session views (§5, FR-017, FR-018)

- **Decision**: `display_status` is a pure function `(Status, Option<NaiveDate>) -> String`
  returning "Pending" / "Attended" / "Absent" / "Cancelled", or
  `"Rescheduled → YYYY-MM-DD"` when the session is cancelled and a makeup replaces it. If several
  makeups replace one session, the latest-created (highest id) one is shown. `is_past =
  now >= date.and_time(start_time)`. `sessions_between(from, to, now)`: sessions dated
  `from..=to` of non-archived active-term courses, ordered by date, start time, id.
  `sessions_for_course(course_id, now)`: all sessions of that course (archived or not, for history
  — T15), same order; `NotFound` if the course does not exist.
- **Rationale**: §5 text; the arrow is U+2192 as in §5.

## R-D9. Attendance summary (R13–R15)

- **Decision**: counts cover every session of the course, regular and makeup, past and future.
  `pct = round(attended / (attended + absent) × 1000) / 10` as `f64` (one decimal), with ties
  rounded half away from zero (`f64::round`, e.g. 6.25 → 6.3), `None` when the denominator is 0. `below_target` compares the **rounded** `pct` with `target` (R14 uses
  `pct`). `attendance_summary` returns `NotFound` for an unknown course and works for archived
  courses (T15). `all_summaries`: non-archived courses of the active term, ordered by code.
- **Rationale**: R13–R15 text; spec Assumptions for `all_summaries` scope (confirmed by router
  decision 2026-10-29).

## R-D10. Dashboard (R16–R20, R3)

- **Decision**: all parts exclude archived courses and use the active term.
  `today` = sessions dated `now.date()` ordered by start time, then course code, then id.
  `next_class` = earliest pending session with start strictly after `now` (ordered by date, start,
  id). `upcoming` = `list_assessments(Upcoming { now })`; `overdue` =
  `list_assessments(Overdue { now })`. `awaiting` = pending sessions with
  `date.and_time(end_time) <= now`, oldest first (date, start, id). `below_target` = the
  `all_summaries` entries whose `below_target` is true.
- **Rationale**: R16–R20 text; reuse keeps one implementation per rule.

## R-D11. `list_assessments` Upcoming / Overdue (R18, R19)

- **Decision**: `Upcoming { now }`: incomplete, `now <= due_at <= now + upcoming_window_days`
  (the stored setting, days × 24 h), ordered by `due_at`, id. `Overdue { now }`: incomplete,
  `due_at < now`, same order. Both keep M1's non-archived active-term scope.
- **Rationale**: filter doc comments in `model.rs` and spec Assumptions.

## R-D12. Time precision

- **Decision**: all comparisons with `now` use minute precision: `now` is truncated to whole
  minutes (seconds dropped), the same as stored values (`YYYY-MM-DDTHH:MM`, §1). Comparisons may be
  done on `YYYY-MM-DDTHH:MM` text (lexicographic order equals time order) or on parsed chrono
  values; both give the same result.
- **Rationale**: stored times have no seconds; truncation makes `now = 10:00:30` behave like
  `10:00` for R12, R17, R20 boundaries.
