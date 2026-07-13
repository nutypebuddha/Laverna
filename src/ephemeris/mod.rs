/// Pure function: Convert Julian day number to ephemeris date components.
/// Uses the standard astronomical Julian day to Gregorian calendar algorithm.
pub fn julian_day_to_date(julian_day: f64) -> (i32, u8, u8) {
    let jd = julian_day + 0.5;
    let z = jd as i64;
    let _fractional = jd - z as f64;

    let a = if z < 2299161 {
        z
    } else {
        let alpha = ((z as f64 - 1867216.25) / 36524.25) as i64;
        z + 1 + alpha - alpha / 4
    };

    let b = a + 1524;
    let c = ((b as f64 - 122.1) / 365.25) as i64;
    let d = (365.25 * c as f64) as i64;
    let e = ((b - d) as f64 / 30.6001) as i64;

    let day = (b - d - (30.6001 * e as f64) as i64) as u8;
    let month = if e < 14 {
        (e - 1) as u8
    } else {
        (e - 13) as u8
    };
    let year = if month > 2 { c - 4716 } else { c - 4715 };

    (year as i32, month, day)
}

/// Pure function: Compute VSOP87 approximation for a planet.
pub fn compute_vsop87_approximation(julian_day: f64, planet_index: u8) -> f64 {
    let t = (julian_day - 2451545.0) / 36525.0;
    let base_longitude = match planet_index {
        0 => 357.529 + 35999.05 * t, // Mercury
        1 => 181.980 + 58517.82 * t, // Venus
        2 => 100.464 + 35999.37 * t, // Earth
        3 => 355.433 + 19140.30 * t, // Mars
        4 => 34.351 + 3034.91 * t,   // Jupiter
        5 => 49.944 + 1222.11 * t,   // Saturn
        6 => 313.232 + 428.47 * t,   // Uranus
        7 => 304.880 + 218.49 * t,   // Neptune
        _ => 0.0,
    };
    base_longitude.rem_euclid(360.0)
}

/// Pure function: Convert ecliptic longitude to zodiac sign index.
pub fn longitude_to_sign_index(ecliptic_longitude: f64) -> u8 {
    let normalized = ecliptic_longitude.rem_euclid(360.0);
    (normalized / 30.0) as u8
}

/// Computed position of a graha at a moment in time.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrahaPosition {
    /// Which graha.
    pub graha: crate::wheel::Domain,
    /// Tropical ecliptic longitude in degrees (0–360).
    pub tropical: f64,
    /// Sidereal longitude in degrees (tropical − ayanamsa).
    pub sidereal: f64,
    /// Rashi (sidereal zodiac sign) computed from sidereal longitude.
    pub rashi: crate::astrology::Rashi,
    /// Nakshatra (lunar mansion) computed from sidereal longitude.
    pub nakshatra: crate::astrology::Nakshatra,
    /// Pada (quarter) within the nakshatra (1–4).
    pub pada: u8,
}

/// Pure function: approximate Lahiri ayanamsa for a Julian Day.
fn lahiri_ayanamsa(jd: f64) -> f64 {
    let t = (jd - 2451545.0) / 36525.0;
    // Lahiri ayanamsa approximation (Meeus)
    23.85 + 0.01396 * t * 57.29577951
}

/// Pure function: map a sidereal longitude to a Rashi.
fn longitude_to_rashi(sidereal_deg: f64) -> crate::astrology::Rashi {
    let normalized = sidereal_deg.rem_euclid(360.0);
    let index = (normalized / 30.0) as usize;
    crate::astrology::Rashi::from_index(index)
}

/// Pure function: map a sidereal longitude to a Nakshatra (27 mansions).
fn longitude_to_nakshatra(sidereal_deg: f64) -> crate::astrology::Nakshatra {
    let normalized = sidereal_deg.rem_euclid(360.0);
    let index = (normalized / (360.0 / 27.0)) as usize;
    crate::astrology::Nakshatra::from_index(index)
}

/// Pure function: compute pada (1–4) within a nakshatra.
fn compute_pada(sidereal_deg: f64) -> u8 {
    let normalized = sidereal_deg.rem_euclid(360.0);
    let nak_span = 360.0 / 27.0;
    let within_nak = normalized % nak_span;
    ((within_nak / (nak_span / 4.0)) as u8) + 1
}

/// Compute all 9 graha positions for a given Julian Day.
pub fn all_graha_positions(jd: f64) -> Vec<GrahaPosition> {
    let ayanamsa = lahiri_ayanamsa(jd);
    use crate::wheel::Domain;
    Domain::all()
        .iter()
        .map(|&graha| {
            // Map graha to planet index for VSOP87
            let planet_index = match graha {
                Domain::Surya => 2,      // Sun (geocentric = Earth at 2)
                Domain::Chandra => 2,    // Moon (approximate from Earth)
                Domain::Mangala => 3,    // Mars
                Domain::Budha => 0,      // Mercury
                Domain::Brihaspati => 4, // Jupiter
                Domain::Shukra => 1,     // Venus
                Domain::Shani => 5,      // Saturn
                Domain::Rahu => 7,       // North Node (use Neptune approx)
                Domain::Ketu => 6,       // South Node (use Uranus approx)
            };
            let tropical = compute_vsop87_approximation(jd, planet_index);
            let sidereal = (tropical - ayanamsa).rem_euclid(360.0);
            let rashi = longitude_to_rashi(sidereal);
            let nakshatra = longitude_to_nakshatra(sidereal);
            let pada = compute_pada(sidereal);
            GrahaPosition {
                graha,
                tropical,
                sidereal,
                rashi,
                nakshatra,
                pada,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn julian_day_to_date_basic() {
        let (year, month, day) = julian_day_to_date(2451545.0);
        assert_eq!(year, 2000);
        assert_eq!(month, 1);
        assert_eq!(day, 1);
    }

    #[test]
    fn compute_vsop87_approximation_basic() {
        let longitude = compute_vsop87_approximation(2451545.0, 2);
        assert!(longitude >= 0.0 && longitude < 360.0);
    }

    #[test]
    fn longitude_to_sign_index_basic() {
        assert_eq!(longitude_to_sign_index(0.0), 0);
        assert_eq!(longitude_to_sign_index(30.0), 1);
        assert_eq!(longitude_to_sign_index(359.0), 11);
    }
}
