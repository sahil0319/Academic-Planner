//! R42: parses and validates a whole import document, collecting every error with its JSON
//! path and every unknown key as a warning. No database access: the facts the checks need
//! (active term, rule sets, stored courses) come in through [`Context`].

use std::collections::HashMap;

use chrono::{NaiveDate, NaiveTime, Weekday};
use serde_json::{Map, Value};

use super::types::{ImportIssue, ImportSlot, ImportTerm, RuleSetRef};
use crate::model::{Course, Term};

const FORMAT: &str = "academic-planner-courses";

const ROOT_KEYS: &[&str] = &["format", "version", "term", "courses"];
const TERM_KEYS: &[&str] = &["name", "start", "end"];
const COURSE_KEYS: &[&str] = &[
    "code",
    "name",
    "credits",
    "rule_set",
    "instructor",
    "room",
    "min_attendance_pct",
    "slots",
];
const SLOT_KEYS: &[&str] = &["day", "start", "end", "room"];

/// R42: database facts the validation needs, read by the caller before parsing.
pub(super) struct Context<'a> {
    /// R47: the active term, if any.
    pub active_term: Option<&'a Term>,
    /// R42: every rule set, for the exact-name lookup.
    pub rule_sets: &'a [RuleSetRef],
    /// R43: courses of the active term, archived included.
    pub courses: &'a [Course],
}

/// R44: a course key that is absent (keep), `null` (clear), or has a value.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Field<T> {
    Absent,
    Null,
    Value(T),
}

/// R42: one course of the file after validation.
#[derive(Debug, Clone)]
pub(super) struct ParsedCourse {
    /// R43: the code as written, trimmed.
    pub code: String,
    /// R43: index into `Context::courses` of the matched stored course.
    pub matched: Option<usize>,
    pub name: Field<String>,
    pub credits: Field<f64>,
    pub rule_set: Field<RuleSetRef>,
    pub instructor: Field<String>,
    pub room: Field<String>,
    pub min_attendance_pct: Field<i64>,
    /// R45: `Absent` leaves the stored slots untouched.
    pub slots: Field<Vec<ImportSlot>>,
}

/// R42: the outcome of parsing. When `errors` is non-empty the other parts must not be used.
#[derive(Debug, Default)]
pub(super) struct Parsed {
    /// R47: the file's valid `term`, if any.
    pub term: Option<ImportTerm>,
    pub courses: Vec<ParsedCourse>,
    pub errors: Vec<ImportIssue>,
    pub warnings: Vec<ImportIssue>,
}

/// R43: the key course codes match on: case and whitespace are ignored.
pub(super) fn normalize_code(code: &str) -> String {
    code.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

/// R42: walks the whole document. Issues come in walk order: `format`, `version`, `term`,
/// then `courses` by index; within an object known keys first, then unknown keys.
pub(super) fn parse(text: &str, ctx: &Context<'_>) -> Parsed {
    let mut walker = Walker::default();
    let value: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            walker.error(
                "document",
                format!(
                    "not valid JSON (line {}, column {}): {}",
                    e.line(),
                    e.column(),
                    e
                ),
            );
            return walker.finish(None, Vec::new());
        }
    };
    let Some(root) = value.as_object() else {
        walker.error("document", "must be a JSON object");
        return walker.finish(None, Vec::new());
    };
    walker.root(root, ctx)
}

#[derive(Default)]
struct Walker {
    errors: Vec<ImportIssue>,
    warnings: Vec<ImportIssue>,
}

fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

fn field_of(obj: &Map<String, Value>, key: &str) -> Field<Value> {
    match obj.get(key) {
        None => Field::Absent,
        Some(Value::Null) => Field::Null,
        Some(v) => Field::Value(v.clone()),
    }
}

impl Walker {
    fn error(&mut self, path: &str, message: impl Into<String>) {
        self.errors.push(ImportIssue {
            path: path.to_string(),
            message: message.into(),
        });
    }

    fn warn(&mut self, path: &str, message: impl Into<String>) {
        self.warnings.push(ImportIssue {
            path: path.to_string(),
            message: message.into(),
        });
    }

    fn finish(self, term: Option<ImportTerm>, courses: Vec<ParsedCourse>) -> Parsed {
        Parsed {
            term,
            courses,
            errors: self.errors,
            warnings: self.warnings,
        }
    }

    /// R42: every key outside `known` is ignored with a warning at its path.
    fn warn_unknown(&mut self, obj: &Map<String, Value>, known: &[&str], path: &str) {
        for key in obj.keys().filter(|k| !known.contains(&k.as_str())) {
            self.warn(&join(path, key), "unknown key, ignored");
        }
    }

    fn root(mut self, root: &Map<String, Value>, ctx: &Context<'_>) -> Parsed {
        match root.get("format") {
            None => self.error("format", "required"),
            Some(Value::String(s)) if s == FORMAT => {}
            Some(_) => self.error("format", format!("must be \"{FORMAT}\"")),
        }
        match root.get("version") {
            None => self.error("version", "required"),
            Some(v) if v.as_i64() == Some(1) => {}
            Some(_) => self.error("version", "unsupported version; expected 1"),
        }

        // R47: a missing or null term is fine when a term is active.
        let term = match root.get("term") {
            None | Some(Value::Null) => {
                if ctx.active_term.is_none() {
                    self.error("term", "required when there is no active term");
                }
                None
            }
            Some(v) => self.term(v, ctx),
        };

        let courses = match root.get("courses") {
            None => {
                self.error("courses", "required");
                Vec::new()
            }
            Some(Value::Array(items)) => self.courses(items, ctx),
            Some(_) => {
                self.error("courses", "must be a list");
                Vec::new()
            }
        };
        self.warn_unknown(root, ROOT_KEYS, "");
        self.finish(term, courses)
    }

    /// R42, R47: validates `term` (also when an active term makes it unnecessary) and warns
    /// when it differs from the active term.
    fn term(&mut self, value: &Value, ctx: &Context<'_>) -> Option<ImportTerm> {
        let Some(obj) = value.as_object() else {
            self.error("term", "must be an object");
            return None;
        };
        let name = self.required_text(obj, "name", "term");
        let start = self.date(obj, "start", "term");
        let end = self.date(obj, "end", "term");
        self.warn_unknown(obj, TERM_KEYS, "term");
        if let (Some(s), Some(e)) = (start, end) {
            if e < s {
                self.error("term.end", "must be on or after start");
                return None;
            }
        }
        let (name, start_date, end_date) = (name?, start?, end?);
        if let Some(active) = ctx.active_term {
            if active.name != name || active.start_date != start_date || active.end_date != end_date
            {
                self.warn(
                    "term",
                    format!(
                        "ignored; the active term is \"{}\" {} to {}",
                        active.name, active.start_date, active.end_date
                    ),
                );
            }
        }
        Some(ImportTerm {
            name,
            start_date,
            end_date,
        })
    }

    /// A required, non-empty text value; the result is trimmed.
    fn required_text(&mut self, obj: &Map<String, Value>, key: &str, path: &str) -> Option<String> {
        let at = join(path, key);
        match obj.get(key) {
            None | Some(Value::Null) => {
                self.error(&at, "required");
                None
            }
            Some(Value::String(s)) if s.trim().is_empty() => {
                self.error(&at, "must not be empty");
                None
            }
            Some(Value::String(s)) => Some(s.trim().to_string()),
            Some(_) => {
                self.error(&at, "must be text");
                None
            }
        }
    }

    fn date(&mut self, obj: &Map<String, Value>, key: &str, path: &str) -> Option<NaiveDate> {
        let at = join(path, key);
        match obj.get(key) {
            None | Some(Value::Null) => {
                self.error(&at, "required");
                None
            }
            Some(v) => {
                let date = v.as_str().and_then(parse_iso_date);
                if date.is_none() {
                    self.error(&at, "must be a date YYYY-MM-DD");
                }
                date
            }
        }
    }

    fn courses(&mut self, items: &[Value], ctx: &Context<'_>) -> Vec<ParsedCourse> {
        // R43: stored courses by normalized code.
        let mut stored: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, c) in ctx.courses.iter().enumerate() {
            stored.entry(normalize_code(&c.code)).or_default().push(i);
        }
        // R42: first position of each normalized code in the file.
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut out = Vec::new();
        for (i, item) in items.iter().enumerate() {
            let path = format!("courses[{i}]");
            let Some(obj) = item.as_object() else {
                self.error(&path, "must be an object");
                continue;
            };
            if let Some(course) = self.course(obj, &path, i, ctx, &stored, &mut seen) {
                out.push(course);
            }
        }
        out
    }

    fn course(
        &mut self,
        obj: &Map<String, Value>,
        path: &str,
        index: usize,
        ctx: &Context<'_>,
        stored: &HashMap<String, Vec<usize>>,
        seen: &mut HashMap<String, usize>,
    ) -> Option<ParsedCourse> {
        let code = self.required_text(obj, "code", path);

        // R42, R43: duplicates in the file and matches against stored courses.
        // `is_new` stays `None` while the code is unusable, so no name error follows it.
        let mut matched = None;
        let mut is_new = None;
        if let Some(code) = &code {
            let norm = normalize_code(code);
            let code_path = join(path, "code");
            if let Some(first) = seen.get(&norm) {
                self.error(&code_path, format!("duplicate of courses[{first}].code"));
            } else {
                seen.insert(norm.clone(), index);
                match stored.get(&norm).map(Vec::as_slice) {
                    None | Some([]) => is_new = Some(true),
                    Some([one]) => {
                        matched = Some(*one);
                        is_new = Some(false);
                        let course = &ctx.courses[*one];
                        if course.archived {
                            self.warn(
                                path,
                                format!(
                                    "matches archived course \"{}\"; it stays archived",
                                    course.code
                                ),
                            );
                        }
                    }
                    Some(many) => {
                        let codes: Vec<&str> =
                            many.iter().map(|&i| ctx.courses[i].code.as_str()).collect();
                        self.error(
                            &code_path,
                            format!("matches more than one course: {}", codes.join(", ")),
                        );
                    }
                }
            }
        }

        let name = self.name(obj, path, is_new == Some(true));
        let credits = self.credits(obj, path);
        let rule_set = self.rule_set(obj, path, ctx);
        let instructor = self.optional_text(obj, "instructor", path);
        let room = self.optional_text(obj, "room", path);
        let min_attendance_pct = self.min_attendance_pct(obj, path);
        let slots = self.slots(obj, path);
        self.warn_unknown(obj, COURSE_KEYS, path);

        Some(ParsedCourse {
            code: code?,
            matched,
            name,
            credits,
            rule_set,
            instructor,
            room,
            min_attendance_pct,
            slots,
        })
    }

    /// R42, R44: `name` is required for a new course and never null or blank.
    fn name(&mut self, obj: &Map<String, Value>, path: &str, is_new: bool) -> Field<String> {
        let at = join(path, "name");
        match obj.get("name") {
            None => {
                if is_new {
                    self.error(&at, "required for a new course");
                }
                Field::Absent
            }
            Some(Value::String(s)) if !s.trim().is_empty() => Field::Value(s.trim().to_string()),
            Some(Value::String(_) | Value::Null) => {
                self.error(&at, "must not be empty");
                Field::Absent
            }
            Some(_) => {
                self.error(&at, "must be text");
                Field::Absent
            }
        }
    }

    /// R42, R44: `credits` is a number greater than 0 (fractions allowed), or `null`.
    fn credits(&mut self, obj: &Map<String, Value>, path: &str) -> Field<f64> {
        match field_of(obj, "credits") {
            Field::Absent => Field::Absent,
            Field::Null => Field::Null,
            Field::Value(v) => match v.as_f64() {
                Some(n) if n.is_finite() && n > 0.0 => Field::Value(n),
                _ => {
                    self.error(&join(path, "credits"), "must be a number greater than 0");
                    Field::Absent
                }
            },
        }
    }

    /// R42: `rule_set` names an existing rule set exactly, or is `null`.
    fn rule_set(
        &mut self,
        obj: &Map<String, Value>,
        path: &str,
        ctx: &Context<'_>,
    ) -> Field<RuleSetRef> {
        let at = join(path, "rule_set");
        match field_of(obj, "rule_set") {
            Field::Absent => Field::Absent,
            Field::Null => Field::Null,
            Field::Value(Value::String(name)) => {
                match ctx.rule_sets.iter().find(|r| r.name == name) {
                    Some(found) => Field::Value(found.clone()),
                    None => {
                        let names: Vec<&str> =
                            ctx.rule_sets.iter().map(|r| r.name.as_str()).collect();
                        self.error(
                            &at,
                            format!(
                                "no rule set named \"{name}\" (available: {})",
                                names.join(", ")
                            ),
                        );
                        Field::Absent
                    }
                }
            }
            Field::Value(_) => {
                self.error(&at, "must be text");
                Field::Absent
            }
        }
    }

    /// R44: `instructor` and `room`: text, or `null` to clear. Blank text clears too.
    fn optional_text(&mut self, obj: &Map<String, Value>, key: &str, path: &str) -> Field<String> {
        match field_of(obj, key) {
            Field::Absent => Field::Absent,
            Field::Null => Field::Null,
            Field::Value(Value::String(s)) if s.trim().is_empty() => Field::Null,
            Field::Value(Value::String(s)) => Field::Value(s.trim().to_string()),
            Field::Value(_) => {
                self.error(&join(path, key), "must be text");
                Field::Absent
            }
        }
    }

    /// R42: `min_attendance_pct` is a whole number from 0 to 100, or `null`.
    fn min_attendance_pct(&mut self, obj: &Map<String, Value>, path: &str) -> Field<i64> {
        match field_of(obj, "min_attendance_pct") {
            Field::Absent => Field::Absent,
            Field::Null => Field::Null,
            Field::Value(v) => match v.as_i64() {
                Some(n) if (0..=100).contains(&n) => Field::Value(n),
                _ => {
                    self.error(
                        &join(path, "min_attendance_pct"),
                        "must be a whole number from 0 to 100",
                    );
                    Field::Absent
                }
            },
        }
    }

    /// R42, R45: `slots` is a list (empty allowed); `null` is an error.
    fn slots(&mut self, obj: &Map<String, Value>, path: &str) -> Field<Vec<ImportSlot>> {
        let at = join(path, "slots");
        match obj.get("slots") {
            None => Field::Absent,
            Some(Value::Array(items)) => {
                let mut slots = Vec::new();
                for (j, item) in items.iter().enumerate() {
                    let slot_path = format!("{at}[{j}]");
                    match item.as_object() {
                        Some(o) => slots.extend(self.slot(o, &slot_path)),
                        None => self.error(&slot_path, "must be an object"),
                    }
                }
                Field::Value(slots)
            }
            Some(_) => {
                self.error(&at, "must be a list; use [] to end all slots");
                Field::Absent
            }
        }
    }

    /// R42, R2: day, 24-hour start and end with end after start, optional room.
    fn slot(&mut self, obj: &Map<String, Value>, path: &str) -> Option<ImportSlot> {
        let day = match obj.get("day") {
            None | Some(Value::Null) => {
                self.error(&join(path, "day"), "required");
                None
            }
            Some(v) => {
                let day = parse_day(v);
                if day.is_none() {
                    self.error(
                        &join(path, "day"),
                        "must be a day name (Mon or Monday) or 1-7",
                    );
                }
                day
            }
        };
        let start = self.time(obj, "start", path);
        let end = self.time(obj, "end", path);
        let room = match self.optional_text(obj, "room", path) {
            Field::Value(r) => Some(r),
            Field::Absent | Field::Null => None,
        };
        self.warn_unknown(obj, SLOT_KEYS, path);
        if let (Some(s), Some(e)) = (start, end) {
            if e <= s {
                self.error(&join(path, "end"), "must be after start");
                return None;
            }
        }
        Some(ImportSlot {
            weekday: day?,
            start: start?,
            end: end?,
            room,
        })
    }

    fn time(&mut self, obj: &Map<String, Value>, key: &str, path: &str) -> Option<NaiveTime> {
        let at = join(path, key);
        match obj.get(key) {
            None | Some(Value::Null) => {
                self.error(&at, "required");
                None
            }
            Some(v) => {
                let time = v.as_str().and_then(parse_clock);
                if time.is_none() {
                    self.error(&at, "must be a 24-hour time HH:MM");
                }
                time
            }
        }
    }
}

/// R42: `YYYY-MM-DD` with zero padding, nothing else.
fn parse_iso_date(s: &str) -> Option<NaiveDate> {
    let b = s.as_bytes();
    let shaped = b.len() == 10
        && b.iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        });
    if !shaped {
        return None;
    }
    NaiveDate::from_ymd_opt(
        s[0..4].parse().ok()?,
        s[5..7].parse().ok()?,
        s[8..10].parse().ok()?,
    )
}

/// R42: 24-hour `H:MM` or `HH:MM`, 00:00 to 23:59; no seconds, no AM/PM.
fn parse_clock(s: &str) -> Option<NaiveTime> {
    let (h, m) = s.split_once(':')?;
    let digits = |t: &str| t.bytes().all(|c| c.is_ascii_digit());
    if !(1..=2).contains(&h.len()) || m.len() != 2 || !digits(h) || !digits(m) {
        return None;
    }
    NaiveTime::from_hms_opt(h.parse().ok()?, m.parse().ok()?, 0)
}

/// R42: `Mon`..`Sun` or `Monday`..`Sunday` in any case (trimmed), or 1-7 as text or integer
/// (1 = Monday).
fn parse_day(value: &Value) -> Option<Weekday> {
    let number = |n: i64| match n {
        1 => Some(Weekday::Mon),
        2 => Some(Weekday::Tue),
        3 => Some(Weekday::Wed),
        4 => Some(Weekday::Thu),
        5 => Some(Weekday::Fri),
        6 => Some(Weekday::Sat),
        7 => Some(Weekday::Sun),
        _ => None,
    };
    match value {
        Value::Number(n) => n.as_i64().and_then(number),
        Value::String(s) => match s.trim().to_lowercase().as_str() {
            "mon" | "monday" | "1" => Some(Weekday::Mon),
            "tue" | "tuesday" | "2" => Some(Weekday::Tue),
            "wed" | "wednesday" | "3" => Some(Weekday::Wed),
            "thu" | "thursday" | "4" => Some(Weekday::Thu),
            "fri" | "friday" | "5" => Some(Weekday::Fri),
            "sat" | "saturday" | "6" => Some(Weekday::Sat),
            "sun" | "sunday" | "7" => Some(Weekday::Sun),
            _ => None,
        },
        _ => None,
    }
}
