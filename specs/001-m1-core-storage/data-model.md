# Data Model: M1 Core Storage and Setup Data

Storage schema is `docs/spec.md` §3 verbatim (migration `001_init.sql`). This file maps each table
to its Rust types in `crates/core/src/model.rs`. Text encodings: dates `YYYY-MM-DD`, times `HH:MM`,
date-times `YYYY-MM-DDTHH:MM` (research D4). All types derive `Debug, Clone, PartialEq`; records
also derive `serde::Serialize, Deserialize` for M3.

## Enums

| Type | Variants | Stored as |
|---|---|---|
| `Status` | `Pending`, `Attended`, `Absent`, `Cancelled` | `'pending'`… (§3 CHECK) |
| `SessionKind` | `Regular`, `Makeup` | `'regular'`, `'makeup'` |
| `AssessmentKind` | `Quiz`, `Exam`, `Assignment`, `Other` | `'quiz'`… |
| `ReminderKind` | `Session`, `Assessment` | `'session'`, `'assessment'` |
| `AssessmentFilter` | `Upcoming { now: NaiveDateTime }`, `Overdue { now: NaiveDateTime }`, `Completed`, `All` | not stored |
| weekday | `chrono::Weekday` | `INTEGER` 1–7, ISO (Mon = 1) |

## Term / Holiday (table `term`, `holiday`)

| Field | Type | Notes |
|---|---|---|
| `TermInput.id` | `Option<i64>` | `None` = create, `Some` = update |
| `TermInput.name` | `String` | |
| `TermInput.start_date`, `end_date` | `NaiveDate` | **R2**: `end_date < start_date` → Validation |
| `Term.{id, name, start_date, end_date}` | as above, `id: i64` | |
| `Term.is_active` | `bool` | **R1**: at most one `true` (unique partial index) |
| `Term.holidays` | `Vec<Holiday>` | sorted by date (research D6) |
| `Holiday.date` | `NaiveDate` | PK with term_id |
| `Holiday.label` | `Option<String>` | |

Transitions: `save_term` → that term becomes active, all others inactive (R1).
`set_holidays(term_id, days)` replaces the term's holiday set.

## Course (table `course`)

| Field | Type | Notes |
|---|---|---|
| `CourseInput.code`, `name` | `String` | **R2**: empty after trim → Validation; code unique per term → Validation |
| `CourseInput.instructor`, `room`, `notes` | `Option<String>` | |
| `CourseInput.min_attendance_pct` | `Option<i64>` | **R2**: outside 0–100 → Validation |
| `Course.{id, term_id}` | `i64` | `term_id` = active term at creation |
| `Course.archived` | `bool` | **R3**: hidden from default lists; data kept |

Transitions: `set_archived(id, true|false)`. No delete operation (§5).

## Slot (table `slot`)

| Field | Type | Notes |
|---|---|---|
| `SlotInput.course_id` | `i64` | must exist (FK) |
| `SlotInput.weekday` | `chrono::Weekday` | |
| `SlotInput.start_time`, `end_time` | `NaiveTime` | **R2**: `end_time <= start_time` → Validation |
| `SlotInput.room` | `Option<String>` | `None` = use course room |
| `SlotInput.valid_from` | `NaiveDate` | inclusive |
| `SlotInput.valid_to` | `Option<NaiveDate>` | inclusive; `None` = until term end |
| `Slot.id` | `i64` | plus all `SlotInput` fields |

## Assessment (table `assessment`)

| Field | Type | Notes |
|---|---|---|
| `AssessmentInput.course_id` | `i64` | |
| `AssessmentInput.kind` | `AssessmentKind` | |
| `AssessmentInput.title` | `String` | **R2**: empty after trim → Validation |
| `AssessmentInput.due_at` | `NaiveDateTime` | |
| `AssessmentInput.notes` | `Option<String>` | |
| `AssessmentInput.remind_minutes_before` | `Option<u32>` | `None` = no reminder |
| `Assessment.{id, completed: bool, course_code: String}` | | `course_code` joined for lists |

## Settings (table `setting`, key/value text)

| Field | Type | Key | Default |
|---|---|---|---|
| `class_reminder_minutes` | `u32` | `class_reminder_minutes` | 15 |
| `upcoming_window_days` | `u32` | `upcoming_window_days` | 7 |
| `notifications_enabled` | `bool` | `notifications_enabled` (`'1'`/`'0'`) | true |

## Types defined in M1, behavior in later milestones

| Type | Fields | Milestone |
|---|---|---|
| `Session` | `id, course_id, slot_id: Option<i64>, kind, date, start_time, end_time, room: Option<String>, status, replaces_session_id: Option<i64>, note: Option<String>` | M2 |
| `SessionView` | `session: Session, course_code, course_name, display_status: String, is_past: bool` | M2 |
| `MakeupInput` | `course_id, date, start_time, end_time, room: Option<String>, note: Option<String>` | M2 |
| `SyncReport` | `created: usize, deleted: usize` | M2 |
| `AttendanceSummary` | `course_id, course_code, course_name, attended, absent, cancelled, pending: u32, pct: Option<f64>, target: Option<i64>, below_target: bool` | M2 |
| `Dashboard` | `today: Vec<SessionView>, next_class: Option<SessionView>, upcoming: Vec<Assessment>, overdue: Vec<Assessment>, awaiting: Vec<SessionView>, below_target: Vec<AttendanceSummary>` | M2 |
| `Reminder` | `kind: ReminderKind, ref_id: i64, fire_at: NaiveDateTime, title: String` | M5 |

## Errors (`crates/core/src/error.rs`)

| Variant | When |
|---|---|
| `Validation(String)` | every R2 case |
| `NotFound { entity: &'static str, id: i64 }` | update/archive of a missing row |
| `NoActiveTerm` | course creation with no active term |
| `SchemaTooNew { found: i64, supported: i64 }` | R25/T26; message: "database schema version {found} is newer than this app supports ({supported}); update the app" |
| `NoDataDir` | R24: no override and no OS data directory |
| `Corrupt(String)` | stored text fails to parse |
| `Db(rusqlite::Error)`, `Io(std::io::Error)` | `#[from]` wrappers |
