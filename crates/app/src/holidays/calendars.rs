//! The built-in country table (M16, R76): ISO code, display name, Google calendar id.
//!
//! The data lives in `data/holiday-calendars.json` (D21) and is parsed once with
//! `serde_json::Value`.

use std::sync::OnceLock;

use serde_json::Value;

/// R76: one built-in country: ISO code, display name, Google calendar id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Calendar {
    pub code: String,
    pub name: String,
    pub id: String,
}

/// R76: the table, parsed once from `include_str!("../../data/holiday-calendars.json")`; empty if
/// the text is invalid (a unit test guarantees it is valid).
pub fn calendars() -> &'static [Calendar] {
    static TABLE: OnceLock<Vec<Calendar>> = OnceLock::new();
    TABLE.get_or_init(|| parse(include_str!("../../data/holiday-calendars.json")))
}

/// R76: the table entry for an ISO code (compared exactly, upper case), if any.
pub fn calendar_for(code: &str) -> Option<&'static Calendar> {
    calendars().iter().find(|c| c.code == code)
}

/// R76, D21: the table sorted by display name, the order of the Settings choice after "Off".
pub fn calendars_by_name() -> Vec<&'static Calendar> {
    let mut sorted: Vec<&Calendar> = calendars().iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    sorted
}

/// Reads the JSON array; an entry without a text `code`, `name`, or `id` is skipped.
fn parse(text: &str) -> Vec<Calendar> {
    let Ok(Value::Array(items)) = serde_json::from_str::<Value>(text) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let field = |key: &str| item.get(key)?.as_str().map(str::to_string);
            Some(Calendar {
                code: field("code")?,
                name: field("name")?,
                id: field("id")?,
            })
        })
        .collect()
}
