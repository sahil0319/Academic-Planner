# Data Model: M8 12-Hour Times

No schema change, no migration, no new setting, no backup or CSV change (FR-004). Stored times
stay `TEXT 'HH:MM'` and `due_at` stays `TEXT 'YYYY-MM-DDTHH:MM'` (§3).

## View-model values (crates/app/src/viewmodel.rs)

| Item | Kind | Meaning |
|---|---|---|
| `TimeText<'a>` | struct, `Copy` | One time field as typed: `digits: &'a str` (raw field text) and `pm: bool` (toggle; `false` = AM). Passed to the form parsers in place of the old `&str`. |
| displayed time | `String` | `h:mm AM` / `h:mm PM` (R31). |
| pre-fill pair | `(String, bool)` | `hmm` digits and `pm` for an existing time (R32). |

## Parse rules (R32)

Input: `digits`, `pm`. Output: `NaiveTime` or the inline error text.

1. Trim spaces; remove every `:`.
2. Remaining text must be 1–4 ASCII digits, else error (covers empty).
3. 1–2 digits: hour = all digits, minute = 0. 3–4 digits: hour = all but the last two, minute =
   the last two.
4. hour must be 1–12 and minute 0–59, else error.
5. 24-hour hour: AM → `hour % 12`; PM → `hour % 12 + 12`.

| digits | pm | result |
|---|---|---|
| `9` | AM | 09:00 |
| `930`, `0930` | AM | 09:30 |
| `1215`, `12:15` | AM | 00:15 |
| `1215` | PM | 12:15 |
| `130` | PM | 13:30 |
| `1260`, `1300`, `0`, empty | either | error |

## Display rules (R31)

`hour12 = h % 12`, shown as 12 when 0; minutes two digits; suffix `AM` when `h < 12`, else `PM`.

| stored | shown |
|---|---|
| 00:00 | 12:00 AM |
| 09:05 | 9:05 AM |
| 12:00 | 12:00 PM |
| 13:30 | 1:30 PM |

Range: `{start}–{end}` with an en dash, no spaces. Date-time: `{long date}, {time}`.

## Page state (Slint globals, new `bool` properties, in-out)

| Global | New properties | Paired text property |
|---|---|---|
| `TimetableState` | `rs-start-pm`, `rs-end-pm` | `rs-start`, `rs-end` (reschedule) |
| `TimetableState` | `slot-start-pm`, `slot-end-pm` | `slot-start`, `slot-end` (Rules slot form) |
| `CoursesState` | `mk-start-pm`, `mk-end-pm` | `mk-start`, `mk-end` (Add makeup class) |
| `AssessmentsState` | `due-time-pm` | `due-time` |

Seven time fields in total. Existing `*-error` properties are reused unchanged.
