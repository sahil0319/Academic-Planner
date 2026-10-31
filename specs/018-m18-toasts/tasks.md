# Tasks: M18 toast notifications and Rules page save confirmation

Spec: `docs/spec.md` section 13.13 (R87-R91, T80-T85). Look: `DESIGN.md` "Toast notification".
Light flow: no plan, checklist, or analyze artifacts. No schema or core change. Every task is one commit with the prefix `M18:`.

## Inventory of confirmation texts (R89)

Searched: `crates/app/src` (`pages/*.rs`, `controller.rs`, `viewmodel.rs`, `viewmodel/*.rs`, `restore.rs`, `updater.rs`, `notify.rs`) and `crates/app/ui/*.slint`.
Pages that have no success text at all: Timetable, Courses, Assessments, Dashboard (their `message` holds errors only already; no change except the shared doc comment "Row-action errors and success notes").

### Convert to toast (11 texts)

| # | Where | Current text (or builder) | Toast? | New viewmodel function |
|---|---|---|---|---|
| 1 | `pages/settings.rs:16` const `TERM_SAVED`, used at `:165` (`save_term`) | "Term saved." | yes | `vm::term_saved_text()` |
| 2 | `pages/settings.rs:17` const `HOLIDAYS_SAVED`, used at `:214` (`add_holiday`) | "Holidays saved." | yes | `vm::holidays_saved_text()` |
| 3 | `pages/settings.rs:18` const `HOLIDAY_REMOVED`, used at `:240` (`remove_holiday`) | "Holiday removed." | yes | `vm::holiday_removed_text()` |
| 4 | `pages/settings.rs:19` const `SETTINGS_SAVED`, used at `:281` (`save_settings`) | "Settings saved." | yes | `vm::settings_saved_text()` |
| 5 | `pages/settings.rs:305` (`backup`) | `vm::backup_saved_message(path)`: "Backup saved to {path}" | yes | keep existing `vm::backup_saved_message(path)` |
| 6 | `pages/settings.rs:362` (`restore_confirm`, via `changed_with_note`) | `vm::restored_message(copy)`: "Backup restored. Previous data saved to {copy}" | yes | keep existing `vm::restored_message(copy)` |
| 7 | `pages/settings.rs:385` (`export_csv`) | `vm::csv_exported_message(folder)`: "Exported attendance.csv and assessments.csv to {folder}" | yes | keep existing `vm::csv_exported_message(folder)` |
| 8 | `pages/import.rs:92` const `TEMPLATE_COPIED`, set at `:64`; shown by `ui/import.slint:170` (`copy-status` Text) | "Template copied" | yes | `vm::template_copied_text()` |
| 9 | `pages/import.rs:94` const `PROMPT_COPIED`, set at `:66`; same Text | "AI prompt copied" | yes | `vm::prompt_copied_text()` |
| 10 | `pages/rules.rs:375-377` (`save_set`, success branch; today shows nothing) | none today; new "Rule set saved." (R90) | yes | `vm::rule_set_saved_text()` |
| 11 | `pages/rules.rs:426-431` (`save_scale`, success branch; today shows nothing) | none today; new "Grade scale saved." (R90) | yes | `vm::grade_scale_saved_text()` |

Notes on the table:
- Existing wording is kept for rows 1-9. Rows 5-7 keep their existing function names (they have unit tests in `viewmodel/tests.rs`); the other functions are new and return `String`. Decision: no rename of existing functions.
- Rows 1-4 and 6 today go through `changed_with_note`, which shows the note only if the refresh left no error in the page `message`. Decision: keep that guard, as `changed_then_toast` (toast only when no error is shown).
- Row 8-9: the `template-copied` / `prompt-copied` callbacks in `import.slint` stay; the `copy-status` property, its Text, and the `set_copy_status` calls in `import.rs` (lines 87, 121, 136) are removed.

### Stay inline (no toast)

| Where | Text | Reason |
|---|---|---|
| `pages/import.rs:178-185` and `viewmodel/import.rs:174` `import_report_text` ("Imported: N added, ...") | Import result list | R89: result list stays inline |
| `ui/import.slint` result lines, preview lines | preview, warnings, errors | R89 |
| `pages/settings.rs:233` | "This holiday no longer exists" | error |
| `pages/settings.rs` form errors, `vm::file_error_message`, `vm::pre_restore_failed_message`, `vm::error_text` | errors | R89: errors stay |
| `ui/settings.slint:92` "No term saved yet. Enter the term below to start." | standing empty state | not a confirmation |
| `ui/settings.slint:280` "Restoring replaces all current data ..." and `confirm_restore` panel | confirm panel | R89 |
| `pages/courses.rs` confirm panel (delete courses), `after()` | confirm panel, errors only | R89 |
| `pages/about.rs` / `viewmodel.rs:1387` `update_view`: "Checking for updates...", "You're up to date.", "Downloading... N%", "Installing...", "Couldn't check for updates", "The update couldn't be installed..." | standing status line of the updater | R89: status lines. Decision: "You're up to date." is a standing result next to the Check button, so it stays inline |
| `viewmodel.rs` `banner_text` ("Version X is available."), `confirm_title`, `confirm_label`, `ui/about.slint:274` confirm notes | update banner and confirm panel | R89 |
| `viewmodel.rs` `BELOW_TARGET_LABEL`, dashboard banner | below-target banner | R89 |
| `pages/timetable.rs:91` first form error; `pages/dashboard.rs`, `pages/assessments.rs` problems | errors | R89 |
| `pages/about.rs:357`, `controller.rs:261` `show_message` for `open_url` failure and row-action errors | errors | R89 |

Result: 11 texts become toasts; the page `message` property holds errors only afterward. Decision: `App::show_message` stays (errors).

## Tasks

- [x] T001 `crates/app/src/toast.rs`: pure `ToastQueue` (R88). API: `new`, `add(text, now: Duration) -> u64`, `close(id)`, `expire(now)`, `visible(now) -> Vec<&Toast>`, `remaining(&Toast, now) -> f32`; `TOAST_LIFETIME` = 5 s; `MAX_TOASTS` = 3. Same text on screen restarts that toast's timer; a fourth toast drops the oldest. No system clock, no slint. Declare `mod toast;` in `main.rs`. Doc comments cite R88. Commit: `M18: toast queue`.
- [x] T002 `crates/app/src/viewmodel.rs`: move `rule_set_lines` out of `pages/rules.rs` (R87; T80 tests exist) and add the new text functions from the table: `term_saved_text`, `holidays_saved_text`, `holiday_removed_text`, `settings_saved_text`, `template_copied_text`, `prompt_copied_text`, `rule_set_saved_text`, `grade_scale_saved_text`, plus the hint text function for R91 (`fix_rows_hint_text`: "Fix the rows marked in red before saving."). Existing wording unchanged. Commit: `M18: toast texts and rule_set_lines in viewmodel`.
- [x] T003 UI and controller (R88). `ToastState` global (in `ui/types.slint` or new `ui/toast.slint`) with the list of toasts (text, remaining fraction, id) and a `dismiss(id)` callback; toast component per DESIGN.md "Toast notification": 320px, `success-bg` fill, 3px success rule on the left edge, 12px padding, 24px cross button with accessible label "Dismiss", 3px border-coloured track with a success bar shrinking right to left, no shadow, no animation. Placed in `app-window.slint` above all pages and dialogs: bottom-right, 16px from the edges, stacked 8px apart, newest at the bottom. All sizes and colors from `Theme.*` (add tokens to `ui/theme.slint` only if missing; no raw hex or px elsewhere). `App::toast(text)` in `controller.rs`; a `slint::Timer` in the controller (50-100 ms) calls `expire` and updates the bar fractions; the cross callback calls `close`. Commit: `M18: toast component and controller`.
- [x] T004 Convert every "yes" row (1-9) of the table to `app.toast(vm::...)`. The page `message` holds errors only; remove `changed_with_note` (replace by the guarded toast), the `copy-status` property and Text in `import.slint`, the `set_copy_status` calls, and the stale "success notes" comments in `ui/*.slint`. Unit tests of the three existing message functions stay green. Commit: `M18: confirmations as toasts`.
- [x] T005 Rules page (R87, R90, R91, rows 10-11):
  - successful rule set save: form stays open, toast `vm::rule_set_saved_text()`, Saved rules rebuilt from the rule set core returns via `vm::rule_set_lines`, every page refreshes; a second save with no change shows the toast again (queue restarts the timer);
  - successful grade scale save: toast `vm::grade_scale_saved_text()`;
  - failed parse (any component or band row): no toast, a line above the Save button with the hint "Fix the rows marked in red before saving."; core's validation text stays inline (R34);
  - a failed reload after a successful save: the error text stays in `message` (today `save_set` overwrites it with an empty string at `rules.rs:377`).
  Commit: `M18: rules page save feedback`.
- [x] T006 Final gate and checks: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`; grep `crates/app/ui/*.slint` for raw hex and px literals outside `ui/theme.slint` (comments included) and fix any; `npx -y @google/design.md@0.4.0 lint DESIGN.md`. Update `.claude/state/progress.json`. Commit: `M18: gate and lint` (only if files changed).

## Tests (written first by the tester; the builder never edits them)

- [x] Tests T80 (R87): `rule_set_lines` unit tests in `crates/app/src/viewmodel/tests.rs`; covered by T002. Done.
- [x] Tests T81 (R90, R88): Rules page save toast and repeat save restarts the timer; text and queue tests in `crates/app/src/viewmodel/tests.rs` (t81) and `crates/app/src/toast/tests.rs`; covered by T005 and T001.
- [x] Tests T82 (R91): failed parse shows the hint and no toast; core validation text inline; Saved rules unchanged; covered by T005.
- [x] Tests T83 (R90): grade scale save toast "Grade scale saved."; covered by T005.
- [x] Tests T84 and T84b (R88): `ToastQueue` dedupe, expiry, cap of three, close, in `crates/app/src/toast/tests.rs`; covered by T001.
- [x] Tests T85 (R89): each confirmation yields one toast with the text from `viewmodel.rs`, page `message` stays empty, an error case shows no toast; covered by T002 and T004.
- [x] Fill in the exact test file paths above once the tester has written them.

## M18a sweep (R89a, T91)

Human request 2026-10-31: every button that saves, adds, applies, deletes, or archives user data toasts on success, on every page. Failures keep the inline error and show no toast. Texts come from `crates/app/src/viewmodel.rs`; each handler calls `App::toast` after the page refresh (`App::changed`), and only when that refresh left no error in the page message where the page has one. Light flow, one commit per step, prefix `M18a:`.

### Handler audit

| Page | Handler (`pages/*.rs`) | Button | Toast text | State before sweep |
|---|---|---|---|---|
| Courses | `save_course` | Save / Add course | "Course added." / "Course saved." (`course_saved_text(created)`) | missing |
| Courses | `delete_selected` | Delete (confirmed) | "Deleted 1 course." / "Deleted 2 courses." (`courses_deleted_text(n)`) | missing |
| Courses | `set_archived` (was a closure in `wire`) | Archive / Unarchive | "Course archived." / "Course unarchived." (`course_archived_text(archived)`) | missing |
| Courses | `add_makeup` | Add makeup class | "Makeup class added." (`makeup_added_text()`) | missing |
| Assessments | `save` | Save / Add (includes score entry) | "Assessment added." / "Assessment saved." (`assessment_saved_text(created)`) | missing |
| Assessments | `delete_assessment` (was a closure in `wire`) | Delete | "Assessment deleted." (`assessment_deleted_text()`) | missing |
| Timetable | `reschedule` | Reschedule Save | "Class rescheduled." (`rescheduled_text()`) | missing |
| Timetable | `save_slot` | Save slot / Add slot | "Slot added." / "Slot saved." (`slot_saved_text(created)`) | missing |
| Timetable | `end_slot` | End slot | "Slot ended." (`slot_ended_text()`) | missing |
| Rules | `save_set` | Save (rule set) | "Rule set saved." | done (M18) |
| Rules | `save_scale` | Save grade scale | "Grade scale saved." | done (M18) |
| Rules | `delete_set` | Delete rule set | "Rule set deleted." (`rule_set_deleted_text()`) | missing |
| Import | `run_apply` | Apply | "Imported 3 courses." (`import_applied_text(courses, term)`); result list stays | missing |
| Import | template / prompt copy | Copy template / prompt | "Template copied" / "AI prompt copied" | done (M18) |
| Settings | `save_term` | Save term | "Term saved." | done (M18) |
| Settings | `archive_confirm` | Archive term (confirmed) | "Term archived." | done (M18) |
| Settings | `add_day_off` (was `add_holiday`, M15) | Add holiday, Add exam period (mid, final) | `day_off_saved_text(kind)`, e.g. "Mid exams saved." | done (M15) |
| Settings | `remove_holiday` | Remove holiday | "Holiday removed." | done (M18) |
| Settings | `save_settings` | Save settings | "Settings saved." | done (M18) |
| Settings | `backup`, `export_csv`, `restore_confirm` | Backup, Export CSV, Restore | existing messages | done (M18) |
| Dashboard | `shift_biweekly` | Move to next week | "Moved to next week." | done (M12) |

No toast by design (the row itself changes, R89a):
- session status buttons (Attended, Absent, Cancelled, Pending) in `dashboard.rs`, `timetable.rs`, `courses.rs`;
- the "completed" checkbox on an assessment row (`assessments.rs` `set_completed`): a row toggle like the status buttons;
- the "Check for updates on start" switch on About (`about.rs` `toggle_check_on_start`): a switch, not a save button; it shows its own error and reloads the switch on failure.

Not built yet (M16 and any later page): public holiday Add selected. The M16 builder adds a toast for it (R89a) and a line to the coverage test (`t91_every_persisting_handler_toasts`).

### Tasks

- [x] T007 Tests first (T91): text functions in `viewmodel/tests.rs` (`t91_*`) and the handler coverage check `t91_every_persisting_handler_toasts`. Commit: `M18a: tests for save toasts (T91)`.
- [x] T008 `viewmodel.rs`: the new text functions. Commit: `M18a: toast texts for save, add, delete, archive`.
- [x] T009 Handlers: courses, assessments, timetable, rules (delete), import. Commit: `M18a: toasts on every save, add, apply, delete, archive`.
- [x] T010 Gate, theme grep, rebase onto `main`, gate again.
