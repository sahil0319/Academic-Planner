//! Reminders (M5): due reminders logged in `reminder_log`.

use chrono::{Duration, NaiveDateTime};
use rusqlite::{params, Connection};

use crate::db::{col_date, col_time, fmt_datetime, Planner};
use crate::error::Result;
use crate::model::{Reminder, ReminderKind};

/// One reminder that may be due: its key parts, event time `T`, lead, and title.
struct Candidate {
    kind: ReminderKind,
    ref_id: i64,
    event_at: NaiveDateTime,
    lead_minutes: i64,
    title: String,
}

impl Planner {
    /// §5, FR-001: reminders due at `now`, logged in `reminder_log` so none is returned twice.
    ///
    /// Candidates are pending sessions (FR-002) of non-archived courses in the active term;
    /// with no active term the result is empty. A candidate is due when
    /// `fire_at <= now < max(T, fire_at + 1 minute)` with `fire_at = T - lead` (FR-004); anything
    /// outside that window is neither returned nor logged, so missed reminders are skipped.
    /// Every due key is inserted with `INSERT OR IGNORE` in one transaction and returned only
    /// when the insert changed a row. The result is sorted by `fire_at`, then sessions before
    /// assessments, then `ref_id` (FR-005). Does not read `notifications_enabled`.
    pub fn due_reminders(&mut self, now: NaiveDateTime) -> Result<Vec<Reminder>> {
        let class_lead = i64::from(self.settings()?.class_reminder_minutes);
        let tx = self.conn.transaction()?;

        let candidates = session_candidates(&tx, now, class_lead)?;

        let mut due = Vec::new();
        {
            let mut insert = tx.prepare(
                "INSERT OR IGNORE INTO reminder_log (kind, ref_id, fire_at) VALUES (?1, ?2, ?3)",
            )?;
            for c in candidates {
                let Some(fire_at) = fire_at_if_due(&c, now) else {
                    continue;
                };
                let changed =
                    insert.execute(params![c.kind.as_db(), c.ref_id, fmt_datetime(fire_at)])?;
                if changed == 1 {
                    due.push(Reminder {
                        kind: c.kind,
                        ref_id: c.ref_id,
                        fire_at,
                        title: c.title,
                    });
                }
            }
        }
        tx.commit()?;

        due.sort_by_key(|r| (r.fire_at, kind_rank(r.kind), r.ref_id));
        Ok(due)
    }
}

/// FR-004: `fire_at = T - lead` when `fire_at <= now < max(T, fire_at + 1 minute)`, else `None`.
/// A lead that underflows the date range skips the candidate.
fn fire_at_if_due(c: &Candidate, now: NaiveDateTime) -> Option<NaiveDateTime> {
    let fire_at = c
        .event_at
        .checked_sub_signed(Duration::minutes(c.lead_minutes))?;
    let window_end = c
        .event_at
        .max(fire_at.checked_add_signed(Duration::minutes(1))?);
    (fire_at <= now && now < window_end).then_some(fire_at)
}

/// FR-005: sessions sort before assessments at the same `fire_at`.
fn kind_rank(kind: ReminderKind) -> u8 {
    match kind {
        ReminderKind::Session => 0,
        ReminderKind::Assessment => 1,
    }
}

/// Earliest event time whose window can still be open at `now`: no window ends later than
/// `T + 1 minute`, and the stored text format sorts like the time it encodes.
fn earliest_event_text(now: NaiveDateTime) -> String {
    let floor = now.checked_sub_signed(Duration::minutes(1)).unwrap_or(now);
    fmt_datetime(floor)
}

/// FR-002, FR-004: pending sessions (regular and makeup) of non-archived courses in the
/// active term, with lead `class_reminder_minutes`.
fn session_candidates(
    conn: &Connection,
    now: NaiveDateTime,
    lead_minutes: i64,
) -> Result<Vec<Candidate>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.date, s.start_time, c.code, c.name, coalesce(s.room, c.room)
         FROM session s
         JOIN course c ON c.id = s.course_id
         JOIN term t ON t.id = c.term_id
         WHERE s.status = 'pending' AND c.archived = 0 AND t.is_active = 1
           AND s.date || 'T' || s.start_time >= ?1",
    )?;
    let rows = stmt.query_map([earliest_event_text(now)], |r| {
        let date = col_date(r, 1)?;
        let start = col_time(r, 2)?;
        let code: String = r.get(3)?;
        let name: String = r.get(4)?;
        let room: Option<String> = r.get(5)?;
        Ok(Candidate {
            kind: ReminderKind::Session,
            ref_id: r.get(0)?,
            event_at: date.and_time(start),
            lead_minutes,
            title: session_title(&code, &name, start.format("%H:%M"), room.as_deref()),
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// FR-005: `"{code} {name} at {HH:MM}"`, plus `" in {room}"` when the room is not blank.
fn session_title(
    code: &str,
    name: &str,
    start: impl std::fmt::Display,
    room: Option<&str>,
) -> String {
    match room.map(str::trim).filter(|r| !r.is_empty()) {
        Some(room) => format!("{code} {name} at {start} in {room}"),
        None => format!("{code} {name} at {start}"),
    }
}
