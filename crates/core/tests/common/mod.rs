//! Shared fixtures for core integration tests (docs/spec.md §8): active term
//! 2026-01-05 (Mon) → 2026-01-25 (Sun); course CSE101 with target 75;
//! slot A Mon 09:00–10:00; slot B Wed 11:00–12:00.

// Each test binary uses a different subset of these helpers.
#![allow(dead_code)]

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use planner_core::{
    AssessmentInput, AssessmentKind, Course, CourseInput, Holiday, MakeupInput, Planner, Session,
    SessionKind, Slot, SlotInput, Status, Term, TermInput,
};

pub fn d(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

pub fn t(s: &str) -> NaiveTime {
    NaiveTime::parse_from_str(s, "%H:%M").unwrap()
}

pub fn dt(s: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").unwrap()
}

pub fn fixture_term() -> TermInput {
    TermInput {
        id: None,
        name: "Winter 2026".into(),
        start_date: d("2026-01-05"),
        end_date: d("2026-01-25"),
    }
}

pub fn course_input(code: &str) -> CourseInput {
    CourseInput {
        code: code.into(),
        name: format!("{code} course"),
        instructor: None,
        room: Some("A-101".into()),
        notes: None,
        min_attendance_pct: None,
    }
}

pub fn fixture_course() -> CourseInput {
    CourseInput {
        name: "Intro to Programming".into(),
        min_attendance_pct: Some(75),
        ..course_input("CSE101")
    }
}

pub fn slot_a(course_id: i64) -> SlotInput {
    SlotInput {
        course_id,
        weekday: Weekday::Mon,
        start_time: t("09:00"),
        end_time: t("10:00"),
        room: None,
        valid_from: d("2026-01-05"),
        valid_to: None,
    }
}

pub fn slot_b(course_id: i64) -> SlotInput {
    SlotInput {
        course_id,
        weekday: Weekday::Wed,
        start_time: t("11:00"),
        end_time: t("12:00"),
        room: None,
        valid_from: d("2026-01-05"),
        valid_to: None,
    }
}

/// In-memory planner with the fixture term (active) and course CSE101.
pub fn fixture() -> (Planner, Term, Course) {
    let mut p = Planner::open_in_memory().unwrap();
    let term = p.save_term(fixture_term()).unwrap();
    let course = p.create_course(fixture_course()).unwrap();
    (p, term, course)
}

// ---- M2 helpers (sessions, holidays, attendance) ----

/// A moment after the whole fixture term: every fixture session has started by then.
pub fn after_term() -> NaiveDateTime {
    dt("2026-02-01T00:00")
}

/// `YYYY-MM-DD` text of a date, for readable assertions.
pub fn ymd(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Dates of the sessions, in the order given, as `YYYY-MM-DD` text.
pub fn dates(sessions: &[Session]) -> Vec<String> {
    sessions.iter().map(|s| ymd(s.date)).collect()
}

/// Fixture with slot A and slot B added and synced with today = 2026-01-05.
/// Returns (planner, term, course, slot A, slot B).
pub fn synced_fixture() -> (Planner, Term, Course, Slot, Slot) {
    let (mut p, term, cse) = fixture();
    let a = p.add_slot(slot_a(cse.id)).unwrap();
    let b = p.add_slot(slot_b(cse.id)).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    (p, term, cse, a, b)
}

/// Every session of a course, ordered by date, start time, then id.
pub fn sessions_of(p: &Planner, course_id: i64) -> Vec<Session> {
    let mut list: Vec<Session> = p
        .sessions_for_course(course_id, after_term())
        .unwrap()
        .into_iter()
        .map(|v| v.session)
        .collect();
    list.sort_by_key(|s| (s.date, s.start_time, s.id));
    list
}

/// The regular session of a course on a date; panics when there is none.
pub fn regular_on(p: &Planner, course_id: i64, date: &str) -> Session {
    sessions_of(p, course_id)
        .into_iter()
        .find(|s| s.kind == SessionKind::Regular && s.date == d(date))
        .unwrap_or_else(|| panic!("no regular session on {date}"))
}

/// Holidays on the given dates, without labels.
pub fn holidays(days: &[&str]) -> Vec<Holiday> {
    days.iter()
        .map(|day| Holiday {
            date: d(day),
            label: None,
        })
        .collect()
}

/// A labeled holiday on every date from `from` to `to`, both inclusive (an exam period).
pub fn holiday_range(from: &str, to: &str, label: &str) -> Vec<Holiday> {
    let mut out = Vec::new();
    let mut day = d(from);
    while day <= d(to) {
        out.push(Holiday {
            date: day,
            label: Some(label.into()),
        });
        day = day.succ_opt().unwrap();
    }
    out
}

/// A one-hour makeup input on `date` starting at `start` (`HH:MM`).
pub fn makeup_input(course_id: i64, date: &str, start: &str) -> MakeupInput {
    let start_time = t(start);
    MakeupInput {
        course_id,
        date: d(date),
        start_time,
        end_time: start_time + chrono::Duration::hours(1),
        room: None,
        note: None,
    }
}

/// Sets a status through the public API.
pub fn mark(p: &mut Planner, session_id: i64, status: Status, now: NaiveDateTime) {
    p.set_status(session_id, status, now).unwrap();
}

/// Adds `n` makeups to a course, all in the past of `after_term()`, each with `status`.
/// Dates cycle through the 21 term days; start times differ so no two makeups coincide.
pub fn add_marked_makeups(p: &mut Planner, course_id: i64, n: usize, status: Status) {
    for i in 0..n {
        let day = d("2026-01-05") + chrono::Duration::days((i % 21) as i64);
        let start = t("06:00") + chrono::Duration::hours((i / 21) as i64);
        let makeup = p
            .add_makeup(MakeupInput {
                course_id,
                date: day,
                start_time: start,
                end_time: start + chrono::Duration::minutes(30),
                room: None,
                note: None,
            })
            .unwrap();
        if status != Status::Pending {
            mark(p, makeup.id, status, after_term());
        }
    }
}

/// An assessment input of the given kind.
pub fn assessment_input(
    course_id: i64,
    kind: AssessmentKind,
    title: &str,
    due: &str,
) -> AssessmentInput {
    AssessmentInput {
        course_id,
        kind,
        title: title.into(),
        due_at: dt(due),
        notes: None,
        remind_minutes_before: None,
    }
}

/// Asserts two optional percentages are equal to within rounding noise.
pub fn assert_pct(actual: Option<f64>, expected: Option<f64>) {
    match (actual, expected) {
        (None, None) => {}
        (Some(a), Some(e)) => assert!((a - e).abs() < 1e-9, "pct {a} != {e}"),
        _ => panic!("pct {actual:?} != {expected:?}"),
    }
}
