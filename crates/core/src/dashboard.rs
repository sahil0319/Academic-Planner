//! Dashboard queries (M2).

use chrono::NaiveDateTime;

use crate::db::{active_term_id, session_end, session_start, to_minute, Planner};
use crate::error::Result;
use crate::model::{AssessmentFilter, Dashboard, Status};
use crate::schedule::{query_views, ACTIVE_SCOPE};

impl Planner {
    /// R16–R20: today's classes, next class, upcoming and overdue assessments, sessions
    /// awaiting attendance, and below-target courses, all relative to `now`.
    ///
    /// R3: every part covers only non-archived courses of the active term; with no active
    /// term the dashboard is empty (research R-D2, R-D10). Times compare at minute precision.
    /// - R16 `today`: sessions dated `now.date()`, by start time, then course code, then id.
    /// - R17 `next_class`: the earliest pending session with start strictly after `now`.
    /// - R18 `upcoming` / R19 `overdue`: `list_assessments` with the same `now`.
    /// - R20 `awaiting`: pending sessions with `end <= now`, oldest first; in-progress
    ///   sessions are not included.
    /// - R14 `below_target`: the `all_summaries` entries that are below their target.
    pub fn dashboard(&self, now: NaiveDateTime) -> Result<Dashboard> {
        if active_term_id(&self.conn)?.is_none() {
            return Ok(Dashboard {
                today: Vec::new(),
                next_class: None,
                upcoming: Vec::new(),
                overdue: Vec::new(),
                awaiting: Vec::new(),
                below_target: Vec::new(),
            });
        }
        let minute = to_minute(now);

        let mut today = self.sessions_between(now.date(), now.date(), now)?;
        today.sort_by(|a, b| {
            (a.session.start_time, &a.course_code, a.session.id).cmp(&(
                b.session.start_time,
                &b.course_code,
                b.session.id,
            ))
        });

        // Pending sessions of the active scope, ordered by date, start time, then id.
        let pending = query_views(
            &self.conn,
            &format!("s.status = ?1 AND {ACTIVE_SCOPE}"),
            [Status::Pending.as_db()],
            now,
        )?;
        let next_class = pending
            .iter()
            .find(|v| session_start(&v.session) > minute)
            .cloned();
        let awaiting = pending
            .into_iter()
            .filter(|v| session_end(&v.session) <= minute)
            .collect();

        Ok(Dashboard {
            today,
            next_class,
            upcoming: self.list_assessments(AssessmentFilter::Upcoming { now })?,
            overdue: self.list_assessments(AssessmentFilter::Overdue { now })?,
            awaiting,
            below_target: self
                .all_summaries()?
                .into_iter()
                .filter(|s| s.below_target)
                .collect(),
        })
    }
}
