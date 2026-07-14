//! # Swiss-Ephemeris Oracle Regression Test
//!
//! Dependency-free stand-in for the "Swiss-oracle CI check" called for in
//! `laverna-lessons-and-dependency-policy.md` Part 3 #1. The lessons doc
//! recommends diffing Laverna's chart against Swiss Ephemeris on every commit
//! and failing the build if any graha drifts past a threshold (it suggests
//! 0.05°).
//!
//! The original plan reached for the `swiss-eph` Rust crate as a test-only
//! oracle. That crate (0.2.1) does not compile on the current toolchain — 31
//! errors from a `c_char`/`u8` mismatch *inside the crate*, unrelated to
//! Laverna. Per the project's suckless dependency policy ("no dependency tree
//! of its own", "removable as a contained change") and lesson 1.5 ("every
//! number must trace to a command that was actually run"), we instead pin the
//! ground-truth values that were themselves verified against Swiss Ephemeris
//! earlier in this project's audit and assert Laverna keeps reproducing them.
//!
//! ## Ground truth provenance (traceable, not invented)
//!
//! Every expected value below comes from `laverna-fixes.md` §1.1, which states
//! it was "verified against Swiss Ephemeris via `pyswisseph 2.10.03`,
//! `SIDM_LAHIRI`" for the chart at datetime `1994-04-14 20:09:00` (JD
//! `2449457.3396`). The audit's own reproduction:
//!
//! ```text
//! $ laverna chart --datetime "1994-04-14 20:09:00"
//! julian_day: 2449457.3396
//! ayanamsa (lahiri): 23.7808°
//! Surya: tropical 24.65°   Chandra: tropical 66.10°   ...   Ketu: tropical 55.59°
//! ```
//!
//! Worst case in that audit was the Moon at 1 arcminute (0.02°) vs Swiss;
//! every other body was within 0.01°. The "Swiss (tropical)" column is the
//! oracle; we assert Laverna reproduces it within 0.05° — comfortably above
//! the 0.02° already observed, tight enough to catch real regressions.
//!
//! This catches the same failure mode the doc targets: a silent drift in the
//! ephemeris (e.g. the kind of method change that produced the lagna's 1.4°
//! gap) now breaks CI instead of shipping.

use laverna::astrology::{Graha, ALL_GRAHAS};
use laverna::chart::ChartSnapshot;
use laverna::ephemeris::{julian_day, tropical_longitude};

/// All 9 grahas in wheel order (`Surya` … `Ketu`).
const WHEEL: &[Graha] = &ALL_GRAHAS;

/// Julian Day for the audit's reference chart: 1994-04-14 20:09:00 UT.
const REFERENCE_JD: f64 = 2_449_457.339_6;

/// Swiss-Ephemeris-verified tropical longitudes (degrees) for the 8 grahas
/// that have an independent ground truth in the audit. Order matches
/// `WHEEL` (Surya, Chandra, Mangala, Budha, Brihaspati, Shukra, Shani,
/// Rahu); Ketu is the opposite point of Rahu and is checked structurally.
const SWISS_TROPICAL: &[(Graha, f64)] = &[
    (Graha::Surya, 24.64),
    (Graha::Chandra, 66.12),
    (Graha::Mangala, 0.07),
    (Graha::Budha, 8.88),
    (Graha::Brihaspati, 221.71),
    (Graha::Shukra, 46.03),
    (Graha::Shani, 338.75),
    (Graha::Rahu, 235.60),
];

/// Threshold recommended by the lessons doc Part 3 #1.
const TOLERANCE_DEG: f64 = 0.05;

fn norm360(deg: f64) -> f64 {
    let d = deg % 360.0;
    if d < 0.0 {
        d + 360.0
    } else {
        d
    }
}

#[test]
fn swiss_oracle_graha_tropical_positions() {
    for &(graha, swiss) in SWISS_TROPICAL {
        let computed = tropical_longitude(graha, REFERENCE_JD);
        let delta = (computed - swiss).abs();
        assert!(
            delta <= TOLERANCE_DEG,
            "{:?}: tropical {computed:.4}° vs Swiss {swiss:.4}° (Δ {delta:.4}° > {TOLERANCE_DEG}°)",
            graha
        );
    }
}

#[test]
fn swiss_oracle_ketu_is_rahu_opposite() {
    // Ketu has no independent audit row; it must be the antipode of Rahu.
    let rahu = tropical_longitude(Graha::Rahu, REFERENCE_JD);
    let ketu = tropical_longitude(Graha::Ketu, REFERENCE_JD);
    let sep = norm360(ketu - rahu).abs();
    let sep = sep.min(360.0 - sep);
    assert!(
        (sep - 180.0).abs() < 1e-6,
        "Rahu–Ketu separation {sep:.6}° != 180°"
    );
}

#[test]
fn swiss_oracle_deterministic_wheel() {
    let a: Vec<f64> = WHEEL
        .iter()
        .map(|&g| tropical_longitude(g, REFERENCE_JD))
        .collect();
    let b: Vec<f64> = WHEEL
        .iter()
        .map(|&g| tropical_longitude(g, REFERENCE_JD))
        .collect();
    assert_eq!(a.len(), 9);
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.to_bits(), y.to_bits(), "ephemeris not deterministic");
    }
}

#[test]
fn swiss_oracle_reference_jd_matches_audit() {
    // Pin the datetime→JD mapping the ground truth was computed against.
    let jd = julian_day(1994, 4, 14, 20.15);
    assert!(
        (jd - REFERENCE_JD).abs() < 1e-4,
        "julian_day(1994-04-14 20:09) = {jd}, expected {REFERENCE_JD}"
    );
}

#[test]
fn swiss_oracle_lagna_degree_matches_swiss() {
    // Audit reference chart (lat 45.4, lon -92.9). The sidereal ascendant (the
    // lagna *degree*) was verified against Swiss Ephemeris at 127.65°. The
    // audit's "1.4° gap" was a mis-comparison of this degree against the
    // whole-sign cusp, not a fault of the ascendant formula — so we oracle the
    // degree itself to catch any future drift in the ascendant computation.
    let snap = ChartSnapshot::new(REFERENCE_JD).with_location(45.4, -92.9);
    let deg = snap
        .ascendant_deg
        .expect("ascendant degree computed with location");
    assert!(
        (deg - 127.65).abs() <= TOLERANCE_DEG,
        "lagna degree {deg:.4}° vs Swiss 127.65° (Δ {:.4}°)",
        (deg - 127.65).abs()
    );
    // The lagna rashi must be the rising sidereal sign (Simha).
    assert_eq!(snap.lagna.expect("lagna computed").name(), "Simha");
}
