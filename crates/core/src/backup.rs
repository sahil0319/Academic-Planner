//! JSON backup and restore (M3; version 2 adds the grading data, R41).

use std::collections::BTreeMap;

use chrono::NaiveDateTime;
use rusqlite::{params, Transaction};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::db::{
    col_week_parity_opt, fmt_datetime, parse_date, parse_datetime, parse_time, Planner,
    DEFAULT_BANDS_MIGRATION, GRADING_SEED, SCHEMA_VERSION, SETTING_CHECK_UPDATES, SETTING_THEME,
};
use crate::error::{Error, Result};
use crate::model::{DayOffKind, ThemeMode, WeekParity};

/// R21: value of the `format` key of every backup document.
const BACKUP_FORMAT: &str = "academic-planner-backup";

/// R75: first schema version whose backups carry `holidays[].kind` (the M15 migration number;
/// renumber with the migration). An older backup restores every day off as `holiday`.
const DAY_OFF_KIND_VERSION: i64 = 3;

/// Reads a nullable field but, unlike serde's default for `Option`, still requires the key.
fn nullable<'de, D, T>(d: D) -> std::result::Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(d)
}

/// R21: the backup document; field order is the key order of the JSON text.
#[derive(Serialize, Deserialize)]
struct Backup {
    format: String,
    schema_version: i64,
    exported_at: String,
    terms: Vec<TermRow>,
    holidays: Vec<HolidayRow>,
    courses: Vec<CourseRow>,
    slots: Vec<SlotRow>,
    sessions: Vec<SessionRow>,
    assessments: Vec<AssessmentRow>,
    settings: BTreeMap<String, String>,
    /// R41: the four grading arrays; required in a version 2 backup, ignored in version 1.
    #[serde(default)]
    rule_sets: Vec<RuleSetRow>,
    #[serde(default)]
    rule_components: Vec<RuleComponentRow>,
    #[serde(default)]
    attendance_bands: Vec<AttendanceBandRow>,
    #[serde(default)]
    grade_scale: Vec<GradeScaleRow>,
}

/// R41: the arrays a version 2 backup must contain.
const GRADING_KEYS: [&str; 4] = [
    "rule_sets",
    "rule_components",
    "attendance_bands",
    "grade_scale",
];

#[derive(Serialize, Deserialize)]
struct TermRow {
    id: i64,
    name: String,
    start_date: String,
    end_date: String,
    is_active: bool,
}

#[derive(Serialize, Deserialize)]
struct HolidayRow {
    term_id: i64,
    date: String,
    #[serde(deserialize_with = "nullable")]
    label: Option<String>,
    /// R75: `holiday`, `midterm`, or `final`; absent in backups older than `DAY_OFF_KIND_VERSION`.
    #[serde(default = "default_day_off_kind")]
    kind: String,
}

/// R75: the kind of a day off in a backup that has no `kind` key.
fn default_day_off_kind() -> String {
    "holiday".to_string()
}

#[derive(Serialize, Deserialize)]
struct CourseRow {
    id: i64,
    term_id: i64,
    code: String,
    name: String,
    #[serde(deserialize_with = "nullable")]
    instructor: Option<String>,
    #[serde(deserialize_with = "nullable")]
    room: Option<String>,
    #[serde(deserialize_with = "nullable")]
    notes: Option<String>,
    #[serde(deserialize_with = "nullable")]
    min_attendance_pct: Option<i64>,
    archived: bool,
    /// R41: `None` = not graded; absent in a version 1 backup.
    #[serde(default, deserialize_with = "nullable")]
    credits: Option<f64>,
    #[serde(default, deserialize_with = "nullable")]
    rule_set_id: Option<i64>,
}

#[derive(Serialize, Deserialize)]
struct SlotRow {
    id: i64,
    course_id: i64,
    weekday: i64,
    start_time: String,
    end_time: String,
    #[serde(deserialize_with = "nullable")]
    room: Option<String>,
    valid_from: String,
    #[serde(deserialize_with = "nullable")]
    valid_to: Option<String>,
    /// R60: absent in version-1 and version-2 backups, which restore as weekly slots.
    #[serde(default)]
    week_parity: Option<WeekParity>,
}

#[derive(Serialize, Deserialize)]
struct SessionRow {
    id: i64,
    course_id: i64,
    #[serde(deserialize_with = "nullable")]
    slot_id: Option<i64>,
    kind: String,
    date: String,
    start_time: String,
    end_time: String,
    #[serde(deserialize_with = "nullable")]
    room: Option<String>,
    status: String,
    #[serde(deserialize_with = "nullable")]
    replaces_session_id: Option<i64>,
    #[serde(deserialize_with = "nullable")]
    note: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct AssessmentRow {
    id: i64,
    course_id: i64,
    kind: String,
    title: String,
    due_at: String,
    #[serde(deserialize_with = "nullable")]
    notes: Option<String>,
    completed: bool,
    #[serde(deserialize_with = "nullable")]
    remind_minutes_before: Option<i64>,
    /// R41: `None` = not counted in marks; absent in a version 1 backup.
    #[serde(default, deserialize_with = "nullable")]
    component_id: Option<i64>,
    #[serde(default, deserialize_with = "nullable")]
    score: Option<f64>,
}

/// R41: a `rule_set` row.
#[derive(Serialize, Deserialize)]
struct RuleSetRow {
    id: i64,
    name: String,
}

/// R41: a `rule_component` row; nullable keys are always written.
#[derive(Serialize, Deserialize)]
struct RuleComponentRow {
    id: i64,
    rule_set_id: i64,
    position: i64,
    name: String,
    kind: String,
    weight_pct: f64,
    #[serde(deserialize_with = "nullable")]
    item_count: Option<i64>,
    #[serde(deserialize_with = "nullable")]
    best_count: Option<i64>,
    #[serde(deserialize_with = "nullable")]
    item_max: Option<f64>,
    is_final: bool,
}

/// R41: an `attendance_band` row.
#[derive(Serialize, Deserialize)]
struct AttendanceBandRow {
    component_id: i64,
    min_pct: f64,
    credit_pct: f64,
}

/// R41: a `grade_scale` row.
#[derive(Serialize, Deserialize)]
struct GradeScaleRow {
    letter: String,
    min_pct: f64,
    points: f64,
}

impl Backup {
    /// R22: every date, time, and date-time text must parse with the stored-format helpers, so
    /// the database never holds text the app cannot read back.
    fn check_texts(&self) -> Result<()> {
        fn bad(what: &str, value: &str) -> Error {
            Error::Validation(format!("backup has an invalid {what}: {value:?}"))
        }
        let date = |what: &str, v: &str| parse_date(v).map(|_| ()).ok_or_else(|| bad(what, v));
        let time = |what: &str, v: &str| parse_time(v).map(|_| ()).ok_or_else(|| bad(what, v));
        let datetime =
            |what: &str, v: &str| parse_datetime(v).map(|_| ()).ok_or_else(|| bad(what, v));

        for t in &self.terms {
            date("term start_date", &t.start_date)?;
            date("term end_date", &t.end_date)?;
        }
        for h in &self.holidays {
            date("holiday date", &h.date)?;
            // R71, R75: only holiday, midterm, or final.
            if DayOffKind::from_db(&h.kind).is_none() {
                return Err(bad("holiday kind", &h.kind));
            }
        }
        for s in &self.slots {
            time("slot start_time", &s.start_time)?;
            time("slot end_time", &s.end_time)?;
            date("slot valid_from", &s.valid_from)?;
            if let Some(v) = &s.valid_to {
                date("slot valid_to", v)?;
            }
        }
        for s in &self.sessions {
            date("session date", &s.date)?;
            time("session start_time", &s.start_time)?;
            time("session end_time", &s.end_time)?;
        }
        for a in &self.assessments {
            datetime("assessment due_at", &a.due_at)?;
        }
        // §5 Settings theme: only exact lowercase system|light|dark; a missing key is fine.
        if let Some(v) = self.settings.get(SETTING_THEME) {
            ThemeMode::from_db(v)
                .map(|_| ())
                .ok_or_else(|| bad("settings theme", v))?;
        }
        // R22, R29: check_updates_on_start must be exactly "0" or "1"; a missing key reads as on.
        if let Some(v) = self.settings.get(SETTING_CHECK_UPDATES) {
            if v != "0" && v != "1" {
                return Err(bad("settings check_updates_on_start", v));
            }
        }
        Ok(())
    }
}

/// R22: a SQLite constraint violation means the backup data is invalid; anything else is a
/// database error.
fn apply_error(e: rusqlite::Error) -> Error {
    match &e {
        rusqlite::Error::SqliteFailure(f, _)
            if f.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            Error::Validation(format!("backup data rejected: {e}"))
        }
        _ => Error::from(e),
    }
}

/// R22, R41: replaces the content of the data tables with the backup rows. A version 1
/// backup (`version < 2`) has no grading data: the grading tables get the seed defaults.
fn apply(tx: &Transaction, b: &Backup, version: i64) -> rusqlite::Result<()> {
    // Foreign keys are checked at commit, so a makeup may precede the session it replaces.
    tx.execute_batch("PRAGMA defer_foreign_keys = ON")?;
    for table in [
        "session",
        "assessment",
        "slot",
        "course",
        "holiday",
        "term",
        "setting",
        "attendance_band",
        "rule_component",
        "rule_set",
        "grade_scale",
    ] {
        tx.execute(&format!("DELETE FROM {table}"), [])?;
    }
    for t in &b.terms {
        tx.execute(
            "INSERT INTO term (id, name, start_date, end_date, is_active) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![t.id, t.name, t.start_date, t.end_date, t.is_active],
        )?;
    }
    for h in &b.holidays {
        tx.execute(
            "INSERT INTO holiday (term_id, date, label, kind) VALUES (?1, ?2, ?3, ?4)",
            params![h.term_id, h.date, h.label, h.kind],
        )?;
    }
    if version >= 2 {
        for r in &b.rule_sets {
            tx.execute(
                "INSERT INTO rule_set (id, name) VALUES (?1, ?2)",
                params![r.id, r.name],
            )?;
        }
        for c in &b.rule_components {
            tx.execute(
                "INSERT INTO rule_component (id, rule_set_id, position, name, kind, weight_pct, \
                 item_count, best_count, item_max, is_final) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    c.id,
                    c.rule_set_id,
                    c.position,
                    c.name,
                    c.kind,
                    c.weight_pct,
                    c.item_count,
                    c.best_count,
                    c.item_max,
                    c.is_final
                ],
            )?;
        }
        for band in &b.attendance_bands {
            tx.execute(
                "INSERT INTO attendance_band (component_id, min_pct, credit_pct) \
                 VALUES (?1, ?2, ?3)",
                params![band.component_id, band.min_pct, band.credit_pct],
            )?;
        }
        for g in &b.grade_scale {
            tx.execute(
                "INSERT INTO grade_scale (letter, min_pct, points) VALUES (?1, ?2, ?3)",
                params![g.letter, g.min_pct, g.points],
            )?;
        }
    } else {
        tx.execute_batch(GRADING_SEED)?;
        // R93: the seed holds the retired band table; the restored sets end with the defaults.
        tx.execute_batch(DEFAULT_BANDS_MIGRATION)?;
    }
    for c in &b.courses {
        tx.execute(
            "INSERT INTO course (id, term_id, code, name, instructor, room, notes, \
             min_attendance_pct, archived, credits, rule_set_id) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                c.id,
                c.term_id,
                c.code,
                c.name,
                c.instructor,
                c.room,
                c.notes,
                c.min_attendance_pct,
                c.archived,
                c.credits,
                c.rule_set_id
            ],
        )?;
    }
    for s in &b.slots {
        tx.execute(
            "INSERT INTO slot (id, course_id, weekday, start_time, end_time, room, valid_from, \
             valid_to, week_parity) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                s.id,
                s.course_id,
                s.weekday,
                s.start_time,
                s.end_time,
                s.room,
                s.valid_from,
                s.valid_to,
                s.week_parity.map(WeekParity::as_db)
            ],
        )?;
    }
    for s in &b.sessions {
        tx.execute(
            "INSERT INTO session (id, course_id, slot_id, kind, date, start_time, end_time, room, \
             status, replaces_session_id, note) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                s.id,
                s.course_id,
                s.slot_id,
                s.kind,
                s.date,
                s.start_time,
                s.end_time,
                s.room,
                s.status,
                s.replaces_session_id,
                s.note
            ],
        )?;
    }
    for a in &b.assessments {
        tx.execute(
            "INSERT INTO assessment (id, course_id, kind, title, due_at, notes, completed, \
             remind_minutes_before, component_id, score) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                a.id,
                a.course_id,
                a.kind,
                a.title,
                a.due_at,
                a.notes,
                a.completed,
                a.remind_minutes_before,
                a.component_id,
                a.score
            ],
        )?;
    }
    for (key, value) in &b.settings {
        tx.execute(
            "INSERT INTO setting (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
    }
    Ok(())
}

/// Collects all rows of a query into a vector.
fn rows<T>(
    tx: &Transaction,
    sql: &str,
    map: impl FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
) -> Result<Vec<T>> {
    let mut stmt = tx.prepare(sql)?;
    let out = stmt.query_map([], map)?.collect::<rusqlite::Result<_>>()?;
    Ok(out)
}

impl Planner {
    /// R21: exports all data as a versioned JSON backup; IDs are preserved. All tables are read
    /// in one transaction, so the document is a single snapshot. `reminder_log` is not exported.
    pub fn export_json(&self, now: NaiveDateTime) -> Result<String> {
        let tx = self.conn.unchecked_transaction()?;
        let doc = Backup {
            format: BACKUP_FORMAT.to_string(),
            schema_version: SCHEMA_VERSION,
            exported_at: fmt_datetime(now),
            terms: rows(
                &tx,
                "SELECT id, name, start_date, end_date, is_active FROM term ORDER BY id",
                |r| {
                    Ok(TermRow {
                        id: r.get(0)?,
                        name: r.get(1)?,
                        start_date: r.get(2)?,
                        end_date: r.get(3)?,
                        is_active: r.get(4)?,
                    })
                },
            )?,
            holidays: rows(
                &tx,
                "SELECT term_id, date, label, kind FROM holiday ORDER BY term_id, date",
                |r| {
                    Ok(HolidayRow {
                        term_id: r.get(0)?,
                        date: r.get(1)?,
                        label: r.get(2)?,
                        kind: r.get(3)?,
                    })
                },
            )?,
            courses: rows(
                &tx,
                "SELECT id, term_id, code, name, instructor, room, notes, min_attendance_pct, \
                 archived, credits, rule_set_id FROM course ORDER BY id",
                |r| {
                    Ok(CourseRow {
                        id: r.get(0)?,
                        term_id: r.get(1)?,
                        code: r.get(2)?,
                        name: r.get(3)?,
                        instructor: r.get(4)?,
                        room: r.get(5)?,
                        notes: r.get(6)?,
                        min_attendance_pct: r.get(7)?,
                        archived: r.get(8)?,
                        credits: r.get(9)?,
                        rule_set_id: r.get(10)?,
                    })
                },
            )?,
            slots: rows(
                &tx,
                "SELECT id, course_id, weekday, start_time, end_time, room, valid_from, valid_to, \
                 week_parity FROM slot ORDER BY id",
                |r| {
                    Ok(SlotRow {
                        id: r.get(0)?,
                        course_id: r.get(1)?,
                        weekday: r.get(2)?,
                        start_time: r.get(3)?,
                        end_time: r.get(4)?,
                        room: r.get(5)?,
                        valid_from: r.get(6)?,
                        valid_to: r.get(7)?,
                        week_parity: col_week_parity_opt(r, 8)?,
                    })
                },
            )?,
            sessions: rows(
                &tx,
                "SELECT id, course_id, slot_id, kind, date, start_time, end_time, room, status, \
                 replaces_session_id, note FROM session ORDER BY id",
                |r| {
                    Ok(SessionRow {
                        id: r.get(0)?,
                        course_id: r.get(1)?,
                        slot_id: r.get(2)?,
                        kind: r.get(3)?,
                        date: r.get(4)?,
                        start_time: r.get(5)?,
                        end_time: r.get(6)?,
                        room: r.get(7)?,
                        status: r.get(8)?,
                        replaces_session_id: r.get(9)?,
                        note: r.get(10)?,
                    })
                },
            )?,
            assessments: rows(
                &tx,
                "SELECT id, course_id, kind, title, due_at, notes, completed, \
                 remind_minutes_before, component_id, score FROM assessment ORDER BY id",
                |r| {
                    Ok(AssessmentRow {
                        id: r.get(0)?,
                        course_id: r.get(1)?,
                        kind: r.get(2)?,
                        title: r.get(3)?,
                        due_at: r.get(4)?,
                        notes: r.get(5)?,
                        completed: r.get(6)?,
                        remind_minutes_before: r.get(7)?,
                        component_id: r.get(8)?,
                        score: r.get(9)?,
                    })
                },
            )?
            .into_iter()
            .collect(),
            settings: rows(&tx, "SELECT key, value FROM setting ORDER BY key", |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?
            .into_iter()
            .collect(),
            rule_sets: rows(&tx, "SELECT id, name FROM rule_set ORDER BY id", |r| {
                Ok(RuleSetRow {
                    id: r.get(0)?,
                    name: r.get(1)?,
                })
            })?,
            rule_components: rows(
                &tx,
                "SELECT id, rule_set_id, position, name, kind, weight_pct, item_count, \
                 best_count, item_max, is_final FROM rule_component ORDER BY id",
                |r| {
                    Ok(RuleComponentRow {
                        id: r.get(0)?,
                        rule_set_id: r.get(1)?,
                        position: r.get(2)?,
                        name: r.get(3)?,
                        kind: r.get(4)?,
                        weight_pct: r.get(5)?,
                        item_count: r.get(6)?,
                        best_count: r.get(7)?,
                        item_max: r.get(8)?,
                        is_final: r.get(9)?,
                    })
                },
            )?,
            attendance_bands: rows(
                &tx,
                "SELECT component_id, min_pct, credit_pct FROM attendance_band \
                 ORDER BY component_id, min_pct",
                |r| {
                    Ok(AttendanceBandRow {
                        component_id: r.get(0)?,
                        min_pct: r.get(1)?,
                        credit_pct: r.get(2)?,
                    })
                },
            )?,
            grade_scale: rows(
                &tx,
                "SELECT letter, min_pct, points FROM grade_scale ORDER BY min_pct DESC",
                |r| {
                    Ok(GradeScaleRow {
                        letter: r.get(0)?,
                        min_pct: r.get(1)?,
                        points: r.get(2)?,
                    })
                },
            )?,
        };
        serde_json::to_string_pretty(&doc)
            .map_err(|e| Error::Corrupt(format!("backup could not be written: {e}")))
    }

    /// R22: validates a backup and replaces all data in one transaction; on any error the
    /// database is unchanged. Ids are preserved. `reminder_log` stays as it is. Restore does not
    /// sync sessions and does not read the clock.
    pub fn restore_json(&mut self, json: &str) -> Result<()> {
        let invalid = |msg: &str| Error::Validation(msg.to_string());

        let mut value: Value = serde_json::from_str(json)
            .map_err(|e| Error::Validation(format!("backup is not valid JSON: {e}")))?;
        let root = value
            .as_object_mut()
            .ok_or_else(|| invalid("backup root is not a JSON object"))?;
        if root.get("format").and_then(Value::as_str) != Some(BACKUP_FORMAT) {
            return Err(invalid(
                "backup format is missing or not academic-planner-backup",
            ));
        }
        let version = root
            .get("schema_version")
            .and_then(Value::as_i64)
            .ok_or_else(|| invalid("backup schema_version is missing or not an integer"))?;
        if version > SCHEMA_VERSION {
            return Err(Error::Validation(format!(
                "backup schema version {version} is newer than this app supports \
                 ({SCHEMA_VERSION}); update the app"
            )));
        }
        // R41: version 2 needs the four grading arrays; version 1 ignores them if present.
        for key in GRADING_KEYS {
            if version >= 2 && !root.contains_key(key) {
                return Err(Error::Validation(format!(
                    "backup has the wrong shape: missing field `{key}`"
                )));
            }
            if version < 2 {
                root.remove(key);
            }
        }
        // R75: a current backup must say the kind of every day off.
        if version >= DAY_OFF_KIND_VERSION {
            let days = root.get("holidays").and_then(Value::as_array);
            if days
                .into_iter()
                .flatten()
                .any(|h| h.as_object().is_some_and(|o| !o.contains_key("kind")))
            {
                return Err(invalid("backup has the wrong shape: missing field `kind`"));
            }
        }
        let mut backup: Backup = serde_json::from_value(value)
            .map_err(|e| Error::Validation(format!("backup has the wrong shape: {e}")))?;
        if version < DAY_OFF_KIND_VERSION {
            // R75: an older backup restores every day off as a holiday.
            for h in &mut backup.holidays {
                h.kind = default_day_off_kind();
            }
        }
        if version < 2 {
            // R41: a version 1 backup restores ungraded.
            for c in &mut backup.courses {
                c.credits = None;
                c.rule_set_id = None;
            }
            for a in &mut backup.assessments {
                a.component_id = None;
                a.score = None;
            }
        }
        backup.check_texts()?;

        let tx = self.conn.transaction()?;
        apply(&tx, &backup, version).map_err(apply_error)?;
        tx.commit().map_err(apply_error)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::db::Planner;
    use crate::model::TermInput;

    const LOG_ROW: &str =
        "INSERT INTO reminder_log (kind, ref_id, fire_at) VALUES ('session', 1, '2026-01-05T08:45')";

    fn log_rows(p: &Planner) -> Vec<(String, i64, String)> {
        let mut stmt = p
            .conn
            .prepare("SELECT kind, ref_id, fire_at FROM reminder_log ORDER BY kind, ref_id")
            .unwrap();
        let rows = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        rows
    }

    /// R22: restore, whether it succeeds or fails, leaves the reminder log as it was.
    #[test]
    fn r22_restore_leaves_reminder_log_untouched() {
        let day = |d: u32| NaiveDate::from_ymd_opt(2026, 1, d).unwrap();
        let now = day(25).and_hms_opt(18, 30, 0).unwrap();

        let mut source = Planner::open_in_memory().unwrap();
        source
            .save_term(TermInput {
                id: None,
                name: "Winter 2026".into(),
                start_date: day(5),
                end_date: day(25),
            })
            .unwrap();
        let backup = source.export_json(now).unwrap();

        let mut p = Planner::open_in_memory().unwrap();
        p.conn.execute(LOG_ROW, []).unwrap();
        let before = log_rows(&p);
        assert_eq!(before.len(), 1);

        p.restore_json(&backup).unwrap();
        assert_eq!(log_rows(&p), before, "successful restore touched the log");

        assert!(p.restore_json("this is not a backup").is_err());
        assert_eq!(log_rows(&p), before, "failed restore touched the log");
    }
}
