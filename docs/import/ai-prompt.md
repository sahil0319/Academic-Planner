# Convert my timetable for Academic Planner

Paste this whole file into an AI assistant, then paste your timetable (text or a photo) after the
last line. When the assistant answers, copy its answer into the Import panel of Academic Planner and
press Preview. Preview shows everything the import would do and any problem in the answer, and
nothing is saved until you press Apply.

## Your task

Convert the timetable that follows this prompt into the `academic-planner-courses` format
described below.

**Output rules. Follow them exactly.**

- Output a single JSON object and nothing else.
- Do not wrap it in code fences (no triple backticks).
- Do not add comments, explanations, greetings, or notes before or after the JSON.
- The JSON must be valid: double quotes, no trailing commas, no comments.

## The format

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
        { "day": "Mon", "start": "10:30", "end": "11:45", "room": null, "biweekly": null },
        { "day": "Fri", "start": "14:30", "end": "15:45" }
      ]
    }
  ]
}
```

### Keys

- Root: `format` (always `"academic-planner-courses"`), `version` (always the number `1`),
  `term`, `courses`.
- `term`: `name`, `start`, `end`. Include `term` only when the timetable states the semester
  dates. If it does not, leave the `term` key out. Never guess dates.
- Course:
  - `code`: the course code, kept exactly as written in the timetable (for example `CSE 4711`).
  - `name`: the course title.
  - `credits`: a number greater than 0, such as `3` or `1.5`. Leave the key out when the timetable
    does not give credits.
  - `rule_set`: `"Theory"` or `"Lab"`. Use `"Lab"` for laboratory and practical courses and
    `"Theory"` for every other course. Use `null` when you cannot tell.
  - `instructor`, `room`: text, or `null` when the timetable does not say.
  - `min_attendance_pct`: a whole number from 0 to 100, or `null` when the timetable does not say.
  - `slots`: the weekly class meetings of the course (see below).
- Slot: `day`, `start`, `end`, and optionally `room` and `biweekly`.
  - `biweekly`: leave it out (or use `null`) for a class that meets every week. Use it only when
    the timetable says the class meets every other week (for example "alternate weeks" or
    "bi-weekly"). Use `"this_week"` when the class meets in the week you are importing in, and
    `"next_week"` when it meets in the week after. Any other value is an error.

Use only these keys. Any other key is ignored by the importer.

### Rules the importer enforces

- Times are 24-hour `HH:MM`, from `00:00` to `23:59`. Write `14:30`, never `2:30 PM`.
- `day` is one of `Mon`, `Tue`, `Wed`, `Thu`, `Fri`, `Sat`, `Sun`.
- Dates are `YYYY-MM-DD`, for example `2026-01-05`.
- `end` must be later than `start`.
- One slot per class meeting. A course that meets on three days has three slots.
- Each course appears once. Put all of its meetings in that one course's `slots` list.
- Do not guess. When a value is not in the timetable, use `null` (or leave the key out where the
  keys section above says so) instead of inventing one.

### Periods and merged cells

Timetables are usually a grid of periods. Turn the period header into clock times first, then
apply these rules:

- A class in one period is one slot. It starts at that period's start time and ends at that
  period's end time.
- A class cell that spans several consecutive periods (a merged cell) is still **one** slot. It
  starts at the **first** period's start time and ends at the **last** period's end time. Do not
  split it into one slot per period.
- A free cell (empty, or marked with a dash) is not a class. Never create a slot for it.
- Lunch, prayer, and break rows are never classes. Never create a slot for them, and never let a
  slot cover them.

## Worked example: 7th semester

This is a real timetable, converted. Use it as a model.

### Period times

| Period | Time |
| --- | --- |
| Period 1 | 08:00-09:15 |
| Period 2 | 09:15-10:30 |
| Period 3 | 10:30-11:45 |
| Period 4 | 11:45-13:00 |
| Lunch and prayer | 13:00-14:30 (never a class) |
| Period 5 | 14:30-15:45 |
| Period 6 | 15:45-17:00 |

### Timetable grid

A range such as "1-2" is one merged cell spanning those periods. Cells that were free in the
original table are not listed.

| Day | Periods | Course |
| --- | --- | --- |
| Monday | 3 | CSE 4711 Artificial Intelligence |
| Monday | 4 | CSE 4739 Data Mining |
| Monday | 5-6 | HUM 4741 Business Communication and Law |
| Tuesday | 1-2 | CSE 4700 Project/Thesis |
| Tuesday | 3 | CSE 4753 Bioinformatics |
| Tuesday | 4 | MATH 4741 Mathematical Analysis |
| Tuesday | 5 | CSE 4703 Theory of Computing |
| Wednesday | 1-2 | CSE 4712 Artificial Intelligence Lab |
| Wednesday | 3-4 | CSE 4754 Bioinformatics Lab |
| Wednesday | 5-6 | CSE 4700 Project/Thesis |
| Thursday | 2 | MATH 4741 Mathematical Analysis |
| Thursday | 3 | CSE 4753 Bioinformatics |
| Thursday | 5 | CSE 4703 Theory of Computing |
| Friday | 2 | CSE 4739 Data Mining |
| Friday | 3-4 | CSE 4700 Project/Thesis |
| Friday | 5 | CSE 4711 Artificial Intelligence |

### How the merged cells become slots

- Monday periods 5-6 (HUM 4741) start with period 5 at 14:30 and end with period 6 at 17:00:
  one slot, 14:30 to 17:00.
- Tuesday periods 1-2 (CSE 4700) start with period 1 at 08:00 and end with period 2 at 10:30:
  one slot, 08:00 to 10:30.
- Wednesday periods 3-4 (CSE 4754) start with period 3 at 10:30 and end with period 4 at 13:00:
  one slot, 10:30 to 13:00. Lunch starts at 13:00, so the slot stops there.
- A single period such as Monday period 3 (CSE 4711) is one slot, 10:30 to 11:45.

### Resulting JSON

The two lab courses, CSE 4712 and CSE 4754, use `"rule_set": "Lab"`. All other courses use
`"Theory"`. The table gives no credits, no instructors, no rooms, no attendance requirement, and
no semester dates, so those keys are left out (or `null`) and not guessed. Because there is no
`term`, this file can be imported only when the planner already has an active term.

```json
{
  "format": "academic-planner-courses",
  "version": 1,
  "courses": [
    {
      "code": "CSE 4711", "name": "Artificial Intelligence", "rule_set": "Theory",
      "slots": [
        { "day": "Mon", "start": "10:30", "end": "11:45" },
        { "day": "Fri", "start": "14:30", "end": "15:45" }
      ]
    },
    {
      "code": "CSE 4739", "name": "Data Mining", "rule_set": "Theory",
      "slots": [
        { "day": "Mon", "start": "11:45", "end": "13:00" },
        { "day": "Fri", "start": "09:15", "end": "10:30" }
      ]
    },
    {
      "code": "HUM 4741", "name": "Business Communication and Law", "rule_set": "Theory",
      "slots": [
        { "day": "Mon", "start": "14:30", "end": "17:00" }
      ]
    },
    {
      "code": "CSE 4700", "name": "Project/Thesis", "rule_set": "Theory",
      "slots": [
        { "day": "Tue", "start": "08:00", "end": "10:30" },
        { "day": "Wed", "start": "14:30", "end": "17:00" },
        { "day": "Fri", "start": "10:30", "end": "13:00" }
      ]
    },
    {
      "code": "CSE 4753", "name": "Bioinformatics", "rule_set": "Theory",
      "slots": [
        { "day": "Tue", "start": "10:30", "end": "11:45" },
        { "day": "Thu", "start": "10:30", "end": "11:45" }
      ]
    },
    {
      "code": "MATH 4741", "name": "Mathematical Analysis", "rule_set": "Theory",
      "slots": [
        { "day": "Tue", "start": "11:45", "end": "13:00" },
        { "day": "Thu", "start": "09:15", "end": "10:30" }
      ]
    },
    {
      "code": "CSE 4703", "name": "Theory of Computing", "rule_set": "Theory",
      "slots": [
        { "day": "Tue", "start": "14:30", "end": "15:45" },
        { "day": "Thu", "start": "14:30", "end": "15:45" }
      ]
    },
    {
      "code": "CSE 4712", "name": "Artificial Intelligence Lab", "rule_set": "Lab",
      "slots": [
        { "day": "Wed", "start": "08:00", "end": "10:30" }
      ]
    },
    {
      "code": "CSE 4754", "name": "Bioinformatics Lab", "rule_set": "Lab",
      "slots": [
        { "day": "Wed", "start": "10:30", "end": "13:00" }
      ]
    }
  ]
}
```

## Check before you save

Paste the JSON into the Import panel and press Preview. Preview lists every error (with the path of
the bad value) and every change the import would make. Fix any error by asking the assistant to
correct that value, then Preview again. Nothing is saved until you press Apply.

## My timetable

(Paste or attach your timetable below this line. Reply with the JSON object only.)
