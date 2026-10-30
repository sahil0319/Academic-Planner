//! M9 rule sets and the grade scale (R34, R39): seeds, validation, in-place updates, delete.

mod common;

use common::*;
use planner_core::{
    AssessmentFilter, AssessmentInput, AssessmentKind, AttendanceBand, ComponentKind, CourseInput,
    Error, GradeRow, Planner, RuleComponent, RuleComponentInput, RuleSet, RuleSetInput,
};
use rusqlite::Connection;
use tempfile::TempDir;

fn planner() -> Planner {
    let mut p = Planner::open_in_memory().unwrap();
    p.save_term(fixture_term()).unwrap();
    p
}

fn band(min_pct: f64, credit_pct: f64) -> AttendanceBand {
    AttendanceBand {
        min_pct,
        credit_pct,
    }
}

fn default_bands() -> Vec<AttendanceBand> {
    vec![
        band(95.0, 100.0),
        band(90.0, 80.0),
        band(85.0, 60.0),
        band(75.0, 20.0),
        band(70.0, 10.0),
        band(0.0, 0.0),
    ]
}

#[allow(clippy::too_many_arguments)]
fn component(
    id: i64,
    rule_set_id: i64,
    position: i64,
    name: &str,
    kind: ComponentKind,
    weight_pct: f64,
    counts: Option<(u32, u32)>,
    item_max: Option<f64>,
    is_final: bool,
) -> RuleComponent {
    RuleComponent {
        id,
        rule_set_id,
        position,
        name: name.into(),
        kind,
        weight_pct,
        item_count: counts.map(|c| c.0),
        best_count: counts.map(|c| c.1),
        item_max,
        is_final,
        bands: if kind == ComponentKind::Attendance {
            default_bands()
        } else {
            Vec::new()
        },
    }
}

fn seeded_rule_sets() -> Vec<RuleSet> {
    use ComponentKind::{Attendance, Items};
    vec![
        RuleSet {
            id: 1,
            name: "Theory".into(),
            components: vec![
                component(1, 1, 0, "Attendance", Attendance, 10.0, None, None, false),
                component(
                    2,
                    1,
                    1,
                    "Quizzes",
                    Items,
                    15.0,
                    Some((4, 3)),
                    Some(15.0),
                    false,
                ),
                component(3, 1, 2, "Mid", Items, 25.0, Some((1, 1)), None, false),
                component(4, 1, 3, "Final", Items, 50.0, Some((1, 1)), None, true),
            ],
        },
        RuleSet {
            id: 2,
            name: "Lab".into(),
            components: vec![
                component(5, 2, 0, "Attendance", Attendance, 30.0, None, None, false),
                component(6, 2, 1, "Lab work", Items, 40.0, Some((1, 1)), None, false),
                component(7, 2, 2, "Lab final", Items, 30.0, Some((1, 1)), None, true),
            ],
        },
    ]
}

fn input_of(set: &RuleSet) -> RuleSetInput {
    RuleSetInput {
        id: Some(set.id),
        name: set.name.clone(),
        components: set
            .components
            .iter()
            .map(|c| RuleComponentInput {
                id: Some(c.id),
                name: c.name.clone(),
                kind: c.kind,
                weight_pct: c.weight_pct,
                item_count: c.item_count,
                best_count: c.best_count,
                item_max: c.item_max,
                is_final: c.is_final,
                bands: c.bands.clone(),
            })
            .collect(),
    }
}

/// The seeded Theory set as an input that saves unchanged.
fn theory_input(p: &Planner) -> RuleSetInput {
    let sets = p.list_rule_sets().unwrap();
    input_of(sets.iter().find(|s| s.name == "Theory").unwrap())
}

fn rule_component_input(name: &str, weight_pct: f64) -> RuleComponentInput {
    RuleComponentInput {
        id: None,
        name: name.into(),
        kind: ComponentKind::Items,
        weight_pct,
        item_count: Some(1),
        best_count: Some(1),
        item_max: None,
        is_final: false,
        bands: Vec::new(),
    }
}

fn comp_mut<'a>(input: &'a mut RuleSetInput, name: &str) -> &'a mut RuleComponentInput {
    input
        .components
        .iter_mut()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no component {name}"))
}

/// Saving `input` must fail with `Error::Validation` and leave the rule sets as they were.
fn assert_rule_set_rejected(p: &mut Planner, why: &str, input: RuleSetInput) {
    let before = p.list_rule_sets().unwrap();
    let err = p
        .save_rule_set(input)
        .expect_err(&format!("{why}: save should fail"));
    assert!(
        matches!(err, Error::Validation(_)),
        "{why}: expected Error::Validation, got {err:?}"
    );
    assert_eq!(
        p.list_rule_sets().unwrap(),
        before,
        "{why}: rule sets changed"
    );
}

fn row(letter: &str, min_pct: f64, points: f64) -> GradeRow {
    GradeRow {
        letter: letter.into(),
        min_pct,
        points,
    }
}

fn ugc_scale() -> Vec<GradeRow> {
    vec![
        row("A+", 80.0, 4.0),
        row("A", 75.0, 3.75),
        row("A-", 70.0, 3.5),
        row("B+", 65.0, 3.25),
        row("B", 60.0, 3.0),
        row("B-", 55.0, 2.75),
        row("C+", 50.0, 2.5),
        row("C", 45.0, 2.25),
        row("D", 40.0, 2.0),
        row("F", 0.0, 0.0),
    ]
}

/// Asserts the scale is rejected with `Error::Validation` and the stored scale is unchanged.
fn assert_scale_rejected(p: &mut Planner, why: &str, rows: &[GradeRow]) {
    let before = p.grade_scale().unwrap();
    let err = p
        .save_grade_scale(rows)
        .expect_err(&format!("{why}: save should fail"));
    assert!(
        matches!(err, Error::Validation(_)),
        "{why}: expected Error::Validation, got {err:?}"
    );
    assert_eq!(p.grade_scale().unwrap(), before, "{why}: scale changed");
}

/// R34: the two seeded rule sets have every component field, position, and attendance band
/// as specified, bands sorted by `min_pct` descending.
#[test]
fn r34_seeded_rule_sets_match_spec() {
    let p = Planner::open_in_memory().unwrap();
    assert_eq!(p.list_rule_sets().unwrap(), seeded_rule_sets());
}

/// R39: the seeded grade scale is the ten UGC rows, by `min_pct` descending.
#[test]
fn r39_seeded_grade_scale_is_ugc() {
    let p = Planner::open_in_memory().unwrap();
    assert_eq!(p.grade_scale().unwrap(), ugc_scale());
}

/// R34 (T36): weights not summing to 100, two finals, `best_count` above `item_count`, and
/// attendance bands without a 0 row are each rejected and change nothing.
#[test]
fn t36_rule_set_validation_errors() {
    let mut p = Planner::open_in_memory().unwrap();

    let mut weights = theory_input(&p);
    comp_mut(&mut weights, "Final").weight_pct = 45.0;
    assert_rule_set_rejected(&mut p, "weights sum to 95", weights);

    let mut two_finals = theory_input(&p);
    comp_mut(&mut two_finals, "Mid").is_final = true;
    assert_rule_set_rejected(&mut p, "two finals", two_finals);

    let mut best = theory_input(&p);
    let quizzes = comp_mut(&mut best, "Quizzes");
    quizzes.best_count = Some(5);
    quizzes.item_count = Some(4);
    assert_rule_set_rejected(&mut p, "best_count above item_count", best);

    let mut bands = theory_input(&p);
    comp_mut(&mut bands, "Attendance").bands.pop();
    assert_rule_set_rejected(&mut p, "no 0 band", bands);
}

/// R34: a weight sum within 0.01 of 100 is accepted.
#[test]
fn r34_weights_within_tolerance_accepted() {
    let mut p = Planner::open_in_memory().unwrap();
    let mut input = theory_input(&p);
    comp_mut(&mut input, "Final").weight_pct = 50.005;

    let saved = p.save_rule_set(input).unwrap();
    let final_weight = saved.components.iter().find(|c| c.is_final).unwrap();
    assert_eq!(final_weight.weight_pct, 50.005);
    assert_eq!(p.list_rule_sets().unwrap()[0], saved);
}

/// R34: a rule set name must be non-empty and unique; component names must be non-empty and
/// unique within the set. Keeping a set's own name is fine.
#[test]
fn r34_empty_or_duplicate_name_rejected() {
    let mut p = Planner::open_in_memory().unwrap();

    let mut blank = theory_input(&p);
    blank.name = "   ".into();
    assert_rule_set_rejected(&mut p, "blank set name", blank);

    let mut taken = theory_input(&p);
    taken.id = None;
    taken.components.iter_mut().for_each(|c| c.id = None);
    assert_rule_set_rejected(&mut p, "new set named like Theory", taken);

    let mut rename = theory_input(&p);
    rename.name = "Lab".into();
    assert_rule_set_rejected(&mut p, "Theory renamed to Lab", rename);

    let mut empty_component = theory_input(&p);
    comp_mut(&mut empty_component, "Mid").name = " ".into();
    assert_rule_set_rejected(&mut p, "blank component name", empty_component);

    let mut twin = theory_input(&p);
    comp_mut(&mut twin, "Mid").name = "Quizzes".into();
    assert_rule_set_rejected(&mut p, "duplicate component name", twin);

    p.save_rule_set(theory_input(&p)).unwrap();
}

/// R34: at most one attendance component per rule set.
#[test]
fn r34_two_attendance_components_rejected() {
    let mut p = Planner::open_in_memory().unwrap();
    let mut input = theory_input(&p);
    comp_mut(&mut input, "Final").weight_pct = 40.0;
    input.components.push(RuleComponentInput {
        kind: ComponentKind::Attendance,
        item_count: None,
        best_count: None,
        bands: default_bands(),
        ..rule_component_input("Lab attendance", 10.0)
    });
    assert_rule_set_rejected(&mut p, "two attendance components", input);
}

/// R34: the final component must be an items component.
#[test]
fn r34_final_must_be_items() {
    let mut p = Planner::open_in_memory().unwrap();
    let mut input = theory_input(&p);
    comp_mut(&mut input, "Final").is_final = false;
    comp_mut(&mut input, "Attendance").is_final = true;
    assert_rule_set_rejected(&mut p, "attendance as final", input);
}

/// R34: an items component needs `item_count` and `best_count` with 1 <= best_count <=
/// item_count.
#[test]
fn r34_items_need_counts() {
    let mut p = Planner::open_in_memory().unwrap();

    let mut no_item_count = theory_input(&p);
    comp_mut(&mut no_item_count, "Quizzes").item_count = None;
    assert_rule_set_rejected(&mut p, "no item_count", no_item_count);

    let mut no_best_count = theory_input(&p);
    comp_mut(&mut no_best_count, "Quizzes").best_count = None;
    assert_rule_set_rejected(&mut p, "no best_count", no_best_count);

    let mut zero_best = theory_input(&p);
    comp_mut(&mut zero_best, "Quizzes").best_count = Some(0);
    assert_rule_set_rejected(&mut p, "best_count 0", zero_best);
}

/// R34: an attendance component needs at least one band.
#[test]
fn r34_attendance_needs_bands() {
    let mut p = Planner::open_in_memory().unwrap();
    let mut input = theory_input(&p);
    comp_mut(&mut input, "Attendance").bands.clear();
    assert_rule_set_rejected(&mut p, "no bands", input);
}

/// R34: saving a set with component ids updates the rows in place: ids stay, and an
/// assessment linked to a component stays linked.
#[test]
fn r34_update_in_place_keeps_component_ids() {
    let mut p = planner();
    let course = graded_course(&mut p, "CSE101", 3.0, "Theory");
    let quizzes = component_id(&p, "Theory", "Quizzes");
    let linked = p
        .create_assessment(AssessmentInput {
            component_id: Some(quizzes),
            ..assessment_input(
                course.id,
                AssessmentKind::Quiz,
                "Quiz 1",
                "2026-01-12T10:00",
            )
        })
        .unwrap();
    let before_ids: Vec<i64> = p.list_rule_sets().unwrap()[0]
        .components
        .iter()
        .map(|c| c.id)
        .collect();

    let mut input = theory_input(&p);
    {
        let quizzes = comp_mut(&mut input, "Quizzes");
        quizzes.name = "Class tests".into();
        quizzes.weight_pct = 20.0;
    }
    comp_mut(&mut input, "Final").weight_pct = 45.0;
    let saved = p.save_rule_set(input).unwrap();

    let after_ids: Vec<i64> = saved.components.iter().map(|c| c.id).collect();
    assert_eq!(after_ids, before_ids);
    let renamed = saved.components.iter().find(|c| c.id == quizzes).unwrap();
    assert_eq!(renamed.name, "Class tests");
    assert_eq!(renamed.weight_pct, 20.0);
    assert_eq!(p.list_rule_sets().unwrap()[0], saved);

    let assessments = p.list_assessments(AssessmentFilter::All).unwrap();
    let after = assessments.iter().find(|a| a.id == linked.id).unwrap();
    assert_eq!(after.component_id, Some(quizzes));
}

/// R34: a new rule set can be created with one items final component of 100%.
#[test]
fn r34_new_rule_set_created() {
    let mut p = Planner::open_in_memory().unwrap();
    let saved = p
        .save_rule_set(RuleSetInput {
            id: None,
            name: "Seminar".into(),
            components: vec![RuleComponentInput {
                is_final: true,
                ..rule_component_input("Report", 100.0)
            }],
        })
        .unwrap();

    assert_eq!(saved.name, "Seminar");
    assert!(saved.id > 2);
    assert_eq!(saved.components.len(), 1);
    assert_eq!(saved.components[0].name, "Report");
    assert_eq!(saved.components[0].position, 0);
    assert!(saved.components[0].is_final);

    let sets = p.list_rule_sets().unwrap();
    assert_eq!(sets.len(), 3);
    assert_eq!(sets[2], saved);
}

/// A Theory input without the Mid component, with its weight moved to the Final.
fn theory_without_mid(p: &Planner) -> RuleSetInput {
    let mut input = theory_input(p);
    input.components.retain(|c| c.name != "Mid");
    comp_mut(&mut input, "Final").weight_pct = 75.0;
    input
}

/// R34: removing a component that has a linked assessment with a score is rejected and
/// leaves the set and the link unchanged.
#[test]
fn r34_remove_component_with_scored_assessment_rejected() {
    let mut p = planner();
    let course = graded_course(&mut p, "CSE101", 3.0, "Theory");
    let mid = component_id(&p, "Theory", "Mid");
    let item = scored(&mut p, course.id, mid, "Mid", 50.0);

    let input = theory_without_mid(&p);
    assert_rule_set_rejected(&mut p, "scored assessment on removed component", input);

    let assessments = p.list_assessments(AssessmentFilter::All).unwrap();
    let after = assessments.iter().find(|a| a.id == item.id).unwrap();
    assert_eq!(after.component_id, Some(mid));
    assert_eq!(after.score, Some(50.0));
}

/// R34: removing a component unlinks assessments that have no score (research D3).
#[test]
fn r34_remove_component_unlinks_unscored_assessments() {
    let mut p = planner();
    let course = graded_course(&mut p, "CSE101", 3.0, "Theory");
    let mid = component_id(&p, "Theory", "Mid");
    let item = p
        .create_assessment(AssessmentInput {
            component_id: Some(mid),
            ..assessment_input(course.id, AssessmentKind::Exam, "Mid", "2026-01-20T10:00")
        })
        .unwrap();

    let input = theory_without_mid(&p);
    let saved = p.save_rule_set(input).unwrap();
    assert_eq!(saved.components.len(), 3);

    let assessments = p.list_assessments(AssessmentFilter::All).unwrap();
    let after = assessments.iter().find(|a| a.id == item.id).unwrap();
    assert_eq!(after.component_id, None);
}

/// R34 (T43): a rule set used by a course cannot be deleted.
#[test]
fn t43_delete_rule_set_used_by_course_rejected() {
    let mut p = planner();
    p.create_course(CourseInput {
        credits: Some(3.0),
        rule_set_id: Some(1),
        ..course_input("CSE101")
    })
    .unwrap();
    let before = p.list_rule_sets().unwrap();

    let err = p.delete_rule_set(1).unwrap_err();
    assert!(matches!(err, Error::Validation(_)), "{err:?}");
    assert_eq!(p.list_rule_sets().unwrap(), before);
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

/// R34: deleting an unused rule set removes it with its components and attendance bands.
#[test]
fn r34_delete_unused_rule_set() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("planner.sqlite");
    {
        let mut p = Planner::open(&path).unwrap();
        p.delete_rule_set(2).unwrap();
        let sets = p.list_rule_sets().unwrap();
        assert_eq!(sets, seeded_rule_sets()[..1].to_vec());
    }

    let conn = Connection::open(&path).unwrap();
    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM rule_set WHERE id = 2"),
        0
    );
    assert_eq!(
        count(
            &conn,
            "SELECT COUNT(*) FROM rule_component WHERE rule_set_id = 2"
        ),
        0
    );
    assert_eq!(
        count(
            &conn,
            "SELECT COUNT(*) FROM attendance_band WHERE component_id IN (5, 6, 7)"
        ),
        0
    );
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM rule_component"), 4);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM attendance_band"), 6);
}

/// R34: deleting an unknown rule set is `NotFound`.
#[test]
fn r34_delete_unknown_rule_set_not_found() {
    let mut p = Planner::open_in_memory().unwrap();
    let err = p.delete_rule_set(99).unwrap_err();
    assert!(matches!(err, Error::NotFound { .. }), "{err:?}");
}

/// R39: a duplicate letter, a duplicate `min_pct`, a missing 0 row, and points that rise as
/// `min_pct` falls are each rejected and leave the scale unchanged.
#[test]
fn r39_grade_scale_validation_errors() {
    let mut p = Planner::open_in_memory().unwrap();

    let mut duplicate_letter = ugc_scale();
    duplicate_letter[1].letter = "A+".into();
    assert_scale_rejected(&mut p, "duplicate letter", &duplicate_letter);

    let mut duplicate_min = ugc_scale();
    duplicate_min[1].min_pct = 80.0;
    assert_scale_rejected(&mut p, "duplicate min_pct", &duplicate_min);

    let mut no_zero = ugc_scale();
    no_zero.pop();
    assert_scale_rejected(&mut p, "no 0 row", &no_zero);

    let mut rising = ugc_scale();
    rising[1].points = 4.5;
    assert_scale_rejected(&mut p, "points rise as min falls", &rising);
}

/// R39: saving a grade scale replaces all rows, returned by `min_pct` descending whatever
/// the input order.
#[test]
fn r39_save_grade_scale_replaces_rows() {
    let mut p = Planner::open_in_memory().unwrap();
    let rows = vec![
        row("Pass", 40.0, 1.0),
        row("Fail", 0.0, 0.0),
        row("High", 70.0, 4.0),
    ];
    p.save_grade_scale(&rows).unwrap();

    assert_eq!(
        p.grade_scale().unwrap(),
        vec![
            row("High", 70.0, 4.0),
            row("Pass", 40.0, 1.0),
            row("Fail", 0.0, 0.0)
        ]
    );
}
