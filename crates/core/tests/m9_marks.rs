//! M9 course marks (R33, R35, R36, R37) and the R36 score checks (T43).

mod common;

use common::*;
use planner_core::{
    AssessmentFilter, AssessmentInput, AssessmentKind, Assumption, ComponentMarks, CourseInput,
    CourseMarks, Error, FinalOutlook, Needed, NeededRow, Planner,
};

fn planner() -> Planner {
    let mut p = Planner::open_in_memory().unwrap();
    p.save_term(fixture_term()).unwrap();
    p
}

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "{actual} is not {expected}"
    );
}

fn component<'a>(marks: &'a CourseMarks, name: &str) -> &'a ComponentMarks {
    marks
        .components
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no component {name}"))
}

/// Scores the four Theory quizzes (or fewer) of a course as "Quiz 1", "Quiz 2", and so on.
fn score_quizzes(p: &mut Planner, course_id: i64, scores: &[f64]) {
    let quizzes = component_id(p, "Theory", "Quizzes");
    for (i, score) in scores.iter().enumerate() {
        scored(p, course_id, quizzes, &format!("Quiz {}", i + 1), *score);
    }
}

/// Theory course (3 credits) with the T39 inputs: attendance 92%, quizzes 12, 11, 13, 14, and
/// a mid score of 60 of 75. No final score.
fn t39_course(p: &mut Planner) -> i64 {
    let course = graded_course(p, "CSE101", 3.0, "Theory");
    mark_attendance(p, course.id, 23, 2);
    score_quizzes(p, course.id, &[12.0, 11.0, 13.0, 14.0]);
    let mid = component_id(p, "Theory", "Mid");
    scored(p, course.id, mid, "Mid", 60.0);
    course.id
}

/// R36, R33 (T37): best 3 of the quiz scores 12, 11, 13, 14 give 39.0 of 45.
#[test]
fn t37_quizzes_best_three_of_four() {
    let mut p = planner();
    let course = graded_course(&mut p, "CSE101", 3.0, "Theory");
    score_quizzes(&mut p, course.id, &[12.0, 11.0, 13.0, 14.0]);

    let marks = p.course_marks(course.id).unwrap();
    let quizzes = component(&marks, "Quizzes");
    close(quizzes.earned, 39.0);
    close(quizzes.marks, 45.0);
    close(quizzes.assessed_fraction, 1.0);
}

/// R35, R33 (T38): 87% attendance falls in the 85 band (90% credit) and earns 27.0 of 30.
#[test]
fn t38_attendance_band_87_pct() {
    let mut p = planner();
    let course = graded_course(&mut p, "CSE101", 3.0, "Theory");
    mark_attendance(&mut p, course.id, 20, 3);

    let marks = p.course_marks(course.id).unwrap();
    let attendance = component(&marks, "Attendance");
    close(attendance.earned, 27.0);
    close(attendance.marks, 30.0);
}

/// R37, R39 (T40): a projected 79.9% is an A and 80.0% is an A+.
#[test]
fn t40_projected_79_9_is_a_and_80_0_is_a_plus() {
    let mut p = planner();
    let mid = component_id(&p, "Theory", "Mid");

    let low = graded_course(&mut p, "CSE101", 3.0, "Theory");
    scored(&mut p, low.id, mid, "Mid", 59.925);
    let marks = p.course_marks(low.id).unwrap();
    assert_pct(marks.projected_pct, Some(79.9));
    assert_eq!(marks.letter.as_deref(), Some("A"));

    let high = graded_course(&mut p, "CSE102", 3.0, "Theory");
    scored(&mut p, high.id, mid, "Mid", 60.0);
    let marks = p.course_marks(high.id).unwrap();
    assert_pct(marks.projected_pct, Some(80.0));
    assert_eq!(marks.letter.as_deref(), Some("A+"));
}

/// R33: a course needs both credits and a rule set to be graded; otherwise there are no
/// components, no projection, and no letter.
#[test]
fn r33_ungraded_course_has_no_marks() {
    let mut p = planner();
    let credits_only = p
        .create_course(CourseInput {
            credits: Some(3.0),
            ..course_input("CSE101")
        })
        .unwrap();
    let rule_set_only = p
        .create_course(CourseInput {
            rule_set_id: Some(1),
            ..course_input("CSE102")
        })
        .unwrap();
    let neither = p.create_course(course_input("CSE103")).unwrap();

    for course in [credits_only, rule_set_only, neither] {
        let marks = p.course_marks(course.id).unwrap();
        assert!(!marks.graded, "{} should not be graded", course.code);
        assert!(marks.components.is_empty());
        assert_eq!(marks.projected_pct, None);
        assert_eq!(marks.letter, None);
        assert_eq!(marks.outlook, None);
    }
}

/// R33: component marks are `weight_pct` / 100 x (100 x credits), in rule-set order.
#[test]
fn r33_component_marks_scale_with_credits() {
    let mut p = planner();
    let course = graded_course(&mut p, "CSE101", 1.5, "Theory");

    let marks = p.course_marks(course.id).unwrap();
    assert!(marks.graded);
    close(marks.total, 150.0);
    let listed: Vec<(&str, f64)> = marks
        .components
        .iter()
        .map(|c| (c.name.as_str(), c.marks))
        .collect();
    assert_eq!(
        listed,
        vec![
            ("Attendance", 15.0),
            ("Quizzes", 22.5),
            ("Mid", 37.5),
            ("Final", 75.0)
        ]
    );
}

/// R35: with no attended or absent session the attendance percentage is unknown, so the
/// component earns 0 and counts as not assessed.
#[test]
fn r35_attendance_without_marked_sessions_not_assessed() {
    let mut p = planner();
    let course = graded_course(&mut p, "CSE101", 3.0, "Theory");
    p.add_slot(slot_a(course.id)).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();

    let marks = p.course_marks(course.id).unwrap();
    let attendance = component(&marks, "Attendance");
    close(attendance.earned, 0.0);
    close(attendance.assessed_fraction, 0.0);
}

/// R36: fewer scores than `best_count` earn pro rata: two of three counted items are
/// assessed.
#[test]
fn r36_fewer_scores_than_best_count() {
    let mut p = planner();
    let course = graded_course(&mut p, "CSE101", 3.0, "Theory");
    score_quizzes(&mut p, course.id, &[12.0, 11.0]);

    let marks = p.course_marks(course.id).unwrap();
    let quizzes = component(&marks, "Quizzes");
    close(quizzes.earned, 23.0);
    close(quizzes.assessed_fraction, 2.0 / 3.0);
    assert_eq!(quizzes.scores_entered, 2);
    assert_eq!(quizzes.missing_items, 2);
}

/// R37: with nothing assessed there is no projected percentage and no letter.
#[test]
fn r37_nothing_assessed_has_no_projection() {
    let mut p = planner();
    let course = graded_course(&mut p, "CSE101", 3.0, "Theory");

    let marks = p.course_marks(course.id).unwrap();
    assert!(marks.graded);
    assert_eq!(marks.projected_pct, None);
    assert_eq!(marks.letter, None);
    close(marks.secured, 0.0);
    close(marks.assessed, 0.0);
}

/// R37 (T39 inputs): secured is the sum of earned marks, assessed the marks of what was
/// assessed, and the projection is secured / assessed.
#[test]
fn r37_secured_and_assessed_sums() {
    let mut p = planner();
    let course_id = t39_course(&mut p);

    let marks = p.course_marks(course_id).unwrap();
    close(marks.secured, 129.0);
    close(marks.assessed, 150.0);
    assert_pct(marks.projected_pct, Some(86.0));
    assert_eq!(marks.letter.as_deref(), Some("A+"));
}

/// R36 (T43): a score above the item max is rejected on create and update. A quiz is out of
/// 15; a mid with no item max is out of its marks (75 on a 3-credit course). A score equal to
/// the maximum is accepted.
#[test]
fn t43_score_above_item_max_rejected() {
    let mut p = planner();
    let course = graded_course(&mut p, "CSE101", 3.0, "Theory");
    let quizzes = component_id(&p, "Theory", "Quizzes");
    let mid = component_id(&p, "Theory", "Mid");
    let linked = |component_id: i64, title: &str, score: f64| AssessmentInput {
        component_id: Some(component_id),
        score: Some(score),
        ..assessment_input(course.id, AssessmentKind::Other, title, "2026-01-12T10:00")
    };

    for (component_id, title, over, max) in
        [(quizzes, "Quiz 1", 16.0, 15.0), (mid, "Mid", 76.0, 75.0)]
    {
        let before = p.list_assessments(AssessmentFilter::All).unwrap().len();
        let err = p
            .create_assessment(linked(component_id, title, over))
            .unwrap_err();
        assert!(
            matches!(err, Error::Validation(_)),
            "create {title}: {err:?}"
        );
        assert_eq!(
            p.list_assessments(AssessmentFilter::All).unwrap().len(),
            before,
            "rejected create {title} must write nothing"
        );

        let ok = p
            .create_assessment(linked(component_id, title, max))
            .unwrap();
        assert_eq!(ok.score, Some(max));
        let err = p
            .update_assessment(ok.id, linked(component_id, title, over))
            .unwrap_err();
        assert!(
            matches!(err, Error::Validation(_)),
            "update {title}: {err:?}"
        );
        let stored = p.list_assessments(AssessmentFilter::All).unwrap();
        let stored = stored.iter().find(|a| a.id == ok.id).unwrap();
        assert_eq!(stored.score, Some(max));
    }
}

/// R36 (OQ-2 A): a linked component must belong to the course's current rule set, so a Lab
/// component on a Theory course, or any component on an ungraded course, is rejected on
/// create and update.
#[test]
fn r36_component_must_belong_to_course_rule_set() {
    let mut p = planner();
    let theory = graded_course(&mut p, "CSE101", 3.0, "Theory");
    let ungraded = p.create_course(course_input("CSE102")).unwrap();
    let lab_work = component_id(&p, "Lab", "Lab work");
    let mid = component_id(&p, "Theory", "Mid");
    let linked = |course_id: i64, component_id: i64| AssessmentInput {
        component_id: Some(component_id),
        ..assessment_input(course_id, AssessmentKind::Other, "Item", "2026-01-12T10:00")
    };

    for (course_id, component_id) in [(theory.id, lab_work), (ungraded.id, mid)] {
        let err = p
            .create_assessment(linked(course_id, component_id))
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)), "create: {err:?}");

        let plain = p
            .create_assessment(assessment_input(
                course_id,
                AssessmentKind::Other,
                "Item",
                "2026-01-12T10:00",
            ))
            .unwrap();
        let err = p
            .update_assessment(plain.id, linked(course_id, component_id))
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)), "update: {err:?}");
    }
}

/// R37: marks of an unknown course are `NotFound`.
#[test]
fn r37_course_marks_unknown_course_not_found() {
    let p = planner();
    let err = p.course_marks(99).unwrap_err();
    assert!(matches!(err, Error::NotFound { .. }), "{err:?}");
}

/// Letters of the default grade scale, in scale order (highest first).
const LETTERS: [&str; 10] = ["A+", "A", "A-", "B+", "B", "B-", "C+", "C", "D", "F"];

/// The `Needed` outlook of a course: rows, final item max, assumptions. Panics on `Actual`.
fn needed_outlook(p: &Planner, course_id: i64) -> (Vec<NeededRow>, f64, Vec<Assumption>) {
    match p.course_marks(course_id).unwrap().outlook {
        Some(FinalOutlook::Needed {
            rows,
            final_item_max,
            assumptions,
        }) => (rows, final_item_max, assumptions),
        other => panic!("expected a Needed outlook, got {other:?}"),
    }
}

/// The row for `letter`; panics when absent.
fn row<'a>(rows: &'a [NeededRow], letter: &str) -> &'a NeededRow {
    rows.iter()
        .find(|r| r.letter == letter)
        .unwrap_or_else(|| panic!("no row for {letter}"))
}

/// Asserts a row needs exactly `expected` (within float noise for scores).
fn assert_needed(rows: &[NeededRow], letter: &str, expected: Needed) {
    match (row(rows, letter).needed, expected) {
        (Needed::Score(a), Needed::Score(e)) => {
            assert!((a - e).abs() < 1e-9, "{letter}: needs {a}, expected {e}")
        }
        (a, e) => assert_eq!(a, e, "{letter}"),
    }
}

/// R38 (T39): 3-credit Theory, attendance 92%, quizzes 12, 11, 13, 14, mid 60 of 75, no final:
/// the final (item max 150) needs 111.0 for an A+ and 96.0 for an A, and so on down the scale.
#[test]
fn t39_needed_in_final() {
    let mut p = planner();
    let course_id = t39_course(&mut p);

    let (rows, final_item_max, assumptions) = needed_outlook(&p, course_id);
    close(final_item_max, 150.0);
    let letters: Vec<&str> = rows.iter().map(|r| r.letter.as_str()).collect();
    assert_eq!(letters, LETTERS.to_vec());
    let expected = [
        ("A+", Needed::Score(111.0)),
        ("A", Needed::Score(96.0)),
        ("A-", Needed::Score(81.0)),
        ("B+", Needed::Score(66.0)),
        ("B", Needed::Score(51.0)),
        ("B-", Needed::Score(36.0)),
        ("C+", Needed::Score(21.0)),
        ("C", Needed::Score(6.0)),
        ("D", Needed::Secured),
        ("F", Needed::Secured),
    ];
    for (letter, needed) in expected {
        assert_needed(&rows, letter, needed);
    }
    assert!(assumptions.is_empty(), "{assumptions:?}");
}

/// R38 (T39, OQ-1 A): a missing quiz counts as 0, raising the need to 114.0 (A+) and 99.0
/// (A), and the outlook lists one assumed-0 quiz.
#[test]
fn t39_missing_quiz_listed_as_assumed_zero() {
    let mut p = planner();
    let course = graded_course(&mut p, "CSE101", 3.0, "Theory");
    mark_attendance(&mut p, course.id, 23, 2);
    score_quizzes(&mut p, course.id, &[12.0, 11.0, 13.0]);
    let mid = component_id(&p, "Theory", "Mid");
    scored(&mut p, course.id, mid, "Mid", 60.0);

    let (rows, _, assumptions) = needed_outlook(&p, course.id);
    assert_needed(&rows, "A+", Needed::Score(114.0));
    assert_needed(&rows, "A", Needed::Score(99.0));
    assert_eq!(
        assumptions,
        vec![Assumption {
            component_name: "Quizzes".into(),
            missing_items: 1
        }]
    );
}

/// R38 (T39): a grade whose need is above the final item max is not reachable; one at or
/// below 0 is secured. Only a mid score of 0 of 75 is entered, so an A+ needs 240 of 150.
#[test]
fn r38_not_reachable() {
    let mut p = planner();
    let course = graded_course(&mut p, "CSE101", 3.0, "Theory");
    let mid = component_id(&p, "Theory", "Mid");
    scored(&mut p, course.id, mid, "Mid", 0.0);

    let (rows, final_item_max, _) = needed_outlook(&p, course.id);
    close(final_item_max, 150.0);
    assert_needed(&rows, "A+", Needed::NotReachable);
    assert_needed(&rows, "B-", Needed::NotReachable);
    assert_needed(&rows, "D", Needed::Score(120.0));
    assert_needed(&rows, "F", Needed::Secured);
}

/// R38 (T39): the needed score is rounded up to 1 decimal. 1.5-credit Theory has final marks
/// and item max of 75; with only mid 23.33 a D needs 60 - 23.33 = 36.67, shown as 36.7, and an
/// A+ needs 96.67, which is above 75.
#[test]
fn r38_needed_rounds_up() {
    let mut p = planner();
    let course = graded_course(&mut p, "CSE101", 1.5, "Theory");
    let mid = component_id(&p, "Theory", "Mid");
    scored(&mut p, course.id, mid, "Mid", 23.33);

    let (rows, final_item_max, _) = needed_outlook(&p, course.id);
    close(final_item_max, 75.0);
    assert_needed(&rows, "D", Needed::Score(36.7));
    assert_needed(&rows, "A+", Needed::NotReachable);
}

/// R38 (T39): float noise must not push a needed score up by 0.1. The worked example needs
/// 111.0 for an A+ (not 111.1), and a 1.5-credit course with only mid 32.5 needs 57.5 for a B
/// (60% of 150 = 90), where the raw float result is 57.50000000000001 (not 57.6).
#[test]
fn r38_float_noise_does_not_round_up() {
    let mut p = planner();
    let worked = t39_course(&mut p);
    let (rows, _, _) = needed_outlook(&p, worked);
    match row(&rows, "A+").needed {
        Needed::Score(v) => assert_eq!(v, 111.0),
        other => panic!("A+: {other:?}"),
    }

    let course = graded_course(&mut p, "CSE102", 1.5, "Theory");
    let mid = component_id(&p, "Theory", "Mid");
    scored(&mut p, course.id, mid, "Mid", 32.5);
    let (rows, _, _) = needed_outlook(&p, course.id);
    match row(&rows, "B").needed {
        Needed::Score(v) => assert_eq!(v, 57.5),
        other => panic!("B: {other:?}"),
    }
}

/// R38 (T39): once the final has a score, the outlook shows the actual grade, the same letter
/// as `course_marks`.
#[test]
fn r38_final_scored_shows_actual_grade() {
    let mut p = planner();
    let course_id = t39_course(&mut p);
    let final_id = component_id(&p, "Theory", "Final");
    scored(&mut p, course_id, final_id, "Final", 120.0);

    let marks = p.course_marks(course_id).unwrap();
    assert!(marks.letter.is_some());
    assert_eq!(
        marks.outlook,
        Some(FinalOutlook::Actual {
            letter: marks.letter.clone()
        })
    );
}

/// R38 (T39, research D8): attendance with no marked session earns 0, adds 0 to secured, and is
/// not listed as an assumption (only missing items are). Only mid 60 is entered: the 4 missing
/// quizzes are listed, attendance is not, and an A- needs 210 - 60 = 150.
#[test]
fn r38_unassessed_attendance_counts_zero_and_is_not_listed() {
    let mut p = planner();
    let course = graded_course(&mut p, "CSE101", 3.0, "Theory");
    let mid = component_id(&p, "Theory", "Mid");
    scored(&mut p, course.id, mid, "Mid", 60.0);

    let (rows, _, assumptions) = needed_outlook(&p, course.id);
    assert_needed(&rows, "A", Needed::NotReachable);
    assert_needed(&rows, "A-", Needed::Score(150.0));
    assert_needed(&rows, "B+", Needed::Score(135.0));
    assert_eq!(
        assumptions,
        vec![Assumption {
            component_name: "Quizzes".into(),
            missing_items: 4
        }]
    );
}
