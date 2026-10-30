//! Rule sets and the grade scale (M9, §13.3).

use std::collections::HashSet;

use rusqlite::{params, Connection, OptionalExtension};

use crate::db::Planner;
use crate::error::{Error, Result};
use crate::model::{
    AttendanceBand, ComponentKind, GradeRow, RuleComponent, RuleComponentInput, RuleSet,
    RuleSetInput,
};

/// R92: the default attendance bands, in descending `min_pct` order: at or above 95% earns
/// 100% of the attendance marks, 90 earns 80, 85 earns 60, 75 earns 20, 70 earns 10, and 0
/// earns 0. Pure (no I/O); the table satisfies R34 (at least one band, one with `min_pct` 0).
pub fn default_attendance_bands() -> Vec<AttendanceBand> {
    [
        (95.0, 100.0),
        (90.0, 80.0),
        (85.0, 60.0),
        (75.0, 20.0),
        (70.0, 10.0),
        (0.0, 0.0),
    ]
    .into_iter()
    .map(|(min_pct, credit_pct)| AttendanceBand {
        min_pct,
        credit_pct,
    })
    .collect()
}

/// R34: weight sums within this distance of 100 are accepted.
const WEIGHT_TOLERANCE: f64 = 0.01;

impl ComponentKind {
    /// R34: the `rule_component.kind` text.
    fn as_db(self) -> &'static str {
        match self {
            ComponentKind::Attendance => "attendance",
            ComponentKind::Items => "items",
        }
    }

    fn from_db(s: &str) -> Option<Self> {
        match s {
            "attendance" => Some(ComponentKind::Attendance),
            "items" => Some(ComponentKind::Items),
            _ => None,
        }
    }
}

impl Planner {
    /// R34: every rule set with its components (by position) and attendance bands.
    pub fn list_rule_sets(&self) -> Result<Vec<RuleSet>> {
        load_rule_sets(&self.conn)
    }

    /// R34: create or update a rule set after validating it; components update in place by id.
    pub fn save_rule_set(&mut self, input: RuleSetInput) -> Result<RuleSet> {
        let tx = self.conn.transaction()?;
        let existing = match input.id {
            Some(id) => Some(load_rule_set(&tx, id)?.ok_or(Error::NotFound {
                entity: "rule set",
                id,
            })?),
            None => None,
        };
        validate_rule_set(&tx, &input, existing.as_ref())?;

        let name = input.name.trim();
        let set_id = match &existing {
            Some(set) => {
                tx.execute(
                    "UPDATE rule_set SET name = ?1 WHERE id = ?2",
                    params![name, set.id],
                )?;
                set.id
            }
            None => {
                tx.execute("INSERT INTO rule_set (name) VALUES (?1)", [name])?;
                tx.last_insert_rowid()
            }
        };

        if let Some(set) = &existing {
            let kept: HashSet<i64> = input.components.iter().filter_map(|c| c.id).collect();
            // R34: removed components lose their unscored links, then go (research D3).
            for old in set.components.iter().filter(|c| !kept.contains(&c.id)) {
                tx.execute(
                    "UPDATE assessment SET component_id = NULL WHERE component_id = ?1",
                    [old.id],
                )?;
                tx.execute("DELETE FROM rule_component WHERE id = ?1", [old.id])?;
            }
            // Park the names of kept components so renames and swaps cannot collide on
            // the unique (rule_set_id, name) pair while rows update one by one.
            for id in &kept {
                tx.execute(
                    "UPDATE rule_component SET name = '#parked' || id WHERE id = ?1",
                    [id],
                )?;
            }
        }

        for (position, c) in input.components.iter().enumerate() {
            let (item_count, best_count, item_max) = match c.kind {
                ComponentKind::Items => (c.item_count, c.best_count, c.item_max),
                ComponentKind::Attendance => (None, None, None),
            };
            let component_id = match c.id {
                Some(id) => {
                    tx.execute(
                        "UPDATE rule_component SET position = ?1, name = ?2, kind = ?3,
                             weight_pct = ?4, item_count = ?5, best_count = ?6, item_max = ?7,
                             is_final = ?8
                         WHERE id = ?9",
                        params![
                            position as i64,
                            c.name.trim(),
                            c.kind.as_db(),
                            c.weight_pct,
                            item_count,
                            best_count,
                            item_max,
                            c.is_final,
                            id
                        ],
                    )?;
                    id
                }
                None => {
                    tx.execute(
                        "INSERT INTO rule_component (rule_set_id, position, name, kind,
                             weight_pct, item_count, best_count, item_max, is_final)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                        params![
                            set_id,
                            position as i64,
                            c.name.trim(),
                            c.kind.as_db(),
                            c.weight_pct,
                            item_count,
                            best_count,
                            item_max,
                            c.is_final
                        ],
                    )?;
                    tx.last_insert_rowid()
                }
            };
            tx.execute(
                "DELETE FROM attendance_band WHERE component_id = ?1",
                [component_id],
            )?;
            if c.kind == ComponentKind::Attendance {
                for b in &c.bands {
                    tx.execute(
                        "INSERT INTO attendance_band (component_id, min_pct, credit_pct)
                         VALUES (?1, ?2, ?3)",
                        params![component_id, b.min_pct, b.credit_pct],
                    )?;
                }
            }
        }

        let saved = load_rule_set(&tx, set_id)?.ok_or(Error::NotFound {
            entity: "rule set",
            id: set_id,
        })?;
        tx.commit()?;
        Ok(saved)
    }

    /// R34: delete an unused rule set; a set that a course uses is an error.
    pub fn delete_rule_set(&mut self, id: i64) -> Result<()> {
        let tx = self.conn.transaction()?;
        if load_rule_set(&tx, id)?.is_none() {
            return Err(Error::NotFound {
                entity: "rule set",
                id,
            });
        }
        let used: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM course WHERE rule_set_id = ?1)",
            [id],
            |r| r.get(0),
        )?;
        if used {
            return Err(invalid(
                "rule set is used by a course; change that course first",
            ));
        }
        tx.execute(
            "UPDATE assessment SET component_id = NULL
             WHERE component_id IN (SELECT id FROM rule_component WHERE rule_set_id = ?1)",
            [id],
        )?;
        tx.execute("DELETE FROM rule_set WHERE id = ?1", [id])?;
        tx.commit()?;
        Ok(())
    }

    /// R39: the grade scale, by `min_pct` descending.
    pub fn grade_scale(&self) -> Result<Vec<GradeRow>> {
        let mut stmt = self
            .conn
            .prepare("SELECT letter, min_pct, points FROM grade_scale ORDER BY min_pct DESC")?;
        let rows = stmt
            .query_map([], |r| {
                Ok(GradeRow {
                    letter: r.get(0)?,
                    min_pct: r.get(1)?,
                    points: r.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        Ok(rows)
    }

    /// R39: validate and replace the whole grade scale.
    pub fn save_grade_scale(&mut self, rows: &[GradeRow]) -> Result<()> {
        validate_grade_scale(rows)?;
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM grade_scale", [])?;
        for row in rows {
            tx.execute(
                "INSERT INTO grade_scale (letter, min_pct, points) VALUES (?1, ?2, ?3)",
                params![row.letter.trim(), row.min_pct, row.points],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}

/// R34: every rule set, ordered by id.
pub(crate) fn load_rule_sets(conn: &Connection) -> Result<Vec<RuleSet>> {
    let mut stmt = conn.prepare("SELECT id, name FROM rule_set ORDER BY id")?;
    let heads = stmt
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    heads
        .into_iter()
        .map(|(id, name)| {
            Ok(RuleSet {
                id,
                name,
                components: load_components(conn, id)?,
            })
        })
        .collect()
}

/// R34: one rule set, or `None` for an unknown id.
pub(crate) fn load_rule_set(conn: &Connection, id: i64) -> Result<Option<RuleSet>> {
    let name: Option<String> = conn
        .query_row("SELECT name FROM rule_set WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .optional()?;
    name.map(|name| {
        Ok(RuleSet {
            id,
            name,
            components: load_components(conn, id)?,
        })
    })
    .transpose()
}

/// R34: components of a set by position; R35: attendance bands by `min_pct` descending.
fn load_components(conn: &Connection, rule_set_id: i64) -> Result<Vec<RuleComponent>> {
    let mut stmt = conn.prepare(
        "SELECT id, rule_set_id, position, name, kind, weight_pct, item_count, best_count,
                item_max, is_final
         FROM rule_component WHERE rule_set_id = ?1 ORDER BY position, id",
    )?;
    let mut components = stmt
        .query_map([rule_set_id], |r| {
            let kind: String = r.get(4)?;
            let kind = ComponentKind::from_db(&kind).ok_or_else(|| {
                rusqlite::Error::FromSqlConversionFailure(
                    4,
                    rusqlite::types::Type::Text,
                    format!("component kind {kind:?} in column 4").into(),
                )
            })?;
            Ok(RuleComponent {
                id: r.get(0)?,
                rule_set_id: r.get(1)?,
                position: r.get(2)?,
                name: r.get(3)?,
                kind,
                weight_pct: r.get(5)?,
                item_count: r.get(6)?,
                best_count: r.get(7)?,
                item_max: r.get(8)?,
                is_final: r.get(9)?,
                bands: Vec::new(),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut bands = conn.prepare(
        "SELECT min_pct, credit_pct FROM attendance_band WHERE component_id = ?1
         ORDER BY min_pct DESC",
    )?;
    for c in &mut components {
        c.bands = bands
            .query_map([c.id], |r| {
                Ok(AttendanceBand {
                    min_pct: r.get(0)?,
                    credit_pct: r.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
    }
    Ok(components)
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Validation(message.into())
}

/// R34: all checks of `save_rule_set`, run before any write. `existing` is the stored set
/// when updating.
fn validate_rule_set(
    conn: &Connection,
    input: &RuleSetInput,
    existing: Option<&RuleSet>,
) -> Result<()> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(invalid("rule set name is required"));
    }
    let taken: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM rule_set WHERE name = ?1 AND id IS NOT ?2)",
        params![name, existing.map(|s| s.id)],
        |r| r.get(0),
    )?;
    if taken {
        return Err(invalid(format!("a rule set named {name:?} already exists")));
    }

    let total: f64 = input.components.iter().map(|c| c.weight_pct).sum();
    if (total - 100.0).abs().is_nan() || (total - 100.0).abs() > WEIGHT_TOLERANCE + 1e-9 {
        return Err(invalid(format!(
            "component weights add up to {total}, not 100"
        )));
    }

    let mut names = HashSet::new();
    let mut attendance = 0;
    let mut finals = 0;
    for c in &input.components {
        validate_component(c, existing)?;
        if !names.insert(c.name.trim().to_owned()) {
            return Err(invalid(format!(
                "component name {:?} is used twice",
                c.name.trim()
            )));
        }
        if c.kind == ComponentKind::Attendance {
            attendance += 1;
        }
        if c.is_final {
            finals += 1;
            if c.kind != ComponentKind::Items {
                return Err(invalid("the final component must be an items component"));
            }
        }
    }
    if attendance > 1 {
        return Err(invalid("a rule set has at most one attendance component"));
    }
    if finals != 1 {
        return Err(invalid("a rule set needs exactly one final component"));
    }

    if let Some(set) = existing {
        let kept: HashSet<i64> = input.components.iter().filter_map(|c| c.id).collect();
        for old in set.components.iter().filter(|c| !kept.contains(&c.id)) {
            let scored: bool = conn.query_row(
                "SELECT EXISTS (SELECT 1 FROM assessment
                                WHERE component_id = ?1 AND score IS NOT NULL)",
                [old.id],
                |r| r.get(0),
            )?;
            if scored {
                return Err(invalid(format!(
                    "component {:?} has scored assessments and cannot be removed",
                    old.name
                )));
            }
        }
    }
    Ok(())
}

/// R34: checks of one component (the weight, id, item, and band checks).
fn validate_component(c: &RuleComponentInput, existing: Option<&RuleSet>) -> Result<()> {
    let name = c.name.trim();
    if name.is_empty() {
        return Err(invalid("component name is required"));
    }
    if !(c.weight_pct > 0.0 && c.weight_pct <= 100.0) {
        return Err(invalid(format!(
            "weight of {name:?} must be above 0 and at most 100"
        )));
    }
    if let Some(id) = c.id {
        let in_set = existing.is_some_and(|set| set.components.iter().any(|o| o.id == id));
        if !in_set {
            return Err(invalid(format!(
                "component {id} does not belong to this rule set"
            )));
        }
    }
    match c.kind {
        ComponentKind::Items => {
            let (Some(item_count), Some(best_count)) = (c.item_count, c.best_count) else {
                return Err(invalid(format!(
                    "{name:?} needs an item count and a best count"
                )));
            };
            if best_count < 1 || best_count > item_count {
                return Err(invalid(format!(
                    "best count of {name:?} must be at least 1 and at most its item count"
                )));
            }
            if c.item_max.is_some_and(|max| max.is_nan() || max <= 0.0) {
                return Err(invalid(format!("item max of {name:?} must be above 0")));
            }
        }
        ComponentKind::Attendance => validate_bands(name, &c.bands)?,
    }
    Ok(())
}

/// R34/R35: an attendance component needs bands, one starting at 0, values in 0 to 100, and
/// distinct `min_pct`.
fn validate_bands(name: &str, bands: &[AttendanceBand]) -> Result<()> {
    if bands.is_empty() {
        return Err(invalid(format!("{name:?} needs at least one band")));
    }
    let in_range = |x: f64| (0.0..=100.0).contains(&x);
    for (i, b) in bands.iter().enumerate() {
        if !in_range(b.min_pct) || !in_range(b.credit_pct) {
            return Err(invalid(format!("band values of {name:?} must be 0 to 100")));
        }
        if bands[..i].iter().any(|o| o.min_pct == b.min_pct) {
            return Err(invalid(format!(
                "{name:?} has two bands starting at {}",
                b.min_pct
            )));
        }
    }
    if !bands.iter().any(|b| b.min_pct == 0.0) {
        return Err(invalid(format!("{name:?} needs a band starting at 0")));
    }
    Ok(())
}

/// R39: letters non-empty and unique; `min_pct` unique and 0 to 100 with a 0 row; points 0 or
/// more and not rising as `min_pct` falls.
fn validate_grade_scale(rows: &[GradeRow]) -> Result<()> {
    let mut letters = HashSet::new();
    for (i, row) in rows.iter().enumerate() {
        let letter = row.letter.trim();
        if letter.is_empty() {
            return Err(invalid("grade letter is required"));
        }
        if !letters.insert(letter.to_owned()) {
            return Err(invalid(format!("grade letter {letter:?} is used twice")));
        }
        if !(0.0..=100.0).contains(&row.min_pct) {
            return Err(invalid(format!("minimum of {letter:?} must be 0 to 100")));
        }
        if rows[..i].iter().any(|o| o.min_pct == row.min_pct) {
            return Err(invalid(format!("two grades start at {}", row.min_pct)));
        }
        if row.points.is_nan() || row.points < 0.0 {
            return Err(invalid(format!("points of {letter:?} must be 0 or more")));
        }
    }
    if !rows.iter().any(|r| r.min_pct == 0.0) {
        return Err(invalid("the grade scale needs a row starting at 0"));
    }
    let mut sorted: Vec<&GradeRow> = rows.iter().collect();
    sorted.sort_by(|a, b| b.min_pct.total_cmp(&a.min_pct));
    if sorted.windows(2).any(|w| w[1].points > w[0].points) {
        return Err(invalid("points must not rise as the minimum falls"));
    }
    Ok(())
}
