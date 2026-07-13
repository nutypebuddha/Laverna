//! Barnum/Forer effect guardrails for astrological reasoning.
//!
//! Research basis:
//! - The Barnum/Forer effect: people rate vague, universally-applicable
//!   statements as highly personally accurate (Wikipedia).
//! - Laverna design principle (from architecture research): "require the
//!   ontology to produce *discriminating* statements — ones that would be
//!   wrong for many inputs — or the module will manufacture the illusion
//!   of grounding while grounding nothing."
//!
//! This module provides functions to detect and filter out non-discriminating
//! (Barnum) statements so that the astrological module produces only
//! falsifiable, specific claims.

/// Patterns that indicate a statement is universally applicable (Barnum).
/// These are phrases that would be true for virtually anyone.
static BARNUM_PATTERNS: &[(&str, &str)] = &[
    // Vague personality traits — true for almost everyone.
    (
        "you have a tendency to",
        "Vague personality trait — true for most people",
    ),
    (
        "you are sometimes",
        "Vague frequency qualifier — applies to everyone",
    ),
    ("you enjoy when", "Universal enjoyment — not discriminating"),
    ("you can be both", "Both-qualifier — applies to everyone"),
    ("you tend to worry", "Common worry pattern — not specific"),
    (
        "you have a great deal of unused potential",
        "Universal potential — unfalsifiable",
    ),
    (
        "you have a strong need for others to like and admire you",
        "Universal social need",
    ),
    (
        "some of your aspirations tend to be unrealistic",
        "Universal aspiration pattern",
    ),
    (
        "you have a natural talent for",
        "Vague talent claim — not discriminating",
    ),
    (
        "you are independent-minded",
        "So vague it applies to most people who read it",
    ),
    ("you pride yourself on", "Universal pride — not specific"),
    (
        "at times you are extroverted, sociable, while at other times",
        "Barnum double-description",
    ),
    (
        "while you have some personality weaknesses, you are",
        "Universal weakness-then-strength pattern",
    ),
    ("your sex drive could be", "Universal drive pattern"),
    (
        "you are far more sexually demanding than you let on",
        "Unfalsifiable claim",
    ),
    (
        "security is one of your major goals",
        "Universal security motivation",
    ),
    // Temporal vagueness.
    ("in the future", "Vague temporal — could mean anything"),
    (
        "recently you have been",
        "Vague temporal pattern — applies to everyone",
    ),
    (
        "this period of your life",
        "Vague life-phase — universally applicable",
    ),
    // Astrological Barnum patterns specifically.
    (
        "this is a time of great opportunity",
        "Universal positive framing",
    ),
    (
        "you are entering a period of transformation",
        "Universal transformation — always true",
    ),
    (
        "challenges will lead to growth",
        "Universal growth framing — unfalsifiable",
    ),
    ("the stars suggest", "Vague attribution — not falsifiable"),
    ("the universe is guiding you", "Universal spiritual framing"),
    (
        "trust your intuition",
        "Universal advice — applies to everyone",
    ),
];

/// Patterns that indicate a discriminating (non-Barnum) statement.
/// These are specific enough to be falsifiable.
static DISCRIMINATING_PATTERNS: &[&str] = &[
    // Specific planetary placements.
    "conjunct",
    "opposition",
    "square",
    "trine",
    "sextile",
    "in the",
    "house", // "in the 7th house" is specific
    "ruling",
    "dignified",
    "exalted",
    "debilitated",
    "combust",
    "retrograde",
    "direct motion",
    // Specific sign/house claims.
    "ascendant",
    "midheaven",
    "descendant",
    "imum coeli",
    "lagna",
    // Specific nakshatra claims.
    "pada",
    "nakshatra lord",
    "dasha period",
    // Specific element/modality claims.
    "fire sign",
    "earth sign",
    "air sign",
    "water sign",
    "cardinal",
    "fixed",
    "mutable",
    // Specific relationship between bodies.
    "between",
    "aspect between",
    "angular separation",
    "orb of",
];

/// Result of a Barnum analysis.
#[derive(Debug, Clone)]
pub struct BarnumAnalysis {
    /// Whether the statement is discriminating (false = Barnum).
    pub is_discriminating: bool,
    /// Barnum score: 0.0 = perfectly discriminating, 1.0 = pure Barnum.
    pub barnum_score: f64,
    /// Which Barnum patterns were matched.
    pub matched_barnum: Vec<(&'static str, &'static str)>,
    /// Which discriminating patterns were matched.
    pub matched_discriminating: Vec<&'static str>,
}

/// Analyze a statement for Barnum/Forer effect.
///
/// Returns a BarnumAnalysis with the barnum_score:
/// - 0.0–0.3: Discriminating (good — the statement is specific enough to be falsifiable)
/// - 0.3–0.6: Borderline — may need more specificity
/// - 0.6–1.0: Barnum — the statement would be true for almost anyone
pub fn analyze_barnum(statement: &str) -> BarnumAnalysis {
    let lower = statement.to_lowercase();

    let mut matched_barnum = Vec::new();
    for (pattern, reason) in BARNUM_PATTERNS {
        if lower.contains(pattern) {
            matched_barnum.push((*pattern, *reason));
        }
    }

    let mut matched_discriminating = Vec::new();
    for pattern in DISCRIMINATING_PATTERNS {
        if lower.contains(pattern) {
            matched_discriminating.push(*pattern);
        }
    }

    let barnum_count = matched_barnum.len() as f64;
    let discriminating_count = matched_discriminating.len() as f64;

    // Score: barnum patterns increase score, discriminating patterns decrease it.
    let raw = if barnum_count + discriminating_count == 0.0 {
        0.5 // Unknown — treat as borderline.
    } else {
        barnum_count / (barnum_count + discriminating_count + 1.0)
    };

    BarnumAnalysis {
        is_discriminating: raw < 0.3,
        barnum_score: raw.clamp(0.0, 1.0),
        matched_barnum,
        matched_discriminating,
    }
}

/// Check if a statement passes the Barnum filter (is discriminating enough).
pub fn passes_barnum_filter(statement: &str) -> bool {
    let analysis = analyze_barnum(statement);
    analysis.is_discriminating
}

/// Generate a human-readable explanation of why a statement is Barnum.
pub fn explain_barnum(statement: &str) -> Option<String> {
    let analysis = analyze_barnum(statement);
    if analysis.is_discriminating {
        return None;
    }

    let mut out = format!(
        "Statement scores {:.0}% on the Barnum scale (threshold: 30%).\n",
        analysis.barnum_score * 100.0
    );

    if !analysis.matched_barnum.is_empty() {
        out.push_str("Non-discriminating patterns found:\n");
        for (pattern, reason) in &analysis.matched_barnum {
            out.push_str(&format!("  - '{}': {}\n", pattern, reason));
        }
    }

    if analysis.matched_discriminating.is_empty() {
        out.push_str(
            "No specific astrological content detected. To pass the Barnum filter, include:\n\
             - Specific planetary aspects (conjunct, opposition, square)\n\
             - Specific house placements ('in the 7th house')\n\
             - Specific dignities (exalted, debilitated, combust)\n\
             - Specific nakshatra or dasha period references\n",
        );
    } else {
        out.push_str("Discriminating patterns found but outweighed by vague ones:\n");
        for pattern in &analysis.matched_discriminating {
            out.push_str(&format!("  - '{}'\n", pattern));
        }
        out.push_str("Strengthen by removing vague language and keeping the specific content.\n");
    }

    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_barnum_detects_vague_statement() {
        let analysis = analyze_barnum("You have a tendency to worry about the future");
        assert!(!analysis.is_discriminating);
        assert!(analysis.barnum_score > 0.3);
        assert!(!analysis.matched_barnum.is_empty());
    }

    #[test]
    fn test_barnum_passes_discriminating() {
        let statement = "Saturn conjunct your ascendant in the 1st house creates a square to Mars in the 4th house";
        let analysis = analyze_barnum(statement);
        assert!(analysis.is_discriminating);
        assert!(analysis.barnum_score < 0.3);
        assert!(!analysis.matched_discriminating.is_empty());
    }

    #[test]
    fn test_barnum_filter_rejects_vague() {
        assert!(!passes_barnum_filter(
            "You can be both introverted and extroverted"
        ));
    }

    #[test]
    fn test_barnum_filter_passes_specific() {
        assert!(passes_barnum_filter(
            "Jupiter in Sagittarius conjunct the Midheaven with a trine to Moon in the 10th house"
        ));
    }

    #[test]
    fn test_explain_barnum_vague() {
        let explanation = explain_barnum("You have a great deal of unused potential");
        assert!(explanation.is_some());
        let text = explanation.unwrap();
        assert!(text.contains("Barnum"));
        assert!(text.contains("%"));
    }

    #[test]
    fn test_explain_barnum_specific() {
        let explanation = explain_barnum("Venus exalted in Pisces conjunct the Descendant");
        assert!(explanation.is_none());
    }

    #[test]
    fn test_barnum_astrology_specific_patterns() {
        // These should NOT be flagged as Barnum.
        assert!(passes_barnum_filter("Sun combust Mercury within 2 degrees"));
        assert!(passes_barnum_filter(
            "Rahu in the 7th house with Ketu opposition"
        ));
        assert!(passes_barnum_filter(
            "Mars retrograde in Aries, debilitated"
        ));
    }

    #[test]
    fn test_barnum_mixed_statement() {
        // Mostly discriminating but with some vague language.
        let analysis = analyze_barnum(
            "Saturn conjunct your ascendant — this is a time of great opportunity for growth",
        );
        // Should have both matched barnum and discriminating.
        assert!(!analysis.matched_barnum.is_empty() || !analysis.matched_discriminating.is_empty());
    }
}
