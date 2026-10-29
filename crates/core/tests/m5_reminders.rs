//! M5 reminders (FR-001..FR-005, docs/spec.md §5): `due_reminders` dedupe, windows, and order.

mod common;

use chrono::NaiveDateTime;
use common::*;
use planner_core::{
    AssessmentInput, AssessmentKind, Planner, Reminder, ReminderKind, Settings, Status,
};

/// A moment with seconds, `YYYY-MM-DDTHH:MM:SS`.
fn dts(s: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").unwrap()
}

fn set_lead(p: &mut Planner, minutes: u32) {
    p.save_settings(&Settings {
        class_reminder_minutes: minutes,
        ..Settings::default()
    })
    .unwrap();
}

fn session_reminder(ref_id: i64, fire_at: &str, title: &str) -> Reminder {
    Reminder {
        kind: ReminderKind::Session,
        ref_id,
        fire_at: dt(fire_at),
        title: title.into(),
    }
}

fn assessment_with_reminder(
    course_id: i64,
    title: &str,
    due: &str,
    minutes: Option<u32>,
) -> AssessmentInput {
    AssessmentInput {
        remind_minutes_before: minutes,
        component_id: None,
        score: None,
        ..assessment_input(course_id, AssessmentKind::Assignment, title, due)
    }
}

// ---- US1: session reminders (T001) ----

/// FR-001, FR-002, FR-005, §5: a due session reminder is returned once and logged.
#[test]
fn m5_session_reminder_fires_once() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let session = regular_on(&p, cse.id, "2026-01-05");
    assert_eq!(
        p.due_reminders(dt("2026-01-05T08:45")).unwrap(),
        vec![session_reminder(
            session.id,
            "2026-01-05T08:45",
            "CSE101 Intro to Programming at 09:00 in A-101"
        )]
    );
    assert!(p.due_reminders(dt("2026-01-05T08:46")).unwrap().is_empty());
    assert!(p.due_reminders(dt("2026-01-05T08:59")).unwrap().is_empty());
}

/// FR-001, FR-004, §5: a call before `fire_at` returns nothing and logs nothing.
#[test]
fn m5_session_reminder_not_before_fire_at() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let session = regular_on(&p, cse.id, "2026-01-05");
    assert!(p.due_reminders(dt("2026-01-05T08:44")).unwrap().is_empty());
    let due = p.due_reminders(dt("2026-01-05T08:45")).unwrap();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].ref_id, session.id);
}

/// FR-004, §5: a reminder never returned before the class starts is skipped, not fired late.
#[test]
fn m5_missed_session_reminder_skipped() {
    let (mut p, _, _, _, _) = synced_fixture();
    assert!(p.due_reminders(dt("2026-01-05T09:00")).unwrap().is_empty());
    assert!(p.due_reminders(dt("2026-01-05T09:30")).unwrap().is_empty());
    let due = p.due_reminders(dt("2026-01-07T10:45")).unwrap();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].fire_at, dt("2026-01-07T10:45"));
}

/// FR-002, §5: a cancelled session gets no reminder.
#[test]
fn m5_cancelled_session_no_reminder() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let session = regular_on(&p, cse.id, "2026-01-05");
    p.set_status(session.id, Status::Cancelled, dt("2026-01-05T08:00"))
        .unwrap();
    assert!(p.due_reminders(dt("2026-01-05T08:45")).unwrap().is_empty());
}

/// FR-002, §5: a rescheduled session is cancelled (no reminder); its makeup is reminded.
#[test]
fn m5_rescheduled_session_reminds_makeup_only() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let original = regular_on(&p, cse.id, "2026-01-05");
    let makeup = p
        .reschedule(original.id, makeup_input(cse.id, "2026-01-06", "14:00"))
        .unwrap();
    assert!(p.due_reminders(dt("2026-01-05T08:45")).unwrap().is_empty());
    assert_eq!(
        p.due_reminders(dt("2026-01-06T13:45")).unwrap(),
        vec![session_reminder(
            makeup.id,
            "2026-01-06T13:45",
            "CSE101 Intro to Programming at 14:00 in A-101"
        )]
    );
}

/// FR-002, §5: a makeup session is reminded like any pending session.
#[test]
fn m5_makeup_session_reminded() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let makeup = p
        .add_makeup(makeup_input(cse.id, "2026-01-08", "16:00"))
        .unwrap();
    assert_eq!(
        p.due_reminders(dt("2026-01-08T15:45")).unwrap(),
        vec![session_reminder(
            makeup.id,
            "2026-01-08T15:45",
            "CSE101 Intro to Programming at 16:00 in A-101"
        )]
    );
    assert!(p.due_reminders(dt("2026-01-08T15:50")).unwrap().is_empty());
}

/// FR-002, §5: only pending sessions are reminded; a marked session is not.
#[test]
fn m5_marked_session_no_reminder() {
    let (mut p, _, cse, _, _) = synced_fixture();
    set_lead(&mut p, 0);
    let session = regular_on(&p, cse.id, "2026-01-05");
    p.set_status(session.id, Status::Attended, dt("2026-01-05T09:00"))
        .unwrap();
    assert!(p
        .due_reminders(dts("2026-01-05T09:00:30"))
        .unwrap()
        .is_empty());
}

/// FR-002, §5: the `class_reminder_minutes` setting sets the session lead time.
#[test]
fn m5_class_reminder_minutes_setting_used() {
    let (mut p, _, _, _, _) = synced_fixture();
    set_lead(&mut p, 30);
    assert!(p.due_reminders(dt("2026-01-05T08:29")).unwrap().is_empty());
    let due = p.due_reminders(dt("2026-01-05T08:30")).unwrap();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].fire_at, dt("2026-01-05T08:30"));
    assert!(p.due_reminders(dt("2026-01-05T08:45")).unwrap().is_empty());
}

/// FR-004, §5: with lead 0 the reminder is due during the start minute, once.
#[test]
fn m5_zero_lead_session_reminder() {
    let (mut p, _, _, _, _) = synced_fixture();
    set_lead(&mut p, 0);
    assert!(p.due_reminders(dt("2026-01-05T08:59")).unwrap().is_empty());
    let due = p.due_reminders(dts("2026-01-05T09:00:30")).unwrap();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].fire_at, dt("2026-01-05T09:00"));
    assert!(p
        .due_reminders(dts("2026-01-05T09:00:45"))
        .unwrap()
        .is_empty());
    assert!(p.due_reminders(dt("2026-01-07T11:01")).unwrap().is_empty());
}

// ---- US2: assessment reminders (T002) ----

/// FR-001, FR-003, FR-005, §5: a due assessment reminder is returned once and logged.
#[test]
fn m5_assessment_reminder_fires_once() {
    let (mut p, _, cse) = fixture();
    let a = p
        .create_assessment(assessment_with_reminder(
            cse.id,
            "HW1",
            "2026-01-07T23:59",
            Some(60),
        ))
        .unwrap();
    assert!(p.due_reminders(dt("2026-01-07T22:58")).unwrap().is_empty());
    assert_eq!(
        p.due_reminders(dt("2026-01-07T22:59")).unwrap(),
        vec![Reminder {
            kind: ReminderKind::Assessment,
            ref_id: a.id,
            fire_at: dt("2026-01-07T22:59"),
            title: "CSE101 Assignment: HW1 due January 7, 2026, 23:59".into(),
        }]
    );
    assert!(p.due_reminders(dt("2026-01-07T23:30")).unwrap().is_empty());
}

/// FR-003, §5: an assessment without `remind_minutes_before` is never reminded.
#[test]
fn m5_assessment_without_reminder_minutes_skipped() {
    let (mut p, _, cse) = fixture();
    p.create_assessment(assessment_with_reminder(
        cse.id,
        "HW1",
        "2026-01-07T23:59",
        None,
    ))
    .unwrap();
    for now in ["2026-01-07T22:59", "2026-01-07T23:30", "2026-01-07T23:58"] {
        assert!(p.due_reminders(dt(now)).unwrap().is_empty());
    }
}

/// FR-003, §5: a completed assessment gets no reminder.
#[test]
fn m5_completed_assessment_no_reminder() {
    let (mut p, _, cse) = fixture();
    let a = p
        .create_assessment(assessment_with_reminder(
            cse.id,
            "HW1",
            "2026-01-07T23:59",
            Some(60),
        ))
        .unwrap();
    p.set_completed(a.id, true).unwrap();
    assert!(p.due_reminders(dt("2026-01-07T22:59")).unwrap().is_empty());
}

/// FR-004, FR-005, §5: an assessment reminder never returned before the due time is skipped.
#[test]
fn m5_missed_assessment_reminder_skipped() {
    let (mut p, _, cse) = fixture();
    p.create_assessment(assessment_with_reminder(
        cse.id,
        "HW1",
        "2026-01-07T23:59",
        Some(60),
    ))
    .unwrap();
    assert!(p.due_reminders(dt("2026-01-07T23:59")).unwrap().is_empty());
    assert!(p.due_reminders(dt("2026-01-08T00:00")).unwrap().is_empty());
}

/// FR-001, FR-003, FR-004, §5: a changed due time gives a new `fire_at`, reminded once more.
#[test]
fn m5_assessment_due_change_fires_new_reminder() {
    let (mut p, _, cse) = fixture();
    let a = p
        .create_assessment(assessment_with_reminder(
            cse.id,
            "HW1",
            "2026-01-07T23:59",
            Some(60),
        ))
        .unwrap();
    assert_eq!(p.due_reminders(dt("2026-01-07T22:59")).unwrap().len(), 1);
    p.update_assessment(
        a.id,
        assessment_with_reminder(cse.id, "HW1", "2026-01-08T23:59", Some(60)),
    )
    .unwrap();
    assert!(p.due_reminders(dt("2026-01-07T23:00")).unwrap().is_empty());
    let due = p.due_reminders(dt("2026-01-08T22:59")).unwrap();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].fire_at, dt("2026-01-08T22:59"));
    assert_eq!(
        due[0].title,
        "CSE101 Assignment: HW1 due January 8, 2026, 23:59"
    );
    assert!(p.due_reminders(dt("2026-01-08T23:00")).unwrap().is_empty());
}

/// FR-004, FR-003, §5: with lead 0 the assessment reminder is due during the due minute, once.
#[test]
fn m5_zero_lead_assessment_reminder() {
    let (mut p, _, cse) = fixture();
    p.create_assessment(assessment_with_reminder(
        cse.id,
        "HW1",
        "2026-01-07T23:59",
        Some(0),
    ))
    .unwrap();
    assert!(p.due_reminders(dt("2026-01-07T23:58")).unwrap().is_empty());
    let due = p.due_reminders(dt("2026-01-07T23:59")).unwrap();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].fire_at, dt("2026-01-07T23:59"));
    assert!(p
        .due_reminders(dts("2026-01-07T23:59:40"))
        .unwrap()
        .is_empty());
}

// ---- Cross-cutting (T003) ----

/// FR-001, FR-005, §5, edge "Several due at once": one call returns all, ordered, once.
#[test]
fn m5_several_due_reminders_ordered() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let mth = p.create_course(course_input("MTH201")).unwrap();
    p.add_slot(slot_a(mth.id)).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    let early = p
        .create_assessment(assessment_with_reminder(
            cse.id,
            "HW1",
            "2026-01-05T09:00",
            Some(20),
        ))
        .unwrap();
    let late = p
        .create_assessment(assessment_with_reminder(
            cse.id,
            "HW2",
            "2026-01-05T09:00",
            Some(15),
        ))
        .unwrap();
    let cse_session = regular_on(&p, cse.id, "2026-01-05");
    let mth_session = regular_on(&p, mth.id, "2026-01-05");

    let due = p.due_reminders(dt("2026-01-05T08:50")).unwrap();
    let got: Vec<(ReminderKind, i64, NaiveDateTime)> =
        due.iter().map(|r| (r.kind, r.ref_id, r.fire_at)).collect();
    assert_eq!(
        got,
        vec![
            (ReminderKind::Assessment, early.id, dt("2026-01-05T08:40")),
            (
                ReminderKind::Session,
                cse_session.id,
                dt("2026-01-05T08:45")
            ),
            (
                ReminderKind::Session,
                mth_session.id,
                dt("2026-01-05T08:45")
            ),
            (ReminderKind::Assessment, late.id, dt("2026-01-05T08:45")),
        ]
    );
    assert_eq!(due[2].title, "MTH201 MTH201 course at 09:00 in A-101");
    assert!(p.due_reminders(dt("2026-01-05T08:51")).unwrap().is_empty());
}

/// FR-002, FR-003, §5: an archived course has no session or assessment reminders.
#[test]
fn m5_archived_course_no_reminders() {
    let (mut p, _, cse, _, _) = synced_fixture();
    p.create_assessment(assessment_with_reminder(
        cse.id,
        "HW1",
        "2026-01-05T12:00",
        Some(60),
    ))
    .unwrap();
    p.set_archived(cse.id, true).unwrap();
    assert!(p.due_reminders(dt("2026-01-05T08:45")).unwrap().is_empty());
    assert!(p.due_reminders(dt("2026-01-05T11:00")).unwrap().is_empty());
}

/// FR-002, §5, edge "No active term": no term means `Ok` and no reminders.
#[test]
fn m5_no_active_term_no_reminders() {
    let mut p = Planner::open_in_memory().unwrap();
    assert!(p.due_reminders(dt("2026-01-05T08:45")).unwrap().is_empty());
}

/// FR-002, FR-003, R1, §5: sessions and assessments of a non-active term are not reminded.
#[test]
fn m5_inactive_term_no_reminders() {
    let (mut p, _, cse, _, _) = synced_fixture();
    p.create_assessment(assessment_with_reminder(
        cse.id,
        "HW1",
        "2026-01-07T23:59",
        Some(60),
    ))
    .unwrap();
    p.save_term(planner_core::TermInput {
        id: None,
        name: "Spring 2026".into(),
        start_date: d("2026-02-01"),
        end_date: d("2026-02-28"),
    })
    .unwrap();
    assert!(p.due_reminders(dt("2026-01-05T08:45")).unwrap().is_empty());
    assert!(p.due_reminders(dt("2026-01-07T22:59")).unwrap().is_empty());
}

/// FR-006, §5, research R-6: core ignores `notifications_enabled`; the app gates the call.
#[test]
fn m5_notifications_switch_ignored_by_core() {
    let (mut p, _, _, _, _) = synced_fixture();
    p.save_settings(&Settings {
        notifications_enabled: false,
        ..Settings::default()
    })
    .unwrap();
    assert_eq!(p.due_reminders(dt("2026-01-05T08:45")).unwrap().len(), 1);
}

/// FR-001, R22, §5, edge "Restore": restore leaves `reminder_log` untouched, no refire.
#[test]
fn m5_restore_does_not_refire() {
    let (mut p, _, _, _, _) = synced_fixture();
    assert_eq!(p.due_reminders(dt("2026-01-05T08:45")).unwrap().len(), 1);
    let json = p.export_json(dt("2026-01-05T08:45")).unwrap();
    p.restore_json(&json).unwrap();
    assert!(p.due_reminders(dt("2026-01-05T08:45")).unwrap().is_empty());
    assert!(p.due_reminders(dt("2026-01-05T08:46")).unwrap().is_empty());
}
