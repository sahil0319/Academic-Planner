//! The public holiday fetch service (M16, R77; D26). Pure apart from the injected fetcher: the
//! worker-thread half (`fetch_events`) never touches the database, and the UI-thread half
//! (`apply_fetch`) writes only on success.

use std::ops::RangeInclusive;

use chrono::{Datelike, NaiveDate};
use planner_core::{Planner, PublicHoliday};

use super::calendars::calendar_for;
use super::fetch::{feed_url, HolidayFetcher};
use super::ics::parse_public_holidays;

/// R77, D5: the years of the active term, start year to end year inclusive. Only events dated in
/// these years are kept.
pub fn term_years(start: NaiveDate, end: NaiveDate) -> RangeInclusive<i32> {
    start.year()..=end.year()
}

/// R77, D26: one fetch of the country's feed; events dated in `years`, sorted by date then name.
/// `Err` when the code is not in the table (the fetcher is not called), the fetch fails, or the
/// body is not a calendar (no `BEGIN:VCALENDAR`), so an error page answered with 200 never
/// replaces the cache. A calendar with no event in `years` is `Ok` with an empty list.
pub fn fetch_events(
    fetcher: &dyn HolidayFetcher,
    code: &str,
    years: RangeInclusive<i32>,
) -> Result<Vec<PublicHoliday>, String> {
    let calendar = calendar_for(code).ok_or_else(|| format!("no calendar for {code}"))?;
    let body = fetcher.fetch(&feed_url(&calendar.id))?;
    if !body.contains("BEGIN:VCALENDAR") {
        return Err("the answer is not a calendar".to_string());
    }
    Ok(parse_public_holidays(&body)
        .into_iter()
        .filter(|day| years.contains(&day.date.year()))
        .collect())
}

/// R77: what `apply_fetch` did to the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchOutcome {
    Updated,
    Failed,
}

/// R77, D26: a success replaces the country's rows (`set_public_holidays`); a failure, or a
/// write error, leaves the cached rows as they are and gives `Failed`.
pub fn apply_fetch(
    planner: &mut Planner,
    code: &str,
    result: Result<Vec<PublicHoliday>, String>,
) -> FetchOutcome {
    match result {
        Ok(days) => match planner.set_public_holidays(code, &days) {
            Ok(()) => FetchOutcome::Updated,
            Err(_) => FetchOutcome::Failed,
        },
        Err(_) => FetchOutcome::Failed,
    }
}
