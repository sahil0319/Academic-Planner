# Data Model: M3 Backup, Restore, and CSV Export

No schema change: migration `001_init.sql` (docs/spec.md §3) is unchanged and no migration is
added. This file fixes the shape of the backup document and the CSV files; the decisions behind them
are in [research.md](research.md).

## Backup document (R21)

Top-level keys, in this order. `schema_version` is `SCHEMA_VERSION` (1 today).

| Key | JSON type | Source |
| --- | --- | --- |
| `format` | string, always `"academic-planner-backup"` | constant |
| `schema_version` | integer | `SCHEMA_VERSION` |
| `exported_at` | string `YYYY-MM-DDTHH:MM` | `fmt_datetime(now)` |
| `terms` | array of term objects, by `id` | table `term` |
| `holidays` | array of holiday objects, by `(term_id, date)` | table `holiday` |
| `courses` | array of course objects, by `id` | table `course` |
| `slots` | array of slot objects, by `id` | table `slot` |
| `sessions` | array of session objects, by `id` | table `session` |
| `assessments` | array of assessment objects, by `id` | table `assessment` |
| `settings` | object, key to string value, by key | table `setting` |

`reminder_log` is not exported.

## Row objects

Keys are the §3 column names. `?` = nullable (JSON `null`). Flags are JSON booleans; every other
INTEGER is a JSON number; TEXT dates, times, and date-times are the stored strings.

| Object | Fields |
| --- | --- |
| term | `id` int, `name` str, `start_date` date, `end_date` date, `is_active` bool |
| holiday | `term_id` int, `date` date, `label` str? |
| course | `id` int, `term_id` int, `code` str, `name` str, `instructor` str?, `room` str?, `notes` str?, `min_attendance_pct` int?, `archived` bool |
| slot | `id` int, `course_id` int, `weekday` int, `start_time` time, `end_time` time, `room` str?, `valid_from` date, `valid_to` date? |
| session | `id` int, `course_id` int, `slot_id` int?, `kind` str, `date` date, `start_time` time, `end_time` time, `room` str?, `status` str, `replaces_session_id` int?, `note` str? |
| assessment | `id` int, `course_id` int, `kind` str, `title` str, `due_at` datetime, `notes` str?, `completed` bool, `remind_minutes_before` int? |

## Restore (R22)

Validation, in order, before the database is touched (research R-D4): valid JSON object; `format`;
`schema_version` present and `<= SCHEMA_VERSION`; all seven collections present and every row
well-typed; every date, time, and date-time text parses. Then one transaction with deferred foreign
keys (R-D5):

1. delete: `session`, `assessment`, `slot`, `course`, `holiday`, `term`, `setting`
2. insert, with stored ids: `term`, `holiday`, `course`, `slot`, `session`, `assessment`, `setting`
3. commit; any error drops the transaction (rollback)

`reminder_log` is left untouched. State transitions: none; restore is a full replacement, not a
merge.

## CSV files (R23)

Both files: first record is the header; terminator CRLF; fields quoted only when they contain a
comma, double quote, or line break.

**attendance.csv** header: `date,start,end,course_code,course_name,kind,status,replaces_date,note`

| Column | Value |
| --- | --- |
| `date` | `session.date` |
| `start` | `session.start_time` |
| `end` | `session.end_time` |
| `course_code` | `course.code` |
| `course_name` | `course.name` |
| `kind` | `session.kind` (`regular` / `makeup`) |
| `status` | `session.status` (`pending` / `attended` / `absent` / `cancelled`) |
| `replaces_date` | `date` of the session `replaces_session_id` points to; empty if none |
| `note` | `session.note`; empty if none |

Sorted by `date`, `start_time`, course `code`, session `id`. All sessions of all courses and terms.

**assessments.csv** header: `course_code,kind,title,due_at,completed,notes`

| Column | Value |
| --- | --- |
| `course_code` | `course.code` |
| `kind` | `assessment.kind` |
| `title` | `assessment.title` |
| `due_at` | `assessment.due_at` (`YYYY-MM-DDTHH:MM`) |
| `completed` | `true` / `false` |
| `notes` | `assessment.notes`; empty if none |

Sorted by `due_at`, course `code`, assessment `id`. All assessments.
