//! A small iCalendar reader that keeps only public holidays (M16, R81).
//!
//! Pure: it takes the feed text and returns the days; it does no I/O. The feed is Google's public
//! holiday calendar, where each VEVENT is already expanded (no `RRULE` handling, D23).

use std::collections::BTreeSet;

use chrono::{Duration, NaiveDate};
use planner_core::PublicHoliday;

/// R81, D22: an event is a public holiday when its unescaped DESCRIPTION contains this text
/// (case sensitive). Google writes "Observance" for the rest, which is dropped.
const PUBLIC_HOLIDAY_MARK: &str = "Public holiday";

/// Longest all-day span expanded into single dates; a guard against a malformed `DTEND`.
const MAX_SPAN_DAYS: i64 = 366;

/// One VEVENT while it is being read.
#[derive(Default)]
struct Event {
    summary: Option<String>,
    description: Option<String>,
    start: Option<String>,
    end: Option<String>,
}

/// R81: the public holidays in an iCalendar feed, one entry per date, sorted by date then name,
/// each `(date, name)` once.
///
/// Rules (D22, D23): lines end in CRLF or LF; a line that starts with a space or a tab continues
/// the previous one (that first character is dropped); the property name is the text before the
/// first `:` or `;`; `\n`, `\N`, `\,`, `\;`, `\\` are unescaped. `DTSTART` is a date or a
/// date-time and its first 8 digits are the date. An all-day event gives one entry per date up
/// to its exclusive `DTEND` (a missing or earlier `DTEND` means one day); a date-time event gives
/// one entry on its `DTSTART` date. An event without `SUMMARY` or `DTSTART` is skipped. Alarms
/// inside an event are ignored.
pub fn parse_public_holidays(ics: &str) -> Vec<PublicHoliday> {
    let mut found: BTreeSet<(NaiveDate, String)> = BTreeSet::new();
    let mut event: Option<Event> = None;
    let mut alarm_depth = 0usize;

    for line in unfold(ics) {
        let Some((name, value)) = split_property(&line) else {
            continue;
        };
        let name = name.to_ascii_uppercase();
        match (name.as_str(), event.as_mut()) {
            ("BEGIN", None) if value.eq_ignore_ascii_case("VEVENT") => {
                event = Some(Event::default());
                alarm_depth = 0;
            }
            ("BEGIN", Some(_)) => alarm_depth += 1,
            ("END", Some(_)) if alarm_depth > 0 => alarm_depth -= 1,
            ("END", Some(_)) if value.eq_ignore_ascii_case("VEVENT") => {
                if let Some(done) = event.take() {
                    collect(&done, &mut found);
                }
            }
            (_, Some(current)) if alarm_depth == 0 => match name.as_str() {
                "SUMMARY" => current.summary = Some(unescape(value)),
                "DESCRIPTION" => current.description = Some(unescape(value)),
                "DTSTART" => current.start = Some(value.to_string()),
                "DTEND" => current.end = Some(value.to_string()),
                _ => {}
            },
            _ => {}
        }
    }

    found
        .into_iter()
        .map(|(date, name)| PublicHoliday { date, name })
        .collect()
}

/// Adds the dates of `event` to `found` when it is a public holiday with a name and a start.
fn collect(event: &Event, found: &mut BTreeSet<(NaiveDate, String)>) {
    let is_public = event
        .description
        .as_deref()
        .is_some_and(|d| d.contains(PUBLIC_HOLIDAY_MARK));
    let (Some(summary), Some(start_text)) = (&event.summary, &event.start) else {
        return;
    };
    if !is_public {
        return;
    }
    let Some(start) = leading_date(start_text) else {
        return;
    };
    let all_day = start_text.len() == 8;
    let mut last = start;
    if all_day {
        if let Some(end) = event.end.as_deref().and_then(leading_date) {
            // DTEND is exclusive: the last day is the day before it.
            last = (end - Duration::days(1)).max(start);
        }
    }
    let mut date = start;
    let mut days = 0;
    while date <= last && days < MAX_SPAN_DAYS {
        found.insert((date, summary.clone()));
        let Some(next) = date.succ_opt() else { break };
        date = next;
        days += 1;
    }
}

/// The date from the first 8 digits of a `DTSTART` or `DTEND` value (`20260414` or
/// `20260505T210000Z`).
fn leading_date(value: &str) -> Option<NaiveDate> {
    let digits = value.get(..8)?;
    NaiveDate::parse_from_str(digits, "%Y%m%d").ok()
}

/// Splits the text into logical lines: CRLF or LF ends a line, and a line that starts with a
/// space or a tab continues the previous one.
fn unfold(ics: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for raw in ics.split('\n') {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        if let Some(rest) = raw.strip_prefix([' ', '\t']) {
            if let Some(last) = lines.last_mut() {
                last.push_str(rest);
                continue;
            }
        }
        lines.push(raw.to_string());
    }
    lines
}

/// Splits a logical line into the property name (before the first `:` or `;`) and its value
/// (after the first `:` that is not inside a quoted parameter value).
fn split_property(line: &str) -> Option<(&str, &str)> {
    let name_end = line.find([':', ';'])?;
    let name = &line[..name_end];
    let mut quoted = false;
    for (i, c) in line.char_indices().skip(name_end) {
        match c {
            '"' => quoted = !quoted,
            ':' if !quoted => return Some((name, &line[i + 1..])),
            _ => {}
        }
    }
    None
}

/// Unescapes iCalendar text: `\n` and `\N` are a line break, `\,` `\;` `\\` stand for the
/// character itself.
fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n' | 'N') => out.push('\n'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}
