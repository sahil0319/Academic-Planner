# Contract: Import Format (docs/spec.md §13.4)

```json
{
  "format": "academic-planner-courses",
  "version": 1,
  "term": { "name": "Semester 7", "start": "2026-01-05", "end": "2026-05-01" },
  "courses": [
    {
      "code": "CSE 4711", "name": "Artificial Intelligence",
      "credits": 3, "rule_set": "Theory",
      "instructor": null, "room": null, "min_attendance_pct": 75,
      "slots": [
        { "day": "Mon", "start": "10:30", "end": "11:45", "room": null },
        { "day": "Fri", "start": "14:30", "end": "15:45" }
      ]
    }
  ]
}
```

## Known keys

- Root: `format`, `version`, `term`, `courses`.
- `term`: `name`, `start`, `end`.
- Course: `code`, `name`, `credits`, `rule_set`, `instructor`, `room`, `min_attendance_pct`,
  `slots`.
- Slot: `day`, `start`, `end`, `room`, `biweekly` (R59, added by M12).

Every other key (e.g. `notes`, `archived`, `teacher`, `comment`) → warning
`{path}: unknown key, ignored` (R42).

## Absent versus null (R44, R45)

| Key | Absent (existing course) | `null` (existing course) | New course |
|---|---|---|---|
| `name` | keep | error | required |
| `credits`, `rule_set`, `instructor`, `room`, `min_attendance_pct` | keep | clear | absent/`null` = none |
| `slots` | untouched | error | absent = no slots |
| `slots: []` | — | — | ends all current slots / new course has none |
| slot `room` | no slot room | no slot room | same |
| slot `biweekly` | every week | every week | same |
| `code` | error (required) | error | error |

## Grammar

- `day`: `Mon`…`Sun` or `Monday`…`Sunday` in any case (trimmed), or `1`…`7` as a string or JSON
  integer (1 = Monday).
- time: `H:MM` or `HH:MM`, 00:00–23:59, 24-hour.
- date: `YYYY-MM-DD`.
- `version`: integer `1`.
- slot `biweekly` (R59): `"this_week"`, `"next_week"`, `null`, or absent. `this_week` means the
  slot meets in the term week that contains the effective date (R49), `next_week` the week
  after it; the importer turns the value into a week parity through R54 before planning, and
  R45 compares slots including parity. `null` or absent means every week.
- `credits`: number > 0. `min_attendance_pct`: integer 0–100.

## Error messages (exact wording is the builder's; these are the required facts)

| Path | Case | Message gist |
|---|---|---|
| `document` | not JSON | `not valid JSON (line L, column C): …` |
| `document` | root not object | `must be a JSON object` |
| `format` | missing / wrong | `required` / `must be "academic-planner-courses"` |
| `version` | missing / not 1 | `required` / `unsupported version; expected 1` |
| `term` | missing, no active term | `required when there is no active term` |
| `term.start`, `term.end`, … | bad date | `must be a date YYYY-MM-DD` |
| `term.end` | before start | `must be on or after start` |
| `courses` | missing / not a list | `required` / `must be a list` |
| `courses[i]` | not an object | `must be an object` |
| `courses[i].code` | missing / null / empty | `required` / `must not be empty` |
| `courses[i].code` | duplicate in file | `duplicate of courses[k].code` |
| `courses[i].code` | OQ-1 ambiguous DB match | `matches more than one course: …` |
| `courses[i].name` | missing for new / null / empty | `required for a new course` / `must not be empty` |
| `courses[i].credits` | ≤ 0 / not a number | `must be a number greater than 0` |
| `courses[i].min_attendance_pct` | outside 0–100 / not an integer | `must be a whole number from 0 to 100` |
| `courses[i].rule_set` | unknown | `no rule set named "X" (available: Theory, Lab)` |
| `courses[i].slots` | null / not a list | `must be a list; use [] to end all slots` |
| `courses[i].slots[j].day` | bad | `must be a day name (Mon or Monday) or 1–7` |
| `courses[i].slots[j].start`/`.end` | bad | `must be a 24-hour time HH:MM` |
| `courses[i].slots[j].end` | ≤ start | `must be after start` |
| `courses[i].slots[j].biweekly` | not `"this_week"`, `"next_week"`, or null | `must be "this_week", "next_week", or null` |
| any string field | wrong JSON type | `must be text` |
