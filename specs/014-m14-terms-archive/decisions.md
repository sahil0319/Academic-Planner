# M14 decisions

Taken unattended with the recommended option (router instruction). One line each; the human may revisit.

- D1 Layout: `archive_term` and `past_terms` live in a new private core module `terms.rs`; R40 logic is shared through `pub(crate) term_gpa_for(term_id)` in `marks.rs`, and `term_gpa` wraps it (signature unchanged).
- D2 The R68 core message formats the end date as "May 1, 2026" (`%B %-d, %Y`), not ISO: other core Validation texts print ISO via `Display`, but this text is user-facing, R66 defines display dates, and `reminders.rs` already formats long dates in core. Exact text: "This term ends on May 1, 2026; archive it after that" (no trailing period, as in the spec).
- D3 `archive_term` on an existing but already inactive term returns `Ok(())` and changes nothing (idempotent), checked before the date rule; an unknown id returns `Error::NotFound { entity: "term", id }`.
- D4 `past_terms` counts courses and sums attendance over every course of the term including archived courses (R3 keeps history); GPA follows R40 exactly (non-archived graded courses); order `end_date DESC, id DESC`.
- D5 Date ranges are written with full dates on both sides, "January 5, 2026 – May 1, 2026", as R66 says; DESIGN.md's example compresses the first year away; the spec wins for text (flag for a later DESIGN.md wording sync).
- D6 Settings card week line: "Week {n} of {N}" inside the term, "Starts {date}" before it, "Ended on {date}" after it.
- D7 Disabled-Archive reason line: "Archive is available after {end}." ("" when enabled).
- D8 Successful archive shows the toast "Term archived." (R88/R89: every completed action is confirmed by a toast; the spec names no text). Archive errors go to the page `message`.
- D9 Past terms empty state: "No past terms yet. A term shows here after you archive it." (DESIGN: every list has an empty state).
- D10 T70 is tested through the pure reducer `term_save_outcome` (toast, clear fields, close form, inline error) because the app has no headless Slint tests; the page code follows the reducer.
- D11 The Dashboard keeps its existing "Getting started" no-term card; the R66 no-term line is added above it (removing the card is not in the spec).
- D12 The "ended" dashboard sentence "Archive it in Settings." is a `LinkButton` (new component, `Theme.link`) that sets `AppState.current-page = 4`, per DESIGN.md "Term line"; `TermLine { text, link }` with `full()` gives the R66 sentence for tests.
- D13 Past-term row figures carry DESIGN.md labels: "{n} courses" (singular "1 course"), "Attendance 87.5%" / "Attendance —", "GPA 3.40" / "GPA —".
- D14 The term form fields are no longer filled from the active term on load; only Edit pre-fills them and a successful save clears them (R67). `term-editing` lives in `SettingsState` (set by Rust, read back on refresh) so the minute timer keeps the form open; no `controller.rs` change.
- D15 Week numbers use an own pure helper `term_week_number(start, date)` in `viewmodel.rs` (R54 numbering, Monday-Sunday weeks, week 1 contains the start); M12's `term_week_parity` is not on this branch.
- D16 T68 needs no core change: sync, dashboard, and reminders already read only the active term (checked in code); the T68 test is the guard.
- D17 Analysis remediation applied: removed the [P] marker from T006 (it needs T005) and corrected the coverage table in plan.md.
- D18 TEST BUG, not fixed (builder may not edit tests): t69 in m14_terms_archive.rs scores the Spring B course 23.25 on Mid, but Mid is 25% of 300 marks = 75 (R36/R37/R40, item_max = marks), so 23.25 is 31% = F (0.00), not 62% = B (3.00); 46.5 is the intended score. With 46.5 all six core tests pass (checked on a scratch copy). Tester must fix the number.
- D19 (router): T69 fixture score corrected 23.25 -> 46.5 (Mid is 75 marks; 62% = 46.5), test data bug, assertions unchanged.
