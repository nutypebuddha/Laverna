//! # Reverse Routing — query → graha forces → strategy
//!
//! The descent engine maps a query's tokens onto the 9-graha wheel
//! (`Domain` nodes). Reverse routing *inverts* that flow: given the dominant
//! graha forces a query activated, synthesize a context-aware strategic
//! framework — without inventing or speculating. The strategy emerges from the
//! query's own semantic structure, so it is deterministic and reproducible.
//!
//! Strategy mapping audited in `laverna_reverse_routing_strategy.md`: each
//! graha is an archetypal force with a standing strategic principle.

use crate::descent::{SettledToken, SettlingMatrix};
use crate::wheel::Domain;

/// Strategic principle carried by each graha (archetypal force). This is the
/// "upward" leg of reverse routing: force → recommended action framework.
pub fn strategy_principle(graha: Domain) -> &'static str {
    match graha {
        Domain::Surya => "Protect the irreducible core; lead from first principles",
        Domain::Chandra => "Listen, adapt, and respect natural cycles",
        Domain::Mangala => "Build, test, verify, and fail fast",
        Domain::Budha => "Articulate clearly and link ideas precisely",
        Domain::Brihaspati => "Extract principles and scale understanding",
        Domain::Shukra => "Bridge domains and integrate systems harmoniously",
        Domain::Shani => "Honor limits and work within structure",
        Domain::Rahu => "Transcend boundaries and evolve",
        Domain::Ketu => "Let go, detach, and consolidate",
    }
}

/// Default assimilation target for a graha force when a repo has no specific
/// profile. Used by `route --repos` to map an unknown repo's dominant force to
/// where it belongs in the Laverna ecosystem.
pub fn graha_default_target(graha: Domain) -> &'static str {
    match graha {
        Domain::Surya => "Protect core / lead (anchor the reboot)",
        Domain::Chandra => "Listen & adapt (UX / iteration)",
        Domain::Mangala => "Build / test / verify (engineering subsystem)",
        Domain::Budha => "Articulate / link (docs & bridges)",
        Domain::Brihaspati => "Extract principles (validation layer)",
        Domain::Shukra => "Bridge / integrate (cross-domain glue)",
        Domain::Shani => "Honor limits (sandbox / structure)",
        Domain::Rahu => "Transcend boundaries (experimental layer)",
        Domain::Ketu => "Let go / consolidate (prune / archive)",
    }
}

/// A synthesized strategy report for a single query.
#[derive(Debug, Clone)]
pub struct StrategyReport {
    /// The original query text.
    pub query: String,
    /// Dominant graha forces, ranked by hit count (desc), then wheel index.
    /// Tuple is `(graha, hit_count, share_of_total_hits [0,1])`.
    pub ranked: Vec<(Domain, usize, f64)>,
    /// Strongest force (primary strategy).
    pub primary: Option<Domain>,
    /// Second force (secondary / balancing strategy).
    pub secondary: Option<Domain>,
    /// Third force (tertiary strategy), if present.
    pub tertiary: Option<Domain>,
}

impl StrategyReport {
    /// Human-readable strategy report.
    pub fn format(&self) -> String {
        let mut out = String::new();
        out.push_str("═══ Reverse-Routing Strategy ═══\n");
        out.push_str(&format!("query: \"{}\"\n\n", self.query));

        out.push_str("GRAHA FORCES (by dominance):\n");
        if self.ranked.is_empty() {
            out.push_str("  (no graha forces resolved — query is outside the wheel's scope)\n");
        } else {
            for (graha, count, share) in &self.ranked {
                out.push_str(&format!(
                    "  {} {} ({}) — {} — {} hits ({:.0}%) — {}\n",
                    graha.symbol(),
                    graha.name(),
                    graha.full_name(),
                    graha.archetype(),
                    count,
                    share * 100.0,
                    strategy_principle(*graha),
                ));
            }
        }

        out.push_str("\nSYNTHESIZED STRATEGY:\n");
        match self.primary {
            Some(g) => out.push_str(&format!(
                "  PRIMARY:    {} ({}) — {}\n",
                g.archetype(),
                g.name(),
                strategy_principle(g),
            )),
            None => out.push_str("  PRIMARY:    (none)\n"),
        }
        match self.secondary {
            Some(g) => out.push_str(&format!(
                "  SECONDARY:  {} ({}) — {}\n",
                g.archetype(),
                g.name(),
                strategy_principle(g),
            )),
            None => out.push_str("  SECONDARY:  (none)\n"),
        }
        match self.tertiary {
            Some(g) => out.push_str(&format!(
                "  TERTIARY:   {} ({}) — {}\n",
                g.archetype(),
                g.name(),
                strategy_principle(g),
            )),
            None => out.push_str("  TERTIARY:   (none)\n"),
        }

        out.push_str(
            "\nThis strategy emerges from the query's semantic structure — no speculation.\n",
        );
        out
    }
}

/// The single strongest graha force a token resolved to. A token may map to
/// several domains; for strategy synthesis we credit only its dominant force,
/// which keeps the resulting strategy focused rather than smeared across all
/// nine grahas. Falls back to the first listed domain when no graha weight was
/// scored (e.g. unit-built tokens).
pub fn dominant_graha_of(token: &SettledToken) -> Option<Domain> {
    let best = token
        .vedic_classification
        .grahas
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    match best {
        Some((i, &w)) if w > 0.0 => Domain::from_index(i),
        _ => token.domains.first().copied(),
    }
}

/// Pure reverse-routing synthesis: each token votes for its dominant graha
/// force; the forces are ranked and the primary/secondary/tertiary are picked.
/// Deterministic — identical queries yield identical reports.
pub fn synthesize_strategy(query: &str, matrix: &SettlingMatrix) -> StrategyReport {
    let mut counts = [0usize; 9];
    for token in &matrix.tokens {
        if let Some(graha) = dominant_graha_of(token) {
            counts[graha.index()] += 1;
        }
    }
    let total: usize = counts.iter().sum();

    let mut ranked: Vec<(Domain, usize, f64)> = Domain::all()
        .iter()
        .map(|&graha| {
            let count = counts[graha.index()];
            let share = if total > 0 {
                count as f64 / total as f64
            } else {
                0.0
            };
            (graha, count, share)
        })
        .filter(|(_, count, _)| *count > 0)
        .collect();

    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.index().cmp(&b.0.index())));

    let primary = ranked.first().map(|(g, _, _)| *g);
    let secondary = ranked.get(1).map(|(g, _, _)| *g);
    let tertiary = ranked.get(2).map(|(g, _, _)| *g);

    StrategyReport {
        query: query.to_string(),
        ranked,
        primary,
        secondary,
        tertiary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descent::{SettledToken, SettlingMatrix};

    /// Build a matrix from token→domain assignments (no registries needed).
    fn matrix_from(tokens: &[(&str, &[Domain])]) -> SettlingMatrix {
        let settled: Vec<SettledToken> = tokens
            .iter()
            .map(|(text, domains)| {
                let mut t = SettledToken::new(text);
                t.domains = domains.to_vec();
                t
            })
            .collect();
        SettlingMatrix::new(settled)
    }

    #[test]
    fn synthesizes_primary_secondary_from_dominant_grahas() {
        let matrix = matrix_from(&[
            ("architect", &[Domain::Mangala]),
            ("build", &[Domain::Mangala]),
            ("test", &[Domain::Mangala]),
            ("verify", &[Domain::Mangala]),
            ("wisdom", &[Domain::Brihaspati]),
        ]);
        let report = synthesize_strategy("how to architect safely", &matrix);

        assert_eq!(report.primary, Some(Domain::Mangala));
        assert_eq!(report.secondary, Some(Domain::Brihaspati));
        assert_eq!(report.tertiary, None);

        // Mangala = 4/5 = 80%, Brihaspati = 1/5 = 20%.
        let mangala = report
            .ranked
            .iter()
            .find(|(g, _, _)| *g == Domain::Mangala)
            .unwrap();
        assert_eq!(mangala.1, 4);
        assert!((mangala.2 - 0.8).abs() < 1e-9);
    }

    #[test]
    fn empty_matrix_yields_no_forces() {
        let matrix = matrix_from(&[]);
        let report = synthesize_strategy("???", &matrix);
        assert!(report.ranked.is_empty());
        assert!(report.primary.is_none());
        assert!(report.format().contains("no graha forces resolved"));
    }

    #[test]
    fn determinism_same_input_same_report() {
        let matrix = matrix_from(&[
            ("integrate", &[Domain::Shukra]),
            ("bridge", &[Domain::Shukra]),
            ("harmonize", &[Domain::Shukra, Domain::Budha]),
        ]);
        let a = synthesize_strategy("x", &matrix);
        let b = synthesize_strategy("x", &matrix);
        assert_eq!(a.ranked, b.ranked);
        assert_eq!(a.primary, b.primary);
    }

    #[test]
    fn graha_default_target_covers_all_grahas() {
        for graha in Domain::all() {
            assert!(!graha_default_target(graha).is_empty());
        }
    }
}
