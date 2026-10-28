# Data Model: M5 Reminders

No schema change. Every table and type below already exists (migration 001, `crates/core/src/model.rs`).

## Reminder (§5 type, `model.rs`, unchanged)

| Field | Type | Meaning |
| --- | --- | --- |
| `kind` | `ReminderKind` (`Session` / `Assessment`) | What the reminder is for; stored as `'session'` / `'assessment'` |
| `ref_id` | `i64` | `session.id` or `assessment.id` |
| `fire_at` | `NaiveDateTime` (whole minute) | `T - lead` (research R-2) |
| `title` | `String` | Display text (FR-005, research R-5) |

## reminder_log (§3 table, unchanged)

| Column | Type | Notes |
| --- | --- | --- |
| `kind` | TEXT, `'session'` or `'assessment'` | |
| `ref_id` | INTEGER | not a foreign key; rows outlive deleted assessments harmlessly |
| `fire_at` | TEXT `YYYY-MM-DDTHH:MM` | written with `db::fmt_datetime` |

Primary key `(kind, ref_id, fire_at)` is the dedupe key. Only `due_reminders` writes rows; restore
and backup never touch the table (R22, M3 decision). No row is ever deleted.

## setting keys read (§3, unchanged)

| Key | Default | Use |
| --- | --- | --- |
| `class_reminder_minutes` | 15 | Lead for every session reminder (core) |
| `notifications_enabled` | 1 | App-side switch: when off the app does not call `due_reminders` |

## Source rows

| Reminder kind | Eligible rows | Event time `T` | Lead `m` |
| --- | --- | --- | --- |
| Session | `session.status = 'pending'`, regular or makeup, course not archived, course in the active term | `session.date` + `session.start_time` | `class_reminder_minutes` |
| Assessment | `assessment.completed = 0`, `remind_minutes_before IS NOT NULL`, course not archived, course in the active term | `assessment.due_at` | `remind_minutes_before` |

## Lifecycle of one reminder key

```text
not yet due        now < fire_at                                   -> not returned, not logged
due, not logged    fire_at <= now < max(T, fire_at + 1 min)        -> returned once, logged
due, logged        same window, key already in reminder_log        -> not returned
missed             now >= max(T, fire_at + 1 min) and never logged -> never returned, never logged
```

A changed `T` or lead gives a new `fire_at`, which is a new key with its own lifecycle.
