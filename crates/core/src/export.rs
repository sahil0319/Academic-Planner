//! CSV exports (M3).

use std::io::Write;

use crate::db::Planner;
use crate::error::{Error, Result};

/// R23: a `csv` write failure is an I/O failure of the caller's writer.
fn csv_err(e: csv::Error) -> Error {
    Error::Io(e.into())
}

impl Planner {
    /// R23: writes `attendance.csv` (header row, sorted by date).
    ///
    /// One record per session of every course, term, and status, ordered by
    /// `date, start_time, course code, session id`. `replaces_date` is the date of the
    /// session named by `replaces_session_id`; missing values are empty fields.
    pub fn export_attendance_csv<W: Write>(&self, w: W) -> Result<()> {
        let mut out = csv::WriterBuilder::new()
            .terminator(csv::Terminator::CRLF)
            .from_writer(w);
        out.write_record([
            "date",
            "start",
            "end",
            "course_code",
            "course_name",
            "kind",
            "status",
            "replaces_date",
            "note",
        ])
        .map_err(csv_err)?;
        let mut stmt = self.conn.prepare(
            "SELECT s.date, s.start_time, s.end_time, c.code, c.name, s.kind, s.status, \
                    r.date, s.note \
             FROM session s \
             JOIN course c ON c.id = s.course_id \
             LEFT JOIN session r ON r.id = s.replaces_session_id \
             ORDER BY s.date, s.start_time, c.code, s.id",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let mut record: Vec<String> = Vec::with_capacity(9);
            for i in 0..9 {
                record.push(row.get::<_, Option<String>>(i)?.unwrap_or_default());
            }
            out.write_record(&record).map_err(csv_err)?;
        }
        out.flush()?;
        Ok(())
    }

    /// R23: writes `assessments.csv` (header row, sorted by date).
    ///
    /// One record per assessment, ordered by `due_at, course code, assessment id`.
    /// `completed` is `true` or `false`; a missing note is an empty field.
    pub fn export_assessments_csv<W: Write>(&self, w: W) -> Result<()> {
        let mut out = csv::WriterBuilder::new()
            .terminator(csv::Terminator::CRLF)
            .from_writer(w);
        out.write_record([
            "course_code",
            "kind",
            "title",
            "due_at",
            "completed",
            "notes",
        ])
        .map_err(csv_err)?;
        let mut stmt = self.conn.prepare(
            "SELECT c.code, a.kind, a.title, a.due_at, a.completed, a.notes \
             FROM assessment a \
             JOIN course c ON c.id = a.course_id \
             ORDER BY a.due_at, c.code, a.id",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let completed: bool = row.get(4)?;
            let record = [
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                completed.to_string(),
                row.get::<_, Option<String>>(5)?.unwrap_or_default(),
            ];
            out.write_record(&record).map_err(csv_err)?;
        }
        out.flush()?;
        Ok(())
    }
}
