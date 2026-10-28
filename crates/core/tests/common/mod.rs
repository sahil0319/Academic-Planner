//! Shared fixtures for core integration tests (docs/spec.md §8): active term
//! 2026-01-05 (Mon) → 2026-01-25 (Sun); course CSE101 with target 75;
//! slot A Mon 09:00–10:00; slot B Wed 11:00–12:00.

// Each test binary uses a different subset of these helpers.
#![allow(dead_code)]

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use planner_core::{Course, CourseInput, Planner, SlotInput, Term, TermInput};

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
