//! M9 backups (R41, amending R21 and R22): grading data in the JSON document, version 1
//! backups restoring ungraded with seeded defaults, and unchanged CSV exports.

mod common;

use common::*;
use planner_core::{
    AssessmentInput, AssessmentKind, ComponentKind, Error, GradeRow, Planner, RuleSetInput,
};
use serde_json::Value;

const NOW: &str = "2026-01-25T18:30";
const LATER: &str = "2026-02-01T08:00";

const GRADING_KEYS: [&str; 4] = [
    "rule_sets",
    "rule_components",
    "attendance_bands",
    "grade_scale",
];

fn grade_row(letter: &str, min_pct: f64, points: f64) -> GradeRow {
    GradeRow {
        letter: letter.into(),
        min_pct,
        points,
    }
}

/// A planner with a graded Theory course, a linked scored assessment, an edited Theory rule
/// set (renamed component, new weights), a new "Seminar" rule set, and an edited grade scale.
fn graded_planner() -> Planner {
    let mut p = Planner::open_in_memory().unwrap();
    p.save_term(fixture_term()).unwrap();
    let course = graded_course(&mut p, "CSE101", 3.0, "Theory");
    let mid = component_id(&p, "Theory", "Mid");
    scored(&mut p, course.id, mid, "Mid", 61.5);

    let mut theory = p.list_rule_sets().unwrap().remove(0);
    theory.components[1].name = "Class tests".into();
    theory.components[1].weight_pct = 20.0;
    theory.components[3].weight_pct = 45.0;
    let input = RuleSetInput {
        id: Some(theory.id),
        name: theory.name.clone(),
        components: theory
            .components
            .iter()
            .map(|c| planner_core::RuleComponentInput {
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
    };
    p.save_rule_set(input).unwrap();
    p.save_rule_set(RuleSetInput {
        id: None,
        name: "Seminar".into(),
        components: vec![planner_core::RuleComponentInput {
            id: None,
            name: "Report".into(),
            kind: ComponentKind::Items,
            weight_pct: 100.0,
            item_count: Some(1),
            best_count: Some(1),
            item_max: None,
            is_final: true,
            bands: Vec::new(),
        }],
    })
    .unwrap();
    p.save_grade_scale(&[
        grade_row("Distinction", 70.0, 4.0),
        grade_row("Merit", 50.0, 3.0),
        grade_row("Fail", 0.0, 0.0),
    ])
    .unwrap();
    p
}

fn export_text(p: &Planner, now: &str) -> String {
    p.export_json(dt(now)).unwrap()
}

fn parse(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn without_exported_at(text: &str) -> Value {
    let mut doc = parse(text);
    doc.as_object_mut().unwrap().remove("exported_at").unwrap();
    doc
}

/// R41 (T42): a version 2 export restored into a fresh planner and exported again gives the
/// same document except `exported_at`, with a graded course, a linked scored assessment, an
/// edited rule set, and an edited grade scale.
#[test]
fn t42_v2_export_restore_export_equal() {
    let source = graded_planner();
    let first = export_text(&source, NOW);

    let doc = parse(&first);
    assert_eq!(doc["schema_version"], 3);
    assert_eq!(doc["courses"][0]["credits"], 3.0);
    assert_eq!(doc["courses"][0]["rule_set_id"], 1);
    assert_eq!(doc["assessments"][0]["score"], 61.5);
    assert_eq!(doc["grade_scale"].as_array().unwrap().len(), 3);
    assert_eq!(doc["rule_sets"].as_array().unwrap().len(), 3);

    let mut fresh = Planner::open_in_memory().unwrap();
    fresh.restore_json(&first).unwrap();
    let second = export_text(&fresh, LATER);

    assert_ne!(parse(&first)["exported_at"], parse(&second)["exported_at"]);
    assert_eq!(without_exported_at(&first), without_exported_at(&second));
}

/// R41, R22 (T42): a version 1 backup restores with ungraded courses and assessments, and the
/// rule sets and grade scale become the seeded defaults, even when the database held edited
/// ones before.
#[test]
fn t42_v1_backup_restores_ungraded_with_defaults() {
    let mut doc = parse(&export_text(&graded_planner(), NOW));
    doc["schema_version"] = Value::from(1);
    let object = doc.as_object_mut().unwrap();
    for key in GRADING_KEYS {
        object.remove(key);
    }
    for row in doc["courses"].as_array_mut().unwrap() {
        let row = row.as_object_mut().unwrap();
        row.remove("credits");
        row.remove("rule_set_id");
    }
    for row in doc["assessments"].as_array_mut().unwrap() {
        let row = row.as_object_mut().unwrap();
        row.remove("component_id");
        row.remove("score");
    }

    let mut target = graded_planner();
    target
        .restore_json(&serde_json::to_string(&doc).unwrap())
        .unwrap();

    let courses = target.list_courses(true).unwrap();
    assert_eq!(courses.len(), 1);
    assert_eq!(courses[0].credits, None);
    assert_eq!(courses[0].rule_set_id, None);
    let assessments = target
        .list_assessments(planner_core::AssessmentFilter::All)
        .unwrap();
    assert_eq!(assessments.len(), 1);
    assert_eq!(assessments[0].component_id, None);
    assert_eq!(assessments[0].score, None);

    let fresh = Planner::open_in_memory().unwrap();
    assert_eq!(
        target.list_rule_sets().unwrap(),
        fresh.list_rule_sets().unwrap()
    );
    assert_eq!(target.grade_scale().unwrap(), fresh.grade_scale().unwrap());
}

/// R41, R22: a version 2 backup missing any of the four grading arrays is rejected as
/// malformed and the database is unchanged.
#[test]
fn r41_v2_backup_missing_grading_arrays_rejected() {
    let good = parse(&export_text(&graded_planner(), NOW));
    for key in GRADING_KEYS {
        let mut doc = good.clone();
        doc.as_object_mut().unwrap().remove(key).unwrap();

        let mut target = graded_planner();
        let before = export_text(&target, NOW);
        let err = target
            .restore_json(&serde_json::to_string(&doc).unwrap())
            .expect_err(&format!("missing {key} should be rejected"));
        assert!(
            matches!(err, Error::Validation(_)),
            "missing {key}: expected Error::Validation, got {err:?}"
        );
        assert_eq!(
            export_text(&target, NOW),
            before,
            "missing {key}: DB changed"
        );
    }
}

/// R41, R23: with grading data present, both CSV exports keep their R23 headers and columns.
#[test]
fn r41_csv_unchanged() {
    let mut p = graded_planner();
    let course = p.list_courses(false).unwrap().remove(0);
    let quiz = component_id(&p, "Theory", "Class tests");
    p.create_assessment(AssessmentInput {
        component_id: Some(quiz),
        score: Some(12.0),
        ..assessment_input(
            course.id,
            AssessmentKind::Quiz,
            "Quiz 1",
            "2026-01-14T09:00",
        )
    })
    .unwrap();
    p.add_slot(slot_a(course.id)).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();

    let mut attendance = Vec::new();
    p.export_attendance_csv(&mut attendance).unwrap();
    let attendance = String::from_utf8(attendance).unwrap();
    assert_eq!(
        attendance.lines().next().unwrap(),
        "date,start,end,course_code,course_name,kind,status,replaces_date,note"
    );

    let mut assessments = Vec::new();
    p.export_assessments_csv(&mut assessments).unwrap();
    assert_eq!(
        String::from_utf8(assessments).unwrap(),
        "course_code,kind,title,due_at,completed,notes\r\n\
         CSE101,other,Mid,2026-01-12T10:00,false,\r\n\
         CSE101,quiz,Quiz 1,2026-01-14T09:00,false,\r\n"
    );
}
