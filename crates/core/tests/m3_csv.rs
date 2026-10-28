//! M3 CSV exports (R23): attendance.csv and assessments.csv, byte-exact (CRLF terminators).

mod common;

use std::io::{self, Write};

use common::*;
use planner_core::{AssessmentInput, AssessmentKind, CourseInput, Error, Planner, Status};

const ATTENDANCE_HEADER: &str =
    "date,start,end,course_code,course_name,kind,status,replaces_date,note";
const ASSESSMENTS_HEADER: &str = "course_code,kind,title,due_at,completed,notes";

fn attendance(p: &Planner) -> String {
    let mut out = Vec::new();
    p.export_attendance_csv(&mut out).unwrap();
    String::from_utf8(out).unwrap()
}

fn assessments(p: &Planner) -> String {
    let mut out = Vec::new();
    p.export_assessments_csv(&mut out).unwrap();
    String::from_utf8(out).unwrap()
}

/// Lines joined by CRLF, with a CRLF after the last one.
fn crlf(lines: &[&str]) -> String {
    let mut text = lines.join("\r\n");
    text.push_str("\r\n");
    text
}

/// A CSV field that needs quoting: wrapped in quotes, inner quotes doubled.
fn quoted(text: &str) -> String {
    format!("\"{}\"", text.replace('"', "\"\""))
}

/// The records after the header, as lists of fields, read back with the `csv` crate.
fn records(text: &str) -> Vec<Vec<String>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(text.as_bytes());
    reader
        .records()
        .map(|r| r.unwrap().iter().map(str::to_string).collect())
        .collect()
}

/// An empty planner with the fixture term.
fn with_term() -> Planner {
    let mut p = Planner::open_in_memory().unwrap();
    p.save_term(fixture_term()).unwrap();
    p
}

/// R23 (T25): the attendance header and rows match the format exactly: sorted by date then
/// start time, a rescheduled original shows as cancelled, its makeup names the replaced date.
#[test]
fn t25_attendance_csv_header_and_rows_exact() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let original = regular_on(&p, cse.id, "2026-01-12");
    let mut input = makeup_input(cse.id, "2026-01-13", "14:00");
    input.note = Some("lab swap".into());
    let makeup = p.reschedule(original.id, input).unwrap();
    mark(&mut p, makeup.id, Status::Attended, after_term());

    let expected = crlf(&[
        ATTENDANCE_HEADER,
        "2026-01-05,09:00,10:00,CSE101,Intro to Programming,regular,pending,,",
        "2026-01-07,11:00,12:00,CSE101,Intro to Programming,regular,pending,,",
        "2026-01-12,09:00,10:00,CSE101,Intro to Programming,regular,cancelled,,",
        "2026-01-13,14:00,15:00,CSE101,Intro to Programming,makeup,attended,2026-01-12,lab swap",
        "2026-01-14,11:00,12:00,CSE101,Intro to Programming,regular,pending,,",
        "2026-01-19,09:00,10:00,CSE101,Intro to Programming,regular,pending,,",
        "2026-01-21,11:00,12:00,CSE101,Intro to Programming,regular,pending,,",
    ]);
    assert_eq!(attendance(&p), expected);
}

/// R23 (T25): the assessments header and rows match the format exactly: sorted by due time,
/// `completed` as true/false, `due_at` as `YYYY-MM-DDTHH:MM`.
#[test]
fn t25_assessments_csv_header_and_rows_exact() {
    let (mut p, _, cse) = fixture();
    p.create_assessment(assessment_input(
        cse.id,
        AssessmentKind::Assignment,
        "Final project",
        "2026-01-23T17:00",
    ))
    .unwrap();
    let quiz = p
        .create_assessment(AssessmentInput {
            notes: Some("ch 1-3".into()),
            ..assessment_input(cse.id, AssessmentKind::Quiz, "Quiz 1", "2026-01-14T09:00")
        })
        .unwrap();
    p.set_completed(quiz.id, true).unwrap();

    let expected = crlf(&[
        ASSESSMENTS_HEADER,
        "CSE101,quiz,Quiz 1,2026-01-14T09:00,true,ch 1-3",
        "CSE101,assignment,Final project,2026-01-23T17:00,false,",
    ]);
    assert_eq!(assessments(&p), expected);
}

/// R23: an empty planner gives header-only files.
#[test]
fn r23_csv_empty_planner_is_header_only() {
    let p = Planner::open_in_memory().unwrap();
    assert_eq!(attendance(&p), crlf(&[ATTENDANCE_HEADER]));
    assert_eq!(assessments(&p), crlf(&[ASSESSMENTS_HEADER]));
}

/// R23: a comma, a double quote, or a line break in a field quotes it, with inner quotes
/// doubled; reading the output back returns the original text.
#[test]
fn r23_csv_quotes_comma_quote_and_line_break() {
    let course_name = "Algebra, \"Part 1\"\nLecture";
    let session_note = "Bring \"notes\", please\nthanks";
    let title = "Essay, \"draft\"\nv2";
    let assessment_notes = "see \"rubric\", p.2\nok";

    let mut p = with_term();
    let course = p
        .create_course(CourseInput {
            name: course_name.into(),
            ..course_input("ALG1")
        })
        .unwrap();
    let mut makeup = makeup_input(course.id, "2026-01-06", "10:00");
    makeup.note = Some(session_note.into());
    p.add_makeup(makeup).unwrap();
    p.create_assessment(AssessmentInput {
        notes: Some(assessment_notes.into()),
        ..assessment_input(
            course.id,
            AssessmentKind::Assignment,
            title,
            "2026-01-20T10:00",
        )
    })
    .unwrap();

    let attendance_text = attendance(&p);
    let expected_attendance = format!(
        "{ATTENDANCE_HEADER}\r\n2026-01-06,10:00,11:00,ALG1,{},makeup,pending,,{}\r\n",
        quoted(course_name),
        quoted(session_note)
    );
    assert_eq!(attendance_text, expected_attendance);
    let rows = records(&attendance_text);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1][4], course_name);
    assert_eq!(rows[1][8], session_note);

    let assessments_text = assessments(&p);
    let expected_assessments = format!(
        "{ASSESSMENTS_HEADER}\r\nALG1,assignment,{},2026-01-20T10:00,false,{}\r\n",
        quoted(title),
        quoted(assessment_notes)
    );
    assert_eq!(assessments_text, expected_assessments);
    let rows = records(&assessments_text);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1][2], title);
    assert_eq!(rows[1][5], assessment_notes);
}

/// R23: attendance.csv holds every session: archived courses, all four statuses, all terms.
#[test]
fn r23_attendance_csv_includes_archived_all_statuses_all_terms() {
    let (mut p, _, cse, _, _) = synced_fixture();
    for (day, status) in [
        ("2026-01-05", Status::Attended),
        ("2026-01-07", Status::Absent),
        ("2026-01-12", Status::Cancelled),
    ] {
        let id = regular_on(&p, cse.id, day).id;
        mark(&mut p, id, status, after_term());
    }
    p.set_archived(cse.id, true).unwrap();

    p.save_term(planner_core::TermInput {
        id: None,
        name: "Spring 2026".into(),
        start_date: d("2026-02-02"),
        end_date: d("2026-02-27"),
    })
    .unwrap();
    let other = p.create_course(course_input("CSE202")).unwrap();
    p.add_makeup(makeup_input(other.id, "2026-02-10", "10:00"))
        .unwrap();

    let rows = records(&attendance(&p));
    // Header, the six sessions of the archived first-term course, one session of the second.
    assert_eq!(rows.len(), 1 + 6 + 1);
    let body = &rows[1..];
    let statuses = |wanted: &str| body.iter().filter(|r| r[6] == wanted).count();
    assert_eq!(statuses("attended"), 1);
    assert_eq!(statuses("absent"), 1);
    assert_eq!(statuses("cancelled"), 1);
    assert_eq!(statuses("pending"), 4);
    assert_eq!(body.iter().filter(|r| r[3] == "CSE101").count(), 6);
    assert_eq!(body.iter().filter(|r| r[3] == "CSE202").count(), 1);
}

/// R23: sessions on the same date and start time are ordered by course code, then session id,
/// not by creation order.
#[test]
fn r23_attendance_csv_ties_ordered_by_course_code() {
    let mut p = with_term();
    let zzz = p.create_course(course_input("ZZZ900")).unwrap();
    let aaa = p.create_course(course_input("AAA100")).unwrap();
    p.add_makeup(makeup_input(zzz.id, "2026-01-06", "09:00"))
        .unwrap();
    p.add_makeup(makeup_input(aaa.id, "2026-01-06", "09:00"))
        .unwrap();

    let rows = records(&attendance(&p));
    let codes: Vec<&str> = rows[1..].iter().map(|r| r[3].as_str()).collect();
    assert_eq!(codes, vec!["AAA100", "ZZZ900"]);
}

/// R23: assessments with the same due time are ordered by course code, then assessment id.
#[test]
fn r23_assessments_csv_ties_ordered_by_course_code() {
    let mut p = with_term();
    let zzz = p.create_course(course_input("ZZZ900")).unwrap();
    let aaa = p.create_course(course_input("AAA100")).unwrap();
    for course in [&zzz, &aaa] {
        p.create_assessment(assessment_input(
            course.id,
            AssessmentKind::Exam,
            "Final",
            "2026-01-22T10:00",
        ))
        .unwrap();
    }

    let rows = records(&assessments(&p));
    let codes: Vec<&str> = rows[1..].iter().map(|r| r[0].as_str()).collect();
    assert_eq!(codes, vec!["AAA100", "ZZZ900"]);
}

/// R23: a missing note, notes, or replaced session is an empty field.
#[test]
fn r23_csv_missing_values_are_empty() {
    let (mut p, _, cse) = fixture();
    p.add_makeup(makeup_input(cse.id, "2026-01-06", "10:00"))
        .unwrap();
    p.create_assessment(assessment_input(
        cse.id,
        AssessmentKind::Exam,
        "Midterm",
        "2026-01-20T10:00",
    ))
    .unwrap();

    assert_eq!(
        attendance(&p),
        crlf(&[
            ATTENDANCE_HEADER,
            "2026-01-06,10:00,11:00,CSE101,Intro to Programming,makeup,pending,,",
        ])
    );
    assert_eq!(
        assessments(&p),
        crlf(&[
            ASSESSMENTS_HEADER,
            "CSE101,exam,Midterm,2026-01-20T10:00,false,",
        ])
    );
}

/// A writer whose every write and flush fails.
struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("disk full"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("disk full"))
    }
}

/// R23: a writer that fails makes both exports return `Error::Io`.
#[test]
fn r23_csv_write_failure_is_io_error() {
    let (mut p, _, cse, _, _) = synced_fixture();
    p.create_assessment(assessment_input(
        cse.id,
        AssessmentKind::Quiz,
        "Quiz 1",
        "2026-01-14T09:00",
    ))
    .unwrap();

    let err = p.export_attendance_csv(FailingWriter).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "attendance: got {err:?}");
    let err = p.export_assessments_csv(FailingWriter).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "assessments: got {err:?}");
}
