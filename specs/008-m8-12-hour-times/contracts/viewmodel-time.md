# Contract: view-model time functions (crates/app/src/viewmodel.rs)

App-internal functions; not part of the core §5 API. All are pure (no clock, no I/O) and are
unit-tested in `crates/app/src/viewmodel/tests.rs`. Doc comments cite R31/R32.

## Changed

```rust
/// R31: `9:05 AM`, `12:00 AM` (midnight), `12:00 PM` (noon); seconds dropped.
pub fn format_time(t: NaiveTime) -> String;

/// R31: `9:00 AM–10:30 AM` (en dash, no spaces). Body unchanged; uses `format_time`.
pub fn format_time_range(start: NaiveTime, end: NaiveTime) -> String;

/// R31: `October 30, 2026, 11:59 PM`. Body unchanged; uses `format_time`.
pub fn format_datetime(dt: NaiveDateTime) -> String;

/// R32: digits plus AM/PM toggle to a 24-hour time; see data-model.md "Parse rules".
/// Error: `TIME_FORMAT_ERR`.
pub fn parse_time(digits: &str, pm: bool) -> Result<NaiveTime, String>;

/// R31: core validation messages pass through `times_to_12h` after the existing capitalisation.
pub fn error_text(e: &planner_core::Error) -> String;
```

Form parsers: only the time parameters change type, from `&str` to `TimeText<'_>`. Parameter
order, error keys (`start`, `end`, `due_time`), and all other checks are unchanged.

```rust
pub fn parse_slot_form(course_id: Option<i64>, weekday: i32, start: TimeText<'_>,
    end: TimeText<'_>, room: &str, valid_from: &str, valid_to: &str)
    -> Result<SlotInput, FormErrors>;
pub fn parse_makeup_form(course_id: i64, date: &str, start: TimeText<'_>, end: TimeText<'_>,
    room: &str, note: &str) -> Result<MakeupInput, FormErrors>;
pub fn parse_assessment_form(course_id: Option<i64>, kind: i32, title: &str, due_date: &str,
    due_time: TimeText<'_>, notes: &str, remind: &str) -> Result<AssessmentInput, FormErrors>;
```

## New

```rust
/// R32: one time field as typed: the digits text and the AM/PM toggle (`pm == false` is AM).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeText<'a> {
    pub digits: &'a str,
    pub pm: bool,
}

/// R32: pre-fill for editing: (`hmm` digits, pm). 09:30 → ("930", false); 00:15 → ("1215", false);
/// 12:05 → ("1205", true); 13:30 → ("130", true).
pub fn format_time_input(t: NaiveTime) -> (String, bool);

/// R31: rewrites every standalone `HH:MM` token (valid time, not adjacent to another digit) to
/// `format_time` form; other text unchanged. "end time 09:00 is not after 10:30" →
/// "end time 9:00 AM is not after 10:30 AM".
pub fn times_to_12h(text: &str) -> String;
```

`TIME_FORMAT_ERR` becomes "Enter the time as digits, e.g. 930 (hour 1–12, minutes 00–59)".
Tests compare against the constant's value, so wording changes need one edit in the test file.

## Tests (tester, round 2)

| Test | Checks |
|---|---|
| `t34_am_inputs_parse` | AM: `9`→09:00, `930`→09:30, `0930`→09:30, `1215`→00:15, `12:15`→00:15 |
| `t34_pm_inputs_parse` | PM: `1215`→12:15, `130`→13:30 |
| `t34_bad_inputs_are_errors` | `1260`, `1300`, `0`, empty with AM and PM → `Err(TIME_FORMAT_ERR)` |
| `t35_times_display_12_hour` | 00:00, 09:05, 12:00, 13:30 → "12:00 AM", "9:05 AM", "12:00 PM", "1:30 PM" |
| pre-fill tests | the four D4 examples; round trip over all 1,440 minutes with both helpers |
| edge tests | 5+ digits, letters, ` 930 ` trimmed, `9:30` |
| `times_to_12h` tests | core slot message; date-time text; text with no time unchanged; `2026-10-30` unchanged |
| updated existing tests | `format_time*`, `format_datetime*`, `parse_time*`, form parser tests use the new forms |
