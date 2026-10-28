# Contract: `Planner::due_reminders` (core, M5)

Signature (frozen, docs/spec.md §5; checked by `crates/core/tests/api_surface.rs`):

```rust
pub fn due_reminders(&mut self, now: NaiveDateTime) -> Result<Vec<Reminder>>;
// logs to reminder_log, never returns one twice
```

Lives in the new private module `crates/core/src/reminders.rs` (moved from `backup.rs`).

## Behavior

1. Reads `class_reminder_minutes` from settings.
2. Collects candidates (data-model.md "Source rows"): pending sessions and incomplete assessments
   with `remind_minutes_before` set, of non-archived courses in the active term. No active term:
   returns `Ok(vec![])`.
3. For each candidate: `fire_at = T - lead`; keeps it only if
   `fire_at <= now < max(T, fire_at + 1 minute)` (FR-004).
4. In one transaction, `INSERT OR IGNORE` each kept `(kind, ref_id, fire_at)` into `reminder_log`;
   returns only rows whose insert changed a row (FR-001).
5. Sorted by `fire_at`, then `Session` before `Assessment`, then `ref_id` (FR-005).
6. Titles (FR-005):
   - Session: `{code} {name} at {HH:MM}`, followed by " in {room}" when `coalesce(session.room, course.room)`
     is not null or blank. Fixture: `CSE101 Intro to Programming at 09:00 in A-101`.
   - Assessment: `{code} {Kind}: {title} due {YYYY-MM-DD HH:MM}`, `{Kind}` in
     `Quiz | Exam | Assignment | Other`. Example: `CSE101 Assignment: HW1 due 2026-01-07 23:59`.
7. Does not read `notifications_enabled` (the app gates the call) and does not read the clock.

## Errors

Only storage errors are possible: `Error::Db` (SQLite) or `Error::Corrupt` (a stored date/time that
does not parse). On error the transaction is dropped and nothing is logged.

## Test coverage (crates/core/tests/m5_reminders.rs)

| Test | Covers |
| --- | --- |
| `m5_session_reminder_fires_once` | US1.1, US1.3, FR-001, title with room |
| `m5_session_reminder_not_before_fire_at` | US1.2, not-yet-due is not logged |
| `m5_missed_session_reminder_skipped` | US1.4, SC-002 |
| `m5_cancelled_session_no_reminder` | US1.5 |
| `m5_rescheduled_session_reminds_makeup_only` | US1.5, US1.6, edge "Time edits" |
| `m5_makeup_session_reminded` | US1.6 |
| `m5_marked_session_no_reminder` | FR-002 (pending only) |
| `m5_archived_course_no_reminders` | US1.5, US2.2 |
| `m5_class_reminder_minutes_setting_used` | FR-002 |
| `m5_zero_lead_session_reminder` | FR-004 lead 0 |
| `m5_assessment_reminder_fires_once` | US2.1, FR-003, assessment title |
| `m5_assessment_without_reminder_minutes_skipped` | US2.2 |
| `m5_completed_assessment_no_reminder` | US2.2 |
| `m5_missed_assessment_reminder_skipped` | US2.4 |
| `m5_assessment_due_change_fires_new_reminder` | US2.3 |
| `m5_zero_lead_assessment_reminder` | FR-004 lead 0 |
| `m5_several_due_reminders_ordered` | edge "Several due at once", FR-005 order |
| `m5_no_active_term_no_reminders` | edge "No active term" |
| `m5_inactive_term_no_reminders` | R1, edge "No active term" |
| `m5_notifications_switch_ignored_by_core` | research R-6 (core does not gate) |
| `m5_restore_does_not_refire` | edge "Restore" |
