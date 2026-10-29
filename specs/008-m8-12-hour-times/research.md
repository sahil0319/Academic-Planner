# Research: M8 12-Hour Times

All decisions stay inside `crates/app`. Nothing in `docs/spec.md` §1 is re-decided; no rule,
schema, backup format, CSV format, or §5 method changes.

## D1. Where display formatting changes

- **Decision**: Change the body of the existing `viewmodel::format_time` to the 12-hour form
  (R31, T35). `format_time_range` and `format_datetime` already call it, so session rows, slot
  rows, assessment rows, and the Dashboard day panel (which calls `format_time` directly) all
  switch with no call-site change.
- **Rationale**: one function, one test; every display path already goes through it (a grep of
  `crates/app/src` shows no other `%H:%M` display formatting).
- **Alternatives considered**: a new `format_time_12h` next to the old one (leaves a 24-hour
  formatter in the app that nothing should call).
- **Not changed**: `pre_restore_file_name` uses `%H%M` in a file name, not a displayed time.

## D2. Parser shape and signature

- **Decision**: `parse_time(digits: &str, pm: bool) -> Result<NaiveTime, String>` replaces the
  strict `HH:MM` parser. Steps: trim; drop every `:`; require 1–4 ASCII digits; 1–2 digits = hour
  with minutes 0; 3–4 digits = all but the last two digits are the hour, last two the minutes;
  hour 1–12 and minutes 0–59 or error; map 12 AM → 0, 12 PM → 12, other PM → hour + 12 (R32,
  T34).
- **Rationale**: keeps one parser with the same name, so every form caller is found by the
  compiler when the signature changes.
- **Alternatives considered**: keep the old parser alongside (dead code after M8); accept other
  separators such as `.` (not in R32).

## D3. Passing the toggle through the form parsers

- **Decision**: add `pub struct TimeText<'a> { pub digits: &'a str, pub pm: bool }` (Copy) and
  change the time parameters of `parse_slot_form`, `parse_makeup_form`, and
  `parse_assessment_form` from `&str` to `TimeText<'_>`. Error keys (`start`, `end`, `due_time`)
  stay the same.
- **Rationale**: adding a separate `bool` per time would push `parse_slot_form` to 9 and
  `parse_assessment_form` to 8 parameters, over clippy's `too_many_arguments` limit of 7, which
  the gate denies.
- **Alternatives considered**: `#[allow(clippy::too_many_arguments)]` (hides a lint the project
  otherwise keeps); a tuple `(&str, bool)` (less readable at call sites).

## D4. Pre-fill

- **Decision**: `format_time_input(t: NaiveTime) -> (String, bool)` returns the `hmm` digits
  (12-hour hour without a leading zero, then two-digit minutes) and `pm`. Examples: 09:30 →
  (`"930"`, false); 13:30 → (`"130"`, true); 00:15 → (`"1215"`, false); 12:05 → (`"1205"`, true).
  Name mirrors the existing `format_date_input`.
- **Rationale**: R32 "pre-fills `hmm` and the toggle"; a round-trip property
  (`parse_time(format_time_input(t)) == t` for all 1,440 minutes) proves it.

## D5. Times inside core's messages

- **Decision**: add `times_to_12h(text: &str) -> String`, which rewrites every standalone
  `HH:MM` token (two digits, colon, two digits, a valid time, not touching another digit) to the
  12-hour form and leaves all other text alone. `error_text` applies it to
  `Error::Validation` messages, so core's R2 slot message reads "Slot end time 9:00 AM is not
  after its start time 10:30 AM".
- **Rationale**: R31 covers every time shown in the UI; core's text stays unchanged (no core
  change), following the M4 precedent `display_status_text`, which rewrites core's ISO date for
  display (T09).
- **Alternatives considered**: change core's message (core change, out of M8's app-only scope);
  pre-check end > start in the app (duplicates an R2 rule that belongs to core).
- **OQ-1 (notification text)**: if the human picks option B, `notify::show` passes the reminder
  title through the same `times_to_12h`. Assessment titles contain "October 30, 2026, 23:59";
  the token rule handles it. Nothing else changes.

## D6. AM/PM toggle and time field in Slint

- **Decision**: add two components to `crates/app/ui/components.slint`:
  - `AmPmToggle`: a pill with two text segments "AM" and "PM", `in-out property <bool> pm`,
    modelled on `AppearanceSwitch`/`AppearanceSegment` (surface-alt pill, active segment on
    surface with a border, `accessible-role: button`, `accessible-label`, Tab focus, Enter/Space
    activates, clicked sets `pm`).
  - `TimeField`: the same label, bordered input, and inline error as `Field`, with the
    `AmPmToggle` beside the input in the input row. Properties: `label`, `text` (in-out), `pm`
    (in-out), `error`.
  Only existing `Theme.*` tokens are used (for example `input-height`, `radius-full`,
  `radius-sm`, `border-width`, `space-xs`, `space-sm`, `surface`, `surface-alt`, `border`,
  `primary`, `on-surface`, `on-surface-muted`, label font tokens). No new tokens, no raw px or
  hex; the theme grep stays clean.
- **Rationale**: matches the router's instruction (existing components and tokens only, no design
  skills) and DESIGN.md's switch pattern; text labels keep the meaning readable without color.
- **Alternatives considered**: a ComboBox with AM/PM (heavier, two clicks); placing a separate
  toggle next to `Field` in each page (misaligns when the field shows an error).

## D7. "Digits-only" field

- **Decision**: the `TimeField` input is a plain single-line text input; the parser enforces
  digits (D2). A typed `:` is accepted by the input and ignored by the parser (R32, T34 `12:15`).
- **Rationale**: no logic in `.slint` beyond bindings (constitution III); Slint's numeric input
  type would block the `:` that R32 says to ignore and behaves differently across platforms.

## D8. New-form defaults and wording

- **Decision**: a new or cleared form sets each time field to empty and its toggle to AM. Labels
  drop "(HH:MM)": "Start", "End", "Due time". `TIME_FORMAT_ERR` becomes
  "Enter the time as digits, e.g. 930 (hour 1–12, minutes 00–59)".
- **Rationale**: naive UI choices; the spec leaves them open and they change no rule.

## D9. Test round structure

- **Decision**: round 1 (builder) changes signatures, adds `TimeText` and stubs (`todo!()`
  bodies only for new functions), adds the `*-pm` bool properties to the page state globals so
  page code can pass them, and makes the existing tests compile with a mechanical call-site
  update only (no expectation changes). Round 2 (tester) writes T34, T35, and the other
  view-model tests, and revises the existing tests whose expectations change. Round 3 (builder)
  implements the functions, adds the Slint components, wires every page (fields, pre-fill,
  clear, labels), and runs the gate.
- **Rationale**: tests must compile before the tester starts, and test expectations are the
  tester's to set.
