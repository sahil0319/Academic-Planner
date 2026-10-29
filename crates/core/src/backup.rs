//! JSON backup and restore (M3).

use std::collections::BTreeMap;

use chrono::NaiveDateTime;
use rusqlite::{params, Transaction};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::db::{
    fmt_datetime, parse_date, parse_datetime, parse_time, Planner, SCHEMA_VERSION, SETTING_THEME,
};
use crate::error::{Error, Result};
use crate::model::ThemeMode;

/// R21: value of the `format` key of every backup document.
const BACKUP_FORMAT: &str = "academic-planner-backup";

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
}

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

/// R22: replaces the content of the seven data tables with the backup rows.
fn apply(tx: &Transaction, b: &Backup) -> rusqlite::Result<()> {
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
            "INSERT INTO holiday (term_id, date, label) VALUES (?1, ?2, ?3)",
            params![h.term_id, h.date, h.label],
        )?;
    }
    for c in &b.courses {
        tx.execute(
            "INSERT INTO course (id, term_id, code, name, instructor, room, notes, \
             min_attendance_pct, archived) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                c.id,
                c.term_id,
                c.code,
                c.name,
                c.instructor,
                c.room,
                c.notes,
                c.min_attendance_pct,
                c.archived
            ],
        )?;
    }
    for s in &b.slots {
        tx.execute(
            "INSERT INTO slot (id, course_id, weekday, start_time, end_time, room, valid_from, \
             valid_to) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                s.id,
                s.course_id,
                s.weekday,
                s.start_time,
                s.end_time,
                s.room,
                s.valid_from,
                s.valid_to
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
             remind_minutes_before) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                a.id,
                a.course_id,
                a.kind,
                a.title,
                a.due_at,
                a.notes,
                a.completed,
                a.remind_minutes_before
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
                "SELECT term_id, date, label FROM holiday ORDER BY term_id, date",
                |r| {
                    Ok(HolidayRow {
                        term_id: r.get(0)?,
                        date: r.get(1)?,
                        label: r.get(2)?,
                    })
                },
            )?,
            courses: rows(
                &tx,
                "SELECT id, term_id, code, name, instructor, room, notes, min_attendance_pct, \
                 archived FROM course ORDER BY id",
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
                    })
                },
            )?,
            slots: rows(
                &tx,
                "SELECT id, course_id, weekday, start_time, end_time, room, valid_from, valid_to \
                 FROM slot ORDER BY id",
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
                 remind_minutes_before FROM assessment ORDER BY id",
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
        };
        serde_json::to_string_pretty(&doc)
            .map_err(|e| Error::Corrupt(format!("backup could not be written: {e}")))
    }

    /// R22: validates a backup and replaces all data in one transaction; on any error the
    /// database is unchanged. Ids are preserved. `reminder_log` stays as it is. Restore does not
    /// sync sessions and does not read the clock.
    pub fn restore_json(&mut self, json: &str) -> Result<()> {
        let invalid = |msg: &str| Error::Validation(msg.to_string());

        let value: Value = serde_json::from_str(json)
            .map_err(|e| Error::Validation(format!("backup is not valid JSON: {e}")))?;
        let root = value
            .as_object()
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
        let backup: Backup = serde_json::from_value(value)
            .map_err(|e| Error::Validation(format!("backup has the wrong shape: {e}")))?;
        backup.check_texts()?;

        let tx = self.conn.transaction()?;
        apply(&tx, &backup).map_err(apply_error)?;
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
