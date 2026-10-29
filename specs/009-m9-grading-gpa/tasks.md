---

description: "Task list for M9 credits, rule sets, marks, GPA"
---

# Tasks: M9 Credits, Rule Sets, Marks, GPA

**Input**: Design documents from `specs/009-m9-grading-gpa/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [quickstart.md](quickstart.md), [contracts/core-api.md](contracts/core-api.md), [contracts/core-backup.md](contracts/core-backup.md), [contracts/ui-grading.md](contracts/ui-grading.md)

**Tests**: Required. T36–T43 come from `docs/spec.md` §13.5; rule tests (`r33_…`–`r41_…`) and
view-model tests come from the contracts. Each tester task runs before its builder task
(constitution II). Tests use in-memory SQLite or a `tempfile::TempDir`, never the real user DB.

**Start condition**: M7 is committed. Core track (T001–T017) can start now, in parallel with the
M8 app work in another worktree. App track (T018–T026) starts only after **M8 is merged** and the
listed core tasks are done.

**Pending router confirmation**: OQ-1 and OQ-2 (research.md). Tasks that use the recommended
option are marked `(OQ-1 A)` / `(OQ-2 A)`; if the router picks another option, only those tasks
change.

## Format: `[ID] [P?] [Story] [crate] (owner) Description`

- **[P]**: can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: user story from spec.md (US1–US6)
- **[core]** / **[app]**: the crate the task belongs to. `[core]` tasks never need app work except
  T002, a mechanical compile fix of four struct literals in crates/app.
- **(tester)** / **(builder)** / **(human)**: who does the task.

## Path Conventions

New: `crates/core/src/migrations/002_grading.sql`, `crates/core/src/grading_seed.sql`,
`crates/core/src/grading.rs`, `crates/core/src/marks.rs`, `crates/core/src/marks/tests.rs`,
`crates/core/tests/{m9_migration.rs, m9_rules.rs, m9_marks.rs, m9_gpa.rs, m9_backup.rs}`,
`crates/app/ui/rules.slint`, `crates/app/src/pages/rules.rs`. Edited: `crates/core/src/{lib.rs,
model.rs, db.rs, backup.rs}`, `crates/core/tests/{common/mod.rs, api_surface.rs, migrations.rs,
m3_backup_export.rs, m3_backup_restore.rs}` and every core test with a `CourseInput` or
`AssessmentInput` literal, `crates/app/src/{viewmodel.rs, viewmodel/tests.rs, main.rs,
controller.rs, pages/mod.rs, pages/courses.rs, pages/assessments.rs}`, `crates/app/ui/{app-window.slint,
types.slint, courses.slint, assessments.slint}`. Not edited: `crates/core/src/migrations/001_init.sql`,
`docs/spec.md`, `DESIGN.md`, `crates/app/ui/theme.slint` (unless a token is missing; then stop and
ask), `.claude/state/` (router only).

---

## Phase 1: Setup (core track)

- [x] T001 [core] (builder) Types and stubs per data-model.md "Rust types": in crates/core/src/model.rs add `ComponentKind`, `AttendanceBand`, `RuleComponent`, `RuleSet`, `RuleComponentInput`, `RuleSetInput`, `GradeRow`, `ComponentMarks`, `Needed`, `NeededRow`, `Assumption`, `FinalOutlook`, `CourseMarks`, `GpaStatus`, `GpaSummary` (derives like existing types; doc comments cite R33–R40); add `credits: Option<f64>, rule_set_id: Option<i64>` to `CourseInput` and `Course`, `component_id: Option<i64>, score: Option<f64>` to `AssessmentInput` and `Assessment` (doc `/// R33: …`, `/// R36: …`); row mapping in crates/core/src/db.rs sets the new output fields to `None` for now (columns arrive in T005). Create crates/core/src/grading.rs with `impl Planner { list_rule_sets, save_rule_set, delete_rule_set, grade_scale, save_grade_scale }` and crates/core/src/marks.rs with `impl Planner { course_marks, term_gpa }`, exact §13.3 signatures, bodies `todo!("M9: R..")`. In marks.rs also stub the pure helpers `pub(crate) fn band_credit(bands: &[AttendanceBand], pct: f64) -> f64`, `pub(crate) fn items_earned(scores: &[f64], best_count: u32, item_max: f64, marks: f64) -> (f64, f64)` (earned, assessed fraction), `pub(crate) fn round1(x: f64) -> f64`, `pub(crate) fn round2(x: f64) -> f64`, `pub(crate) fn ceil1(x: f64) -> f64`, `pub(crate) fn letter_for(scale: &[GradeRow], pct: f64) -> Option<String>`, plus `#[cfg(test)] mod tests;` with an empty crates/core/src/marks/tests.rs. Add `mod grading; mod marks;` to crates/core/src/lib.rs. Add `credits: None, rule_set_id: None` / `component_id: None, score: None` to every `CourseInput`/`AssessmentInput` literal in crates/core/tests (common/mod.rs, terms_courses.rs, m2_attendance.rs, m3_csv.rs, m3_backup_export.rs, m3_backup_restore.rs, validation.rs, assessments.rs, m5_reminders.rs). `#[allow(dead_code)]` only on unused stub helpers. All existing core tests green.
- [x] T002 [core] (builder) Mechanical app compile fix, nothing else: in crates/app/src/viewmodel.rs `parse_course_form` and `parse_assessment_form` add `credits: None, rule_set_id: None` / `component_id: None, score: None`; same in the two literals in crates/app/src/viewmodel/tests.rs. Commit together with T001 so the workspace builds. (If M8 has already changed these functions, apply the same four additions on top.) Needs T001
- [x] T003 [P] [core] (tester) In crates/core/tests/api_surface.rs add a `// M9 (§13.3)` block binding `list_rule_sets: fn(&Planner) -> Result<Vec<RuleSet>>`, `save_rule_set: fn(&mut Planner, RuleSetInput) -> Result<RuleSet>`, `delete_rule_set: fn(&mut Planner, i64) -> Result<()>`, `grade_scale: fn(&Planner) -> Result<Vec<GradeRow>>`, `save_grade_scale: fn(&mut Planner, &[GradeRow]) -> Result<()>`, `course_marks: fn(&Planner, i64) -> Result<CourseMarks>`, `term_gpa: fn(&Planner) -> Result<GpaSummary>`. Compiles and passes against the stubs. Needs T001

---

## Phase 2: Foundational - migration 002 and the new columns (core track)

- [x] T004 [core] (tester) Version pins (research D14) and migration tests. In crates/core/tests/migrations.rs replace `assert_eq!(SCHEMA_VERSION, 1)` with `2` and fake the "newer" DB with `SCHEMA_VERSION + 1` instead of `2`. In crates/core/tests/m3_backup_export.rs change only `doc["schema_version"] == json!(1)` to `json!(2)` (key lists are T015). In crates/core/tests/m3_backup_restore.rs make T30's "newer" cases `SCHEMA_VERSION + 1` and 99, and `r22_restore_accepts_equal_and_lower_schema_version` loop `1..=SCHEMA_VERSION`. Create crates/core/tests/m9_migration.rs: `m9_migration_002_upgrades_v1_file_db` (in a `TempDir`, build a v1 file DB with rusqlite: `PRAGMA foreign_keys=ON`, execute `include_str!("../src/migrations/001_init.sql")`, `PRAGMA user_version=1`, insert a term, a course, an assessment; then `Planner::open` → `user_version` 2, the course and assessment are intact with `credits`, `rule_set_id`, `component_id`, `score` all `None`, `backups/pre-migrate-v1-to-v2.sqlite` exists, and `rule_set` has 2 rows, `rule_component` 7, `attendance_band` 12, `grade_scale` 10); `r33_course_credits_and_rule_set_stored` (create with `credits: Some(1.5), rule_set_id: Some(1)`, read back via `list_courses`, update to `None`/`None`, read back); `r33_credits_must_be_positive` (`Some(0.0)` and `Some(-1.0)` → `Error::Validation`); `r33_unknown_rule_set_rejected` (`rule_set_id: Some(99)` → `Error::Validation`); `r36_score_stored_and_negative_rejected` (score `Some(12.0)` with `component_id: None` round-trips via `list_assessments`; `Some(-1.0)` → `Error::Validation`). Doc comments cite R25, R26, R33, R36, §13.3. Run them and record which fail. Needs T001
- [x] T005 [core] (builder) Create crates/core/src/migrations/002_grading.sql = the §13.3 "Migration 002" SQL block verbatim. Create crates/core/src/grading_seed.sql with the `INSERT`s of data-model.md "Seeds" (fixed ids: rule_set 1 Theory, 2 Lab; components 1–7 with positions, kinds, weights, `item_count`/`best_count`/`item_max`/`is_final` exactly as the table; bands `(90,100),(85,90),(80,80),(75,70),(70,60),(0,0)` for components 1 and 5; grade scale `A+ 80 4.00, A 75 3.75, A- 70 3.50, B+ 65 3.25, B 60 3.00, B- 55 2.75, C+ 50 2.50, C 45 2.25, D 40 2.00, F 0 0.00`). In crates/core/src/db.rs extend `MIGRATIONS` with `concat!(include_str!("migrations/002_grading.sql"), include_str!("grading_seed.sql"))` (research D2; `SCHEMA_VERSION` becomes 2 by itself) and expose the seed text as `pub(crate) const GRADING_SEED: &str = include_str!("grading_seed.sql");` for T016. Store and read `course.credits`, `course.rule_set_id`, `assessment.component_id`, `assessment.score` in create/update/list. Validation (research D4, `Error::Validation`): `credits` must be `> 0` when present; `rule_set_id` must name an existing rule set; `score` must be `>= 0` when present. Doc comments `/// R33: …`, `/// R36: …`. T003, T004 and all earlier core tests pass. Needs T004

**Checkpoint**: schema v2 with seeds; existing behavior unchanged; core gate green.

---

## Phase 3: User Story 1 - Grading rules exist and are validated (P1, core track)

**Goal**: seeded rule sets and grade scale; R34 and R39 validation; component update in place.

**Independent Test**: `cargo test -p planner-core --test m9_rules` passes.

- [x] T006 [US1] [core] (tester) Create crates/core/tests/m9_rules.rs per contracts/core-api.md "R34 checks" and "R39 checks". Seeds: `r34_seeded_rule_sets_match_spec` (both sets, every component field and position, bands sorted by min_pct descending) and `r39_seeded_grade_scale_is_ugc` (10 rows by min_pct descending). T36: `t36_rule_set_validation_errors` building inputs from the seeded Theory set: weights summing to 95; two `is_final` components; `best_count: Some(5), item_count: Some(4)`; attendance bands without a 0 row → each `Error::Validation`, and `list_rule_sets` unchanged after each. More R34: `r34_weights_within_tolerance_accepted` (sum 100.005 saves), `r34_empty_or_duplicate_name_rejected`, `r34_two_attendance_components_rejected`, `r34_final_must_be_items`, `r34_items_need_counts`, `r34_attendance_needs_bands`, `r34_update_in_place_keeps_component_ids` (rename a component and change its weight with its id: same id afterwards), `r34_new_rule_set_created` (a set "Seminar" with one items final component of 100% saves and lists), `r34_remove_component_with_scored_assessment_rejected`, `r34_remove_component_unlinks_unscored_assessments` (research D3; after save the assessment has `component_id: None`). T43 part 1: `t43_delete_rule_set_used_by_course_rejected` (course with `rule_set_id: Some(1)` → `Error::Validation`; set still listed), `r34_delete_unused_rule_set` (and its components and bands are gone), `r34_delete_unknown_rule_set_not_found`. R39: `r39_grade_scale_validation_errors` (duplicate letter, duplicate min_pct, no 0 row, points rising as min falls → each `Error::Validation`, scale unchanged), `r39_save_grade_scale_replaces_rows`. Linking assessments uses a course with `credits: Some(3.0), rule_set_id: Some(1)` (OQ-2 A). Doc comments cite R34, R39, T36, T43. Run; record failures. Needs T005
- [ ] T007 [US1] [core] (builder) Implement crates/core/src/grading.rs per contracts/core-api.md and research D3–D6: `list_rule_sets` (sets by id, components by position, bands by min_pct descending); `save_rule_set` (all R34 checks before any write: name non-empty after trim and unique; weights each in (0, 100] and sum within 100 ± 0.01; component names non-empty and unique; at most one attendance; exactly one `is_final` and it is `items`; items need `item_count` and `best_count` with `1 ≤ best_count ≤ item_count` and `item_max` None or > 0; attendance needs ≥ 1 band, a band with `min_pct` 0, band values in 0–100, unique band mins; input ids must belong to this set; removed components with a linked scored assessment → error; then in one transaction update by id, insert new, unlink unscored assessments of removed components, delete removed, replace bands, store NULL item fields for attendance and no bands for items, `position` = index); `delete_rule_set` (NotFound; Validation if any course row uses it; else unlink assessments of its components and delete); `grade_scale` (by min_pct descending); `save_grade_scale` (R39 checks: letters non-empty and unique, min_pct unique and 0–100, a 0 row, points ≥ 0 and non-increasing by min_pct descending; replace all rows in one transaction). Doc comments `/// R34: …`, `/// R39: …`. T006 passes; all core tests pass. Needs T006

---

## Phase 4: User Story 2 - Course marks and projected grade (P1, core track)

**Goal**: R33, R35–R37 and the R36 score check.

**Independent Test**: `cargo test -p planner-core --test m9_marks` (T37, T38, T40, T43 score) and `cargo test -p planner-core marks::` pass.

- [x] T008 [P] [US2] [core] (tester) Unit tests in crates/core/src/marks/tests.rs for the pure helpers (research D7): `band_credit` with the default bands at 92.0 → 100, 90.0 → 100, 89.9 → 90, 85.0 → 90, 84.9 → 80, 0.0 → 0; `items_earned(&[12.,11.,13.,14.], 3, 15., 45.)` → (39.0, 1.0); `items_earned(&[12.,11.], 3, 15., 45.)` → (23.0, 2/3); `items_earned(&[], 1, 75., 75.)` → (0.0, 0.0); five scores for best 3 use the best three; `round1(79.94)` → 79.9, `round1(79.95)` → 80.0; `round2(3.6666666666666665)` → 3.67; `ceil1(111.00000000000001)` → 111.0, `ceil1(111.01)` → 111.1, `ceil1(96.0)` → 96.0, `ceil1(-5.0)` → -5.0; `letter_for(seeded scale, 79.9)` → "A", `80.0` → "A+", `0.0` → "F". Doc comments cite R35, R36, R37, R38, R40, T40. Needs T001
- [x] T009 [P] [US2] [core] (tester) Fixture helpers in crates/core/tests/common/mod.rs: `graded_course(p: &mut Planner, code: &str, credits: f64, rule_set_name: &str) -> Course` (looks the set up by name in `list_rule_sets`), `component_id(p: &Planner, rule_set_name: &str, component_name: &str) -> i64`, `scored(p: &mut Planner, course_id: i64, component_id: i64, title: &str, score: f64) -> Assessment`, and use the existing `add_marked_makeups` for attendance (87% = 20 attended + 3 absent → R13 87.0; 92% = 23 attended + 2 absent). Create crates/core/tests/m9_marks.rs: `t37_quizzes_best_three_of_four` (3-credit Theory, quiz scores 12, 11, 13, 14 → Quizzes `earned` 39.0, `marks` 45.0, `assessed_fraction` 1.0), `t38_attendance_band_87_pct` (Attendance `earned` 27.0 of `marks` 30.0), `t40_projected_79_9_is_a_and_80_0_is_a_plus` (3-credit Theory with only a mid score: 59.925 → `projected_pct` 79.9, letter "A"; 60.0 → 80.0, "A+"), `r33_ungraded_course_has_no_marks` (credits only, rule set only, neither → `graded: false`, no components, no letter), `r33_component_marks_scale_with_credits` (1.5-credit Theory: 15, 22.5, 37.5, 75), `r35_attendance_without_marked_sessions_not_assessed` (fraction 0, earned 0), `r36_fewer_scores_than_best_count` (two quizzes 12, 11 → earned 23.0, fraction 2/3), `r37_nothing_assessed_has_no_projection` (`projected_pct` None, letter None), `r37_secured_and_assessed_sums` (T39 inputs: secured 129.0, assessed 150.0, projected 86.0, "A+"), `t43_score_above_item_max_rejected` (quiz score 16 with item_max 15 → `Error::Validation`; mid score 76 on a 3-credit course where the mid's marks are 75 → `Error::Validation`; on create and on update), `r36_component_must_belong_to_course_rule_set` (OQ-2 A: linking a Lab component to a Theory course, or any component to an ungraded course → `Error::Validation`), `r37_course_marks_unknown_course_not_found`. Doc comments cite R33, R35–R37, T37, T38, T40, T43. Run; record failures. Needs T005
- [ ] T010 [US2] [core] (builder) Implement crates/core/src/marks.rs per data-model.md "Computation": the pure helpers (`ceil1(x) = (x * 10.0 - 1e-9).ceil() / 10.0`; `round1`/`round2` round half away from zero like the R13 helper), then `course_marks`: NotFound for an unknown id; ungraded (R33) → `graded: false`; else total = 100 × credits, per component marks = weight_pct / 100 × total, attendance from the R13 `pct` of the course's counts (reuse `attendance::pct`) via `band_credit` (None → earned 0, fraction 0), items from scores of assessments linked to that component (only components of the course's current rule set, OQ-2 A), effective item_max = `item_max` or the component's marks, `scores_entered`, `missing_items = max(item_count − entered, 0)` (OQ-1 A); secured, assessed, `projected_pct = round1(secured / assessed × 100)` or None, letter via `letter_for` and `grade_scale()`; `fully_assessed` = every attendance pct known and every items component has `missing_items == 0` (OQ-1 A); `outlook: None` for now (T012). In crates/core/src/db.rs `create_assessment`/`update_assessment`: when `component_id` is set it must be a component of the course's current rule set (OQ-2 A) and a present `score` must be `<=` the effective item_max (R36), else `Error::Validation`. Doc comments `/// R35: …` … `/// R37: …`. T008, T009 pass; all core tests pass. Needs T008, T009, T007 (the T009 fixtures read `list_rule_sets`)

---

## Phase 5: User Story 3 - Needed in final (P1, core track)

**Goal**: R38 needed-in-final table with assumptions, or the actual grade.

**Independent Test**: `cargo test -p planner-core --test m9_marks t39_ r38_` passes.

- [ ] T011 [US3] [core] (tester) Append to crates/core/tests/m9_marks.rs: `t39_needed_in_final` (3-credit Theory: attendance 92% (23 attended, 2 absent), quizzes 12, 11, 13, 14, mid 60 of 75, no final score → `FinalOutlook::Needed` with `final_item_max` 150.0, rows in scale order, A+ `Needed::Score(111.0)`, A `Needed::Score(96.0)`, A- `Score(81.0)`, B+ `Score(66.0)`, B `Score(51.0)`, B- `Score(36.0)`, C+ `Score(21.0)`, C `Score(6.0)`, D and F `Needed::Secured`; `assumptions` empty); `t39_missing_quiz_listed_as_assumed_zero` (same but quizzes 12, 11, 13 only → A+ `Score(114.0)`, A `Score(99.0)`, `assumptions == [Assumption { component_name: "Quizzes", missing_items: 1 }]`) (OQ-1 A); `r38_not_reachable` (3-credit Theory, only mid 0 of 75 entered → A+ needs 240 > 150 → `Needed::NotReachable`); `r38_needed_rounds_up` (a case where need / final marks × item_max is not a multiple of 0.1, e.g. 1.5-credit Theory (final marks and item max 75) with only mid 23.33 → D need = 60 − 23.33 = 36.67 → `Score(36.7)`; A+ need 96.67 > 75 → `NotReachable`); `r38_final_scored_shows_actual_grade` (link a score to Final → `FinalOutlook::Actual { letter }` equal to `course_marks().letter`); `r38_unassessed_attendance_counts_zero_and_is_not_listed` (research D8). Doc comments cite R38, T39. Run; record failures. Needs T010
- [ ] T012 [US3] [core] (builder) In crates/core/src/marks.rs fill `outlook` per data-model.md: if any assessment linked to the final component has a score → `Actual { letter }`; else for each grade row (min_pct descending) need = min_pct / 100 × total − Σ earned of non-final components, value = `ceil1(need / final_marks × final_item_max)`, `Secured` if ≤ 0, `NotReachable` if > final_item_max, else `Score(value)`; `assumptions` = one entry per non-final items component with `missing_items > 0` (OQ-1 A), in position order. Doc comments `/// R38: …`. T011 passes; all core tests pass. Needs T011

---

## Phase 6: User Story 4 - Term GPA (P2, core track)

**Goal**: R40.

**Independent Test**: `cargo test -p planner-core --test m9_gpa` passes.

- [ ] T013 [P] [US4] [core] (tester) Create crates/core/tests/m9_gpa.rs: `t41_gpa_three_credit_a_plus_and_one_and_half_credit_b` (3-credit Theory with only mid 60 of 75 → 80.0% A+; 1.5-credit Theory with only mid 23.25 of 37.5 → 62.0% B; `gpa == Some(3.67)`, `status == Projected`, `included == 2`, `left_out == 0`); `r40_none_when_no_course_qualifies` (no graded course → `gpa: None`); `r40_left_out_counts_graded_courses_without_marks` (a graded course with nothing assessed → `left_out == 1`, not in the average); `r40_ungraded_and_archived_courses_excluded`; `r40_final_when_everything_assessed` (one 3-credit Theory course with attendance marked, 4 quiz scores, mid and final scores → `status == Final`), and `r40_projected_while_a_quiz_is_missing` (same with 3 quiz scores → `Projected`, OQ-1 A); `r40_no_active_term` (research D12). Doc comments cite R40, T41. Run; record failures. Needs T010
- [ ] T014 [US4] [core] (builder) Implement `term_gpa` in crates/core/src/marks.rs: no active term → `GpaSummary { gpa: None, status: Projected, included: 0, left_out: 0 }`; over non-archived courses of the active term that are graded, call the `course_marks` logic; with `projected_pct` → included (points from `letter`'s grade row × credits), without → `left_out`; `gpa = round2(Σ points × credits / Σ credits)` or None; `status = Final` iff at least one included course and every included course is `fully_assessed`. Doc comments `/// R40: …`. T013 passes. Needs T013, T012

---

## Phase 7: User Story 5 - Backups carry grading data (P2, core track)

**Goal**: R41.

**Independent Test**: `cargo test -p planner-core --test m9_backup` and the m3 backup tests pass.

- [x] T015 [P] [US5] [core] (tester) Per contracts/core-backup.md. In crates/core/tests/m3_backup_export.rs update `r21_export_has_exact_top_level_keys` to the 14 keys (add `rule_sets`, `rule_components`, `attendance_bands`, `grade_scale`) and the course and assessment row expectations in `r21_export_rows_keep_ids_and_value_types` to include `credits`, `rule_set_id`, `component_id`, `score`; `r21_export_empty_database` expects the seeded grading rows. Create crates/core/tests/m9_backup.rs: `t42_v2_export_restore_export_equal` (graded course, a linked scored assessment, an edited rule set, an edited grade scale; export → `restore_json` into a fresh in-memory planner → export; the two JSON texts are equal after removing `exported_at`); `t42_v1_backup_restores_ungraded_with_defaults` (on a DB whose rule sets and grade scale were edited, restore a v1 document made by exporting, setting `schema_version` to 1 and removing the four arrays and the four new row keys → courses have `credits`/`rule_set_id` None, assessments `component_id`/`score` None, `list_rule_sets()` and `grade_scale()` equal a fresh `open_in_memory()`); `r41_v2_backup_missing_grading_arrays_rejected` (`Error::Validation`, DB unchanged); `r41_csv_unchanged` (with grading data present, both CSV header rows equal the R23 headers). Doc comments cite R21, R22, R41, T42. Run; record failures. Needs T005
- [ ] T016 [US5] [core] (builder) Implement R41 in crates/core/src/backup.rs per contracts/core-backup.md and research D13: the four arrays (row structs with every column, nullable keys always written) appended after `settings`, ordered as specified; course and assessment rows gain the four keys (`#[serde(default)]` on read); restore clears `attendance_band`, `rule_component`, `rule_set`, `grade_scale` too; `schema_version` 2 requires the four arrays, inserts them; `schema_version` 1 ignores them, stores the four new row fields as NULL, and runs `GRADING_SEED`. Doc comments `/// R41: …`. T015 and all m3 tests pass. Needs T015, T007

- [ ] T017 [core] (builder) Core checkpoint: remove leftover `#[allow(dead_code)]` from T001; run `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` and `cargo test -p planner-core t36_ t37_ t38_ t39_ t40_ t41_ t42_ t43_`; paste the summary. Needs T003, T007, T010, T012, T014, T016

**Checkpoint**: core track done and gate-green; the app still builds with T002 only.

---

## Phase 8: User Story 6 - Screens for grading (P2, app track)

**Start condition**: M8 merged into this branch. Each task lists the core task it depends on.

**Goal**: §13.3 UI per contracts/ui-grading.md; formatting in view-model functions; `Theme.*` only.

**Independent Test**: `cargo test -p app viewmodel` passes; theme grep clean.

- [ ] T018 [US6] [app] (builder) View-model stubs: in crates/app/src/viewmodel.rs add the functions of contracts/ui-grading.md "View-model functions" with bodies `todo!("M9")` (`format_mark`, `format_marks`, `format_projected`, `needed_text`, `needed_rows`, `assumption_text`, `actual_grade_text`, `gpa_header_text`, `component_summary`, `band_text`, `score_max_label`, `parse_component_row`, `parse_band_row`, `parse_grade_row`) and extend `parse_course_form` with `credits: &str, rule_set_id: Option<i64>` and `parse_assessment_form` with `component_id: Option<i64>, score: &str` (keep old behavior; new params unused for now; update callers in crates/app/src/pages/courses.rs and pages/assessments.rs to pass `""`/`None`). Add the Slint structs needed later to crates/app/ui/types.slint (`MarkRow`, `NeededRowView`, `RuleSetRow`, `ComponentRow`, `BandRow`, `GradeRowView`). Builds. Needs T001 (types), M8 merged
- [ ] T019 [P] [US6] [app] (tester) Formatting tests in crates/app/src/viewmodel/tests.rs with the exact examples of contracts/ui-grading.md: `format_marks(39.0, 45.0) == "39.0 of 45"` (T37), `format_marks(27.0, 30.0) == "27.0 of 30"` (T38), `format_mark(37.5) == "37.5"`, `format_projected(Some(80.0), Some("A+"))`, `format_projected(None, None) == "—"`, `needed_text` for `Score(111.0)` → "111.0", `Secured`, `NotReachable` (T39), `assumption_text` 1 and 2 items, `actual_grade_text`, `gpa_header_text` for 3.67 projected (T41), final, None, and left_out 1, `component_summary` for the seeded Attendance, Quizzes, Final, `band_text`, `score_max_label(Some(15.0)) == "/ 15"`. Doc comments cite R37, R38, R40, T37–T41. Run; record failures. Needs T018
- [ ] T020 [P] [US6] [app] (tester) Form tests in crates/app/src/viewmodel/tests.rs: `parse_course_form` credits "" → None, "3" → 3.0, "1.5" → 1.5, "abc" / "0" / "-1" → inline error on `credits`, rule set `None` passes through; `parse_assessment_form` score "" → None, "12" → 12.0, "12.5", "x" / "-1" → inline error on `score`; `parse_component_row` (empty item max → None, non-number weight → inline error), `parse_band_row`, `parse_grade_row` number errors inline. R34/R39 range checks are not duplicated here. Run; record failures. Needs T018
- [ ] T021 [US6] [app] (builder) Implement the T018 functions in crates/app/src/viewmodel.rs so T019 and T020 pass. Needs T019, T020
- [ ] T022 [US6] [app] (builder) Rules page per contracts/ui-grading.md "Rules page": new crates/app/ui/rules.slint (bindings and callbacks only, `Theme.*` only), crates/app/src/pages/rules.rs (`refresh`, `wire`; calls `list_rule_sets`, `save_rule_set`, `delete_rule_set`, `grade_scale`, `save_grade_scale`; core `Error::Validation` text shown inline via `error_text`), `pub mod rules;` in crates/app/src/pages/mod.rs, a nav button after Assessments with page index 6 in crates/app/ui/app-window.slint, wiring in crates/app/src/main.rs / controller.rs. After a save, refresh the Courses page (marks and GPA may change). Needs T021, T007
- [ ] T023 [US6] [app] (builder) Course form: **Credits** text and **Rule set** choice ("Not graded" + names from `list_rule_sets`) in crates/app/ui/courses.slint and crates/app/src/pages/courses.rs; edit pre-fills both; pass to `parse_course_form`. Needs T021, T005, T007
- [ ] T024 [US6] [app] (builder) Assessment form: **Counts toward** choice ("Not counted" + components of the selected course's rule set, empty when not graded) and **Score** text with the `score_max_label` from `course_marks(course).components[i].item_max` in crates/app/ui/assessments.slint and crates/app/src/pages/assessments.rs; edit pre-fills; core validation errors (R36, OQ-2 A) shown inline. After a save, refresh the Courses page. Needs T021, T010
- [ ] T025 [US6] [app] (builder) Course detail: **Marks** card (one `MarkRow` per component via `format_marks`, secured, `format_projected`; an ungraded line when `graded` is false) and **Needed in final** table (`needed_rows`, assumption lines via `assumption_text`, or `actual_grade_text`) in crates/app/ui/courses.slint and crates/app/src/pages/courses.rs. Needs T021, T012
- [ ] T026 [US6] [app] (builder) Courses page header: `gpa_header_text(&term_gpa()?)` in crates/app/ui/courses.slint and crates/app/src/pages/courses.rs, refreshed with the course list. Needs T021, T014

---

## Phase 9: Polish, gate, Done-when

- [ ] T027 [core] [app] (builder) Run the gate `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` and the CI theme grep (`! grep -rnE '#[0-9a-fA-F]{6}|[0-9]+px' crates/app/ui --include=*.slint | grep -v theme.slint`); remove leftover `#[allow(dead_code)]`/`todo!`; paste the summary. Needs T017, T022–T026
- [ ] T028 (human) Manual look per quickstart.md "Manual look" with a scratch `ACADEMIC_PLANNER_DB`: Rules page values, the T39 course shows Quizzes 39.0 of 45 and A+ 111.0 / A 96.0, GPA header shows "(projected)", a v1 DB copy upgrades with a pre-migrate backup. Needs T027

---

## Dependencies & Execution Order

- Core track: T001 → T002 (same commit) → T003 [P] and T004 → T005 → stories. US1 (T006–T007),
  US2 tests (T008, T009) and US5 tester (T015) can start right after T005 (T008 only needs T001);
  T010 also needs T007 because the T009 fixtures read `list_rule_sets`.
  US3 (T011–T012) needs T010. US4 (T013–T014) needs T010 for tests and T012 for the
  implementation. T016 needs T007 (rule-set reads for export). T017 closes the core track.
- Within a story: tester task → run it failing → builder task.
- App track starts after **M8 is merged** (shared files: crates/app/src/viewmodel.rs,
  viewmodel/tests.rs, ui/assessments.slint, ui/courses.slint, pages/assessments.rs,
  pages/courses.rs). App → core dependencies:

| App task | Needs core task(s) | Why |
| --- | --- | --- |
| T018 view-model stubs | T001 | new core types in signatures |
| T019, T020 tests | T018 (and so T001) | compile against stubs |
| T021 implement | T019, T020 | — |
| T022 Rules page | T007 | rule-set and grade-scale CRUD at runtime |
| T023 course form | T005, T007 | credits/rule set stored; rule-set names |
| T024 assessment form | T010 | `course_marks` item_max; R36/OQ-2 validation |
| T025 marks card + needed table | T012 | `CourseMarks.outlook` |
| T026 GPA header | T014 | `term_gpa` |

- The only app edit inside the core track is T002 (four struct literals).

## Parallel Examples

- After T001: T003, T004, T008 in parallel (different files).
- After T005: T006, T009, T015 in parallel (m9_rules.rs, m9_marks.rs + common/mod.rs,
  m9_backup.rs + m3_backup_export.rs).
- After T018: T019 and T020 (same file; run sequentially if one tester, else split by section).
- T022 and T023–T026 touch different .slint/.rs files except T023/T025/T026 (all courses.*):
  run those three sequentially.

## Implementation Strategy

1. Core MVP = Setup + migration + US1 + US2 (T001–T010): rules and marks, gate-green.
2. Add US3 (needed in final), US4 (GPA), US5 (backup) → T017 core checkpoint.
3. After M8 merges: US6 app track (T018–T026).
4. Gate (T027), human look (T028).

## Owners

- **tester**: T003, T004, T006, T008, T009, T011, T013, T015 (core); T019, T020 (app)
- **builder**: T001, T002, T005, T007, T010, T012, T014, T016, T017 (core); T018, T021–T026 (app); T027 (both)
- **human**: T028
