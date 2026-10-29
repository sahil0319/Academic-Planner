# Data Model: M10 Courses and Timetable Import

No schema change and no migration. M10 reads and writes existing tables only:

| Table | Read | Written by `apply_import` |
|---|---|---|
| `term` | active term (R47) | insert one active term when none exists (R47) |
| `course` | active-term courses incl. archived (R43, R44); M9 columns `credits`, `rule_set_id` | insert new courses; update changed fields of matched courses |
| `slot` | slots of matched courses not ended before E (R45) | set `valid_to = E − 1` on ended slots; insert new slots from E |
| `session` | via sync | via sync only (R4, R5) |
| `rule_set` (M9) | `id`, `name` for R42 lookup | never |

## In-memory entities (core, public; exact Rust names in contracts/core-import.md)

### ImportPlan

| Field | Meaning |
|---|---|
| `term` | `Option<TermInput>`-like value: the term to create (name, start, end) when there is no active term; `None` otherwise |
| `add` | courses to create, in file order, each with its slots |
| `update` | matched courses with at least one field change or a slot change, in file order |
| `warnings` | `ImportIssue` list (unknown keys, archived match, term ignored) |
| `errors` | `ImportIssue` list; when non-empty, `term`, `add`, `update` are empty |

`is_empty()` = `term.is_none() && add.is_empty() && update.is_empty()` (warnings do not count).

### ImportIssue

`path: String` (D4), `message: String`. Display: `"{path}: {message}"`.

### CourseAdd

Code as written (trimmed), name, instructor, room, min_attendance_pct, credits, rule set (id and
name), slots (`ImportSlot` list, all starting at E). Absent and `null` both give `None` for a new
course.

### CourseUpdate

`course_id`, stored `code`, `archived` (for the preview's label), `changes: Vec<FieldChange>`,
`slots: Option<SlotChange>` (`None` = untouched or equal).

### FieldChange

`field` (one of Name, Credits, RuleSet, Instructor, Room, MinAttendancePct), `old` and `new` as a
typed value (`FieldValue`: text, integer, number, or rule set name; each optional). Formatting is
the app's job.

### SlotChange

`end: Vec<Slot>` (the stored slots that get `valid_to = E − 1`), `add: Vec<ImportSlot>`.

### ImportSlot

`weekday: Weekday`, `start: NaiveTime`, `end: NaiveTime`, `room: Option<String>`.

### ImportReport

`plan: ImportPlan` (what was applied), `sync: SyncReport`.

## Validation rules (R42, R2)

| Path | Rule |
|---|---|
| `format` | required, equals `academic-planner-courses` |
| `version` | required, integer 1 |
| `term` | object; required if no active term; `name` non-empty, `start`/`end` dates, end ≥ start |
| `courses` | required list (may be empty) |
| `courses[i]` | object |
| `courses[i].code` | required non-empty string; normalized code unique in file (R43) |
| `courses[i].name` | non-empty string; required for a new course; `null` is an error |
| `courses[i].credits` | number > 0, or `null` |
| `courses[i].rule_set` | name of an existing rule set, or `null` |
| `courses[i].instructor`, `.room` | string or `null` |
| `courses[i].min_attendance_pct` | integer 0–100, or `null` |
| `courses[i].slots` | list (may be empty); `null` is an error |
| `courses[i].slots[j].day` | D7 |
| `courses[i].slots[j].start`, `.end` | D8; end > start |
| `courses[i].slots[j].room` | string, `null`, or absent |
| any other key | warning, ignored |

## State transitions

```text
paste text ──Preview──▶ ImportPlan (errors?) ──no errors, not empty──▶ Apply ──▶ ImportReport
     ▲                         │ errors / empty                                   │
     └──── edit text or date ◀─┘                                                  ▼
                                                                 DB changed + synced (one tx)
```
