//! Session generation, slot changes, exceptions, and attendance marking (M2).

use chrono::{NaiveDate, NaiveDateTime};
use rusqlite::{params, Connection, Params};

use crate::db::{
    fmt_date, fmt_time, load_course, load_session, session_from_row, session_start, to_minute,
    Planner, SESSION_COLS,
};
use crate::error::{Error, Result};
use crate::model::*;

/// R3: the SQL condition (on `course c`) for non-archived courses of the active term.
pub(crate) const ACTIVE_SCOPE: &str =
    "c.archived = 0 AND c.term_id = (SELECT id FROM term WHERE is_active = 1)";

impl Planner {
    /// R7: ends the old slot the day before `effective_from`, inserts the changed slot from
    /// `effective_from`, then syncs. Past sessions keep their stored values.
    pub fn change_slot(
        &mut self,
        id: i64,
        input: SlotInput,
        effective_from: NaiveDate,
    ) -> Result<Slot> {
        let _ = (id, input, effective_from);
        todo!("M2: R7")
    }

    /// R7: makes `last_date` the slot's last valid date, then syncs.
    pub fn end_slot(&mut self, id: i64, last_date: NaiveDate) -> Result<()> {
        let _ = (id, last_date);
        todo!("M2: R7")
    }

    /// R4, R5: idempotently creates the regular sessions the slots produce and deletes
    /// future pending ones they no longer produce.
    pub fn sync_sessions(&mut self, today: NaiveDate) -> Result<SyncReport> {
        let _ = today;
        todo!("M2: R4, R5")
    }

    /// R3, §5: sessions dated `from..=to` of non-archived active-term courses, ordered by
    /// date, start time, then id; `is_past` is relative to `now`. No active term: empty.
    pub fn sessions_between(
        &self,
        from: NaiveDate,
        to: NaiveDate,
        now: NaiveDateTime,
    ) -> Result<Vec<SessionView>> {
        query_views(
            &self.conn,
            &format!("s.date >= ?1 AND s.date <= ?2 AND {ACTIVE_SCOPE}"),
            params![fmt_date(from), fmt_date(to)],
            now,
        )
    }

    /// R3, R15, §5: every session of one course, archived or not (history), ordered by
    /// date, start time, then id; `is_past` is relative to `now`.
    pub fn sessions_for_course(
        &self,
        course_id: i64,
        now: NaiveDateTime,
    ) -> Result<Vec<SessionView>> {
        load_course(&self.conn, course_id)?;
        query_views(&self.conn, "s.course_id = ?1", [course_id], now)
    }

    /// R11, R12: sets a session's status; attended/absent only once the session has started
    /// (`now >= start`, minute precision). Pending and cancelled are allowed at any time.
    pub fn set_status(&mut self, id: i64, status: Status, now: NaiveDateTime) -> Result<()> {
        let session = load_session(&self.conn, id)?;
        let marks_presence = matches!(status, Status::Attended | Status::Absent);
        if marks_presence && to_minute(now) < session_start(&session) {
            return Err(Error::Validation(format!(
                "session {id} has not started yet; attendance can be marked from {} {}",
                fmt_date(session.date),
                fmt_time(session.start_time)
            )));
        }
        self.conn.execute(
            "UPDATE session SET status = ?1 WHERE id = ?2",
            params![status.as_db(), id],
        )?;
        Ok(())
    }

    /// R9: creates a makeup that replaces session `id` and cancels the original, in one
    /// transaction. Rescheduling an attended or absent session is an error.
    pub fn reschedule(&mut self, id: i64, makeup: MakeupInput) -> Result<Session> {
        let _ = (id, makeup);
        todo!("M2: R9")
    }

    /// R10: adds a one-off makeup class (`replaces_session_id = NULL`), pending.
    pub fn add_makeup(&mut self, makeup: MakeupInput) -> Result<Session> {
        let tx = self.conn.transaction()?;
        let session = insert_makeup(&tx, &makeup, None)?;
        tx.commit()?;
        Ok(session)
    }
}

/// §3, §5, FR-018: the status text shown for a session. A cancelled session replaced by a
/// makeup shows "Rescheduled → YYYY-MM-DD" with the makeup's date.
pub(crate) fn display_status(status: Status, replaced_by: Option<NaiveDate>) -> String {
    match (status, replaced_by) {
        (Status::Cancelled, Some(date)) => format!("Rescheduled → {}", fmt_date(date)),
        (Status::Pending, _) => "Pending".into(),
        (Status::Attended, _) => "Attended".into(),
        (Status::Absent, _) => "Absent".into(),
        (Status::Cancelled, None) => "Cancelled".into(),
    }
}

/// §5: session views matching `condition` (SQL on `session s` and `course c`), ordered by
/// date, start time, then id. `replaced_by` is the date of the latest (highest-id) makeup
/// that replaces the session; `is_past = now >= start` at minute precision.
pub(crate) fn query_views(
    conn: &Connection,
    condition: &str,
    params: impl Params,
    now: NaiveDateTime,
) -> Result<Vec<SessionView>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SESSION_COLS}, c.code, c.name,
             (SELECT m.date FROM session m WHERE m.replaces_session_id = s.id
              ORDER BY m.id DESC LIMIT 1)
         FROM session s
         JOIN course c ON c.id = s.course_id
         WHERE {condition}
         ORDER BY s.date, s.start_time, s.id"
    ))?;
    let now = to_minute(now);
    let views = stmt
        .query_map(params, |r| {
            let session = session_from_row(r)?;
            let replaced_by = crate::db::col_date_opt(r, 13)?;
            Ok(SessionView {
                display_status: display_status(session.status, replaced_by),
                is_past: now >= session_start(&session),
                course_code: r.get(11)?,
                course_name: r.get(12)?,
                session,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(views)
}

/// R9, R10: inserts a pending makeup session from `makeup`, replacing `replaces` if given.
/// The end must be after the start, and the course must exist.
fn insert_makeup(
    conn: &Connection,
    makeup: &MakeupInput,
    replaces: Option<i64>,
) -> Result<Session> {
    if makeup.end_time <= makeup.start_time {
        return Err(Error::Validation(format!(
            "makeup end time {} is not after its start time {}",
            fmt_time(makeup.end_time),
            fmt_time(makeup.start_time)
        )));
    }
    load_course(conn, makeup.course_id)?;
    conn.execute(
        "INSERT INTO session (course_id, slot_id, kind, date, start_time, end_time, room,
             status, replaces_session_id, note)
         VALUES (?1, NULL, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            makeup.course_id,
            SessionKind::Makeup.as_db(),
            fmt_date(makeup.date),
            fmt_time(makeup.start_time),
            fmt_time(makeup.end_time),
            makeup.room,
            Status::Pending.as_db(),
            replaces,
            makeup.note
        ],
    )?;
    load_session(conn, conn.last_insert_rowid())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 1, 13).unwrap()
    }

    #[test]
    fn display_status_plain_statuses() {
        assert_eq!(display_status(Status::Pending, None), "Pending");
        assert_eq!(display_status(Status::Attended, None), "Attended");
        assert_eq!(display_status(Status::Absent, None), "Absent");
        assert_eq!(display_status(Status::Cancelled, None), "Cancelled");
    }

    #[test]
    fn display_status_rescheduled_shows_makeup_date() {
        assert_eq!(
            display_status(Status::Cancelled, Some(day())),
            "Rescheduled → 2026-01-13"
        );
    }

    #[test]
    fn display_status_ignores_replacement_unless_cancelled() {
        assert_eq!(display_status(Status::Pending, Some(day())), "Pending");
        assert_eq!(display_status(Status::Attended, Some(day())), "Attended");
        assert_eq!(display_status(Status::Absent, Some(day())), "Absent");
    }
}
