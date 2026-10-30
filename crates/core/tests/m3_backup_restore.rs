//! M3 backup restore (R22): validation, all-or-nothing replacement, ids and links kept.

mod common;

use common::*;
use planner_core::{
    AssessmentFilter, AssessmentInput, AssessmentKind, DayOffKind, Error, Holiday, Planner,
    Settings, Status, TermInput, ThemeMode, SCHEMA_VERSION,
};
use serde_json::{json, Value};

const NOW: &str = "2026-01-25T18:30";
const LATER: &str = "2026-02-01T08:00";

struct Populated {
    p: Planner,
    term_id: i64,
    course_id: i64,
    original_id: i64,
    makeup_id: i64,
}

/// The §8 fixture, synced, with a holiday, the 01-12 session rescheduled to an attended
/// makeup, two assessments (one completed), and changed settings.
fn populated() -> Populated {
    let (mut p, term, course, _, _) = synced_fixture();
    p.set_holidays(
        term.id,
        &[Holiday {
            date: d("2026-01-19"),
            label: Some("Break".into()),
            kind: DayOffKind::Holiday,
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

    p.create_assessment(AssessmentInput {
        notes: Some("bring calculator".into()),
        remind_minutes_before: Some(30),
        component_id: None,
        score: None,
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
        original_id: original.id,
        makeup_id: makeup.id,
    }
}

/// A planner whose rows all differ from `populated()`: another term, courses, slot, session,
/// and assessment, with ids that overlap the populated ones.
fn target() -> Planner {
    let mut p = Planner::open_in_memory().unwrap();
    p.save_term(TermInput {
        id: None,
        name: "Other term".into(),
        start_date: d("2026-03-02"),
        end_date: d("2026-03-29"),
    })
    .unwrap();
    let bio = p.create_course(course_input("BIO200")).unwrap();
    p.create_course(course_input("CHE300")).unwrap();
    p.add_slot(slot_b(bio.id)).unwrap();
    p.add_makeup(makeup_input(bio.id, "2026-03-04", "10:00"))
        .unwrap();
    p.create_assessment(assessment_input(
        bio.id,
        AssessmentKind::Assignment,
        "Lab report",
        "2026-03-20T12:00",
    ))
    .unwrap();
    p
}

fn parse(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

/// The document of a populated planner, ready to be edited into a bad one.
fn good_doc() -> Value {
    parse(&populated().p.export_json(dt(NOW)).unwrap())
}

/// A document, parsed and without `exported_at`, for comparing two exports.
fn without_exported_at(text: &str) -> Value {
    let mut doc = parse(text);
    doc.as_object_mut().unwrap().remove("exported_at").unwrap();
    doc
}

/// The good document after `edit`, as JSON text.
fn edited(edit: impl FnOnce(&mut Value)) -> String {
    let mut doc = good_doc();
    edit(&mut doc);
    serde_json::to_string(&doc).unwrap()
}

fn remove_key(doc: &mut Value, key: &str) {
    doc.as_object_mut().unwrap().remove(key).unwrap();
}

/// The first row of `collection` that satisfies `pick`.
fn row_mut<'a>(
    doc: &'a mut Value,
    collection: &str,
    pick: impl Fn(&Value) -> bool,
) -> &'a mut Value {
    doc[collection]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| pick(r))
        .unwrap_or_else(|| panic!("no matching row in {collection}"))
}

/// Restoring `bad` must fail with `Error::Validation` and leave the database as it was.
fn assert_rejected(p: &mut Planner, why: &str, bad: &str) {
    let before = p.export_json(dt(NOW)).unwrap();
    let err = p
        .restore_json(bad)
        .expect_err(&format!("{why}: restore should fail"));
    assert!(
        matches!(err, Error::Validation(_)),
        "{why}: expected Error::Validation, got {err:?}"
    );
    assert_eq!(
        p.export_json(dt(NOW)).unwrap(),
        before,
        "{why}: database changed"
    );
}

/// Every case must be rejected, one after the other, on the same target planner.
fn assert_all_rejected(cases: Vec<(&str, String)>) {
    let mut p = target();
    for (why, bad) in cases {
        assert_rejected(&mut p, why, &bad);
    }
}

/// R22, R21 (T30): export, restore into an empty planner, export again: the documents are
/// equal except `exported_at`.
#[test]
fn t30_export_restore_into_empty_export_equal_except_exported_at() {
    let source = populated().p;
    let first = source.export_json(dt(NOW)).unwrap();

    let mut empty = Planner::open_in_memory().unwrap();
    empty.restore_json(&first).unwrap();
    let second = empty.export_json(dt(LATER)).unwrap();

    assert_ne!(parse(&first)["exported_at"], parse(&second)["exported_at"]);
    assert_eq!(without_exported_at(&first), without_exported_at(&second));
}

/// R22 (T30): a wrong, missing, or non-string `format` is rejected and the database is unchanged.
#[test]
fn t30_bad_format_rejected_and_db_unchanged() {
    assert_all_rejected(vec![
        (
            "wrong format",
            edited(|doc| doc["format"] = json!("something-else")),
        ),
        ("missing format", edited(|doc| remove_key(doc, "format"))),
        ("non-string format", edited(|doc| doc["format"] = json!(7))),
    ]);
}

/// R22 (T30): a `schema_version` above the app's, missing, or not an integer is rejected and
/// the database is unchanged.
#[test]
fn t30_newer_schema_version_rejected_and_db_unchanged() {
    assert_all_rejected(vec![
        (
            "schema_version one above the app's",
            edited(|doc| doc["schema_version"] = json!(SCHEMA_VERSION + 1)),
        ),
        (
            "schema_version 99",
            edited(|doc| doc["schema_version"] = json!(99)),
        ),
        (
            "missing schema_version",
            edited(|doc| remove_key(doc, "schema_version")),
        ),
        (
            "string schema_version",
            edited(|doc| doc["schema_version"] = json!("1")),
        ),
        (
            "fractional schema_version",
            edited(|doc| doc["schema_version"] = json!(1.5)),
        ),
    ]);
}

/// R22: a `schema_version` equal to or lower than the app's is accepted.
#[test]
fn r22_restore_accepts_equal_and_lower_schema_version() {
    let expected = without_exported_at(&populated().p.export_json(dt(NOW)).unwrap());
    for version in 1..=SCHEMA_VERSION {
        let text = edited(|doc| doc["schema_version"] = json!(version));
        let mut p = target();
        p.restore_json(&text)
            .unwrap_or_else(|e| panic!("schema_version {version} should restore: {e:?}"));

        // The re-export carries the app's own schema version, so it equals the original.
        assert_eq!(
            without_exported_at(&p.export_json(dt(NOW)).unwrap()),
            expected
        );
    }
}

/// R22: restore replaces all data; it does not merge with what the planner held before.
#[test]
fn r22_restore_replaces_not_merges() {
    let a = populated().p;
    let backup = a.export_json(dt(NOW)).unwrap();

    let mut b = target();
    b.create_course(course_input("MAT400")).unwrap();
    assert!(b.list_courses(true).unwrap().len() >= 3);
    b.restore_json(&backup).unwrap();

    assert_eq!(
        without_exported_at(&b.export_json(dt(NOW)).unwrap()),
        without_exported_at(&backup)
    );
    let codes: Vec<String> = b
        .list_courses(true)
        .unwrap()
        .into_iter()
        .map(|c| c.code)
        .collect();
    assert_eq!(codes, vec!["CSE101".to_string()]);
    assert_eq!(b.active_term().unwrap().unwrap().name, "Winter 2026");
    assert_eq!(b.list_assessments(AssessmentFilter::All).unwrap().len(), 2);
}

/// R22, R21: ids and every link survive a restore: course to term, slot to course, session to
/// slot, makeup to the session it replaces, assessment to course.
#[test]
fn r22_restore_preserves_ids_and_links() {
    let pop = populated();
    let backup = pop.p.export_json(dt(NOW)).unwrap();

    let mut b = target();
    b.restore_json(&backup).unwrap();

    let term = b.active_term().unwrap().unwrap();
    assert_eq!(term.id, pop.term_id);
    assert_eq!(Some(term.clone()), pop.p.active_term().unwrap());

    let courses = b.list_courses(true).unwrap();
    assert_eq!(courses, pop.p.list_courses(true).unwrap());
    assert_eq!(courses[0].id, pop.course_id);
    assert_eq!(courses[0].term_id, pop.term_id);

    assert_eq!(
        b.list_slots(d("2026-01-12")).unwrap(),
        pop.p.list_slots(d("2026-01-12")).unwrap()
    );
    assert_eq!(
        b.list_slots(d("2026-01-12")).unwrap()[0].course_id,
        pop.course_id
    );

    assert_eq!(
        sessions_of(&b, pop.course_id),
        sessions_of(&pop.p, pop.course_id)
    );
    let regular = regular_on(&b, pop.course_id, "2026-01-05");
    assert!(regular.slot_id.is_some());
    assert_eq!(
        regular.slot_id,
        regular_on(&pop.p, pop.course_id, "2026-01-05").slot_id
    );

    let views = b.sessions_for_course(pop.course_id, after_term()).unwrap();
    let original = views
        .iter()
        .find(|v| v.session.id == pop.original_id)
        .unwrap();
    assert_eq!(original.display_status, "Rescheduled → 2026-01-13");
    let makeup = views
        .iter()
        .find(|v| v.session.id == pop.makeup_id)
        .unwrap();
    assert_eq!(makeup.session.replaces_session_id, Some(pop.original_id));
    assert_eq!(makeup.display_status, "Attended");

    let assessments = b.list_assessments(AssessmentFilter::All).unwrap();
    assert_eq!(
        assessments,
        pop.p.list_assessments(AssessmentFilter::All).unwrap()
    );
    assert!(assessments.iter().all(|a| a.course_id == pop.course_id));
    assert_eq!(b.settings().unwrap(), pop.p.settings().unwrap());
}

/// R22: a makeup whose id is lower than the id of the session it replaces still restores
/// (foreign keys are checked at commit, not row by row).
#[test]
fn r22_restore_makeup_with_lower_id_than_original() {
    let pop = populated();
    let mut doc = parse(&pop.p.export_json(dt(NOW)).unwrap());

    let moved_id = 1000;
    row_mut(&mut doc, "sessions", |r| r["id"] == pop.original_id)["id"] = json!(moved_id);
    row_mut(&mut doc, "sessions", |r| r["id"] == pop.makeup_id)["replaces_session_id"] =
        json!(moved_id);
    doc["sessions"]
        .as_array_mut()
        .unwrap()
        .sort_by_key(|r| r["id"].as_i64().unwrap());
    assert!(pop.makeup_id < moved_id);

    let mut b = target();
    b.restore_json(&serde_json::to_string(&doc).unwrap())
        .unwrap();

    let views = b.sessions_for_course(pop.course_id, after_term()).unwrap();
    let original = views.iter().find(|v| v.session.id == moved_id).unwrap();
    assert_eq!(original.display_status, "Rescheduled → 2026-01-13");
    let makeup = views
        .iter()
        .find(|v| v.session.id == pop.makeup_id)
        .unwrap();
    assert_eq!(makeup.session.replaces_session_id, Some(moved_id));
}

/// R22: text that is not a JSON object is rejected and the database is unchanged.
#[test]
fn r22_invalid_json_rejected_and_db_unchanged() {
    assert_all_rejected(vec![
        ("garbage text", "this is not json {".to_string()),
        ("empty string", String::new()),
        ("array root", "[]".to_string()),
        ("number root", "42".to_string()),
        ("null root", "null".to_string()),
    ]);
}

/// R22: a missing collection is rejected and the database is unchanged.
#[test]
fn r22_missing_collection_rejected_and_db_unchanged() {
    assert_all_rejected(vec![
        (
            "missing sessions",
            edited(|doc| remove_key(doc, "sessions")),
        ),
        (
            "missing settings",
            edited(|doc| remove_key(doc, "settings")),
        ),
    ]);
}

/// R22: an ill-typed field or collection is rejected and the database is unchanged.
#[test]
fn r22_ill_typed_field_rejected_and_db_unchanged() {
    assert_all_rejected(vec![
        (
            "completed as number",
            edited(|doc| doc["assessments"][0]["completed"] = json!(1)),
        ),
        (
            "is_active as text",
            edited(|doc| doc["terms"][0]["is_active"] = json!("yes")),
        ),
        (
            "course id as text",
            edited(|doc| doc["courses"][0]["id"] = json!("x")),
        ),
        (
            "sessions as object",
            edited(|doc| doc["sessions"] = json!({})),
        ),
    ]);
}

/// R22: a text value that is not a stored-format date, time, or date-time is rejected and the
/// database is unchanged.
#[test]
fn r22_malformed_date_rejected_and_db_unchanged() {
    assert_all_rejected(vec![
        (
            "term start_date",
            edited(|doc| doc["terms"][0]["start_date"] = json!("01/05/2026")),
        ),
        (
            "session start_time",
            edited(|doc| doc["sessions"][0]["start_time"] = json!("9am")),
        ),
        (
            "assessment due_at",
            edited(|doc| doc["assessments"][0]["due_at"] = json!("2026-01-05")),
        ),
    ]);
}

/// R22: a reference to a row that is not in the backup fails at commit and rolls back.
#[test]
fn r22_dangling_reference_rolls_back() {
    assert_all_rejected(vec![
        (
            "session course_id 999",
            edited(|doc| doc["sessions"][0]["course_id"] = json!(999)),
        ),
        (
            "session replaces_session_id 999",
            edited(|doc| {
                row_mut(doc, "sessions", |r| r["kind"] == "makeup")["replaces_session_id"] =
                    json!(999)
            }),
        ),
        (
            "slot course_id 999",
            edited(|doc| doc["slots"][0]["course_id"] = json!(999)),
        ),
    ]);
}

/// R22: rows that break a schema constraint roll the whole restore back.
#[test]
fn r22_constraint_violation_rolls_back() {
    assert_all_rejected(vec![
        (
            "two active terms",
            edited(|doc| {
                let mut second = doc["terms"][0].clone();
                second["id"] = json!(2);
                second["name"] = json!("Second active");
                doc["terms"].as_array_mut().unwrap().push(second);
            }),
        ),
        (
            "session ends when it starts",
            edited(|doc| {
                let start = doc["sessions"][0]["start_time"].clone();
                doc["sessions"][0]["end_time"] = start;
            }),
        ),
        (
            "duplicate (slot_id, date)",
            edited(|doc| {
                let mut copy = row_mut(doc, "sessions", |r| r["kind"] == "regular").clone();
                copy["id"] = json!(9999);
                doc["sessions"].as_array_mut().unwrap().push(copy);
            }),
        ),
    ]);
}
