# Import and export of courses

`template.json` and `ai-prompt.md` are shown in Courses, Import, with Copy buttons (R50). The
format is the `academic-planner-courses` document of docs/spec.md section 13.4.

## Exporting (R96-R100)

Courses, then **Copy JSON** or **Save JSON...**, writes the same format. Tick courses in the list
to export only those; with none ticked, every course that is not archived is exported. The other
person pastes the text (or the contents of the file) into Courses, Import, and presses Preview.

An export holds the active term, and for each course its code, name, credits, rule set name,
instructor, room, attendance target, and the slots that have not ended. It holds no notes,
sessions, attendance, assessments, marks, or holidays.

The other planner must have rule sets with the same names as the `rule_set` values (Theory and Lab
exist by default). Otherwise Preview reports an error at `courses[n].rule_set` and nothing is
imported.

## The `biweekly` key in an export

An export is written relative to the day it was made (the week of "today"):

- no `biweekly` key: the slot meets every week;
- `"this_week"`: the slot meets in the week of the export date;
- `"next_week"`: the slot meets in the week after it.

On import the weeks are counted from the effective date (R49), which defaults to today. Import on
the same day as the export, or set the effective date in the preview to the export date, so
`this_week` lands on the right week.
