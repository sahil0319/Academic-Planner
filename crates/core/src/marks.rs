//! Course marks, needed-in-final, and term GPA (M9, §13.3). Pure helpers take all inputs
//! as arguments; nothing here reads the clock.

use rusqlite::params;

use crate::db::{load_course, Planner};
use crate::error::{Error, Result};
use crate::grading::load_rule_set;
use crate::model::{
    Assumption, AttendanceBand, ComponentKind, ComponentMarks, CourseMarks, FinalOutlook,
    GpaStatus, GpaSummary, GradeRow, Needed, NeededRow,
};

impl Planner {
    /// R35-R38: marks, projected grade, and needed-in-final of one course.
    ///
    /// R33: a course without credits or without a rule set is not graded and has no marks.
    /// R37: the projection is `secured / assessed`; R38: `outlook` is the needed-in-final table, or
    /// the actual grade once the final has a score.
    pub fn course_marks(&self, course_id: i64) -> Result<CourseMarks> {
        let course = load_course(&self.conn, course_id)?;
        let (Some(credits), Some(rule_set_id)) = (course.credits, course.rule_set_id) else {
            return Ok(CourseMarks {
                course_id,
                graded: false,
                total: 0.0,
                components: Vec::new(),
                secured: 0.0,
                assessed: 0.0,
                projected_pct: None,
                letter: None,
                fully_assessed: false,
                outlook: None,
            });
        };
        let rule_set = load_rule_set(&self.conn, rule_set_id)?.ok_or(Error::NotFound {
            entity: "rule set",
            id: rule_set_id,
        })?;
        let total = 100.0 * credits;
        let attendance_pct = self.attendance_summary(course_id)?.pct;

        let mut score_stmt = self.conn.prepare(
            "SELECT score FROM assessment
             WHERE course_id = ?1 AND component_id = ?2 AND score IS NOT NULL",
        )?;
        let mut components = Vec::with_capacity(rule_set.components.len());
        for c in &rule_set.components {
            let marks = component_total(c.weight_pct, total);
            let mut part = ComponentMarks {
                component_id: c.id,
                name: c.name.clone(),
                kind: c.kind,
                is_final: c.is_final,
                marks,
                earned: 0.0,
                assessed_fraction: 0.0,
                item_max: None,
                scores_entered: 0,
                missing_items: 0,
            };
            match c.kind {
                ComponentKind::Attendance => {
                    // R35: an unknown percentage earns nothing and is not assessed.
                    if let Some(pct) = attendance_pct {
                        part.earned = band_credit(&c.bands, pct) / 100.0 * marks;
                        part.assessed_fraction = 1.0;
                    }
                }
                ComponentKind::Items => {
                    // R36: only scores linked to a component of the course's current rule set.
                    let scores = score_stmt
                        .query_map(params![course_id, c.id], |r| r.get::<_, f64>(0))?
                        .collect::<rusqlite::Result<Vec<_>>>()?;
                    let item_max = c.item_max.unwrap_or(marks);
                    let best_count = c.best_count.unwrap_or(1);
                    let (earned, fraction) = items_earned(&scores, best_count, item_max, marks);
                    part.earned = earned;
                    part.assessed_fraction = fraction;
                    part.item_max = Some(item_max);
                    part.scores_entered = scores.len() as u32;
                    // R38: missing items = item_count - entered, never below 0.
                    part.missing_items = c
                        .item_count
                        .unwrap_or(0)
                        .saturating_sub(part.scores_entered);
                }
            }
            components.push(part);
        }

        let secured: f64 = components.iter().map(|c| c.earned).sum();
        let assessed: f64 = components
            .iter()
            .map(|c| c.marks * c.assessed_fraction)
            .sum();
        // R37: nothing assessed means no projection and no letter.
        let projected_pct = (assessed > 0.0).then(|| round1(secured / assessed * 100.0));
        let scale = self.grade_scale()?;
        let letter = projected_pct.and_then(|pct| letter_for(&scale, pct));
        // R40: fully assessed = attendance percentage known and no item missing.
        let fully_assessed = components.iter().all(|c| match c.kind {
            ComponentKind::Attendance => c.assessed_fraction >= 1.0,
            ComponentKind::Items => c.missing_items == 0,
        });

        let outlook = final_outlook(&components, &scale, total, letter.clone());

        Ok(CourseMarks {
            course_id,
            graded: true,
            total,
            components,
            secured,
            assessed,
            projected_pct,
            letter,
            fully_assessed,
            outlook,
        })
    }

    /// R40: GPA over the non-archived graded courses of the active term.
    ///
    /// A graded course with a projected percentage is included (its letter's points weighted
    /// by credits); a graded course without one is left out. Ungraded (R33) and archived (R3)
    /// courses count in neither. `gpa = round2(sum(points x credits) / sum(credits))`, `None`
    /// when nothing is included. The status is `Final` only when at least one course is
    /// included and every included course is fully assessed; no active term gives an empty,
    /// projected summary.
    pub fn term_gpa(&self) -> Result<GpaSummary> {
        let mut summary = GpaSummary {
            gpa: None,
            status: GpaStatus::Projected,
            included: 0,
            left_out: 0,
        };
        if self.active_term()?.is_none() {
            return Ok(summary);
        }
        let scale = self.grade_scale()?;
        let mut weighted_points = 0.0;
        let mut credits_sum = 0.0;
        let mut all_final = true;
        for course in self.list_courses(false)? {
            let Some(credits) = course.credits else {
                continue;
            };
            let marks = self.course_marks(course.id)?;
            if !marks.graded {
                continue;
            }
            let points = marks
                .letter
                .as_deref()
                .and_then(|l| scale.iter().find(|row| row.letter == l))
                .map(|row| row.points);
            let Some(points) = points else {
                summary.left_out += 1;
                continue;
            };
            summary.included += 1;
            weighted_points += points * credits;
            credits_sum += credits;
            all_final &= marks.fully_assessed;
        }
        if summary.included > 0 && credits_sum > 0.0 {
            summary.gpa = Some(round2(weighted_points / credits_sum));
            if all_final {
                summary.status = GpaStatus::Final;
            }
        }
        Ok(summary)
    }
}

/// R38: the outlook of a graded course. When the final has a score it is the actual grade
/// (`letter`, the R37 letter); otherwise one row per grade-scale row (highest `min_pct`
/// first) with `need = min_pct / 100 x total - sum(earned of non-final components)` and
/// `ceil1(need / final_marks x final_item_max)`: `Secured` at or below 0, `NotReachable`
/// above the final item max. Assumptions list the non-final items components with missing
/// items (counted as 0), in position order. `None` when no final component exists.
fn final_outlook(
    components: &[ComponentMarks],
    scale: &[GradeRow],
    total: f64,
    letter: Option<String>,
) -> Option<FinalOutlook> {
    let final_part = components.iter().find(|c| c.is_final)?;
    if final_part.scores_entered > 0 {
        return Some(FinalOutlook::Actual { letter });
    }
    let final_item_max = final_part.item_max.unwrap_or(final_part.marks);
    let others_earned: f64 = components
        .iter()
        .filter(|c| !c.is_final)
        .map(|c| c.earned)
        .sum();
    let mut grades: Vec<&GradeRow> = scale.iter().collect();
    grades.sort_by(|a, b| b.min_pct.total_cmp(&a.min_pct));
    let rows = grades
        .into_iter()
        .map(|g| {
            let need = g.min_pct / 100.0 * total - others_earned;
            let needed = if need <= 0.0 {
                Needed::Secured
            } else {
                let value = ceil1(need / final_part.marks * final_item_max);
                if value > final_item_max {
                    Needed::NotReachable
                } else {
                    Needed::Score(value)
                }
            };
            NeededRow {
                letter: g.letter.clone(),
                min_pct: g.min_pct,
                needed,
            }
        })
        .collect();
    let assumptions = components
        .iter()
        .filter(|c| !c.is_final && c.kind == ComponentKind::Items && c.missing_items > 0)
        .map(|c| Assumption {
            component_name: c.name.clone(),
            missing_items: c.missing_items,
        })
        .collect();
    Some(FinalOutlook::Needed {
        rows,
        final_item_max,
        assumptions,
    })
}

/// R33: the marks of a component, `weight_pct` / 100 x the course total (100 x credits).
pub(crate) fn component_total(weight_pct: f64, total: f64) -> f64 {
    weight_pct / 100.0 * total
}

/// R35: share of the component's marks earned at attendance `pct`, as a percentage: the
/// `credit_pct` of the band with the highest `min_pct <= pct`.
pub(crate) fn band_credit(bands: &[AttendanceBand], pct: f64) -> f64 {
    bands
        .iter()
        .filter(|b| b.min_pct <= pct)
        .max_by(|a, b| a.min_pct.total_cmp(&b.min_pct))
        .map_or(0.0, |b| b.credit_pct)
}

/// R36: `(earned, assessed fraction)` of an items component from the entered `scores`:
/// earned = best `best_count` summed / (`best_count` x `item_max`) x `marks`; fraction =
/// min(entered, K) / K.
pub(crate) fn items_earned(
    scores: &[f64],
    best_count: u32,
    item_max: f64,
    marks: f64,
) -> (f64, f64) {
    if best_count == 0 || item_max <= 0.0 {
        return (0.0, 0.0);
    }
    let mut best = scores.to_vec();
    best.sort_by(|a, b| b.total_cmp(a));
    best.truncate(best_count as usize);
    let sum: f64 = best.iter().sum();
    let k = f64::from(best_count);
    (sum / (k * item_max) * marks, best.len() as f64 / k)
}

/// R37: round to 1 decimal, half away from zero.
pub(crate) fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

/// R40: round to 2 decimals, half away from zero.
pub(crate) fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// R38: round up to 1 decimal; float noise below 1e-9 does not push the result up.
pub(crate) fn ceil1(x: f64) -> f64 {
    (x * 10.0 - 1e-9).ceil() / 10.0
}

/// R37: letter of the grade-scale row with the highest `min_pct <= pct`.
pub(crate) fn letter_for(scale: &[GradeRow], pct: f64) -> Option<String> {
    scale
        .iter()
        .filter(|row| row.min_pct <= pct)
        .max_by(|a, b| a.min_pct.total_cmp(&b.min_pct))
        .map(|row| row.letter.clone())
}

#[cfg(test)]
mod tests;
