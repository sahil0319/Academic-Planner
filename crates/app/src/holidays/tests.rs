//! Unit tests for the public holiday module (R76, R77, R81; T75, T76, T79). The parser runs on
//! saved sample feeds in `testdata/`; the service runs with a fake fetcher. Nothing here opens a
//! network connection, and the database is in memory.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use chrono::NaiveDate;
use planner_core::{Planner, PublicHoliday};

use super::calendars::{calendar_for, calendars, calendars_by_name};
use super::fetch::{feed_url, HolidayFetcher};
use super::ics::parse_public_holidays;
use super::locale::{country_from_locale, resolve_country, setting_value, Country};
use super::service::{apply_fetch, fetch_events, term_years, FetchOutcome};
use crate::viewmodel::{public_holiday_retry_visible, public_holiday_status_text, HolidayFetch};

const SAMPLE: &str = include_str!("testdata/sample.ics");
const DATETIME: &str = include_str!("testdata/datetime.ics");
const YEARS: &str = include_str!("testdata/years.ics");

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

fn ph(date: NaiveDate, name: &str) -> PublicHoliday {
    PublicHoliday {
        date,
        name: name.to_string(),
    }
}

fn planner() -> Planner {
    Planner::open_in_memory().unwrap()
}

/// One all-day VEVENT with no `DTEND` (one day) and the given description line text.
fn all_day_event(date: &str, name: &str, description: &str) -> String {
    format!(
        "BEGIN:VEVENT\r\nDTSTART;VALUE=DATE:{date}\r\nSUMMARY:{name}\r\n\
         DESCRIPTION:{description}\r\nEND:VEVENT\r\n"
    )
}

/// Wraps VEVENT texts in a VCALENDAR.
fn feed(events: &[String]) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n{}END:VCALENDAR\r\n",
        events.concat()
    )
}

/// The fake fetcher: returns a fixed result and counts its calls (no network).
struct FakeFetcher {
    reply: Result<String, String>,
    calls: AtomicUsize,
    urls: Mutex<Vec<String>>,
}

impl FakeFetcher {
    fn ok(body: &str) -> Self {
        Self::with(Ok(body.to_string()))
    }

    fn err(message: &str) -> Self {
        Self::with(Err(message.to_string()))
    }

    fn with(reply: Result<String, String>) -> Self {
        FakeFetcher {
            reply,
            calls: AtomicUsize::new(0),
            urls: Mutex::new(Vec::new()),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl HolidayFetcher for FakeFetcher {
    fn fetch(&self, url: &str) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.urls.lock().unwrap().push(url.to_string());
        self.reply.clone()
    }
}

/// R81, T75: the public holiday is kept, the observance is dropped, the two-day event (exclusive
/// `DTEND`) gives two dates, folded lines (space and tab continuations) are joined, escaped text
/// is unescaped, and the result is sorted by date.
#[test]
fn t75_public_holiday_kept_observance_dropped_folded_name_joined() {
    let days = parse_public_holidays(SAMPLE);

    assert_eq!(
        days,
        vec![
            ph(d(2026, 2, 21), "International Mother Language Day"),
            ph(d(2026, 3, 20), "Eid ul-Fitr, Holiday"),
            ph(d(2026, 3, 21), "Eid ul-Fitr, Holiday"),
            ph(d(2026, 4, 14), "Pohela Boishakh (Bengali New Year)"),
        ]
    );
}

/// R81, T75: a date-time `DTSTART` gives one entry on its own date, also when the end is on the
/// next day or the time carries a `Z`.
#[test]
fn t75_date_time_event_uses_its_date() {
    let days = parse_public_holidays(DATETIME);

    assert_eq!(
        days,
        vec![
            ph(d(2026, 5, 5), "Night Event"),
            ph(d(2026, 6, 1), "Evening Event"),
        ]
    );
}

/// R81, T75: lines split on a bare LF as well as CRLF, and folding still works.
#[test]
fn t75_lf_only_line_endings() {
    let lf = SAMPLE.replace("\r\n", "\n");
    assert!(!lf.contains('\r'));
    assert_eq!(parse_public_holidays(&lf), parse_public_holidays(SAMPLE));

    let folded = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nDTSTART;VALUE=DATE:20260110\n\
                  SUMMARY:Found\n ers Day\nDESCRIPTION:Public holiday\nEND:VEVENT\n\
                  END:VCALENDAR\n";
    assert_eq!(
        parse_public_holidays(folded),
        vec![ph(d(2026, 1, 10), "Founders Day")]
    );
}

/// R81, T75: an event without a `SUMMARY` or without a `DTSTART` is skipped; the others stay.
#[test]
fn t75_event_without_summary_or_start_is_skipped() {
    let no_summary = "BEGIN:VEVENT\r\nDTSTART;VALUE=DATE:20260301\r\n\
                      DESCRIPTION:Public holiday\r\nEND:VEVENT\r\n"
        .to_string();
    let no_start = "BEGIN:VEVENT\r\nSUMMARY:No Start Day\r\n\
                    DESCRIPTION:Public holiday\r\nEND:VEVENT\r\n"
        .to_string();
    let ok = all_day_event("20260302", "Kept Day", "Public holiday");

    let days = parse_public_holidays(&feed(&[no_summary, no_start, ok]));

    assert_eq!(days, vec![ph(d(2026, 3, 2), "Kept Day")]);
}

/// R81, T75: the same date and name twice is listed once; two names on one date are both listed,
/// ordered by name.
#[test]
fn t75_same_date_and_name_twice_is_listed_once() {
    let events = [
        all_day_event("20260410", "Beta Day", "Public holiday"),
        all_day_event("20260410", "Alpha Day", "Public holiday"),
        all_day_event("20260410", "Beta Day", "Public holiday"),
    ];

    let days = parse_public_holidays(&feed(&events));

    assert_eq!(
        days,
        vec![
            ph(d(2026, 4, 10), "Alpha Day"),
            ph(d(2026, 4, 10), "Beta Day")
        ]
    );
}

/// R81, T75 (D22): the "Public holiday" check is case sensitive; an event with another text or
/// no description is dropped.
#[test]
fn t75_description_check_is_case_sensitive() {
    let no_description = "BEGIN:VEVENT\r\nDTSTART;VALUE=DATE:20260504\r\n\
                          SUMMARY:No Description Day\r\nEND:VEVENT\r\n"
        .to_string();
    let events = [
        all_day_event("20260501", "Lower Day", "public holiday"),
        all_day_event("20260502", "Upper Day", "PUBLIC HOLIDAY"),
        all_day_event("20260503", "Exact Day", "Public holiday"),
        no_description,
        all_day_event("20260505", "Other Day", "Observance"),
    ];

    let days = parse_public_holidays(&feed(&events));

    assert_eq!(days, vec![ph(d(2026, 5, 3), "Exact Day")]);
}

/// R76, T76, D8, D38: the region part of a locale maps to a table country or Off.
#[test]
fn t76_locales_map_to_country_or_off() {
    let code = |c: &str| Country::Code(c.to_string());

    assert_eq!(country_from_locale("en-BD"), code("BD"));
    assert_eq!(country_from_locale("bn-BD"), code("BD"));
    assert_eq!(country_from_locale("en-US"), code("US"));
    assert_eq!(country_from_locale("fr"), Country::Off);
    assert_eq!(country_from_locale("xx-ZZ"), Country::Off);
    assert_eq!(country_from_locale("bn_BD.UTF-8"), code("BD"));
    assert_eq!(country_from_locale("en-Latn-BD"), code("BD"));
    assert_eq!(country_from_locale(""), Country::Off);
    assert_eq!(country_from_locale("en-bd"), code("BD"));
}

/// R76, D8, D27: the stored setting wins over the OS region; `off` is Off; an absent setting
/// follows the OS region; a stored code outside the table, or in lower case, is Off.
#[test]
fn t76_resolve_country_setting_wins() {
    let code = |c: &str| Country::Code(c.to_string());

    assert_eq!(resolve_country(Some("off"), Some("en-BD")), Country::Off);
    assert_eq!(resolve_country(Some("GB"), None), code("GB"));
    assert_eq!(resolve_country(Some("GB"), Some("en-BD")), code("GB"));
    assert_eq!(resolve_country(None, Some("en-BD")), code("BD"));
    assert_eq!(resolve_country(None, Some("fr")), Country::Off);
    assert_eq!(resolve_country(None, None), Country::Off);
    assert_eq!(resolve_country(Some("ZZ"), Some("en-BD")), Country::Off);
    assert_eq!(resolve_country(Some("bd"), Some("en-BD")), Country::Off);

    assert_eq!(setting_value(&Country::Off), "off");
    assert_eq!(setting_value(&code("BD")), "BD");
}

/// R76, D6, D21: the built-in table parses, holds the required countries once each, and every
/// id is a Google holiday calendar id.
#[test]
fn t76_calendar_table_is_valid() {
    let required = [
        "BD", "IN", "PK", "NP", "LK", "MY", "SG", "ID", "AE", "SA", "GB", "IE", "US", "CA", "AU",
        "NZ", "DE", "FR",
    ];
    let table = calendars();

    for code in required {
        assert!(calendar_for(code).is_some(), "{code} missing");
    }
    for calendar in table {
        let same = table.iter().filter(|c| c.code == calendar.code).count();
        assert_eq!(same, 1, "{} listed {same} times", calendar.code);
        assert!(
            !calendar.name.trim().is_empty(),
            "{} has no name",
            calendar.code
        );
        assert!(
            calendar
                .id
                .ends_with("#holiday@group.v.calendar.google.com"),
            "{}: {}",
            calendar.code,
            calendar.id
        );
    }
    assert_eq!(
        calendar_for("BD").unwrap().id,
        "en.bd#holiday@group.v.calendar.google.com"
    );
    assert!(calendar_for("ZZ").is_none());

    let names: Vec<&str> = calendars_by_name()
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    assert_eq!(names.len(), table.len());
}

/// R77: the request URL percent-encodes the calendar id (`#` and `@`).
#[test]
fn r77_feed_url_encodes_the_calendar_id() {
    assert_eq!(
        feed_url("en.bd#holiday@group.v.calendar.google.com"),
        "https://calendar.google.com/calendar/ical/\
         en.bd%23holiday%40group.v.calendar.google.com/public/basic.ics"
    );
}

/// T79, R77: a failed fetch leaves the cached rows; the status reads "Couldn't load public
/// holidays" and Retry is shown.
#[test]
fn t79_failed_fetch_keeps_cached_rows() {
    let mut planner = planner();
    let cached = vec![
        ph(d(2026, 2, 21), "International Mother Language Day"),
        ph(d(2026, 3, 26), "Independence Day"),
    ];
    planner.set_public_holidays("BD", &cached).unwrap();
    let fetcher = FakeFetcher::err("timed out");

    let result = fetch_events(&fetcher, "BD", 2026..=2026);

    assert!(result.is_err());
    assert_eq!(fetcher.calls(), 1);
    assert_eq!(
        apply_fetch(&mut planner, "BD", result),
        FetchOutcome::Failed
    );
    assert_eq!(
        planner
            .public_holidays_between("BD", d(2026, 1, 1), d(2026, 12, 31))
            .unwrap(),
        cached
    );
    let today = d(2026, 10, 31);
    assert_eq!(
        public_holiday_status_text(HolidayFetch::Failed, today),
        "Couldn't load public holidays"
    );
    assert!(public_holiday_retry_visible(HolidayFetch::Failed));
}

/// R77: a successful fetch replaces the country's rows with the events of the term years and
/// leaves other countries alone; the request goes to the table's feed URL.
#[test]
fn r77_successful_fetch_replaces_country_rows() {
    let mut planner = planner();
    planner
        .set_public_holidays("BD", &[ph(d(2026, 6, 1), "Old Day")])
        .unwrap();
    let india = vec![ph(d(2026, 1, 26), "Republic Day")];
    planner.set_public_holidays("IN", &india).unwrap();
    let fetcher = FakeFetcher::ok(SAMPLE);

    let result = fetch_events(&fetcher, "BD", 2026..=2026);

    let expected = vec![
        ph(d(2026, 2, 21), "International Mother Language Day"),
        ph(d(2026, 3, 20), "Eid ul-Fitr, Holiday"),
        ph(d(2026, 3, 21), "Eid ul-Fitr, Holiday"),
        ph(d(2026, 4, 14), "Pohela Boishakh (Bengali New Year)"),
    ];
    assert_eq!(result.as_ref().unwrap(), &expected);
    assert_eq!(fetcher.calls(), 1);
    let id = &calendar_for("BD").unwrap().id;
    assert_eq!(fetcher.urls.lock().unwrap().as_slice(), [feed_url(id)]);
    assert_eq!(
        apply_fetch(&mut planner, "BD", result),
        FetchOutcome::Updated
    );
    let (from, to) = (d(2026, 1, 1), d(2026, 12, 31));
    assert_eq!(
        planner.public_holidays_between("BD", from, to).unwrap(),
        expected
    );
    assert_eq!(
        planner.public_holidays_between("IN", from, to).unwrap(),
        india
    );
}

/// R77, D26: a calendar with no event in the term years is a success that clears the country.
#[test]
fn r77_empty_calendar_clears_country_rows() {
    let mut planner = planner();
    planner
        .set_public_holidays("BD", &[ph(d(2026, 6, 1), "Old Day")])
        .unwrap();
    let fetcher = FakeFetcher::ok(&feed(&[]));

    let result = fetch_events(&fetcher, "BD", 2026..=2026);

    assert_eq!(result, Ok(Vec::new()));
    assert_eq!(
        apply_fetch(&mut planner, "BD", result),
        FetchOutcome::Updated
    );
    assert!(planner
        .public_holidays_between("BD", d(2026, 1, 1), d(2026, 12, 31))
        .unwrap()
        .is_empty());
}

/// R77, D5: only events dated within the term years are kept (start year to end year).
#[test]
fn r77_only_term_years_are_kept() {
    let fetcher = FakeFetcher::ok(YEARS);

    let one_year = term_years(d(2026, 1, 5), d(2026, 12, 20));
    assert_eq!(one_year, 2026..=2026);
    assert_eq!(
        fetch_events(&fetcher, "BD", one_year),
        Ok(vec![
            ph(d(2026, 3, 26), "Independence Day"),
            ph(d(2026, 12, 16), "Victory Day"),
        ])
    );

    let two_years = term_years(d(2026, 9, 1), d(2027, 1, 31));
    assert_eq!(two_years, 2026..=2027);
    assert_eq!(
        fetch_events(&fetcher, "BD", two_years),
        Ok(vec![
            ph(d(2026, 3, 26), "Independence Day"),
            ph(d(2026, 12, 16), "Victory Day"),
            ph(d(2027, 3, 26), "Independence Day"),
            ph(d(2027, 12, 16), "Victory Day"),
        ])
    );
}

/// R77, D26: a code outside the table is an error and the fetcher is not called; a 200 answer
/// that is not a calendar is an error; the cached rows stay either way.
#[test]
fn r77_unknown_code_and_non_calendar_body_are_errors() {
    let mut planner = planner();
    let cached = vec![ph(d(2026, 3, 26), "Independence Day")];
    planner.set_public_holidays("BD", &cached).unwrap();

    let unknown = FakeFetcher::ok(SAMPLE);
    assert!(fetch_events(&unknown, "ZZ", 2026..=2026).is_err());
    assert_eq!(unknown.calls(), 0);

    let html = FakeFetcher::ok("<html><body>Sign in to continue</body></html>");
    let result = fetch_events(&html, "BD", 2026..=2026);
    assert!(result.is_err());
    assert_eq!(html.calls(), 1);
    assert_eq!(
        apply_fetch(&mut planner, "BD", result),
        FetchOutcome::Failed
    );

    assert_eq!(
        planner
            .public_holidays_between("BD", d(2026, 1, 1), d(2026, 12, 31))
            .unwrap(),
        cached
    );
}
