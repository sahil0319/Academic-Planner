//! Session generation, slot changes, exceptions, and attendance marking (M2).

use chrono::{NaiveDate, NaiveDateTime};

use crate::db::Planner;
use crate::error::Result;
use crate::model::*;

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

    /// Sessions dated `from..=to` for non-archived courses; `is_past` is relative to `now`.
    pub fn sessions_between(
        &self,
        from: NaiveDate,
        to: NaiveDate,
        now: NaiveDateTime,
    ) -> Result<Vec<SessionView>> {
        let _ = (from, to, now);
        todo!("M2: R3, R16")
    }

    /// All sessions of one course; `is_past` is relative to `now`.
    pub fn sessions_for_course(
        &self,
        course_id: i64,
        now: NaiveDateTime,
    ) -> Result<Vec<SessionView>> {
        let _ = (course_id, now);
        todo!("M2: R15")
    }

    /// R11, R12: sets a session's status; attended/absent only once the session has started.
    pub fn set_status(&mut self, id: i64, status: Status, now: NaiveDateTime) -> Result<()> {
        let _ = (id, status, now);
        todo!("M2: R11, R12")
    }

    /// R9: creates a makeup that replaces session `id` and cancels the original, in one
    /// transaction. Rescheduling an attended or absent session is an error.
    pub fn reschedule(&mut self, id: i64, makeup: MakeupInput) -> Result<Session> {
        let _ = (id, makeup);
        todo!("M2: R9")
    }

    /// R10: adds a one-off makeup class.
    pub fn add_makeup(&mut self, makeup: MakeupInput) -> Result<Session> {
        let _ = makeup;
        todo!("M2: R10")
    }
}
