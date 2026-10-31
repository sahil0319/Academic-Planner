//! Locale to public holiday country (M16, R76; D8, D27, D38). The mapping is pure; only
//! `os_locale` touches the system. There is never an IP lookup.

use super::calendars::calendar_for;

/// R76: a country in the built-in table, or Off.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Country {
    Off,
    Code(String),
}

/// The setting text that means Off (D8).
const OFF: &str = "off";

/// R76, T76, D38: the region of a locale tag as a table country, else Off. `en-BD` is BD,
/// `bn_BD.UTF-8` is BD, `en-Latn-BD` is BD, `fr` (no region) is Off, `xx-ZZ` (not in the table)
/// is Off. A `.encoding` or `@modifier` suffix is dropped, the language subtag is skipped, a
/// 4-letter script subtag is ignored, and the first 2-letter alphabetic subtag is the region
/// (upper-cased).
pub fn country_from_locale(tag: &str) -> Country {
    let base = tag.split(['.', '@']).next().unwrap_or("");
    let region = base
        .split(['-', '_'])
        .skip(1)
        .find(|part| part.len() == 2 && part.chars().all(|c| c.is_ascii_alphabetic()));
    match region {
        Some(region) => {
            let code = region.to_ascii_uppercase();
            if calendar_for(&code).is_some() {
                Country::Code(code)
            } else {
                Country::Off
            }
        }
        None => Country::Off,
    }
}

/// R76, D8, D27: the country to use. The stored setting wins: `off` is Off; an ISO code in the
/// table (exact, upper case) is that country; any other stored text is Off. With no stored
/// setting the OS region decides; no OS locale is Off.
pub fn resolve_country(setting: Option<&str>, os_locale: Option<&str>) -> Country {
    match setting {
        Some(OFF) => Country::Off,
        Some(code) if calendar_for(code).is_some() => Country::Code(code.to_string()),
        Some(_) => Country::Off,
        None => os_locale.map_or(Country::Off, country_from_locale),
    }
}

/// R76, D27: the text stored in `public_holiday_country`: `off` or the ISO code.
pub fn setting_value(country: &Country) -> String {
    match country {
        Country::Off => OFF.to_string(),
        Country::Code(code) => code.clone(),
    }
}

/// R76: the OS locale tag, such as `en-BD`. A thin wrapper over `sys_locale::get_locale()`; not
/// unit tested.
pub fn os_locale() -> Option<String> {
    sys_locale::get_locale()
}
