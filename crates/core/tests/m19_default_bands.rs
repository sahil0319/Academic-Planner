//! M19 default attendance bands (docs/spec.md §13.14, R92-R95, T86-T89). T90 is in the app
//! view model tests.

mod common;

use std::path::Path;

use common::*;
use planner_core::{
    default_attendance_bands, AttendanceBand, ComponentKind, Planner, RuleComponentInput,
    RuleSetInput, SCHEMA_VERSION,
};
use rusqlite::Connection;
use serde_json::Value;
use tempfile::TempDir;

const NEW_TABLE: [(f64, f64); 6] = [
    (95.0, 100.0),
    (90.0, 80.0),
    (85.0, 60.0),
    (75.0, 20.0),
    (70.0, 10.0),
    (0.0, 0.0),
];

const OLD_TABLE: [(f64, f64); 6] = [
    (90.0, 100.0),
    (85.0, 90.0),
    (80.0, 80.0),
    (75.0, 70.0),
    (70.0, 60.0),
    (0.0, 0.0),
];

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "{actual} is not {expected}"
    );
}

/// The attendance bands of the named component in the named rule set, as `(min, credit)`.
fn bands_of(p: &Planner, rule_set: &str, component: &str) -> Vec<(f64, f64)> {
    p.list_rule_sets()
        .unwrap()
        .into_iter()
        .find(|set| set.name == rule_set)
        .unwrap_or_else(|| panic!("no rule set {rule_set}"))
        .components
        .into_iter()
        .find(|c| c.name == component)
        .unwrap_or_else(|| panic!("no component {component}"))
        .bands
        .iter()
        .map(|b| (b.min_pct, b.credit_pct))
        .collect()
}

/// The bands of a component read straight from the file, highest `min_pct` first.
fn sql_bands(conn: &Connection, component_id: i64) -> Vec<(f64, f64)> {
    let mut stmt = conn
        .prepare(
            "SELECT min_pct, credit_pct FROM attendance_band \
             WHERE component_id = ?1 ORDER BY min_pct DESC",
        )
        .unwrap();
    stmt.query_map([component_id], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
}

/// Every `rule_set` and `rule_component` row as text, ordered by id, for before/after checks.
fn rule_rows(conn: &Connection) -> Vec<String> {
    let mut rows = Vec::new();
    let mut sets = conn
        .prepare("SELECT id || '|' || name FROM rule_set ORDER BY id")
        .unwrap();
    rows.extend(
        sets.query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(|r| r.unwrap()),
    );
    let mut comps = conn
        .prepare(
            "SELECT id || '|' || rule_set_id || '|' || position || '|' || name || '|' || kind \
             || '|' || weight_pct || '|' || ifnull(item_count, '-') || '|' \
             || ifnull(best_count, '-') || '|' || ifnull(item_max, '-') || '|' || is_final \
             FROM rule_component ORDER BY id",
        )
        .unwrap();
    rows.extend(
        comps
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(|r| r.unwrap()),
    );
    rows
}

/// Builds a version 2 database file the way the shipped app left it: migrations 001 and 002,
/// the grading seed, then user_version 2.
fn make_v2_db(path: &Path) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
    conn.execute_batch(include_str!("../src/migrations/001_init.sql"))
        .unwrap();
    conn.execute_batch(include_str!("../src/migrations/002_grading.sql"))
        .unwrap();
    conn.execute_batch(include_str!("../src/grading_seed.sql"))
        .unwrap();
    conn.pragma_update(None, "user_version", 2).unwrap();
}

fn replace_bands(conn: &Connection, component_id: i64, bands: &[(f64, f64)]) {
    conn.execute(
        "DELETE FROM attendance_band WHERE component_id = ?1",
        [component_id],
    )
    .unwrap();
    for (min_pct, credit_pct) in bands {
        conn.execute(
            "INSERT INTO attendance_band (component_id, min_pct, credit_pct) VALUES (?1, ?2, ?3)",
            rusqlite::params![component_id, min_pct, credit_pct],
        )
        .unwrap();
    }
}

/// Adds a rule set with one attendance component (weight 10) and a final, with the given bands.
fn add_custom_rule_set(conn: &Connection, set_id: i64, attendance_id: i64, bands: &[(f64, f64)]) {
    conn.execute(
        "INSERT INTO rule_set (id, name) VALUES (?1, ?2)",
        rusqlite::params![set_id, format!("Custom {set_id}")],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO rule_component \
         (id, rule_set_id, position, name, kind, weight_pct, item_count, best_count, item_max, \
          is_final) VALUES (?1, ?2, 0, 'Presence', 'attendance', 10, NULL, NULL, NULL, 0)",
        rusqlite::params![attendance_id, set_id],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO rule_component \
         (id, rule_set_id, position, name, kind, weight_pct, item_count, best_count, item_max, \
          is_final) VALUES (?1, ?2, 1, 'Exam', 'items', 90, 1, 1, NULL, 1)",
        rusqlite::params![attendance_id + 100, set_id],
    )
    .unwrap();
    replace_bands(conn, attendance_id, bands);
}

/// R92, R34 (T86): the defaults are exactly the human's table in descending order, and a rule
/// set holding them as its attendance component passes the R34 validation and reads back.
#[test]
fn t86_default_bands_are_the_human_table_and_pass_validation() {
    let bands = default_attendance_bands();
    let pairs: Vec<(f64, f64)> = bands.iter().map(|b| (b.min_pct, b.credit_pct)).collect();
    assert_eq!(pairs, NEW_TABLE.to_vec());
    assert!(bands.windows(2).all(|w| w[0].min_pct > w[1].min_pct));
    assert!(bands.iter().any(|b| b.min_pct == 0.0));

    let mut p = Planner::open_in_memory().unwrap();
    let saved = p
        .save_rule_set(RuleSetInput {
            id: None,
            name: "Seminar".into(),
            components: vec![
                RuleComponentInput {
                    id: None,
                    name: "Presence".into(),
                    kind: ComponentKind::Attendance,
                    weight_pct: 10.0,
                    item_count: None,
                    best_count: None,
                    item_max: None,
                    is_final: false,
                    bands: default_attendance_bands(),
                },
                RuleComponentInput {
                    id: None,
                    name: "Exam".into(),
                    kind: ComponentKind::Items,
                    weight_pct: 90.0,
                    item_count: Some(1),
                    best_count: Some(1),
                    item_max: None,
                    is_final: true,
                    bands: Vec::new(),
                },
            ],
        })
        .unwrap();
    let presence = saved
        .components
        .iter()
        .find(|c| c.name == "Presence")
        .unwrap();
    assert_eq!(presence.bands, default_attendance_bands());
    assert_eq!(
        bands_of(&p, "Seminar", "Presence"),
        NEW_TABLE.to_vec(),
        "stored bands differ from the defaults"
    );
}

/// R93, R26 (T87): migrating a version 2 file replaces the old placeholder table with the new
/// defaults, leaves an edited component, a component already on the new table, a near-miss
/// table (5 of 6 old bands plus an extra), and a table with one changed credit alone, keeps
/// weights and names, and writes the pre-migration copy.
#[test]
fn t87_migration_replaces_only_the_old_placeholder_table() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("planner.sqlite");
    make_v2_db(&path);

    let lab_edited = [(90.0, 100.0), (0.0, 0.0)];
    let five_plus_extra = [
        (95.0, 100.0),
        (90.0, 100.0),
        (85.0, 90.0),
        (75.0, 70.0),
        (70.0, 60.0),
        (0.0, 0.0),
    ];
    let one_credit_changed = [
        (90.0, 100.0),
        (85.0, 90.0),
        (80.0, 80.0),
        (75.0, 70.0),
        (70.0, 61.0),
        (0.0, 0.0),
    ];
    let rows_before = {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        assert_eq!(sql_bands(&conn, 1), OLD_TABLE.to_vec());
        replace_bands(&conn, 5, &lab_edited);
        add_custom_rule_set(&conn, 3, 8, &NEW_TABLE);
        add_custom_rule_set(&conn, 4, 9, &five_plus_extra);
        add_custom_rule_set(&conn, 5, 10, &one_credit_changed);
        rule_rows(&conn)
    };

    let p = Planner::open(&path).unwrap();

    assert_eq!(bands_of(&p, "Theory", "Attendance"), NEW_TABLE.to_vec());
    assert_eq!(bands_of(&p, "Lab", "Attendance"), lab_edited.to_vec());
    assert_eq!(bands_of(&p, "Custom 3", "Presence"), NEW_TABLE.to_vec());
    assert_eq!(
        bands_of(&p, "Custom 4", "Presence"),
        five_plus_extra.to_vec()
    );
    assert_eq!(
        bands_of(&p, "Custom 5", "Presence"),
        one_credit_changed.to_vec()
    );
    drop(p);

    let conn = Connection::open(&path).unwrap();
    assert_eq!(rule_rows(&conn), rows_before, "weights or names changed");
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, SCHEMA_VERSION);

    let copy = dir.path().join(format!(
        "backups/pre-migrate-v2-to-v{SCHEMA_VERSION}.sqlite"
    ));
    assert!(copy.exists(), "missing {}", copy.display());
    let old = Connection::open(&copy).unwrap();
    assert_eq!(sql_bands(&old, 1), OLD_TABLE.to_vec());
}

/// R93, R25 (T87): a migrated database is at the current version, so reopening it does not
/// run the migration again: a component the user later sets back to the old placeholder
/// table keeps it.
#[test]
fn t87_reopen_after_migration_keeps_bands() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("planner.sqlite");
    make_v2_db(&path);
    drop(Planner::open(&path).unwrap());

    let conn = Connection::open(&path).unwrap();
    replace_bands(&conn, 5, &OLD_TABLE);
    drop(conn);

    let p = Planner::open(&path).unwrap();
    assert_eq!(bands_of(&p, "Theory", "Attendance"), NEW_TABLE.to_vec());
    assert_eq!(bands_of(&p, "Lab", "Attendance"), OLD_TABLE.to_vec());
}

/// R93 (T88): a fresh database seeds Theory and Lab with the new table.
#[test]
fn t88_fresh_database_has_new_table_in_theory_and_lab() {
    let p = Planner::open_in_memory().unwrap();
    assert_eq!(bands_of(&p, "Theory", "Attendance"), NEW_TABLE.to_vec());
    assert_eq!(bands_of(&p, "Lab", "Attendance"), NEW_TABLE.to_vec());
    let defaults: Vec<AttendanceBand> = default_attendance_bands();
    let theory = p.list_rule_sets().unwrap().remove(0);
    assert_eq!(theory.components[0].bands, defaults);
}

/// R93, R41 (T88): restoring a version 1 backup re-seeds the default rule sets, and both end
/// with the new table, even when the database held edited bands before.
#[test]
fn t88_v1_backup_restore_ends_with_new_table() {
    let mut source = Planner::open_in_memory().unwrap();
    source.save_term(fixture_term()).unwrap();
    let mut doc: Value =
        serde_json::from_str(&source.export_json(dt("2026-01-25T18:30")).unwrap()).unwrap();
    doc["schema_version"] = Value::from(1);
    let object = doc.as_object_mut().unwrap();
    for key in [
        "rule_sets",
        "rule_components",
        "attendance_bands",
        "grade_scale",
    ] {
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

    let mut target = Planner::open_in_memory().unwrap();
    let mut theory = target.list_rule_sets().unwrap().remove(0);
    theory.components[0].bands = vec![
        AttendanceBand {
            min_pct: 50.0,
            credit_pct: 50.0,
        },
        AttendanceBand {
            min_pct: 0.0,
            credit_pct: 0.0,
        },
    ];
    target
        .save_rule_set(RuleSetInput {
            id: Some(theory.id),
            name: theory.name.clone(),
            components: theory
                .components
                .iter()
                .map(|c| RuleComponentInput {
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
        })
        .unwrap();
    assert_eq!(bands_of(&target, "Theory", "Attendance").len(), 2);

    target
        .restore_json(&serde_json::to_string(&doc).unwrap())
        .unwrap();

    assert_eq!(
        bands_of(&target, "Theory", "Attendance"),
        NEW_TABLE.to_vec()
    );
    assert_eq!(bands_of(&target, "Lab", "Attendance"), NEW_TABLE.to_vec());
}

/// R95, R35 (T89): with the seeded Theory bands, a 3-credit course at 87% attendance earns
/// the 85 band (60%) = 18.0 of 30, and at 95% earns the 95 band (100%) = 30.0 of 30.
#[test]
fn t89_seeded_theory_attendance_87_and_95_pct() {
    let mut p = Planner::open_in_memory().unwrap();
    p.save_term(fixture_term()).unwrap();
    let course = graded_course(&mut p, "CSE101", 3.0, "Theory");
    mark_attendance(&mut p, course.id, 20, 3);
    let marks = p.course_marks(course.id).unwrap();
    let attendance = marks
        .components
        .iter()
        .find(|c| c.name == "Attendance")
        .unwrap();
    close(attendance.earned, 18.0);
    close(attendance.marks, 30.0);

    let mut q = Planner::open_in_memory().unwrap();
    q.save_term(fixture_term()).unwrap();
    let course = graded_course(&mut q, "CSE101", 3.0, "Theory");
    mark_attendance(&mut q, course.id, 19, 1);
    let marks = q.course_marks(course.id).unwrap();
    let attendance = marks
        .components
        .iter()
        .find(|c| c.name == "Attendance")
        .unwrap();
    close(attendance.earned, 30.0);
    close(attendance.marks, 30.0);
}
