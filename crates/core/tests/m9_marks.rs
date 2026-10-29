//! M9 course marks (R33, R35, R36, R37) and the R36 score checks (T43).

mod common;

use common::*;
use planner_core::{
    AssessmentFilter, AssessmentInput, AssessmentKind, ComponentMarks, CourseInput, CourseMarks,
    Error, Planner,
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
