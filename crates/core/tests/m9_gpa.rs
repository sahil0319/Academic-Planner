//! M9 term GPA (R40, T41).

mod common;

use common::*;
use planner_core::{Course, GpaStatus, GpaSummary, Planner};

fn planner() -> Planner {
    let mut p = Planner::open_in_memory().unwrap();
    p.save_term(fixture_term()).unwrap();
    p
}

fn close(actual: Option<f64>, expected: f64) {
    let actual = actual.unwrap_or_else(|| panic!("gpa is None, expected {expected}"));
    assert!(
        (actual - expected).abs() < 1e-9,
        "{actual} is not {expected}"
    );
}

/// A Theory course with only a mid score (a projected percentage but no other marks).
fn mid_only(p: &mut Planner, code: &str, credits: f64, mid_score: f64) -> Course {
    let course = graded_course(p, code, credits, "Theory");
    let mid = component_id(p, "Theory", "Mid");
    scored(p, course.id, mid, "Mid", mid_score);
    course
}

/// A 3-credit Theory course with attendance marked, `quizzes` quiz scores, a mid and a final.
fn full_course(p: &mut Planner, quizzes: usize) -> Course {
    let course = graded_course(p, "CSE101", 3.0, "Theory");
    mark_attendance(p, course.id, 23, 2);
    let quiz_component = component_id(p, "Theory", "Quizzes");
    for i in 0..quizzes {
        scored(
            p,
            course.id,
            quiz_component,
            &format!("Quiz {}", i + 1),
            12.0,
        );
    }
    let mid = component_id(p, "Theory", "Mid");
    scored(p, course.id, mid, "Mid", 60.0);
    let final_id = component_id(p, "Theory", "Final");
    scored(p, course.id, final_id, "Final", 120.0);
    course
}

/// R40 (T41): a 3-credit A+ (80.0%, 4.00) and a 1.5-credit B (62.0%, 3.00) give
/// (4.00 x 3 + 3.00 x 1.5) / 4.5 = 3.67, projected, with two courses included and none left out.
#[test]
fn t41_gpa_three_credit_a_plus_and_one_and_half_credit_b() {
    let mut p = planner();
    let a_plus = mid_only(&mut p, "CSE101", 3.0, 60.0);
    let b = mid_only(&mut p, "CSE102", 1.5, 23.25);
    assert_eq!(
        p.course_marks(a_plus.id).unwrap().letter.as_deref(),
        Some("A+")
    );
    assert_eq!(p.course_marks(b.id).unwrap().letter.as_deref(), Some("B"));

    let gpa = p.term_gpa().unwrap();
    close(gpa.gpa, 3.67);
    assert_eq!(gpa.status, GpaStatus::Projected);
    assert_eq!(gpa.included, 2);
    assert_eq!(gpa.left_out, 0);
}

/// R40: with no graded course the GPA is `None`.
#[test]
fn r40_none_when_no_course_qualifies() {
    let mut p = planner();
    p.create_course(course_input("CSE101")).unwrap();

    let gpa = p.term_gpa().unwrap();
    assert_eq!(gpa.gpa, None);
    assert_eq!(gpa.included, 0);
    assert_eq!(gpa.left_out, 0);
}

/// R40: a graded course with nothing assessed is counted as left out and is not averaged.
#[test]
fn r40_left_out_counts_graded_courses_without_marks() {
    let mut p = planner();
    mid_only(&mut p, "CSE101", 3.0, 60.0);
    graded_course(&mut p, "CSE102", 3.0, "Theory");

    let gpa = p.term_gpa().unwrap();
    close(gpa.gpa, 4.0);
    assert_eq!(gpa.included, 1);
    assert_eq!(gpa.left_out, 1);

    let mut only_unmarked = planner();
    graded_course(&mut only_unmarked, "CSE101", 3.0, "Theory");
    let gpa = only_unmarked.term_gpa().unwrap();
    assert_eq!(gpa.gpa, None);
    assert_eq!(gpa.left_out, 1);
}

/// R40, R33: ungraded courses (R33) and archived courses (R3) are neither included nor left
/// out.
#[test]
fn r40_ungraded_and_archived_courses_excluded() {
    let mut p = planner();
    mid_only(&mut p, "CSE101", 3.0, 60.0);
    p.create_course(course_input("CSE102")).unwrap();
    let archived = mid_only(&mut p, "CSE103", 3.0, 30.0);
    let archived_empty = graded_course(&mut p, "CSE104", 3.0, "Theory");
    p.set_archived(archived.id, true).unwrap();
    p.set_archived(archived_empty.id, true).unwrap();

    let gpa = p.term_gpa().unwrap();
    close(gpa.gpa, 4.0);
    assert_eq!(gpa.included, 1);
    assert_eq!(gpa.left_out, 0);
}

/// R40: the GPA is final when every component of every included course is fully assessed
/// (attendance marked, all 4 quiz scores, mid and final).
#[test]
fn r40_final_when_everything_assessed() {
    let mut p = planner();
    full_course(&mut p, 4);

    let gpa = p.term_gpa().unwrap();
    assert_eq!(gpa.status, GpaStatus::Final);
    assert_eq!(gpa.included, 1);
    assert!(gpa.gpa.is_some());
}

/// R40 (OQ-1 A): a missing quiz keeps the GPA projected.
#[test]
fn r40_projected_while_a_quiz_is_missing() {
    let mut p = planner();
    full_course(&mut p, 3);

    let gpa = p.term_gpa().unwrap();
    assert_eq!(gpa.status, GpaStatus::Projected);
    assert_eq!(gpa.included, 1);
}

/// R40 (research D12): without an active term the summary is empty and projected, not an
/// error.
#[test]
fn r40_no_active_term() {
    let p = Planner::open_in_memory().unwrap();

    assert_eq!(
        p.term_gpa().unwrap(),
        GpaSummary {
            gpa: None,
            status: GpaStatus::Projected,
            included: 0,
            left_out: 0
        }
    );
}
