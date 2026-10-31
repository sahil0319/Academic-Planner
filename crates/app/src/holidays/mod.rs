//! Government public holidays of the user's country (M16, R76, R77, R81).
//!
//! A pure iCalendar parser (`ics`), a pure locale-to-country mapping (`locale`), the built-in
//! calendar table (`calendars`), a `HolidayFetcher` trait with its HTTP implementation (`fetch`),
//! and a service layer that takes the fetcher (`service`). Only the app crate uses the network
//! (R27); tests inject a fake fetcher and never open a socket.

#[cfg(test)]
mod tests;
