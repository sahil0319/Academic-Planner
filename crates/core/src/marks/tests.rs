//! Unit tests for the pure marks helpers (research D7).

use super::*;
use crate::model::{AttendanceBand, GradeRow};

fn default_bands() -> Vec<AttendanceBand> {
    [
        (90.0, 100.0),
        (85.0, 90.0),
        (80.0, 80.0),
        (75.0, 70.0),
        (70.0, 60.0),
        (0.0, 0.0),
    ]
    .into_iter()
    .map(|(min_pct, credit_pct)| AttendanceBand {
        min_pct,
        credit_pct,
    })
    .collect()
}

fn seeded_scale() -> Vec<GradeRow> {
    [
        ("A+", 80.0, 4.0),
        ("A", 75.0, 3.75),
        ("A-", 70.0, 3.5),
        ("B+", 65.0, 3.25),
        ("B", 60.0, 3.0),
        ("B-", 55.0, 2.75),
        ("C+", 50.0, 2.5),
        ("C", 45.0, 2.25),
        ("D", 40.0, 2.0),
        ("F", 0.0, 0.0),
    ]
    .into_iter()
    .map(|(letter, min_pct, points)| GradeRow {
        letter: letter.into(),
        min_pct,
        points,
    })
    .collect()
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

/// R35, T38: the band with the highest `min_pct <= pct` gives the credit; band edges are
/// inclusive.
#[test]
fn band_credit_uses_highest_band_at_or_below_pct() {
    let bands = default_bands();
    assert_close(band_credit(&bands, 92.0), 100.0);
    assert_close(band_credit(&bands, 90.0), 100.0);
    assert_close(band_credit(&bands, 89.9), 90.0);
    assert_close(band_credit(&bands, 85.0), 90.0);
    assert_close(band_credit(&bands, 84.9), 80.0);
    assert_close(band_credit(&bands, 0.0), 0.0);
}

/// R36, T37: the best `best_count` scores count; four quizzes, best three of 15 each.
#[test]
fn items_earned_best_three_of_four() {
    let (earned, fraction) = items_earned(&[12.0, 11.0, 13.0, 14.0], 3, 15.0, 45.0);
    assert_close(earned, 39.0);
    assert_close(fraction, 1.0);
}

/// R36: fewer scores than `best_count` earn what was entered and assess a part of the item.
#[test]
fn items_earned_fewer_scores_than_best_count() {
    let (earned, fraction) = items_earned(&[12.0, 11.0], 3, 15.0, 45.0);
    assert_close(earned, 23.0);
    assert_close(fraction, 2.0 / 3.0);
}

/// R36: no scores earn nothing and assess nothing.
#[test]
fn items_earned_no_scores() {
    let (earned, fraction) = items_earned(&[], 1, 75.0, 75.0);
    assert_close(earned, 0.0);
    assert_close(fraction, 0.0);
}

/// R36: with more scores than `best_count`, only the best ones are summed.
#[test]
fn items_earned_five_scores_best_three_used() {
    let (earned, fraction) = items_earned(&[5.0, 15.0, 10.0, 14.0, 1.0], 3, 15.0, 45.0);
    assert_close(earned, 39.0 / 45.0 * 45.0);
    assert_close(fraction, 1.0);
}

/// R37: round to 1 decimal, half away from zero.
#[test]
fn round1_rounds_half_away_from_zero() {
    assert_close(round1(79.94), 79.9);
    assert_close(round1(79.95), 80.0);
}

/// R40: round to 2 decimals.
#[test]
fn round2_rounds_to_two_decimals() {
    assert_close(round2(3.666_666_666_666_666_5), 3.67);
}

/// R38, T39: round up to 1 decimal, ignoring float noise below 1e-9.
#[test]
fn ceil1_rounds_up_with_float_tolerance() {
    assert_close(ceil1(111.000_000_000_000_01), 111.0);
    assert_close(ceil1(111.01), 111.1);
    assert_close(ceil1(96.0), 96.0);
    assert_close(ceil1(-5.0), -5.0);
}

/// R37, R40, T40: the letter of the row with the highest `min_pct <= pct`.
#[test]
fn letter_for_picks_highest_row_at_or_below_pct() {
    let scale = seeded_scale();
    assert_eq!(letter_for(&scale, 79.9).as_deref(), Some("A"));
    assert_eq!(letter_for(&scale, 80.0).as_deref(), Some("A+"));
    assert_eq!(letter_for(&scale, 0.0).as_deref(), Some("F"));
}
