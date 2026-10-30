# Implementation Plan: M14 Terms on the dashboard, save feedback, archive

**Spec directory**: `specs/014-m14-terms-archive` (work happens on the current branch) | **Date**: 2026-10-31
**Spec**: `docs/spec.md` §13.9 (R66-R70, T66-T70). Related: R1, R3, R13, R40, R54 (week numbering
only), §13.13 R88/R89 (toasts). Visuals: `DESIGN.md` "Term line (Dashboard)", "Term card
(Settings)", "Past terms list (Settings)", "Danger button", "Toast notification".
**Decisions**: [decisions.md](decisions.md) (one line each; taken unattended with the recommended option).

## Summary

`docs/spec.md` §13.9 is the plan. No schema change, no migration: an archived term is a term with
`is_active = 0` (terms made inactive by an earlier `save_term` count too). Three parts:

1. **Core (R68, R70; T67-T69).** `Planner::archive_term` and `Planner::past_terms` in a new private
   module `crates/core/src/terms.rs`; `PastTermSummary` in `model.rs` (exported by the existing
   `pub use model::*`). The GPA of a past term reuses the R40 logic through a crate-private helper
   that takes a term id; `term_gpa` becomes a thin wrapper over it (T41 stays green).
2. **View model (R66, R67, R68, R70; T66, T70).** Pure functions in `crates/app/src/viewmodel.rs`
   with unit tests in `crates/app/src/viewmodel/tests.rs`: term week number (own R54-style helper,
   M12 is not on this branch), dashboard term line, Settings term card lines, archive enablement,
   hint and confirm texts, past-term rows, and a term-save outcome reducer that makes T70 testable
   without a headless Slint window.
3. **UI (R66, R67, R69, R70).** Dashboard term line under the page title; Settings Term card
   read-only with Edit / Archive, the form only when there is no active term or after Edit, an
   inline archive confirm panel; a Past terms card of read-only rows. `Theme.*` only.

M12 (`term_week_parity`) is not on this branch. Nothing here depends on it.

## Technical Context

**Language/Version**: Rust (workspace edition as in `Cargo.toml`), Slint UI
**Primary Dependencies**: `rusqlite`, `chrono` (core); `slint` (app). No new dependency.
**Storage**: SQLite; no schema change (§13.9).
**Testing**: `cargo test --workspace`; core integration tests on in-memory SQLite; app unit tests
on pure view-model functions.
**Target Platform**: desktop (macOS, Windows, Linux) as before.
**Project Type**: Cargo workspace: `crates/core` (library), `crates/app` (Slint desktop app).
**Constraints**: core reads no clock (`today` is passed in); `.slint` holds bindings only;
`Theme.*` only, no raw hex or px outside `ui/theme.slint` (comments included); square corners;
40px list rows (`Theme.row-height`).
**Scale/Scope**: a handful of terms per user; `past_terms` does one query per past term.

No NEEDS CLARIFICATION remains; every interpretation is in decisions.md.

## Constitution Check

| Principle | Status |
|---|---|
| I. Spec is source of truth | Pass. Only §13.9 API added; no rule, schema, or API change. Wording choices not fixed by the spec are listed in decisions.md. |
| II. Test-first, traceable | Pass. T66-T70 and formatting tests are written first (tasks T001-T002); names follow `t67_archive_before_end_is_rejected`; doc comments cite R-ids. |
| III. Pure, clock-free core | Pass. `archive_term` takes `today`; every UI string is built in `viewmodel.rs`; `.slint` binds properties and calls callbacks only. |
| IV. Append-only migrations | Pass. No migration. |
| V. Gate before done | Pass. Final task runs the gate plus the theme grep. |

Re-check after design: still pass.

## Existing behaviour confirmed (T68)

An archived term is `is_active = 0`, and the read paths below already scope to the active term,
so T68 needs no core change, only the test:

- `sync_sessions` → `schedule::sync_in` returns an empty `SyncReport` when `active_term_id` is `None`
  (`crates/core/src/schedule.rs` ~l.173).
- `dashboard` returns an empty `Dashboard` when `active_term_id` is `None` (`dashboard.rs` ~l.23).
- `due_reminders`: both candidate queries filter `t.is_active = 1` (`reminders.rs` ~l.104, ~l.146).
- `list_courses`, `list_slots`, `list_assessments`, `all_summaries`, `sessions_between` filter the
  active term (`db.rs`, `schedule.rs` `ACTIVE_SCOPE`, `attendance.rs`).

Task T009 (gate) re-runs the T68 test as the check; if it fails, the builder fixes the query that
leaks and records it.

## Frozen interfaces

Tester and builder both code against these names and signatures. Changing one needs the router.

### Core (`crates/core`)

`crates/core/src/model.rs` (exactly as §13.9):

```rust
/// R70: one inactive term with its course count, overall attendance, and GPA.
#[derive(Debug, Clone, PartialEq)]
pub struct PastTermSummary {
    pub term: Term,
    pub courses: u32,
    pub attendance_pct: Option<f64>,
    pub gpa: Option<f64>,
}
```

`crates/core/src/terms.rs` (new private module, `mod terms;` in `lib.rs`):

```rust
impl Planner {
    /// R68: archives (sets `is_active = 0`) the term `id` when `today > end_date`.
    pub fn archive_term(&mut self, id: i64, today: NaiveDate) -> Result<()>;
    /// R70: every inactive term, newest `end_date` first (tie: `id` descending).
    pub fn past_terms(&self) -> Result<Vec<PastTermSummary>>;
}
```

`archive_term` behaviour, in this order:

1. Term `id` does not exist: `Error::NotFound { entity: "term", id }` (same as `save_term`).
2. Term exists but is already inactive: `Ok(())`, nothing changes (idempotent no-op; D3).
3. `today <= end_date`: `Error::Validation` with the exact text
   `"This term ends on {end}; archive it after that"`, `{end}` formatted `"%B %-d, %Y"`
   (e.g. `This term ends on May 1, 2026; archive it after that`; D2). The term stays active.
4. Otherwise `UPDATE term SET is_active = 0 WHERE id = ?1`. Courses, slots, sessions,
   assessments, holidays are untouched (R68 "All data stays").

`past_terms` behaviour, per inactive term:

- `term`: `load_term` (holidays included).
- `courses`: count of every course of the term, archived courses included (R3 keeps history).
- `attendance_pct`: `attendance::pct(attended, absent)` over the summed counts of every session of
  every course of the term, archived courses included (R13 rounding; `None` when no session is
  attended or absent).
- `gpa`: `term_gpa_for(term.id)?.gpa` (R40 applied to that term: non-archived graded courses with
  a projected pct, credits-weighted, `round2`; `None` when no course qualifies).
- Order: `ORDER BY end_date DESC, id DESC`.

Crate-private helpers:

```rust
// crates/core/src/db.rs
/// R1, R3: courses of term `term_id`, ordered by code; archived ones only when asked.
pub(crate) fn courses_of_term(conn: &Connection, term_id: i64, include_archived: bool) -> Result<Vec<Course>>;

// crates/core/src/marks.rs (impl Planner)
/// R40: GPA over the non-archived graded courses of term `term_id`.
pub(crate) fn term_gpa_for(&self, term_id: i64) -> Result<GpaSummary>;
```

`term_gpa` keeps its signature and doc: no active term returns the empty projected summary, else
`self.term_gpa_for(active_id)`. The loop body moves unchanged into `term_gpa_for`, iterating
`courses_of_term(&self.conn, term_id, false)` instead of `list_courses(false)`.

`crates/core/tests/api_surface.rs` gains (under a `// M14 (§13.9)` comment):

```rust
let _: fn(&mut Planner, i64, NaiveDate) -> Result<()> = Planner::archive_term;
let _: fn(&Planner) -> Result<Vec<PastTermSummary>> = Planner::past_terms;
```

### App view model (`crates/app/src/viewmodel.rs`)

All pure; `today` is passed in. Dates use the existing `format_date` (`"January 5, 2026"`).
Separators: `" · "` is space, U+00B7, space; `" – "` is space, U+2013, space.

```rust
/// R54 numbering (own helper; M12 is not on this branch): weeks run Monday-Sunday, week 1 is the
/// week containing `start`. `(week_start(date) - week_start(start)).num_days().div_euclid(7) + 1`;
/// 0 or negative before the term's first week.
pub fn term_week_number(start: NaiveDate, date: NaiveDate) -> i64;

/// R66: `"January 5, 2026 – May 1, 2026"` (full dates both sides; D5).
pub fn term_date_range(start: NaiveDate, end: NaiveDate) -> String;

/// R66: the dashboard term line, split so the trailing sentence can be a link (DESIGN.md).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TermLine {
    /// Plain text part.
    pub text: String,
    /// `"Archive it in Settings."` after the end date, else `""`.
    pub link: String,
}
impl TermLine {
    /// `text`, then a space and `link` when `link` is not empty (the R66 sentence).
    pub fn full(&self) -> String;
}

/// R66: `term` is the active term (None = no active term).
/// - `today < start`: text `"{name} · starts {start}"`, no link.
/// - `start <= today <= end`: text `"{name} · {start} – {end} · Week {n} of {N}"`,
///   n = term_week_number(start, today), N = term_week_number(start, end), no link.
/// - `today > end`: text `"{name} ended on {end}."`, link `"Archive it in Settings."`.
/// - None: text `"No active term. Set one up in Settings."`, no link.
pub fn dashboard_term_line(term: Option<&Term>, today: NaiveDate) -> TermLine;

/// R67: read-only Term card lines of the active term.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TermCard {
    pub title: String, // term name
    pub dates: String, // term_date_range(start, end)
    pub week: String,  // term_week_text(term, today)
}
pub fn term_card(term: &Term, today: NaiveDate) -> TermCard;

/// R67 week line: `"Week {n} of {N}"` within the term, `"Starts {start}"` before it,
/// `"Ended on {end}"` after it (D6).
pub fn term_week_text(term: &Term, today: NaiveDate) -> String;

/// R67, R69: the term form shows only when there is no active term or after Edit.
pub fn term_form_open(has_term: bool, editing: bool) -> bool; // !has_term || editing

/// R68: Archive is enabled only when `today > end`.
pub fn archive_enabled(end: NaiveDate, today: NaiveDate) -> bool;

/// R68: muted line under a disabled Archive: `"Archive is available after {end}."`;
/// `""` when enabled (D7).
pub fn archive_hint_text(end: NaiveDate, today: NaiveDate) -> String;

/// R68 exact: `"Archive {name}? Its courses, attendance, assessments, and marks are kept and
/// shown under Past terms. You can then set up the next term."`
pub fn archive_confirm_text(name: &str) -> String;

/// R88, R89: toast after a successful archive: `"Term archived."` (D8).
pub fn term_archived_text() -> String;

/// R70: one read-only Past terms row (DESIGN.md "Past terms list").
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PastTermRow {
    pub name: String,       // term name
    pub dates: String,      // term_date_range(start, end)
    pub courses: String,    // "1 course" / "3 courses" / "0 courses"
    pub attendance: String, // "Attendance 87.5%" / "Attendance —" (format_pct)
    pub gpa: String,        // "GPA 3.40" / "GPA —" (two decimals)
}
pub fn past_term_row(s: &PastTermSummary) -> PastTermRow;

/// R67, R88, T70: what the Settings page does after `save_term` returned `saved`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TermSaveOutcome {
    pub toast: Option<String>, // Some(term_saved_text()) on success, None on error
    pub clear_fields: bool,    // true on success
    pub close_form: bool,      // true on success
    pub form_error: String,    // "" on success, error_text(e) on error (inline, fields kept)
}
pub fn term_save_outcome(saved: &planner_core::Result<Term>) -> TermSaveOutcome;
```

Existing functions reused unchanged: `format_date`, `format_pct`, `week_start`, `error_text`,
`term_saved_text`, `parse_term_form`, the private `fixed` and `count_text`.

### Slint (`crates/app/ui`)

`types.slint`:

```slint
// R70: one read-only past term row.
export struct PastTermRow { name: string, dates: string, courses: string, attendance: string, gpa: string }
```

`dashboard.slint` `DashboardState` additions:

```slint
// R66: term line under the page title, and its trailing link ("" = none).
in property <string> term-line;
in property <string> term-link;
```

The term line is a muted body `Text` directly under `PageTitle`; when `term-link != ""` a
`LinkButton` follows on the same row and its click sets `AppState.current-page = 4` (Settings),
the same binding the nav items use.

`components.slint`: new `LinkButton` (text in `Theme.link`, body type, `TouchArea`, accessible
role button, Return/Space via `FocusScope`, `FocusRing` on focus), all sizes from `Theme.*`.

`settings.slint` `SettingsState` additions:

```slint
// R67: read-only Term card of the active term.
in property <string> term-title;
in property <string> term-dates;
in property <string> term-week;
// R67, R69: the form shows only with no active term or after Edit (Rust computes it).
in property <bool> term-form-open;
in property <bool> term-editing;   // set by Rust; Rust reads it back on refresh
// R68: Archive enablement, the muted reason line, and the inline confirm panel.
in property <bool> archive-enabled;
in property <string> archive-hint;
in property <bool> confirm-archive;
in property <string> archive-confirm-text;
// R70: Past terms rows.
in property <[PastTermRow]> past-terms;

callback edit-term();
callback cancel-term();
callback archive-term();     // opens the confirm panel
callback archive-confirm();
callback archive-cancel();
```

Term card layout: with `has-term && !term-form-open`: title (body-emphasis), dates and week line
(muted body), a row with `SecondaryButton "Edit"` and `DangerButton "Archive"`
(`enabled: archive-enabled`), and under it the `archive-hint` line (muted) when not empty. With
`confirm-archive`: an inline panel styled like the restore confirm (surface fill, divider-width
`Theme.border` edge, square, `Theme.space-lg` padding) with `archive-confirm-text`, a
`DangerButton "Archive"` (`archive-confirm`) and `SecondaryButton "Cancel"` (`archive-cancel`).
With `term-form-open`: the existing fields and "Save term", plus `SecondaryButton "Cancel"` only
when `has-term`. The "No term saved yet…" empty state stays for `!has-term`.

Past terms card (after the Term card): `SectionCard { title: "Past terms" }`; rows are `ListRow`
(40px) with name (body-emphasis), dates, courses, attendance, gpa (muted); no buttons. Empty state
text: `"No past terms yet. A term shows here after you archive it."` (D9).

`bridge.rs`: `pub fn past_term_row(r: &vm::PastTermRow) -> PastTermRow`.

### Page wiring (`crates/app/src`)

`pages/dashboard.rs` `refresh`: before the `has_term` match, set `term-line` / `term-link` from
`vm::dashboard_term_line(term.as_ref(), app.today())` (shown in both the term and no-term cases).

`pages/settings.rs`:

- `refresh(app)`: from `active_term()` and `app.today()`: `has-term`; card lines via
  `vm::term_card`; `archive-enabled` / `archive-hint`; `term-form-open =
  vm::term_form_open(has_term, editing)` where `editing = has_term && get_term_editing()` (no active
  term resets `term-editing` to false and `confirm-archive` to false); `past-terms` from
  `past_terms()` via `vm::past_term_row` and `bridge::past_term_row`. Errors go to `message`.
  Refresh never touches form text (the minute timer must not wipe typing).
- `load_forms`: stops filling the term fields from the active term (R67: fields are filled only by
  Edit, cleared after a save).
- `edit_term`: pre-fills name/start/end from the active term (`format_date_input`), clears field
  errors, sets `term-editing = true`, `confirm-archive = false`, then refreshes the page.
- `cancel_term`: `term-editing = false`, clears fields and their errors, refreshes. No data change.
- `save_term`: parse as today (field errors inline, form stays). Then
  `let outcome = vm::term_save_outcome(&saved)`; on `close_form` set `term-editing = false`; on
  `clear_fields` clear name/start/end and field errors; set `term-form-error` to
  `outcome.form_error`; on `Some(text)` call `changed_then_toast(app, true, &text)` (R6 sync,
  every page refreshes, toast only when no error was shown, R89).
- `archive_term` (button): sets `confirm-archive = true` and `archive-confirm-text =
  vm::archive_confirm_text(&term.name)`.
- `archive_confirm`: `planner_mut().archive_term(term.id, app.today())`; Ok: `confirm-archive =
  false`, `term-editing = false`, clear term fields, `changed_then_toast(app, true,
  &vm::term_archived_text())`; Err: `confirm-archive = false`, `message = vm::error_text(&e)`.
- `archive_cancel`: `confirm-archive = false`.

No change to `controller.rs` (`App::toast`, `App::changed`, `refresh_all` already cover R67
"every page refreshes" and R88).

## Project Structure

```text
specs/014-m14-terms-archive/
├── plan.md        # this file (includes the analysis report at the end)
├── tasks.md
└── decisions.md

crates/core/src/
├── lib.rs         # + mod terms;
├── model.rs       # + PastTermSummary
├── terms.rs       # new: archive_term, past_terms (R68, R70)
├── db.rs          # + courses_of_term (crate-private)
└── marks.rs       # term_gpa -> term_gpa_for(term_id) helper (R40)
crates/core/tests/
├── m14_terms_archive.rs   # new: T67, T68, T69
└── api_surface.rs         # + M14 bindings
crates/app/src/
├── viewmodel.rs           # + R66-R70 functions and T70 reducer
├── viewmodel/tests.rs     # + T66, T70, formatting tests
├── bridge.rs              # + past_term_row
└── pages/{dashboard,settings}.rs
crates/app/ui/
├── types.slint            # + PastTermRow
├── components.slint       # + LinkButton
├── dashboard.slint        # term line
└── settings.slint         # Term card, confirm panel, Past terms
```

No separate research.md, data-model.md, contracts/ or quickstart.md: the frozen interfaces above
are the contract, and the decisions file replaces research (router instruction: write only
plan.md, tasks.md, decisions.md).

## Test plan (written first)

Core, `crates/core/tests/m14_terms_archive.rs` (in-memory SQLite, `mod common;`):

- `t67_archive_before_or_on_end_is_rejected`: term 2026-01-05..2026-05-01 with a course and synced
  sessions; `archive_term(id, 2026-04-30)` and `(id, 2026-05-01)` give `Error::Validation` whose
  text is `"This term ends on May 1, 2026; archive it after that"`; `active_term()` still that term.
- `t67_archive_after_end_keeps_data`: `archive_term(id, 2026-05-02)` is Ok; `active_term()` is
  None; the course row and its session count are unchanged (read via `past_terms` /
  direct counts in the test with the same planner, e.g. `attendance_summary(course_id)` totals).
- `t67_archive_unknown_term_is_not_found` and `t67_archive_inactive_term_is_noop` (D3).
- `t68_archived_term_is_ignored_by_sync_dashboard_and_reminders`: archive after end, then call
  `sync_sessions`, `dashboard`, `due_reminders` with a `today`/`now` inside the old term range (core
  does not tie them to the archive date): sync creates and deletes nothing; dashboard lists are
  empty; no reminder for the archived term's sessions or assessments.
- `t69_past_terms_newest_first_with_counts_attendance_and_gpa`: two past terms with attendance and
  graded courses (built while each is active, then archived after its end or made inactive by the
  next `save_term`) and one past term with a course but nothing marked; order newest `end_date`
  first; `courses` (archived course counted), `attendance_pct` (summed counts, R13), `gpa` (R40,
  archived and ungraded courses excluded); `None` for the one with nothing marked. Active term is
  never listed.
- `term_gpa` after the `term_gpa_for` refactor: the existing T41 tests in `m9_gpa.rs` cover it; no
  new test.

App, `crates/app/src/viewmodel/tests.rs`:

- `t66_term_line_active_shows_week_of_total` (01-21: `"Spring 2026 · January 5, 2026 – May 1, 2026 · Week 3 of 17"`),
  `t66_term_line_before_start` (01-01: `"Spring 2026 · starts January 5, 2026"`),
  `t66_term_line_after_end` (05-02: `full()` = `"Spring 2026 ended on May 1, 2026. Archive it in Settings."`, link set),
  `t66_term_line_no_term` (`"No active term. Set one up in Settings."`).
- `r54_term_week_number_monday_to_sunday` (start on a Wednesday; Sunday same week = 1, next Monday = 2;
  start date itself = 1; last day = N).
- `r67_term_card_lines`, `r67_term_form_open`, `r68_archive_enabled_only_after_end`,
  `r68_archive_hint_text`, `r68_archive_confirm_text_exact`, `r70_past_term_row_formats`
  (singular course, "—" for None, two-decimal GPA).
- `t70_term_save_success_toasts_clears_and_closes`, `t70_term_save_error_stays_inline`
  (Err(Validation "term end date 2026-01-04 is before its start date 2026-01-05") → no toast,
  fields kept, form open, `form_error` = `error_text`).

## Complexity Tracking

None. No constitution violation.

## Analysis report (speckit-analyze, read-only)

Artifacts compared: `docs/spec.md` §13.9 (+ R1, R3, R13, R40, R54, R88, R89), this plan, tasks.md,
constitution, DESIGN.md components.

| ID | Category | Severity | Location | Summary | Resolution |
|---|---|---|---|---|---|
| A1 | Inconsistency | LOW | DESIGN.md "Term line" vs R66 | DESIGN example compresses the range ("January 5 – May 1, 2026"); R66 says dates as "January 5, 2026" | Spec wins for text: full dates both sides (D5). Flag to human for a later DESIGN.md wording sync. |
| A2 | Underspecification | MEDIUM | R68, R88/R89 | No archive confirmation text given; R88/R89 say completed actions confirm by toast | Toast "Term archived." (D8). |
| A3 | Underspecification | LOW | R68 | Archiving an already inactive term not specified | Idempotent `Ok(())` (D3). |
| A4 | Underspecification | LOW | R68 | Date format inside the core message | Long form "May 1, 2026" (D2). |
| A5 | Underspecification | LOW | R67, DESIGN "Term card" | Week line outside the term dates and the disabled-Archive reason wording | D6, D7. |
| A6 | Coverage | LOW | T70 (app) | No headless Slint tests | Covered by the pure `term_save_outcome` reducer plus page wiring that follows it (D10). |
| A7 | Duplication | LOW | dashboard.slint | Existing "No term yet" card plus the new R66 no-term line say the same thing | Keep both: R66 adds the line; removing the card is not in the spec (D11). |
| A8 | Coverage | — | T68 | Archived-term exclusion already holds in core | Test only; confirmed by code reading (section above). |

| A9 | Inconsistency | LOW | tasks.md T006 | T006 was marked [P] but needs `vm::PastTermRow` from T005 | Remediated: [P] removed; dependency stated in tasks.md "Dependencies". |
| A10 | Constitution | LOW | dashboard.slint link click | `AppState.current-page = 4` in a `.slint` click handler | Same pattern as the nav items; a binding, not logic. No violation. |

Coverage summary:

| Requirement | Has task? | Task IDs | Notes |
|---|---|---|---|
| R66 dashboard term line | yes | T002, T005, T006, T007 | T66 |
| R67 term card, Edit, save feedback, Cancel | yes | T002, T005, T008 | T70 via reducer |
| R68 archive_term, disabled Archive, confirm | yes | T001, T002, T004, T005, T008 | T67 |
| R69 no new term while one is active | yes | T008 | form hidden while active; core R1 unchanged |
| R70 past terms | yes | T001, T002, T003, T004, T005, T006, T008 | T69 |
| T68 archived term ignored | yes | T001, T009 | no core change expected |
| API surface (§13.9) | yes | T001 | `api_surface.rs` |

Metrics: 5 rules + 5 tests; 9 tasks; coverage 100%; ambiguities 4 (A2-A5, all resolved in
decisions.md); duplications 1 (A7, kept on purpose); critical 0; high 0. Unmapped tasks: none.
Next action: proceed to implementation (tester T001-T002, then builder T003-T009).
