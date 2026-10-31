# M18 decisions

## M18a sweep decisions (overnight, 2026-10-31)

- Texts: "Course added." / "Course saved.", "Deleted N course(s).", "Course archived." / "Course unarchived.", "Makeup class added.", "Assessment added." / "Assessment saved.", "Assessment deleted.", "Class rescheduled.", "Slot added." / "Slot saved.", "Slot ended.", "Rule set deleted.", "Imported N courses.". Short, one line, ending in a full stop (as the M18 texts).
- The existing "Term archived." is kept (T-tests pin it); the spec example "Archived Spring 2026." says "such as".
- Import Apply toast counts courses added plus updated; a plan with only a term gives "Imported the term.".
- The assessment "completed" checkbox and the About update-check switch do not toast (row toggle and switch, like the status buttons).
- Pages with an inline `message` toast only when the refresh left that message empty (same guard as `changed_then_toast`); forms that show errors next to fields toast after `App::changed`.
- The handler coverage check is a source-scanning unit test (no headless Slint backend exists): each named handler body must contain a `toast(` call.
