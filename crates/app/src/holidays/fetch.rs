//! The public holiday feed request (M16, R77). This is the only file in the app that builds an
//! HTTP client (R27); tests use a fake `HolidayFetcher` and never open a socket.

use std::time::Duration;

/// R77, D5: how long one feed request may take.
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

/// R77: `https://calendar.google.com/calendar/ical/{url-encoded id}/public/basic.ics`. Every
/// character except the unreserved ones (`A-Z a-z 0-9 - . _ ~`) is percent-encoded, so `#` becomes
/// `%23` and `@` becomes `%40`.
pub fn feed_url(calendar_id: &str) -> String {
    let mut encoded = String::with_capacity(calendar_id.len());
    for byte in calendar_id.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    format!("https://calendar.google.com/calendar/ical/{encoded}/public/basic.ics")
}

/// R77: gets the text behind a URL. A trait so tests inject a fake (D4).
pub trait HolidayFetcher {
    /// Returns the body of one GET, or a short error text.
    fn fetch(&self, url: &str) -> Result<String, String>;
}

/// R77: the real fetcher: one blocking GET with `FETCH_TIMEOUT`; a non-2xx status is an error.
pub struct HttpFetcher;

impl HolidayFetcher for HttpFetcher {
    fn fetch(&self, url: &str) -> Result<String, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(FETCH_TIMEOUT)
            .build()
            .map_err(|e| e.to_string())?;
        let response = client.get(url).send().map_err(|e| e.to_string())?;
        let status = response.status();
        if !status.is_success() {
            return Err(format!("HTTP {status}"));
        }
        response.text().map_err(|e| e.to_string())
    }
}
