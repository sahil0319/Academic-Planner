//! Government public holidays of the user's country (M16, R76, R77, R81).
//!
//! A pure iCalendar parser (`ics`), a pure locale-to-country mapping (`locale`), the built-in
//! calendar table (`calendars`), a `HolidayFetcher` trait with its HTTP implementation (`fetch`),
//! and a service layer that takes the fetcher (`service`). Only the app crate uses the network
//! (R27); tests inject a fake fetcher and never open a socket.

// TODO(M16, T010-T011): the page glue and the dashboard use these items; remove this allow when
// they are wired, so the module is checked for dead code again.
#![allow(dead_code)]

pub mod calendars;
pub mod fetch;
pub mod ics;
pub mod locale;
pub mod service;

#[cfg(test)]
mod tests;
