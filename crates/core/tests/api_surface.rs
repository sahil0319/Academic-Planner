//! Freezes the docs/spec.md §5 API: every method is bound to its exact signature, so a
//! signature change fails to compile. No stub is called.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use chrono::{NaiveDate, NaiveDateTime};
use planner_core::*;

#[test]
fn api_signatures_match_spec_section_5() {
    let _: fn(Option<OsString>, Option<PathBuf>) -> Result<PathBuf> = resolve_db_path;
    let _: fn() -> Result<PathBuf> = default_db_path;

    let _: fn(&Path) -> Result<Planner> = Planner::open;
    let _: fn() -> Result<Planner> = Planner::open_in_memory;

    // terms & holidays
    let _: fn(&Planner) -> Result<Option<Term>> = Planner::active_term;
    let _: fn(&mut Planner, TermInput) -> Result<Term> = Planner::save_term;
    let _: fn(&mut Planner, i64, &[Holiday]) -> Result<()> = Planner::set_holidays;

    // courses
    let _: fn(&mut Planner, CourseInput) -> Result<Course> = Planner::create_course;
    let _: fn(&mut Planner, i64, CourseInput) -> Result<Course> = Planner::update_course;
    let _: fn(&mut Planner, i64, bool) -> Result<()> = Planner::set_archived;
    let _: fn(&Planner, bool) -> Result<Vec<Course>> = Planner::list_courses;

    // timetable
    let _: fn(&mut Planner, SlotInput) -> Result<Slot> = Planner::add_slot;
    let _: fn(&mut Planner, i64, SlotInput, NaiveDate) -> Result<Slot> = Planner::change_slot;
    let _: fn(&mut Planner, i64, NaiveDate) -> Result<()> = Planner::end_slot;
    let _: fn(&Planner, NaiveDate) -> Result<Vec<Slot>> = Planner::list_slots;

    // sessions
    let _: fn(&mut Planner, NaiveDate) -> Result<SyncReport> = Planner::sync_sessions;
    let _: fn(&Planner, NaiveDate, NaiveDate, NaiveDateTime) -> Result<Vec<SessionView>> =
        Planner::sessions_between;
    let _: fn(&Planner, i64, NaiveDateTime) -> Result<Vec<SessionView>> =
        Planner::sessions_for_course;
    let _: fn(&mut Planner, i64, Status, NaiveDateTime) -> Result<()> = Planner::set_status;
    let _: fn(&mut Planner, i64, MakeupInput) -> Result<Session> = Planner::reschedule;
    let _: fn(&mut Planner, MakeupInput) -> Result<Session> = Planner::add_makeup;

    // attendance & dashboard
    let _: fn(&Planner, i64) -> Result<AttendanceSummary> = Planner::attendance_summary;
    let _: fn(&Planner) -> Result<Vec<AttendanceSummary>> = Planner::all_summaries;
    let _: fn(&Planner, NaiveDateTime) -> Result<Dashboard> = Planner::dashboard;

    // assessments
    let _: fn(&mut Planner, AssessmentInput) -> Result<Assessment> = Planner::create_assessment;
    let _: fn(&mut Planner, i64, AssessmentInput) -> Result<Assessment> =
        Planner::update_assessment;
    let _: fn(&mut Planner, i64, bool) -> Result<()> = Planner::set_completed;
    let _: fn(&mut Planner, i64) -> Result<()> = Planner::delete_assessment;
    let _: fn(&Planner, AssessmentFilter) -> Result<Vec<Assessment>> = Planner::list_assessments;

    // settings, backup, export
    let _: fn(&Planner) -> Result<Settings> = Planner::settings;
    let _: fn(&mut Planner, &Settings) -> Result<()> = Planner::save_settings;
    let _: fn(&Planner, NaiveDateTime) -> Result<String> = Planner::export_json;
    let _: fn(&mut Planner, &str) -> Result<()> = Planner::restore_json;
    let _: fn(&Planner, Vec<u8>) -> Result<()> = Planner::export_attendance_csv::<Vec<u8>>;
    let _: fn(&Planner, Vec<u8>) -> Result<()> = Planner::export_assessments_csv::<Vec<u8>>;

    // M5
    let _: fn(&mut Planner, NaiveDateTime) -> Result<Vec<Reminder>> = Planner::due_reminders;

    // M9 (§13.3)
    let _: fn(&Planner) -> Result<Vec<RuleSet>> = Planner::list_rule_sets;
    let _: fn(&mut Planner, RuleSetInput) -> Result<RuleSet> = Planner::save_rule_set;
    let _: fn(&mut Planner, i64) -> Result<()> = Planner::delete_rule_set;
    let _: fn(&Planner) -> Result<Vec<GradeRow>> = Planner::grade_scale;
    let _: fn(&mut Planner, &[GradeRow]) -> Result<()> = Planner::save_grade_scale;
    let _: fn(&Planner, i64) -> Result<CourseMarks> = Planner::course_marks;
    let _: fn(&Planner) -> Result<GpaSummary> = Planner::term_gpa;

    // M10 (§13.4)
    let _: fn(&Planner, &str, NaiveDate) -> Result<ImportPlan> = Planner::preview_import;
    let _: fn(&mut Planner, &str, NaiveDate, NaiveDate) -> Result<ImportReport> =
        Planner::apply_import;

    // M11 (§13.6)
    let _: fn(&mut Planner, &[i64]) -> Result<()> = Planner::delete_courses;

    // M14 (§13.9)
    let _: fn(&mut Planner, i64, NaiveDate) -> Result<()> = Planner::archive_term;
    let _: fn(&Planner) -> Result<Vec<PastTermSummary>> = Planner::past_terms;

    // M15 (§13.10)
    let _kinds = [DayOffKind::Holiday, DayOffKind::Midterm, DayOffKind::Final];
    let _ = Holiday {
        date: NaiveDate::default(),
        label: None,
        kind: DayOffKind::Midterm,
    };

    // M16 (§13.11)
    let _ = PublicHoliday {
        date: NaiveDate::default(),
        name: String::new(),
    };
    let _: fn(&mut Planner, &str, &[PublicHoliday]) -> Result<()> = Planner::set_public_holidays;
}
