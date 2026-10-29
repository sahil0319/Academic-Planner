# Contract: Import Panel (§13.4 UI, R49, R50)

## Placement

Courses page header gets an "Import…" button. It opens an `ImportPanel` (in `ui/import.slint`)
in the same way the page's existing forms open. Close returns to the course list. `.slint` holds
only bindings and callbacks; every string comes from `viewmodel/import.rs`. `Theme.*` only.

## Panel contents (top to bottom)

1. Paste box (multi-line `TextInput`, monospace if `Theme` has one, else default font).
2. Buttons: **Copy template**, **Copy AI prompt** (research D17: two read-only `TextInput`s
   holding the embedded texts; copy = select-all, copy, clear-selection). After copying, a short
   status line "Template copied" / "AI prompt copied".
3. **Effective date** field, pre-filled with today (`format_date_input(today)`); inline error from
   `parse_date` (R49).
4. **Preview** button (disabled while the date is invalid or the paste box is blank).
5. Result area:
   - errors: a heading "N errors — nothing will be imported" and one line per error
     `path: message`;
   - otherwise a summary line and sections: "Term to create", "Courses to add" (code, name,
     slots), "Courses to update" (code, "(archived)" tag, field change lines, slot change lines),
     "Warnings" (one line each). Empty plan: "No changes. Everything already matches."
6. **Apply** button, enabled per `can_apply` (research D19). After apply: a result line from
   `import_report_text`, the preview is cleared, and the controller refreshes all pages.

Editing the paste box or the date clears the result area (stale preview, D19).

## View-model functions (`crates/app/src/viewmodel/import.rs`, unit-tested)

| Function | Output examples |
|---|---|
| `issue_line(&ImportIssue) -> String` | `courses[2].slots[0].end: must be after start` |
| `errors_heading(n: usize) -> String` | `1 error — nothing will be imported`, `3 errors — nothing will be imported` |
| `plan_summary(&ImportPlan) -> String` | `Creates term "Semester 7" · adds 5 courses · updates 1 course`; empty: `No changes. Everything already matches.` |
| `term_line(&ImportTerm) -> String` | `Semester 7, 5 Jan 2026 – 1 May 2026` (uses `format_date`) |
| `course_add_line(&CourseAdd) -> String` | `CSE 4711 Artificial Intelligence · 3 credits · Theory` |
| `field_change_line(&FieldChange) -> String` | `Instructor: — → Dr. Rahman`, `Credits: 3 → 1.5`, `Rule set: Theory → Lab`, `Target: 75% → —` |
| `slot_line(&ImportSlot) -> String` | `Mon 10:30 AM–11:45 AM · Room 301` / without room: `Fri 2:30 PM–3:45 PM` (uses `weekday_name`, `format_time_range`) |
| `slot_change_lines(&SlotChange, effective: NaiveDate) -> Vec<String>` | `Ends after 13 Jan: Mon 10:30 AM–11:45 AM`, `From 14 Jan: Mon 11:00 AM–12:15 PM` |
| `import_report_text(&ImportReport) -> String` | `Imported: 5 added, 1 updated. 72 sessions created, 3 removed.` |
| `can_apply(preview: Option<&PreviewState>, text: &str, date_text: &str) -> bool` | true only if the preview is for this exact text and date, has no errors, and is not empty |

Labels for fields: Name, Credits, Rule set, Instructor, Room, Target (matches the course form's
"target" wording; builder aligns with the existing form label). `None` shows as `—` (`or_dash`).

## Wiring (`crates/app/src/pages/import.rs`)

- `wire(&Rc<App>)`: callbacks preview, apply, copy-done, text-edited, date-edited, open, close.
- Preview calls `planner.preview_import(text, date)`; Apply calls
  `planner.apply_import(text, date, today)` with today from the app clock (app crate may read the
  clock); then `App::refresh_all`.
- `Err` from either call shows `error_text(&e)` in the result area.
- Texts: `const TEMPLATE: &str = include_str!(…template.json)`,
  `const AI_PROMPT: &str = include_str!(…ai-prompt.md)` (research D18).
