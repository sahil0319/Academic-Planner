//! Dashboard: today's classes, next class, upcoming, overdue, awaiting attendance.

mod common;

use common::*;
use planner_core::{AssessmentKind, SlotInput, Status};

fn titles(list: &[planner_core::Assessment]) -> Vec<&str> {
    list.iter().map(|a| a.title.as_str()).collect()
}

/// R20: a session in progress is not awaiting attendance; at its end moment it is (T13).
#[test]
fn t13_awaiting_excludes_in_progress_includes_at_end() {
    let (p, _, cse, _, _) = synced_fixture();
    let first = regular_on(&p, cse.id, "2026-01-05");

    let in_progress = p.dashboard(dt("2026-01-05T09:30")).unwrap();
    assert!(in_progress
        .awaiting
        .iter()
        .all(|v| v.session.id != first.id));

    let ended = p.dashboard(dt("2026-01-05T10:00")).unwrap();
    let ids: Vec<i64> = ended.awaiting.iter().map(|v| v.session.id).collect();
    assert_eq!(ids, [first.id]);
    assert_eq!(ended.awaiting[0].course_code, "CSE101");
}

/// R19: an incomplete assessment due before now is overdue; a completed one is not (T16).
#[test]
fn t16_overdue_only_incomplete_and_strictly_before_now() {
    let (mut p, _, cse) = fixture();
    let late = p
        .create_assessment(assessment_input(
            cse.id,
            AssessmentKind::Quiz,
            "Late",
            "2026-01-09T10:00",
        ))
        .unwrap();
    let done = p
        .create_assessment(assessment_input(
            cse.id,
            AssessmentKind::Quiz,
            "Done",
            "2026-01-09T11:00",
        ))
        .unwrap();
    p.create_assessment(assessment_input(
        cse.id,
        AssessmentKind::Quiz,
        "Due now",
        "2026-01-10T12:00",
    ))
    .unwrap();
    p.set_completed(done.id, true).unwrap();

    let board = p.dashboard(dt("2026-01-10T12:00")).unwrap();
    // "Due now" is not strictly before now.
    assert_eq!(titles(&board.overdue), ["Late"]);
    assert_eq!(board.overdue[0].id, late.id);
}

/// R18: an assessment due exactly now + 7 days is upcoming; one minute later is not (T17).
#[test]
fn t17_upcoming_includes_due_at_window_edge() {
    let (mut p, _, cse) = fixture();
    for (title, due) in [
        ("Edge", "2026-01-12T09:00"),
        ("Too late", "2026-01-12T09:01"),
        ("Now", "2026-01-05T09:00"),
    ] {
        p.create_assessment(assessment_input(cse.id, AssessmentKind::Quiz, title, due))
            .unwrap();
    }

    let board = p.dashboard(dt("2026-01-05T09:00")).unwrap();
    assert_eq!(titles(&board.upcoming), ["Now", "Edge"]);
    assert!(board.overdue.is_empty());
}

/// R17: with the earliest future session cancelled, next class is the following pending one (T18).
#[test]
fn t18_next_class_skips_cancelled_session() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let first = regular_on(&p, cse.id, "2026-01-05");
    let second = regular_on(&p, cse.id, "2026-01-07");
    mark(&mut p, first.id, Status::Cancelled, dt("2026-01-05T08:00"));

    let board = p.dashboard(dt("2026-01-05T08:00")).unwrap();
    let next = board.next_class.expect("a next class");
    assert_eq!(next.session.id, second.id);
}

/// R17: a session starting exactly now is not "next" (strictly after now) (T18).
#[test]
fn t18_session_starting_now_is_not_next() {
    let (p, _, cse, _, _) = synced_fixture();
    let second = regular_on(&p, cse.id, "2026-01-07");

    let board = p.dashboard(dt("2026-01-05T09:00")).unwrap();
    let next = board.next_class.expect("a next class");
    assert_eq!(next.session.id, second.id);
}

/// R16: today's classes of two courses are sorted by start time, each with its status (T29).
#[test]
fn t29_today_sorted_by_start_time_across_courses() {
    let (mut p, _, _, _, _) = synced_fixture();
    let mth = p.create_course(course_input("MTH110")).unwrap();
    for (start, end) in [("13:00", "14:00"), ("08:00", "08:50")] {
        p.add_slot(SlotInput {
            start_time: t(start),
            end_time: t(end),
            ..slot_a(mth.id)
        })
        .unwrap();
    }
    p.sync_sessions(d("2026-01-05")).unwrap();
    let afternoon = sessions_of(&p, mth.id)
        .into_iter()
        .find(|s| s.date == d("2026-01-05") && s.start_time == t("13:00"))
        .expect("MTH110 13:00 session");
    mark(
        &mut p,
        afternoon.id,
        Status::Cancelled,
        dt("2026-01-05T08:00"),
    );

    let board = p.dashboard(dt("2026-01-05T08:00")).unwrap();
    let rows: Vec<(String, String, String)> = board
        .today
        .iter()
        .map(|v| {
            (
                v.session.start_time.format("%H:%M").to_string(),
                v.course_code.clone(),
                v.display_status.clone(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            ("08:00".into(), "MTH110".into(), "Pending".into()),
            ("09:00".into(), "CSE101".into(), "Pending".into()),
            ("13:00".into(), "MTH110".into(), "Cancelled".into()),
        ]
    );
    assert!(board
        .today
        .iter()
        .all(|v| v.session.date == d("2026-01-05")));
}
