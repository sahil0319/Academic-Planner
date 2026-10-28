//! User Story 7: midterm and final exam periods entered as holidays on every date of the
//! period, prep days included. No T-ID exists; these reuse the T06/T07 mechanics.

mod common;

use common::*;
use planner_core::{AssessmentKind, Status};

const PERIOD: (&str, &str) = ("2026-01-14", "2026-01-23");

fn set_midterm(p: &mut planner_core::Planner, term_id: i64) {
    p.set_holidays(term_id, &holiday_range(PERIOD.0, PERIOD.1, "Midterm exams"))
        .unwrap();
}

/// R4, R5, FR-006a: a Wed-to-Fri period removes future pending classes inside it, keeps
/// earlier ones, and none of them awaits attendance.
#[test]
fn exam_period_holidays_remove_future_pending_sessions() {
    let (mut p, term, cse, _, _) = synced_fixture();
    set_midterm(&mut p, term.id);

    let report = p.sync_sessions(d("2026-01-09")).unwrap();
    assert_eq!(report.created, 0);
    assert_eq!(report.deleted, 3);
    assert_eq!(
        dates(&sessions_of(&p, cse.id)),
        ["2026-01-05", "2026-01-07", "2026-01-12"]
    );

    // Nothing in the period awaits attendance, and only the three kept sessions count.
    let board = p.dashboard(dt("2026-01-24T00:00")).unwrap();
    assert!(board
        .awaiting
        .iter()
        .all(|v| v.session.date < d(PERIOD.0) || v.session.date > d(PERIOD.1)));
    let summary = p.attendance_summary(cse.id).unwrap();
    assert_eq!(summary.pending, 3);
    assert_pct(summary.pct, None);
}

/// R4, R18, R19, FR-006a: exam assessments inside the period behave like any assessment,
/// and the prep days between them have no class session.
#[test]
fn exam_period_exams_appear_as_assessments_and_prep_days_are_class_free() {
    let (mut p, term, cse, _, _) = synced_fixture();
    set_midterm(&mut p, term.id);
    p.sync_sessions(d("2026-01-09")).unwrap();
    for (title, due) in [
        ("Exam 1", "2026-01-15T10:00"),
        ("Exam 2", "2026-01-20T10:00"),
    ] {
        let exam = p
            .create_assessment(assessment_input(cse.id, AssessmentKind::Exam, title, due))
            .unwrap();
        assert_eq!(exam.kind, AssessmentKind::Exam);
    }

    // Prep days 01-16..01-19 include Monday 01-19, which has no class.
    let sessions = sessions_of(&p, cse.id);
    assert!(sessions
        .iter()
        .all(|s| s.date < d("2026-01-16") || s.date > d("2026-01-19")));

    let upcoming = p.dashboard(dt("2026-01-14T09:00")).unwrap();
    let titles: Vec<&str> = upcoming.upcoming.iter().map(|a| a.title.as_str()).collect();
    assert_eq!(titles, ["Exam 1", "Exam 2"]);
    assert!(upcoming.overdue.is_empty());

    let later = p.dashboard(dt("2026-01-21T09:00")).unwrap();
    let titles: Vec<&str> = later.overdue.iter().map(|a| a.title.as_str()).collect();
    assert_eq!(titles, ["Exam 1", "Exam 2"]);
    assert!(later.upcoming.is_empty());
}

/// R4: removing one date from the period makes sync generate that date's class again.
#[test]
fn exam_period_removing_a_date_restores_its_sessions() {
    let (mut p, term, cse, _, _) = synced_fixture();
    set_midterm(&mut p, term.id);
    p.sync_sessions(d("2026-01-09")).unwrap();
    assert!(!dates(&sessions_of(&p, cse.id)).contains(&"2026-01-19".to_string()));

    let shortened: Vec<_> = holiday_range(PERIOD.0, PERIOD.1, "Midterm exams")
        .into_iter()
        .filter(|h| h.date != d("2026-01-19"))
        .collect();
    p.set_holidays(term.id, &shortened).unwrap();
    let report = p.sync_sessions(d("2026-01-09")).unwrap();

    assert_eq!(report.created, 1);
    assert_eq!(report.deleted, 0);
    let restored = regular_on(&p, cse.id, "2026-01-19");
    assert_eq!(restored.status, Status::Pending);
    assert_eq!(restored.start_time, t("09:00"));
}

/// R5: a session already cancelled inside a future exam period is kept when the period is
/// entered; its pending neighbours are removed.
#[test]
fn exam_period_keeps_session_cancelled_before_period_was_entered() {
    let (mut p, term, cse, _, _) = synced_fixture();
    let cancelled = regular_on(&p, cse.id, "2026-01-14");
    mark(
        &mut p,
        cancelled.id,
        Status::Cancelled,
        dt("2026-01-09T08:00"),
    );

    set_midterm(&mut p, term.id);
    let report = p.sync_sessions(d("2026-01-09")).unwrap();

    assert_eq!(report.deleted, 2);
    assert_eq!(
        dates(&sessions_of(&p, cse.id)),
        ["2026-01-05", "2026-01-07", "2026-01-12", "2026-01-14"]
    );
    assert_eq!(
        regular_on(&p, cse.id, "2026-01-14").status,
        Status::Cancelled
    );
}
