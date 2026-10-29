# Worked example source: 7th semester timetable

The human's own timetable, supplied in the design conversation on 2026-10-29. Task T027 turns it
into the worked example in `docs/import/ai-prompt.md`. Times are 24-hour, as the import format
requires. A merged cell (one course spanning periods) becomes ONE slot from the first period's
start to the last period's end.

## Periods

| Period | Time |
| --- | --- |
| Period 1 | 08:00–09:15 |
| Period 2 | 09:15–10:30 |
| Period 3 | 10:30–11:45 |
| Period 4 | 11:45–13:00 |
| Lunch and prayer | 13:00–14:30 (never a class) |
| Period 5 | 14:30–15:45 |
| Period 6 | 15:45–17:00 |

## Grid (day → periods → course)

| Day | Periods | Course |
| --- | --- | --- |
| Monday | 3 | CSE 4711 Artificial Intelligence |
| Monday | 4 | CSE 4739 Data Mining |
| Monday | 5–6 | HUM 4741 Business Communication and Law |
| Tuesday | 1–2 | CSE 4700 Project/Thesis |
| Tuesday | 3 | CSE 4753 Bioinformatics |
| Tuesday | 4 | MATH 4741 Mathematical Analysis |
| Tuesday | 5 | CSE 4703 Theory of Computing |
| Wednesday | 1–2 | CSE 4712 Artificial Intelligence Lab |
| Wednesday | 3–4 | CSE 4754 Bioinformatics Lab |
| Wednesday | 5–6 | CSE 4700 Project/Thesis |
| Thursday | 2 | MATH 4741 Mathematical Analysis |
| Thursday | 3 | CSE 4753 Bioinformatics |
| Thursday | 5 | CSE 4703 Theory of Computing |
| Friday | 2 | CSE 4739 Data Mining |
| Friday | 3–4 | CSE 4700 Project/Thesis |
| Friday | 5 | CSE 4711 Artificial Intelligence |

Cells marked "—" in the original are free periods. Credits are not in the table (the example
leaves them out or uses placeholders). CSE 4712 and CSE 4754 are labs (rule set "Lab"); the rest
use "Theory".

## Resulting slots (24-hour)

- Mon: CSE 4711 10:30–11:45; CSE 4739 11:45–13:00; HUM 4741 14:30–17:00
- Tue: CSE 4700 08:00–10:30; CSE 4753 10:30–11:45; MATH 4741 11:45–13:00; CSE 4703 14:30–15:45
- Wed: CSE 4712 08:00–10:30; CSE 4754 10:30–13:00; CSE 4700 14:30–17:00
- Thu: MATH 4741 09:15–10:30; CSE 4753 10:30–11:45; CSE 4703 14:30–15:45
- Fri: CSE 4739 09:15–10:30; CSE 4700 10:30–13:00; CSE 4711 14:30–15:45
