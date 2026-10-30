//! M11: deleting courses with everything that belongs to them (R51).

mod common;

use common::*;
use planner_core::{AssessmentFilter, AssessmentKind, Error, Planner};
use rusqlite::Connection;
use tempfile::TempDir;

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

/// Ids of the courses of the active term, archived included.
fn course_ids(p: &Planner) -> Vec<i64> {
    p.list_courses(true)
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect()
}

/// R51: the deleted courses lose their slots, sessions, and assessments; the kept course, the
/// term with its holidays, and the rule sets are unchanged, and sync recreates nothing (T51).
#[test]
fn t51_delete_two_of_three_courses() {
    let (mut p, term, cse, _, _) = synced_fixture();
    let mat = p.create_course(course_input("MAT102")).unwrap();
    p.add_slot(slot_a(mat.id)).unwrap();
    let phy = p.create_course(course_input("PHY103")).unwrap();
    p.add_slot(slot_b(phy.id)).unwrap();
    p.set_holidays(term.id, &holidays(&["2026-01-19"])).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    for id in [cse.id, mat.id, phy.id] {
        p.create_assessment(assessment_input(
            id,
            AssessmentKind::Quiz,
            "Quiz 1",
            "2026-01-20T10:00",
        ))
        .unwrap();
    }
    p.add_makeup(makeup_input(cse.id, "2026-01-10", "09:00"))
        .unwrap();
    p.set_archived(mat.id, true).unwrap();

    let phy_sessions = sessions_of(&p, phy.id);
    assert!(!phy_sessions.is_empty());
    let term_before = p.active_term().unwrap();
    let rule_sets_before = p.list_rule_sets().unwrap();

    p.delete_courses(&[cse.id, mat.id]).unwrap();

    assert_eq!(course_ids(&p), vec![phy.id]);
    for id in [cse.id, mat.id] {
        assert!(matches!(
            p.sessions_for_course(id, after_term()),
            Err(Error::NotFound { .. })
        ));
    }
    let slots = p.list_slots(d("2026-01-12")).unwrap();
    assert!(!slots.is_empty());
    assert!(slots.iter().all(|s| s.course_id == phy.id));
    let assessments = p.list_assessments(AssessmentFilter::All).unwrap();
    assert_eq!(assessments.len(), 1);
    assert_eq!(assessments[0].course_id, phy.id);

    let report = p.sync_sessions(d("2026-01-05")).unwrap();
    assert_eq!((report.created, report.deleted), (0, 0));
    assert_eq!(sessions_of(&p, phy.id), phy_sessions);
    assert_eq!(p.active_term().unwrap(), term_before);
    assert_eq!(p.list_rule_sets().unwrap(), rule_sets_before);
}

/// R51: an unknown id fails the whole call with `NotFound` and deletes nothing (T52).
#[test]
fn t52_unknown_id_deletes_nothing() {
    let (mut p, _, cse, _, _) = synced_fixture();
    p.create_assessment(assessment_input(
        cse.id,
        AssessmentKind::Exam,
        "Midterm",
        "2026-01-15T10:00",
    ))
    .unwrap();
    let sessions = sessions_of(&p, cse.id);

    let err = p.delete_courses(&[cse.id, 999]).unwrap_err();

    assert!(matches!(
        err,
        Error::NotFound {
            entity: "course",
            id: 999
        }
    ));
    assert_eq!(course_ids(&p), vec![cse.id]);
    assert_eq!(sessions_of(&p, cse.id), sessions);
    assert_eq!(p.list_assessments(AssessmentFilter::All).unwrap().len(), 1);
}

/// R51: reminder log rows of the deleted sessions and assessments go too, a makeup of another
/// course that replaced a deleted session loses the link, and an empty list or duplicate ids
/// are fine (T53).
#[test]
fn t53_reminder_log_and_replacement_links_cleaned() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("planner.sqlite");
    let (cse, mat, makeup_id, session_id, assessment_id, kept_assessment_id) = {
        let mut p = Planner::open(&path).unwrap();
        p.save_term(fixture_term()).unwrap();
        let cse = p.create_course(fixture_course()).unwrap();
        p.add_slot(slot_a(cse.id)).unwrap();
        let mat = p.create_course(course_input("MAT102")).unwrap();
        p.sync_sessions(d("2026-01-05")).unwrap();
        let original = regular_on(&p, cse.id, "2026-01-12");
        let makeup = p
            .reschedule(original.id, makeup_input(mat.id, "2026-01-14", "15:00"))
            .unwrap();
        let assessment = p
            .create_assessment(assessment_input(
                cse.id,
                AssessmentKind::Quiz,
                "Quiz 1",
                "2026-01-20T10:00",
            ))
            .unwrap();
        let kept = p
            .create_assessment(assessment_input(
                mat.id,
                AssessmentKind::Quiz,
                "Quiz 1",
                "2026-01-20T10:00",
            ))
            .unwrap();
        (cse, mat, makeup.id, original.id, assessment.id, kept.id)
    };

    let conn = Connection::open(&path).unwrap();
    for (kind, id) in [
        ("session", session_id),
        ("assessment", assessment_id),
        ("session", makeup_id),
        ("assessment", kept_assessment_id),
    ] {
        conn.execute(
            "INSERT INTO reminder_log (kind, ref_id, fire_at) VALUES (?1, ?2, '2026-01-01T09:00')",
            rusqlite::params![kind, id],
        )
        .unwrap();
    }

    let mut p = Planner::open(&path).unwrap();
    p.delete_courses(&[]).unwrap();
    assert_eq!(course_ids(&p), vec![cse.id, mat.id]);
    p.delete_courses(&[cse.id, cse.id]).unwrap();

    assert_eq!(course_ids(&p), vec![mat.id]);
    let mat_sessions = sessions_of(&p, mat.id);
    assert_eq!(mat_sessions.len(), 1);
    assert_eq!(mat_sessions[0].id, makeup_id);
    assert_eq!(mat_sessions[0].replaces_session_id, None);
    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM reminder_log"),
        2,
        "only the kept makeup's and kept assessment's log rows remain"
    );
    assert_eq!(
        count(
            &conn,
            &format!(
                "SELECT COUNT(*) FROM reminder_log WHERE ref_id IN ({makeup_id}, {kept_assessment_id})"
            )
        ),
        2
    );
}
