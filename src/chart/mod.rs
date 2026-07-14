pub mod personality;

use serde::{Deserialize, Serialize};

use crate::astrology::{Graha, House, Rashi};
use crate::ephemeris::{self, GrahaPosition};

/// A real astronomical aspect between two grahas, computed from their actual
/// ecliptic longitude separation on a given date (with standard orb
/// tolerances). Distinct from `crate::wheel::CompositionAspect`, which is a
/// fixed structural relationship on the 9-node compositional wheel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AstroAspect {
    /// ~0° separation (± orb).
    Conjunction,
    /// ~60° separation (± orb).
    Sextile,
    /// ~90° separation (± orb).
    Square,
    /// ~120° separation (± orb).
    Trine,
    /// ~180° separation (± orb).
    Opposition,
}

pub type Aspect = AstroAspect;

pub use personality::{
    derive_personality, AspectModifier, PersonalityProfile, Pillar, WatchArchetype,
};

/// A complete sky snapshot at a moment in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSnapshot {
    /// Julian Day of this snapshot.
    pub julian_day: f64,
    /// Observer's geographic latitude in degrees (N positive).
    pub latitude: Option<f64>,
    /// Observer's geographic longitude in degrees (E positive).
    pub longitude: Option<f64>,
    /// All 9 graha positions computed from ephemeris.
    pub graha_positions: Vec<GrahaPosition>,
    /// Lagna (ascendant) rashi — computed if latitude/longitude are set.
    pub lagna: Option<Rashi>,
    /// Lagna (ascendant) sidereal ecliptic degree [0, 360) — the exact
    /// ascendant point, computed if latitude/longitude are set.
    pub ascendant_deg: Option<f64>,
    /// House cusps (bhavas) — computed if latitude/longitude are set.
    /// Each entry is (House, tropical longitude in degrees, sidereal rashi).
    pub house_cusps: Vec<HouseCusp>,
    /// Aspect matrix: aspect between each pair of grahas (9×9).
    /// Indexed by Graha::index() (0=Surya..8=Ketu).
    pub aspect_matrix: Vec<Vec<Option<Aspect>>>,
    /// Human-readable label (e.g. "birth chart for major_depression").
    pub label: Option<String>,
}

/// A single house cusp: the house, its tropical longitude, and sidereal rashi.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HouseCusp {
    /// Which house (1st through 12th).
    pub house: House,
    /// Tropical ecliptic longitude in degrees [0, 360).
    pub tropical: f64,
    /// Sidereal rashi at this cusp longitude.
    pub rashi: Rashi,
}

impl ChartSnapshot {
    /// Create a new snapshot for the given Julian Day.
    pub fn new(jd: f64) -> Self {
        let graha_positions = ephemeris::all_graha_positions(jd);
        let aspect_matrix = Self::compute_aspect_matrix(&graha_positions);

        ChartSnapshot {
            julian_day: jd,
            latitude: None,
            longitude: None,
            graha_positions,
            lagna: None,
            ascendant_deg: None,
            house_cusps: Vec::new(),
            aspect_matrix,
            label: None,
        }
    }

    /// Set observer location and recompute lagna + house cusps.
    pub fn with_location(mut self, latitude: f64, longitude: f64) -> Self {
        self.latitude = Some(latitude);
        self.longitude = Some(longitude);
        self.lagna = self.compute_lagna(latitude, longitude);
        self.ascendant_deg = Some(self.compute_ascendant_sidereal_deg(latitude, longitude));
        self.house_cusps = self.compute_house_cusps(latitude, longitude);
        self
    }

    /// Set a label for this snapshot.
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Compute the aspect matrix from actual sky positions (tropical longitudes).
    /// Returns an n×n matrix where matrix[i][j] = Some(Aspect) if the angular
    /// separation between graha i and graha j matches a known aspect orb.
    fn compute_aspect_matrix(positions: &[GrahaPosition]) -> Vec<Vec<Option<Aspect>>> {
        let n = positions.len();
        let mut matrix = Vec::with_capacity(n);
        for i in 0..n {
            let mut row = Vec::with_capacity(n);
            for j in 0..n {
                let result = if i == j {
                    Some(Aspect::Conjunction)
                } else {
                    let diff = (positions[i].tropical - positions[j].tropical).abs();
                    let diff = if diff > 180.0 { 360.0 - diff } else { diff };
                    angular_diff_to_aspect(diff)
                };
                row.push(result);
            }
            matrix.push(row);
        }
        matrix
    }

    /// Sidereal ecliptic degree [0, 360) of the ascendant (lagna point) for a
    /// given observer — the exact rising point, before converting to a rashi.
    ///
    /// Shared by `compute_lagna` (rashi) and `compute_house_cusps`
    /// (rising-sign start) so the ascendant is computed exactly once and
    /// cannot drift between the two. Pipeline:
    /// 1. GMST from JD (xalen_houses::gmst)
    /// 2. Local Sidereal Time from GMST + longitude
    /// 3. RAMC from LST (xalen_houses::compute_ramc)
    /// 4. Ascendant ecliptic longitude (xalen_houses::compute_ascendant)
    /// 5. Subtract ayanamsa → sidereal degree
    fn compute_ascendant_sidereal_deg(&self, latitude: f64, longitude: f64) -> f64 {
        let gmst_hours = xalen_houses::gmst(self.julian_day);
        let lst_hours = xalen_houses::local_sidereal_time(gmst_hours, longitude);
        let ramc_rad = xalen_houses::compute_ramc(lst_hours);

        let epsilon_rad = xalen_coords::mean_obliquity(
            (self.julian_day - xalen_time::J2000_JD) / xalen_time::DAYS_PER_JULIAN_CENTURY,
        );

        let lat_rad = latitude.to_radians();
        let asc_tropical =
            norm360(xalen_houses::compute_ascendant(ramc_rad, epsilon_rad, lat_rad).to_degrees());

        let ayanamsa = ephemeris::lahiri_ayanamsa(self.julian_day);
        norm360(asc_tropical - ayanamsa)
    }

    fn compute_lagna(&self, latitude: f64, longitude: f64) -> Option<Rashi> {
        let asc_sidereal = self.compute_ascendant_sidereal_deg(latitude, longitude);
        let xrashi = xalen_vedic::rashi::Rashi::from_longitude_deg(asc_sidereal);
        Some(Rashi::from_index(xrashi.index()))
    }

    /// Compute all 12 house cusps (bhavas) using the sidereal Whole Sign system.
    ///
    /// Sidereal Whole Sign: the 1st house is the *entire sidereal sign* that
    /// contains the sidereal ascendant, and the 12 houses are the 12 sidereal
    /// signs in order. The 1st cusp is therefore the sidereal start of the
    /// rising sign — which is what Swiss Ephemeris produces for `b'W'` with the
    /// sidereal flag.
    ///
    /// This is computed directly from the sidereal ascendant rather than via
    /// `xalen_houses::compute_houses_sidereal`: that helper builds the cusps in
    /// the *tropical* sign frame of the ascendant and only then subtracts the
    /// ayanamsa, which mis-frames the houses (it lands on the wrong sign
    /// boundary by the ayanamsa amount). Building from the sidereal ascendant
    /// keeps every cusp on a true sidereal sign boundary and keeps the 1st
    /// house's rashi identical to the lagna rashi.
    fn compute_house_cusps(&self, latitude: f64, longitude: f64) -> Vec<HouseCusp> {
        let asc_sidereal = self.compute_ascendant_sidereal_deg(latitude, longitude);

        // 1st cusp = sidereal start of the rising sign; the rest step by 30°.
        let rising_sign_start = (asc_sidereal / 30.0).floor() * 30.0;
        let mut result = Vec::with_capacity(12);
        for i in 0..12 {
            let cusp_sidereal = norm360(rising_sign_start + i as f64 * 30.0);
            let xrashi = xalen_vedic::rashi::Rashi::from_longitude_deg(cusp_sidereal);
            result.push(HouseCusp {
                house: House::from_index(i),
                tropical: cusp_sidereal,
                rashi: Rashi::from_index(xrashi.index()),
            });
        }
        result
    }

    /// Get the position of a specific graha.
    pub fn graha_position(&self, graha: Graha) -> Option<&GrahaPosition> {
        self.graha_positions.iter().find(|p| p.graha == graha)
    }

    /// Get the aspect between two grahas in this snapshot.
    pub fn aspect_between(&self, a: Graha, b: Graha) -> Option<Aspect> {
        let i = a.index();
        let j = b.index();
        self.aspect_matrix
            .get(i)
            .and_then(|row| row.get(j))
            .copied()
            .flatten()
    }

    /// Compute synastry (inter-chart aspects) between this snapshot and another.
    ///
    /// Returns a list of (graha_a, graha_b, aspect) pairs where the aspect is
    /// determined by the angular difference between the same graha in each chart.
    pub fn synastry_with(&self, other: &ChartSnapshot) -> Vec<SynastryAspect> {
        let mut results = Vec::new();
        for pos_a in &self.graha_positions {
            for pos_b in &other.graha_positions {
                let diff = (pos_a.tropical - pos_b.tropical).abs();
                let diff = if diff > 180.0 { 360.0 - diff } else { diff };
                let aspect = angular_diff_to_aspect(diff);
                results.push(SynastryAspect {
                    graha_a: pos_a.graha,
                    graha_b: pos_b.graha,
                    angular_distance: diff,
                    aspect,
                });
            }
        }
        results
    }

    /// Format the chart as a readable display string.
    pub fn format(&self) -> String {
        let mut out = String::new();
        let label = self.label.as_deref().unwrap_or("Chart Snapshot");

        out.push_str(&format!("═══ {} ═══\n", label));
        out.push_str(&format!(
            "JD: {:.5}  ({})\n\n",
            self.julian_day,
            self.format_date(),
        ));

        if let Some(lagna) = self.lagna {
            let deg_line = match self.ascendant_deg {
                Some(deg) => format!("  —  {deg:.4}° sidereal"),
                None => String::new(),
            };
            out.push_str(&format!(
                "Lagna (Ascendant): {} {} ({:?}){}\n\n",
                lagna.symbol(),
                lagna.name(),
                lagna,
                deg_line,
            ));
        }

        if !self.house_cusps.is_empty() {
            out.push_str("── Bhavas (House Cusps) ──\n");
            out.push_str(&format!(
                "{:8} | {:>9} | {:10} | {}\n",
                "house", "longitude", "rashi", "domain"
            ));
            for cusp in &self.house_cusps {
                out.push_str(&format!(
                    "{:8} | {:>8.2}° | {:10} | {}\n",
                    cusp.house.name(),
                    cusp.tropical,
                    cusp.rashi.name(),
                    cusp.house.domain(),
                ));
            }
            out.push('\n');
        }

        out.push_str(&format!(
            "{:12} | {:>9} | {:>9} | {:10} | {:16} | pada\n",
            "graha", "tropical", "sidereal", "rashi", "nakshatra"
        ));
        for pos in &self.graha_positions {
            out.push_str(&format!(
                "{:12} | {:8.4}° | {:8.4}° | {:10} | {:16} | {}\n",
                pos.graha.name(),
                pos.tropical,
                pos.sidereal,
                pos.rashi.name(),
                pos.nakshatra.name(),
                pos.pada,
            ));
        }

        out.push_str("\n── Aspect Matrix ──\n");
        out.push_str(&format!("{:12}", ""));
        for g in Graha::all() {
            out.push_str(&format!(" {:>8}", g.symbol()));
        }
        out.push('\n');

        for g in Graha::all() {
            let gi = g.index();
            out.push_str(&format!("{:12}", g.symbol()));
            for gj in 0..9 {
                let aspect = self.aspect_matrix[gi][gj];
                match aspect {
                    Some(a) => out.push_str(&format!(" {:>8}", format!("{:?}", a))),
                    None => out.push_str(&format!(" {:>8}", "")),
                }
            }
            out.push('\n');
        }

        out
    }

    /// Format the Julian Day as an approximate date string.
    fn format_date(&self) -> String {
        let (year, month, day) = ephemeris::julian_day_to_date(self.julian_day);
        format!("{}-{:02}-{:02}", year, month, day)
    }
}

/// A synastry aspect between two grahas from different charts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynastryAspect {
    /// Graha in the first chart.
    pub graha_a: Graha,
    /// Graha in the second chart.
    pub graha_b: Graha,
    /// Angular distance in degrees (0–180).
    pub angular_distance: f64,
    /// Mapped aspect type.
    pub aspect: Option<Aspect>,
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Normalize an angle in degrees to [0, 360).
fn norm360(deg: f64) -> f64 {
    let d = deg % 360.0;
    if d < 0.0 {
        d + 360.0
    } else {
        d
    }
}

/// Map an angular difference (0–180°) to a wheel aspect.
///
/// Uses traditional orbs around each exact aspect angle; a separation
/// outside every orb is genuinely no aspect:
/// - Conjunction   0° ± 10°
/// - Sextile      60° ±  6°
/// - Square       90° ±  8°
/// - Trine       120° ±  8°
/// - Opposition  180° ± 10°
fn angular_diff_to_aspect(diff: f64) -> Option<Aspect> {
    let diff = diff.min(180.0);
    if diff <= 10.0 {
        Some(Aspect::Conjunction)
    } else if (diff - 60.0).abs() <= 6.0 {
        Some(Aspect::Sextile)
    } else if (diff - 90.0).abs() <= 8.0 {
        Some(Aspect::Square)
    } else if (diff - 120.0).abs() <= 8.0 {
        Some(Aspect::Trine)
    } else if diff >= 170.0 {
        Some(Aspect::Opposition)
    } else {
        None
    }
}

/// Create a ChartSnapshot for the current datetime.
impl Default for ChartSnapshot {
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let secs_since_epoch = now.as_secs_f64();
        let jd = 2440587.5 + secs_since_epoch / 86400.0;

        let mut snap = ChartSnapshot::new(jd);
        snap.label = Some("now (system time)".to_string());
        snap
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_snapshot_has_9_grahas() {
        let snap = ChartSnapshot::new(2451545.0);
        assert_eq!(snap.graha_positions.len(), 9);
        assert_eq!(snap.graha_positions[0].graha, Graha::Surya);
        assert_eq!(snap.graha_positions[8].graha, Graha::Ketu);
    }

    #[test]
    fn aspect_matrix_9x9() {
        let snap = ChartSnapshot::new(2451545.0);
        assert_eq!(snap.aspect_matrix.len(), 9);
        for row in &snap.aspect_matrix {
            assert_eq!(row.len(), 9);
        }
        for i in 0..9 {
            assert_eq!(snap.aspect_matrix[i][i], Some(Aspect::Conjunction));
        }
    }

    #[test]
    fn aspect_matrix_matches_longitudes() {
        let snap = ChartSnapshot::new(2447728.345138889);
        for i in 0..9 {
            for j in 0..9 {
                if i == j {
                    continue;
                }
                let diff =
                    (snap.graha_positions[i].tropical - snap.graha_positions[j].tropical).abs();
                let diff = if diff > 180.0 { 360.0 - diff } else { diff };
                assert_eq!(
                    snap.aspect_matrix[i][j],
                    angular_diff_to_aspect(diff),
                    "aspect[{i}][{j}] disagrees with angular separation {diff:.2}°"
                );
            }
        }
    }

    #[test]
    fn lagna_with_location() {
        let snap = ChartSnapshot::new(2451545.0).with_location(51.5, 0.0);
        assert!(snap.lagna.is_some());
        let lagna = snap.lagna.unwrap();
        assert!((lagna.index()) < 12);
    }

    #[test]
    fn whole_sign_cusps_align_with_sidereal_ascendant() {
        // Audit reference chart: 1994-04-14 20:09 UT, lat 45.4, lon -92.9.
        // Sidereal ascendant = 127.648° (Simha). In sidereal Whole Sign the 1st
        // cusp is the *start of the rising sidereal sign* (120.0° Simha) — not
        // the ascendant degree, and not a tropical-frame-shifted boundary.
        let jd = ephemeris::julian_day(1994, 4, 14, 20.15);
        let snap = ChartSnapshot::new(jd).with_location(45.4, -92.9);
        let cusps = &snap.house_cusps;
        assert_eq!(cusps.len(), 12);
        assert!(
            (cusps[0].tropical - 120.0).abs() < 1e-6,
            "cusp1 should be start of rising sidereal sign, got {}",
            cusps[0].tropical
        );
        for i in 0..12 {
            let expected = (120.0 + i as f64 * 30.0) % 360.0;
            assert!(
                (cusps[i].tropical - expected).abs() < 1e-6,
                "cusp{} = {}, expected {}",
                i + 1,
                cusps[i].tropical,
                expected
            );
        }
        // The 1st house rashi must match the lagna rashi (both Simha).
        assert_eq!(cusps[0].rashi, snap.lagna.unwrap());
    }

    #[test]
    fn synastry_self() {
        let snap = ChartSnapshot::new(2451545.0);
        let aspects = snap.synastry_with(&snap);
        for a in &aspects {
            if a.graha_a == a.graha_b {
                assert_eq!(a.aspect, Some(Aspect::Conjunction));
            }
        }
    }

    #[test]
    fn angular_diff_to_aspect_cases() {
        assert_eq!(angular_diff_to_aspect(0.0), Some(Aspect::Conjunction));
        assert_eq!(angular_diff_to_aspect(60.0), Some(Aspect::Sextile));
        assert_eq!(angular_diff_to_aspect(90.0), Some(Aspect::Square));
        assert_eq!(angular_diff_to_aspect(120.0), Some(Aspect::Trine));
        assert_eq!(angular_diff_to_aspect(180.0), Some(Aspect::Opposition));
        assert_eq!(angular_diff_to_aspect(10.0), Some(Aspect::Conjunction));
        assert_eq!(angular_diff_to_aspect(54.0), Some(Aspect::Sextile));
        assert_eq!(angular_diff_to_aspect(66.0), Some(Aspect::Sextile));
        assert_eq!(angular_diff_to_aspect(82.0), Some(Aspect::Square));
        assert_eq!(angular_diff_to_aspect(98.0), Some(Aspect::Square));
        assert_eq!(angular_diff_to_aspect(112.0), Some(Aspect::Trine));
        assert_eq!(angular_diff_to_aspect(128.0), Some(Aspect::Trine));
        assert_eq!(angular_diff_to_aspect(170.0), Some(Aspect::Opposition));
        assert_eq!(angular_diff_to_aspect(15.0), None);
        assert_eq!(angular_diff_to_aspect(45.0), None);
        assert_eq!(angular_diff_to_aspect(40.0), None);
        assert_eq!(angular_diff_to_aspect(105.0), None);
        assert_eq!(angular_diff_to_aspect(140.0), None);
        assert_eq!(angular_diff_to_aspect(169.9), None);
    }

    #[test]
    fn opposition_inclusive_at_180() {
        assert_eq!(angular_diff_to_aspect(180.0), Some(Aspect::Opposition));
        assert_eq!(angular_diff_to_aspect(175.0), Some(Aspect::Opposition));
        assert_eq!(angular_diff_to_aspect(179.999), Some(Aspect::Opposition));
    }

    #[test]
    fn rahu_ketu_always_opposition() {
        for jd in [2451545.0, 2459204.5, 2460676.5, 2460841.5] {
            let snap = ChartSnapshot::new(jd);
            assert_eq!(
                snap.aspect_between(Graha::Rahu, Graha::Ketu),
                Some(Aspect::Opposition),
                "Rahu–Ketu not Opposition at JD {jd}"
            );
        }
    }

    #[test]
    fn chart_is_deterministic() {
        let jd = ephemeris::julian_day(2026, 7, 7, 12.0);
        let a = ChartSnapshot::new(jd);
        let b = ChartSnapshot::new(jd);
        for (pa, pb) in a.graha_positions.iter().zip(b.graha_positions.iter()) {
            assert_eq!(pa.tropical.to_bits(), pb.tropical.to_bits());
        }
    }

    #[test]
    fn graha_position_lookup() {
        let snap = ChartSnapshot::new(2451545.0);
        let surya = snap.graha_position(Graha::Surya);
        assert!(surya.is_some());
        assert_eq!(surya.unwrap().graha, Graha::Surya);

        let ketu = snap.graha_position(Graha::Ketu);
        assert!(ketu.is_some());
        assert_eq!(ketu.unwrap().graha, Graha::Ketu);
    }

    #[test]
    fn aspect_between_consistent_with_matrix() {
        let snap = ChartSnapshot::new(2451545.0);
        for i in 0..9 {
            for j in 0..9 {
                let a = Graha::from_index(i).unwrap();
                let b = Graha::from_index(j).unwrap();
                let expected = if i == j {
                    Some(Aspect::Conjunction)
                } else {
                    let pos_a = snap.graha_position(a).unwrap();
                    let pos_b = snap.graha_position(b).unwrap();
                    let diff = (pos_a.tropical - pos_b.tropical).abs();
                    let diff = if diff > 180.0 { 360.0 - diff } else { diff };
                    angular_diff_to_aspect(diff)
                };
                assert_eq!(
                    snap.aspect_between(a, b),
                    expected,
                    "aspect between {a:?} and {b:?} at J2000.0"
                );
            }
        }
    }

    #[test]
    fn format_does_not_panic() {
        let snap = ChartSnapshot::new(2451545.0)
            .with_location(40.7, -74.0)
            .with_label("test chart");
        let output = snap.format();
        assert!(!output.is_empty());
        assert!(output.contains("Surya"));
        assert!(output.contains("Aspect Matrix"));
    }

    // ── D6 regression: house cusps (bhavas) ──

    #[test]
    fn house_cusps_computed_with_location() {
        let snap = ChartSnapshot::new(2451545.0).with_location(51.5, 0.0);
        assert_eq!(snap.house_cusps.len(), 12, "must have 12 house cusps");
        // First cusp (1st house) should match the lagna longitude
        let first_cusp = &snap.house_cusps[0];
        assert_eq!(first_cusp.house, crate::astrology::House::First);
    }

    #[test]
    fn house_cusps_empty_without_location() {
        let snap = ChartSnapshot::new(2451545.0);
        assert!(snap.house_cusps.is_empty());
    }

    #[test]
    fn house_cusps_all_12_houses_present() {
        let snap = ChartSnapshot::new(2451545.0).with_location(40.7, -74.0);
        for i in 0..12 {
            let cusp = &snap.house_cusps[i];
            assert_eq!(cusp.house, crate::astrology::House::from_index(i));
            assert!(
                cusp.tropical >= 0.0 && cusp.tropical < 360.0,
                "cusp longitude must be in [0, 360): {}",
                cusp.tropical
            );
        }
    }

    #[test]
    fn house_cusps_deterministic() {
        let a = ChartSnapshot::new(2451545.0).with_location(51.5, 0.0);
        let b = ChartSnapshot::new(2451545.0).with_location(51.5, 0.0);
        for (ca, cb) in a.house_cusps.iter().zip(b.house_cusps.iter()) {
            assert_eq!(ca.tropical.to_bits(), cb.tropical.to_bits());
        }
    }

    #[test]
    fn format_includes_bhavas() {
        let snap = ChartSnapshot::new(2451545.0)
            .with_location(40.7, -74.0)
            .with_label("test");
        let output = snap.format();
        assert!(output.contains("Bhavas"));
        assert!(output.contains("1st"));
        assert!(output.contains("10th"));
    }
}
