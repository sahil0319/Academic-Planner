//! M3 backup export (R21): the versioned JSON document.

mod common;

use std::collections::BTreeSet;

use chrono::NaiveTime;
use common::*;
use planner_core::{
    AssessmentInput, AssessmentKind, Holiday, Planner, Session, SessionKind, Settings, Status,
    TermInput, ThemeMode,
};
use serde_json::{json, Value};

const NOW: &str = "2026-01-25T18:30";

struct Populated {
    p: Planner,
    term_id: i64,
    course_id: i64,
    slot_a_id: i64,
    slot_b_id: i64,
    original_id: i64,
    makeup_id: i64,
    exam_id: i64,
    quiz_id: i64,
}

/// The §8 fixture, synced, with a holiday, the 01-12 session rescheduled to an attended
/// makeup, two assessments (one completed), and changed settings.
fn populated() -> Populated {
    let (mut p, term, course, slot_a, slot_b) = synced_fixture();
    p.set_holidays(
        term.id,
        &[Holiday {
            date: d("2026-01-19"),
            label: Some("Break".into()),
        }],
    )
    .unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();

    let first = regular_on(&p, course.id, "2026-01-05");
    mark(&mut p, first.id, Status::Absent, after_term());

    let original = regular_on(&p, course.id, "2026-01-12");
    let mut input = makeup_input(course.id, "2026-01-13", "14:00");
    input.note = Some("lab swap".into());
    let makeup = p.reschedule(original.id, input).unwrap();
    mark(&mut p, makeup.id, Status::Attended, after_term());

    let exam = p
        .create_assessment(AssessmentInput {
            notes: Some("bring calculator".into()),
            remind_minutes_before: Some(30),
            ..assessment_input(
                course.id,
                AssessmentKind::Exam,
                "Midterm",
                "2026-01-20T10:00",
            )
        })
        .unwrap();
    let quiz = p
        .create_assessment(assessment_input(
            course.id,
            AssessmentKind::Quiz,
            "Quiz 1",
            "2026-01-14T09:00",
        ))
        .unwrap();
    p.set_completed(quiz.id, true).unwrap();

    p.save_settings(&Settings {
        class_reminder_minutes: 30,
        upcoming_window_days: 14,
        notifications_enabled: false,
        theme: ThemeMode::System,
        check_updates_on_start: true,
    })
    .unwrap();

    Populated {
        p,
        term_id: term.id,
        course_id: course.id,
        slot_a_id: slot_a.id,
        slot_b_id: slot_b.id,
        original_id: original.id,
        makeup_id: makeup.id,
        exam_id: exam.id,
        quiz_id: quiz.id,
    }
}

fn export(p: &Planner, now: &str) -> Value {
    serde_json::from_str(&p.export_json(dt(now)).unwrap()).unwrap()
}

fn time_text(time: NaiveTime) -> String {
    time.format("%H:%M").to_string()
}

fn kind_text(kind: SessionKind) -> &'static str {
    match kind {
        SessionKind::Regular => "regular",
        SessionKind::Makeup => "makeup",
    }
}

fn status_text(status: Status) -> &'static str {
    match status {
        Status::Pending => "pending",
        Status::Attended => "attended",
        Status::Absent => "absent",
        Status::Cancelled => "cancelled",
    }
}

fn session_json(s: &Session) -> Value {
    json!({
        "id": s.id,
        "course_id": s.course_id,
        "slot_id": s.slot_id,
        "kind": kind_text(s.kind),
        "date": ymd(s.date),
        "start_time": time_text(s.start_time),
        "end_time": time_text(s.end_time),
        "room": s.room,
        "status": status_text(s.status),
        "replaces_session_id": s.replaces_session_id,
        "note": s.note,
    })
}

fn ids(rows: &Value) -> Vec<i64> {
    rows.as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_i64().unwrap())
        .collect()
}

/// R21: the document has exactly the ten top-level keys, the format name, the app's schema
/// version, and the caller's `now` as `exported_at`.
#[test]
fn r21_export_has_exact_top_level_keys() {
    let pop = populated();
    let doc = export(&pop.p, NOW);

    let keys: BTreeSet<&str> = doc
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let expected: BTreeSet<&str> = [
        "format",
        "schema_version",
        "exported_at",
        "terms",
        "holidays",
        "courses",
        "slots",
        "sessions",
        "assessments",
        "settings",
    ]
    .into_iter()
    .collect();
    assert_eq!(keys, expected);

    assert_eq!(doc["format"], json!("academic-planner-backup"));
    assert_eq!(doc["schema_version"], json!(1));
    assert_eq!(doc["exported_at"], json!("2026-01-25T18:30"));
}

/// R21: every row appears once with its stored id, the §3 column keys, JSON booleans for the
/// flags, JSON null for NULL columns, and the stored text for dates and times.
#[test]
fn r21_export_rows_keep_ids_and_value_types() {
    let pop = populated();
    let doc = export(&pop.p, NOW);

    assert_eq!(
        doc["terms"],
        json!([{
            "id": pop.term_id,
            "name": "Winter 2026",
            "start_date": "2026-01-05",
            "end_date": "2026-01-25",
            "is_active": true,
        }])
    );
    assert_eq!(
        doc["holidays"],
        json!([{"term_id": pop.term_id, "date": "2026-01-19", "label": "Break"}])
    );
    assert_eq!(
        doc["courses"],
        json!([{
            "id": pop.course_id,
            "term_id": pop.term_id,
            "code": "CSE101",
            "name": "Intro to Programming",
            "instructor": null,
            "room": "A-101",
            "notes": null,
            "min_attendance_pct": 75,
            "archived": false,
        }])
    );
    assert_eq!(
        doc["slots"],
        json!([
            {
                "id": pop.slot_a_id,
                "course_id": pop.course_id,
                "weekday": 1,
                "start_time": "09:00",
                "end_time": "10:00",
                "room": null,
                "valid_from": "2026-01-05",
                "valid_to": null,
            },
            {
                "id": pop.slot_b_id,
                "course_id": pop.course_id,
                "weekday": 3,
                "start_time": "11:00",
                "end_time": "12:00",
                "room": null,
                "valid_from": "2026-01-05",
                "valid_to": null,
            },
        ])
    );

    let mut stored = sessions_of(&pop.p, pop.course_id);
    stored.sort_by_key(|s| s.id);
    let expected_sessions: Vec<Value> = stored.iter().map(session_json).collect();
    assert_eq!(doc["sessions"], Value::Array(expected_sessions));
    assert_eq!(ids(&doc["sessions"]).len(), stored.len());

    let rows = doc["sessions"].as_array().unwrap();
    let makeup = rows.iter().find(|r| r["id"] == pop.makeup_id).unwrap();
    assert_eq!(makeup["kind"], json!("makeup"));
    assert_eq!(makeup["slot_id"], Value::Null);
    assert_eq!(makeup["status"], json!("attended"));
    assert_eq!(makeup["note"], json!("lab swap"));
    assert_eq!(makeup["replaces_session_id"], json!(pop.original_id));
    let original = rows.iter().find(|r| r["id"] == pop.original_id).unwrap();
    assert_eq!(original["status"], json!("cancelled"));
    assert_eq!(original["replaces_session_id"], Value::Null);

    assert_eq!(
        doc["assessments"],
        json!([
            {
                "id": pop.exam_id,
                "course_id": pop.course_id,
                "kind": "exam",
                "title": "Midterm",
                "due_at": "2026-01-20T10:00",
                "notes": "bring calculator",
                "completed": false,
                "remind_minutes_before": 30,
            },
            {
                "id": pop.quiz_id,
                "course_id": pop.course_id,
                "kind": "quiz",
                "title": "Quiz 1",
                "due_at": "2026-01-14T09:00",
                "notes": null,
                "completed": true,
                "remind_minutes_before": null,
            },
        ])
    );
    assert_eq!(
        doc["settings"],
        json!({
            "class_reminder_minutes": "30",
            "upcoming_window_days": "14",
            "notifications_enabled": "0",
            "theme": "system",
            "check_updates_on_start": "1",
        })
    );
}

/// R21: rows are ordered by id, holidays by (term_id, date), and `settings` holds every key.
#[test]
fn r21_export_collections_ordered() {
    let (mut p, term1, cse) = fixture();
    p.set_holidays(
        term1.id,
        &[
            Holiday {
                date: d("2026-01-21"),
                label: None,
            },
            Holiday {
                date: d("2026-01-07"),
                label: Some("Mid".into()),
            },
        ],
    )
    .unwrap();
    p.add_slot(slot_b(cse.id)).unwrap();
    p.add_slot(slot_a(cse.id)).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    p.create_assessment(assessment_input(
        cse.id,
        AssessmentKind::Other,
        "Late one",
        "2026-01-24T10:00",
    ))
    .unwrap();
    p.create_assessment(assessment_input(
        cse.id,
        AssessmentKind::Other,
        "Early one",
        "2026-01-08T10:00",
    ))
    .unwrap();

    let term2 = p
        .save_term(TermInput {
            id: None,
            name: "Spring 2026".into(),
            start_date: d("2026-02-02"),
            end_date: d("2026-02-27"),
        })
        .unwrap();
    p.set_holidays(
        term2.id,
        &[
            Holiday {
                date: d("2026-02-10"),
                label: None,
            },
            Holiday {
                date: d("2026-02-03"),
                label: None,
            },
        ],
    )
    .unwrap();
    // Codes in reverse alphabetical order: ids still follow creation order.
    let zzz = p.create_course(course_input("ZZZ900")).unwrap();
    let aaa = p.create_course(course_input("AAA100")).unwrap();

    let doc = export(&p, NOW);

    assert_eq!(ids(&doc["terms"]), vec![term1.id, term2.id]);
    assert_eq!(ids(&doc["courses"]), vec![cse.id, zzz.id, aaa.id]);
    for key in ["slots", "sessions", "assessments"] {
        let list = ids(&doc[key]);
        assert!(list.len() >= 2, "{key} should have several rows");
        assert!(
            list.windows(2).all(|w| w[0] < w[1]),
            "{key} not ordered by id: {list:?}"
        );
    }

    let holiday_keys: Vec<(i64, String)> = doc["holidays"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| {
            (
                h["term_id"].as_i64().unwrap(),
                h["date"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        holiday_keys,
        vec![
            (term1.id, "2026-01-07".to_string()),
            (term1.id, "2026-01-21".to_string()),
            (term2.id, "2026-02-03".to_string()),
            (term2.id, "2026-02-10".to_string()),
        ]
    );

    let settings_keys: BTreeSet<&str> = doc["settings"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let expected: BTreeSet<&str> = [
        "class_reminder_minutes",
        "upcoming_window_days",
        "notifications_enabled",
    ]
    .into_iter()
    .collect();
    assert_eq!(settings_keys, expected);
}

/// R21: archived courses, inactive terms, and sessions of every status are all exported.
#[test]
fn r21_export_includes_archived_inactive_and_all_statuses() {
    let (mut p, term1, cse, _, _) = synced_fixture();
    let session = |p: &Planner, day: &str| regular_on(p, cse.id, day).id;
    let (attended, absent, cancelled) = (
        session(&p, "2026-01-05"),
        session(&p, "2026-01-07"),
        session(&p, "2026-01-12"),
    );
    mark(&mut p, attended, Status::Attended, after_term());
    mark(&mut p, absent, Status::Absent, after_term());
    mark(&mut p, cancelled, Status::Cancelled, after_term());
    let stored_sessions = sessions_of(&p, cse.id).len();
    p.set_archived(cse.id, true).unwrap();

    let term2 = p
        .save_term(TermInput {
            id: None,
            name: "Spring 2026".into(),
            start_date: d("2026-02-02"),
            end_date: d("2026-02-27"),
        })
        .unwrap();
    let other = p.create_course(course_input("CSE202")).unwrap();

    let doc = export(&p, NOW);

    let terms = doc["terms"].as_array().unwrap();
    assert_eq!(terms.len(), 2);
    let first = terms.iter().find(|t| t["id"] == term1.id).unwrap();
    assert_eq!(first["is_active"], json!(false));
    let second = terms.iter().find(|t| t["id"] == term2.id).unwrap();
    assert_eq!(second["is_active"], json!(true));

    let courses = doc["courses"].as_array().unwrap();
    assert_eq!(courses.len(), 2);
    let archived = courses.iter().find(|c| c["id"] == cse.id).unwrap();
    assert_eq!(archived["archived"], json!(true));
    let active = courses.iter().find(|c| c["id"] == other.id).unwrap();
    assert_eq!(active["archived"], json!(false));

    let rows = doc["sessions"].as_array().unwrap();
    assert_eq!(rows.len(), stored_sessions);
    let statuses: BTreeSet<&str> = rows.iter().map(|r| r["status"].as_str().unwrap()).collect();
    let expected: BTreeSet<&str> = ["pending", "attended", "absent", "cancelled"]
        .into_iter()
        .collect();
    assert_eq!(statuses, expected);
}

/// R21: an empty database exports six empty arrays and the default settings.
#[test]
fn r21_export_empty_database() {
    let p = Planner::open_in_memory().unwrap();
    let doc = export(&p, NOW);

    for key in [
        "terms",
        "holidays",
        "courses",
        "slots",
        "sessions",
        "assessments",
    ] {
        assert_eq!(doc[key], json!([]), "{key} should be empty");
    }
    assert_eq!(
        doc["settings"],
        json!({
            "class_reminder_minutes": "15",
            "upcoming_window_days": "7",
            "notifications_enabled": "1",
        })
    );
}

/// R21: two exports of unchanged data with the same `now` are byte-identical.
#[test]
fn r21_export_is_deterministic() {
    let pop = populated();
    let first = pop.p.export_json(dt(NOW)).unwrap();
    let second = pop.p.export_json(dt(NOW)).unwrap();
    assert_eq!(first, second);
}
