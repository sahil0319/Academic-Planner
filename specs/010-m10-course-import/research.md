# Research: M10 Courses and Timetable Import

All decisions implement `docs/spec.md` §13.4 as written. Where the text leaves a gap, the decision
picks the reading closest to existing rules (R2, R4–R7) and is recorded here. Gaps that could change
a rule are listed as open questions (OQ) with a default the plan uses until the human answers.

## D1. Module layout

- **Decision**: private module folder `crates/core/src/import/` (`mod.rs`, `types.rs`, `parse.rs`,
  `plan.rs`, `apply.rs`); public types re-exported from `lib.rs` like `model::*`.
- **Rationale**: parse (no DB), plan (DB reads), and apply (writes) are separately testable through
  the public API, and the files do not collide with M8/M9 edits.
- **Alternatives**: one `import.rs` file (would exceed ~800 lines); types in `model.rs` (M9 also
  edits it, merge conflicts).

## D2. Parse with `serde_json::Value`, not derive

- **Decision**: `serde_json::from_str::<Value>` then a hand-written walk.
- **Rationale**: R42 needs every error (not the first), a JSON path per error, unknown keys as
  warnings, and absent versus `null` (R44). A derive stops at the first error and loses the path.
- **Alternatives**: `serde` derive with `deny_unknown_fields` (stops at first error, unknown keys
  become errors); `serde_path_to_error` (still first error only, new dependency).

## D3. Tri-state fields

- **Decision**: a private `enum Field<T> { Absent, Null, Value(T) }` for course fields `name`,
  `credits`, `rule_set`, `instructor`, `room`, `min_attendance_pct`, and for `slots`.
- **Rationale**: R44 absent = keep, `null` = clear; R45 missing `slots` = untouched.
- `code` and `name` cannot be `null` (R2: non-empty); `"slots": null` is an error
  ("must be a list; use [] to end all slots"); a slot's `room` absent or `null` both mean no slot
  room (use the course room), matching the template's second slot.

## D4. JSON paths

- **Decision**: dotted keys and `[i]` indexes from the root, e.g. `term.start`,
  `courses[2].slots[0].end`. Errors about the whole text (not JSON, root not an object) use the path
  `document`. Issue text is `"{path}: {message}"`.
- **Rationale**: §13.4 example format; `document` reads naturally for a non-technical user.
- **Ordering**: issues are reported in walk order: `format`, `version`, `term`, then `courses` by
  index; within an object, known keys in the order of the §13.4 example, then unknown keys.

## D5. How errors are returned

- **Decision**: `preview_import` returns `Ok(ImportPlan)` with `errors: Vec<ImportIssue>`; when
  `errors` is non-empty the change lists are empty. `Err` is reserved for database failures.
  `apply_import` returns `Err(Error::Validation(msg))` when the plan has errors, where `msg` holds
  every `path: message` line, and writes nothing.
- **Rationale**: the preview must show all errors with paths (R48 UI); keeping them in the plan
  avoids a new `Error` variant (no change to the existing error API). T48 can check both functions.
- **Alternatives**: new `Error::Import(Vec<ImportIssue>)` variant (changes the shared error type
  while M9 also edits it).

## D6. Course code matching (R43)

- **Decision**: `normalize(code) = code.chars().filter(|c| !c.is_whitespace()).flat_map(char::to_lowercase)`.
  The same key detects duplicates within the file (R42) and matches active-term courses, archived
  included (R44). A matched course keeps its stored code (never renamed). A new course is created
  with the code exactly as written (trimmed of leading/trailing whitespace, as R2's emptiness check
  trims).
- **OQ-1** (open): two active-term courses with the same normalized code. Default: error at the file
  course's `code` path, "matches more than one course (…, …)". Alternative: match the lowest id.

## D7. Day parsing

- **Decision**: string, trimmed, any case: `mon`…`sun`, `monday`…`sunday`, `"1"`…`"7"`; or a JSON
  integer 1–7. ISO numbering (§1: 1 = Monday). Anything else, including `0`, `8`, `"Mo"`, `"Tues"`,
  or `1.5`, is an error "must be a day name (Mon or Monday) or 1–7".
- **Rationale**: exactly the §13.4 variants; no guessing of other abbreviations.

## D8. Time and date parsing

- **Decision**: time = `H:MM` or `HH:MM`, hour 0–23, minutes exactly two digits 00–59, nothing
  else (no seconds, no AM/PM): own small parser, not chrono's lenient `%H:%M`. Date =
  `YYYY-MM-DD` via the existing core date parser. Slot `end` must be after `start` (R2);
  term `end` must not be before `start` (R2).
- **Rationale**: §13.4 says 24-hour `HH:MM`; a one-digit hour is unambiguous and common in AI
  output, so it is accepted.

## D9. Numbers

- **Decision**: `version` must be the integer 1. `credits` any JSON number > 0 (fractional
  allowed, R33). `min_attendance_pct` an integer 0–100 (schema column is INTEGER); `75.5` is an
  error "must be a whole number from 0 to 100". Strings are never coerced to numbers (except day,
  D7).

## D10. Rule set lookup

- **Decision**: exact match of `rule_set` against `rule_set.name`; an unknown name is an error at
  `courses[i].rule_set` listing the existing names. The plan stores the resolved id.
- **Rationale**: names are UNIQUE as stored (§13.3); a case-insensitive match could be ambiguous.

## D11. Slot set comparison and ending (R45)

- **Decision**:
  - "Slots valid on E" = the course's slots with `valid_from ≤ E` and (`valid_to` is NULL or
    `≥ E`).
  - Compare as multisets of `(weekday, start, end, room)`: sort both lists and compare; order in
    the file does not matter; duplicates count.
  - If different (or `slots: []` with any current slot): end each "current" slot by setting
    `valid_to = E − 1`, then insert the file's slots with `valid_from = E`, `valid_to = NULL`.
  - **OQ-3** (open): which slots are "current". Default: every slot of the course not ended before
    E (`valid_to` NULL or `≥ E`), so slots that start after E (left by an earlier import with a
    later effective date) are ended too and cannot duplicate the new slots. Alternative: only slots
    valid on E (literal), which can leave a future-dated slot running beside the new ones.
  - A slot that starts on E and is ended gets `valid_to = E − 1` (empty range; generates nothing),
    exactly as `change_slot` does.
- **Rationale**: R45 text plus R7. Sorting makes T45 idempotent.

## D12. Sync after apply

- **Decision**: after writing, `sync_in(tx, min(effective_from, today))`, inside the same
  transaction.
- **Rationale**: R45 says R7 semantics; M2 (research R-D4, human decision) syncs `change_slot` with
  the effective date as today, so a back-dated change removes the old slot's pending sessions from
  E on. When E is in the future, syncing with `today` is the normal R6 sync and still removes the
  old slot's pending sessions from E on (they are ≥ today). Attended, absent, and cancelled sessions
  are never removed (R5).
- **Alternatives**: always `today` (back-dated import would leave old pending sessions next to new
  ones on the same days); always `E` (a future E would skip the regular sync up to E).

## D13. Apply = re-plan inside the transaction

- **Decision**: `apply_import` starts a transaction, builds the plan with the same code
  `preview_import` uses (reading through the transaction), returns `Error::Validation` if it has
  errors, returns early with nothing written and no sync if the plan is empty, otherwise writes in
  order: term, new courses with their slots, updated course fields, slot ends, new slots; then syncs
  (D12) and commits. `ImportReport { plan, sync }` returns the plan it applied.
- **Rationale**: R48 "applies exactly that plan": same input, same database → same plan (T50
  compares `report.plan` with the preview). No plan object is passed back in, so the API stays the
  §13.4 signature.

## D14. Field changes (R44)

- **Decision**: for a matched course, a field is listed only when present in the file and different
  from the stored value after parsing (`credits` compared as `f64`, `3` = `3.0`; `rule_set` compared
  by id; text compared exactly). Fields: `name`, `credits`, `rule_set`, `instructor`, `room`,
  `min_attendance_pct`. `notes` and `archived` are not import fields (unknown keys). The write uses
  the stored course with only the changed fields replaced, so notes and archived are kept.

## D15. Term (R47)

- **Decision**: no active term: `term` missing → error at `term` "required when there is no active
  term"; present and valid → the plan creates it, active (as `save_term` does). Active term exists:
  `term` is still validated (R42 whole document) but ignored; a warning at `term` when name, start,
  or end differs, naming both values. Courses are matched and created in the active (or new) term.

## D16. Warnings

- Unknown key at any level: `{path}: unknown key, ignored`.
- Archived match: `courses[i]: matches archived course "{code}"; it stays archived`.
- Term differs: `term: ignored; the active term is "{name}" {start} to {end}`.
- Warnings never block apply and do not make a plan non-empty.

## D17. Clipboard in the app

- **Decision**: no new crate. The panel holds two read-only `TextInput`s (the template and prompt
  texts, shown in a scrollable preview area); each Copy button calls `select-all()` then `copy()`
  then `clear-selection()` on its `TextInput`. Slint already ships its clipboard backend (`copypasta`
  is in `Cargo.lock`).
- **Fallback**: if the builder finds this unreliable on a platform, add `arboard` in the app crate
  only and note it (app-only dependency, no rule change).

## D18. Embedding the docs

- **Decision**: `include_str!("../../../../docs/import/template.json")` and `…/ai-prompt.md` in
  `crates/app/src/pages/import.rs` (path relative to that file); the core template test uses
  `include_str!("../../../docs/import/template.json")` from `crates/core/tests/m10_import.rs`. A
  missing file fails the build.

## D19. Apply enablement and staleness

- **Decision**: the page stores the text and effective date of the last preview. Apply is enabled
  only when the current text and date equal those, the preview had no errors, and the plan is not
  empty. Editing the text or the date clears the shown preview. `can_apply` is a view-model
  function with unit tests.
- **Rationale**: R48 "apply_import applies exactly that plan" from the user's point of view.

## D20. Effective date input

- **Decision**: a date field pre-filled with today (`format_date_input`), parsed with the existing
  `viewmodel::parse_date`; a bad date shows the inline error and disables Preview and Apply (R49).

## Open questions

- **OQ-1** ambiguous code match (D6). Recommended: error.
- **OQ-2** the user's 7th-semester table for the worked example in `ai-prompt.md` is not in the
  repository; the human provides it (task T001).
- **OQ-3** which slots R45 ends (D11). Recommended: every slot not ended before E.
