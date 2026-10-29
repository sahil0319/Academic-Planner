# Contract: time fields in the UI (crates/app/ui, crates/app/src/pages)

## Components (crates/app/ui/components.slint)

```slint
// R32: AM/PM toggle. Two text segments "AM" and "PM"; `pm == false` selects AM.
export component AmPmToggle inherits Rectangle {
    in-out property <bool> pm;
}

// R32: labelled digits field plus AM/PM toggle, with the inline error below (like Field).
export component TimeField inherits VerticalLayout {
    in property <string> label;
    in-out property <string> text;
    in-out property <bool> pm;
    in property <string> error;
}
```

- Built from base elements and the existing `AppearanceSegment` pattern; `Theme.*` tokens only;
  no new theme tokens; no raw hex or px (theme grep stays clean).
- Each segment shows its text label, has `accessible-role: button` and `accessible-label`
  ("AM"/"PM"), takes Tab focus, and activates on click, Enter, or Space.
- No logic beyond bindings: clicking a segment sets `pm` to `false` or `true`.

## Fields per page

| Page / form | Text property | Toggle property | Label |
|---|---|---|---|
| Timetable, reschedule | `TimetableState.rs-start` / `rs-end` | `rs-start-pm` / `rs-end-pm` | "Start" / "End" |
| Timetable Rules, slot form | `TimetableState.slot-start` / `slot-end` | `slot-start-pm` / `slot-end-pm` | "Start" / "End" |
| Courses detail, Add makeup class | `CoursesState.mk-start` / `mk-end` | `mk-start-pm` / `mk-end-pm` | "Start" / "End" |
| Assessments form | `AssessmentsState.due-time` | `due-time-pm` | "Due time" |

## Page code (crates/app/src/pages)

- **Open for edit** (reschedule a session, edit a slot, edit an assessment): set text and toggle
  from `vm::format_time_input(stored_time)`.
- **New or cleared form**: text empty, toggle `false` (AM).
- **Save**: build `vm::TimeText { digits: &text, pm }` from the two properties and pass it to the
  form parser; show errors through the existing `*-error` properties.
- **Display**: no change in page code; rows use `format_time_range` / `format_datetime`, and the
  Dashboard day panel uses `format_time`, all now 12-hour.
- **Core errors**: shown through `vm::error_text`, which now rewrites times (R31).
- **OQ-1 = B only**: `notify::show` passes `reminder.title` through `vm::times_to_12h`.
