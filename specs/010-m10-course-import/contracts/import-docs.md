# Contract: Shipped Import Docs (R50)

## `docs/import/template.json`

- Exactly the §13.4 example (one course `CSE 4711` with two slots, term "Semester 7").
- Must stay valid: core test `r50_template_previews_cleanly` (in `crates/core/tests/m10_import.rs`)
  loads it with `include_str!`, previews it on a fresh in-memory database (M9 seeds "Theory" and
  "Lab") with any effective date inside the term, and asserts: no errors, no warnings, the plan
  creates the term and adds one course with two slots.
- Embedded in the app by `include_str!` for Copy template.

## `docs/import/ai-prompt.md`

Plain Markdown the user pastes into an AI assistant together with their timetable. It must:

1. Ask the assistant to convert the attached timetable to the `academic-planner-courses` format
   and to output only the JSON (no code fences, no comments), so it pastes cleanly.
2. Include the format (the template text) and the known keys with their meaning.
3. State the rules the importer enforces: 24-hour `HH:MM` times; `day` as `Mon`…`Sun`; dates
   `YYYY-MM-DD`; one slot per class meeting; `rule_set` must be one of the user's rule set names
   ("Theory" or "Lab" by default) or `null`; unknown values as `null` rather than guessed.
4. State the **period → time rule for merged cells**: when a class cell spans several consecutive
   periods (merged cells), it is one slot that starts at the first period's start time and ends at
   the last period's end time; a class in one period uses that period's start and end.
5. Include the user's **7th-semester table** as a worked example: the period-time header, the
   grid (with at least one merged cell), and the resulting JSON with 24-hour times. The table is
   provided by the human (OQ-2, task T001); the builder transcribes it.
6. Mention that the result can be checked with Preview before anything is saved.

No test parses the prompt; the human reviews it in the final manual check.
