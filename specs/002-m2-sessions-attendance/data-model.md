# Data Model: M2 Sessions, Exceptions, Attendance, and Dashboard

No schema change: storage is migration `001_init.sql` (`docs/spec.md` §3) and the Rust types in
`crates/core/src/model.rs` are the M1 types, unchanged. This file records how M2 uses them.

## Session (table `session`, type `Session`)

| Field | M2 use |
|---|---|
| `kind` | `Regular` = created by sync from a slot (`slot_id` set); `Makeup` = created by `reschedule` / `add_makeup` (`slot_id NULL`) |
| `date`, `start_time`, `end_time`, `room` | snapshot copied at creation (R4, R7); never updated by sync or slot changes. Regular: from the slot (`room` = slot room, else course room; research R-D3). Makeup: from `MakeupInput` |
| `status` | see lifecycle below (R11) |
| `replaces_session_id` | set only on a makeup created by `reschedule` (R9) |
| `note` | makeup: from `MakeupInput.note`; regular: `NULL` |
| uniqueness | `UNIQUE(slot_id, date)`: at most one regular session per slot per date (R4, T05, T28) |

### Status lifecycle (R8–R12)

- New sessions start `Pending` (R11).
- Any status → `Pending` or `Cancelled` at any time (R12). `Attended`/`Absent` only when
  `now >= start` (R12).
- `reschedule` sets `Pending` or `Cancelled` originals to `Cancelled`; `Attended`/`Absent`
  originals are rejected (R9, T19).
- Sync never changes a status. It only deletes regular, pending sessions dated on or after today
  that R4 no longer produces, and never for archived courses (R5, FR-006).

## Slot (table `slot`, type `Slot`)

- `change_slot(id, input, effective_from)`: old row `valid_to = effective_from - 1`; new row from
  `input` with `valid_from = effective_from` (R7).
- `end_slot(id, last_date)`: `valid_to = last_date`.
- A slot whose `valid_to < valid_from` (possible after `change_slot` with an effective date on or
  before the old slot's `valid_from`) produces no sessions; its existing sessions follow R5.

## Holiday (table `holiday`)

A date with no regular sessions (R4). Exam periods are holidays on every date of the period,
prep days included (FR-006a). Makeups may still be added on holidays.

## Derived values (not stored)

| Value | Definition | Rule |
|---|---|---|
| session start / end | `date.and_time(start_time)` / `date.and_time(end_time)` | R12, R17, R20 |
| `SessionView.is_past` | `now >= start` | §5 |
| `SessionView.display_status` | "Pending" / "Attended" / "Absent" / "Cancelled"; "Rescheduled → YYYY-MM-DD" if cancelled and replaced by a makeup (latest makeup's date) | §3, §5 |
| `AttendanceSummary.pct` | `attended / (attended + absent) × 100`, 1 decimal; `None` if denominator 0 | R13 |
| `AttendanceSummary.below_target` | `target.is_some() && pct.is_some() && pct < target` | R14 |
| `SyncReport` | rows inserted / deleted by one sync | §5 |
