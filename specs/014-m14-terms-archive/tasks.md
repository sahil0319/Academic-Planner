# Tasks: M14 Terms on the dashboard, save feedback, archive

Spec: `docs/spec.md` §13.9 (R66-R70, T66-T70); toasts §13.13 (R88, R89). Plan and frozen
interfaces: [plan.md](plan.md). Decisions: [decisions.md](decisions.md). Look: `DESIGN.md` "Term
line (Dashboard)", "Term card (Settings)", "Past terms list (Settings)", "Danger button".

Rules for every task:

- One task = one commit, message prefixed `M14:` (e.g. `M14: archive_term and past_terms`). Tick
  the box in this file in the same commit.
- Tests are written first by the tester; the builder never edits them. Names follow
  `t67_archive_before_or_on_end_is_rejected`; doc comments cite the R-id.
- No schema change, no migration, no change to `docs/spec.md`, README, or other docs.
- `.claude/state/` is updated by the router, not in these commits.

Stories (for labels):

- **US1** Core archive and past terms (R68, R70; T67, T68, T69).
- **US2** Dashboard term line (R66; T66).
- **US3** Settings Term card, save feedback, archive confirm, Past terms (R67-R70; T70).

## Phase 1: Tests first (tester)

- [x] T001 [P] [US1] Write core integration tests in `crates/core/tests/m14_terms_archive.rs`
  (`mod common;`, in-memory SQLite) exactly as plan.md "Test plan" core list:
  `t67_archive_before_or_on_end_is_rejected` (term 2026-01-05..2026-05-01; today 04-30 and 05-01
  → `Error::Validation("This term ends on May 1, 2026; archive it after that")`, still active),
  `t67_archive_after_end_keeps_data` (05-02 → Ok, `active_term()` None, course and session counts
  unchanged), `t67_archive_unknown_term_is_not_found` (`Error::NotFound { entity: "term", .. }`),
  `t67_archive_inactive_term_is_noop` (Ok, nothing changes),
  `t68_archived_term_is_ignored_by_sync_dashboard_and_reminders` (sync with a `today` inside the
  old range creates/deletes nothing; `dashboard(now)` lists empty; `due_reminders(now)` returns
  nothing for the archived term's sessions and assessments),
  `t69_past_terms_newest_first_with_counts_attendance_and_gpa` (two past terms with attendance and
  graded courses, one with a course and nothing marked; newest `end_date` first; course count
  includes an archived course; attendance summed over all courses with R13 rounding; GPA per R40
  excluding archived and ungraded courses; `None` attendance and GPA for the unmarked one; the
  active term is not listed).
  Also add to `crates/core/tests/api_surface.rs` under `// M14 (§13.9)`:
  `let _: fn(&mut Planner, i64, NaiveDate) -> Result<()> = Planner::archive_term;` and
  `let _: fn(&Planner) -> Result<Vec<PastTermSummary>> = Planner::past_terms;`.
  Commit: `M14: core tests T67-T69`.
- [x] T002 [P] [US2] [US3] Write app unit tests in `crates/app/src/viewmodel/tests.rs` against the
  frozen signatures in plan.md "App view model": `t66_term_line_active_shows_week_of_total`
  (term "Spring 2026" 2026-01-05..2026-05-01, today 01-21 → text
  `"Spring 2026 · January 5, 2026 – May 1, 2026 · Week 3 of 17"`, link `""`),
  `t66_term_line_before_start` (01-01 → `"Spring 2026 · starts January 5, 2026"`),
  `t66_term_line_after_end` (05-02 → text `"Spring 2026 ended on May 1, 2026."`, link
  `"Archive it in Settings."`, `full()` = `"Spring 2026 ended on May 1, 2026. Archive it in Settings."`),
  `t66_term_line_no_term` (`"No active term. Set one up in Settings."`),
  `r54_term_week_number_monday_to_sunday`, `r67_term_card_lines` (incl. `"Starts January 5, 2026"`
  and `"Ended on May 1, 2026"`), `r67_term_form_open`, `r68_archive_enabled_only_after_end`
  (04-30 false, 05-01 false, 05-02 true), `r68_archive_hint_text`
  (`"Archive is available after May 1, 2026."`, `""` when enabled),
  `r68_archive_confirm_text_exact`, `r70_past_term_row_formats` (`"1 course"`, `"3 courses"`,
  `"Attendance 87.5%"`, `"Attendance —"`, `"GPA 3.40"`, `"GPA —"`, date range),
  `t70_term_save_success_toasts_clears_and_closes` (toast `Some("Term saved.")`, clear and close
  true, `form_error` `""`), `t70_term_save_error_stays_inline` (`Err(Error::Validation("term end
  date 2026-01-04 is before its start date 2026-01-05"))` → no toast, clear and close false,
  `form_error` = `error_text` of it). Build `PastTermSummary` and `Term` values directly.
  Commit: `M14: viewmodel tests T66, T70`.

## Phase 2: Core (builder) — US1

- [ ] T003 [US1] Add `PastTermSummary` (exact §13.9 fields, `#[derive(Debug, Clone, PartialEq)]`,
  doc `/// R70`) to `crates/core/src/model.rs`; add crate-private
  `courses_of_term(conn, term_id, include_archived) -> Result<Vec<Course>>` to
  `crates/core/src/db.rs` (same columns and order as `list_courses`); in
  `crates/core/src/marks.rs` move the `term_gpa` loop into
  `pub(crate) fn term_gpa_for(&self, term_id: i64) -> Result<GpaSummary>` iterating
  `courses_of_term(.., false)`, and make `term_gpa` return the empty projected summary with no
  active term, else `term_gpa_for(active_id)`. T41 (`m9_gpa.rs`) stays green.
  Commit: `M14: term_gpa_for helper and PastTermSummary`.
- [ ] T004 [US1] Create `crates/core/src/terms.rs` (`mod terms;` in `crates/core/src/lib.rs`) with
  `Planner::archive_term(&mut self, id: i64, today: NaiveDate) -> Result<()>` (order: NotFound;
  already inactive → `Ok(())`; `today <= end_date` → `Error::Validation(format!("This term ends on
  {}; archive it after that", end.format("%B %-d, %Y")))`; else `UPDATE term SET is_active = 0`)
  and `Planner::past_terms(&self) -> Result<Vec<PastTermSummary>>` (`WHERE is_active = 0 ORDER BY
  end_date DESC, id DESC`; courses = count of all the term's courses incl. archived; attendance =
  `attendance::pct` over summed attended/absent of all the term's sessions incl. archived courses;
  gpa = `term_gpa_for(id)?.gpa`). Doc comments cite R68, R70, R1, R3, R13, R40. T001 tests pass.
  Commit: `M14: archive_term and past_terms`.

## Phase 3: View model (builder) — US2, US3

- [ ] T005 [US2] [US3] Add to `crates/app/src/viewmodel.rs` exactly the plan.md "App view model"
  items: `term_week_number`, `term_date_range`, `TermLine` (+ `full`), `dashboard_term_line`,
  `TermCard`, `term_card`, `term_week_text`, `term_form_open`, `archive_enabled`,
  `archive_hint_text`, `archive_confirm_text`, `term_archived_text`, `PastTermRow`,
  `past_term_row`, `TermSaveOutcome`, `term_save_outcome`. Reuse `format_date`, `format_pct`,
  `week_start`, `error_text`, `term_saved_text`, `fixed`, `count_text`. Separators `" · "`
  (U+00B7) and `" – "` (U+2013). T002 tests pass. Commit: `M14: term view model`.

## Phase 4: UI and pages (builder) — US2, US3

- [ ] T006 [US3] Add `export struct PastTermRow { name, dates, courses, attendance, gpa }` (all
  `string`) to `crates/app/ui/types.slint` and `pub fn past_term_row(r: &vm::PastTermRow) ->
  PastTermRow` to `crates/app/src/bridge.rs`; add `LinkButton` (text in `Theme.link`, body type,
  `TouchArea`, `accessible-role: button`, Return/Space through `FocusScope`, `FocusRing` on focus;
  `Theme.*` only) to `crates/app/ui/components.slint`. Commit: `M14: past term row and link button`.
- [ ] T007 [US2] Dashboard term line (R66): add `term-line` and `term-link` to `DashboardState` in
  `crates/app/ui/dashboard.slint`; a muted body `Text` bound to `term-line` directly under
  `PageTitle`, followed on the same row by `if term-link != "": LinkButton` whose click sets
  `AppState.current-page = 4`. In `crates/app/src/pages/dashboard.rs` `refresh`, set both from
  `vm::dashboard_term_line(term.as_ref(), app.today())` for the term and no-term cases. The
  existing "Getting started" card stays (D11). Commit: `M14: dashboard term line`.
- [ ] T008 [US3] Settings Term card, save, archive, Past terms (R67-R70) in
  `crates/app/ui/settings.slint` and `crates/app/src/pages/settings.rs`, exactly as plan.md
  "Slint" and "Page wiring": new `SettingsState` properties and callbacks (`term-title`,
  `term-dates`, `term-week`, `term-form-open`, `term-editing`, `archive-enabled`, `archive-hint`,
  `confirm-archive`, `archive-confirm-text`, `past-terms`; `edit-term`, `cancel-term`,
  `archive-term`, `archive-confirm`, `archive-cancel`); read-only card with
  `SecondaryButton "Edit"` and `DangerButton "Archive"` (disabled before the end, hint line
  muted); inline confirm panel styled like the restore panel with `DangerButton "Archive"` and
  `SecondaryButton "Cancel"`; form only when `term-form-open` with `Cancel` when `has-term`;
  `load_forms` no longer fills term fields; `save_term` follows `vm::term_save_outcome` and calls
  `changed_then_toast(app, true, ..)` on success; `archive_confirm` calls
  `archive_term(id, app.today())`, toasts `vm::term_archived_text()` on success, errors to
  `message`; "Past terms" `SectionCard` of read-only `ListRow`s (name body-emphasis, other fields
  muted) with empty state `"No past terms yet. A term shows here after you archive it."`.
  Refresh never clears form text and keeps `term-editing` across the minute timer.
  Commit: `M14: settings term card, archive, past terms`.

## Phase 5: Gate (builder)

- [ ] T009 Run `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo
  test --workspace` and paste the summary; run the theme scan
  `grep -nE '#[0-9a-fA-F]{3,8}\b|[0-9]+(\.[0-9]+)?px' crates/app/ui/*.slint | grep -v '^crates/app/ui/theme.slint'`
  (must print nothing; comments count) and fix any hit by using `Theme.*` (add a token to
  `ui/theme.slint` only if missing). Confirm T68 passes without a core query change (plan.md
  "Existing behaviour confirmed"); if it fails, fix the leaking query and add a line to
  decisions.md. Fill in any test file path above that changed. Commit: `M14: gate` (only if files
  changed).

## Dependencies

- T001, T002 first (parallel). T003 → T004 (needs `term_gpa_for`, `PastTermSummary`).
- T005 needs T003 (`PastTermSummary` type) and T002.
- T006 can start after T005 (needs `vm::PastTermRow`); T007 needs T005, T006 (`LinkButton`);
  T008 needs T004, T005, T006.
- T009 last.

## Parallel examples

- Tester: T001 and T002 together (different crates).
- Builder: T006 alongside T007's slint edits once T005 is in (different files), committed separately.

## Implementation strategy

MVP = US1 (T001, T003, T004): core archive and past terms with tests green. Then US2 (dashboard
line) and US3 (Settings) on top. Each commit leaves the tree compiling once its tests' targets
exist; the tests-first commits (T001, T002) may fail to compile until T003-T005 land, as in
earlier milestones.
