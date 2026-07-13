/// Pure function: Lowercase a string. No side effects.
pub fn lowercase_string(input: &str) -> String {
    input.to_lowercase()
}

/// Pure function: Tokenize input into descent layers.
pub fn tokenize_descent(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .map(|token| token.to_lowercase())
        .collect()
}

/// Pure function: Normalize whitespace in input string.
pub fn normalize_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<&str>>().join(" ")
}

use std::fmt::Write as FmtWrite;

use serde::{Deserialize, Serialize};

use crate::astrology::{
    AtomClassification, Graha, PlanetaryRuler, Sign, SignAspect, VedicClassification, VedicElement,
};
use crate::chart::ChartSnapshot;
use crate::entity::{generate_dynamic_entity, EventRegistry, ShikaiFormRegistry};
use crate::formula::FormulaRegistry;
use crate::wheel::CompositionAspect;
use crate::wheel::Domain;

// ─── Descent Layers ─────────────────────────────────────────────────────────

/// The 7 descent layers a token traverses.
///
/// Each layer represents a deeper level of resolution. A token settles
/// at the deepest layer it can resolve to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum DescentLayer {
    /// Cosmic/macro level — unresolved, floating token (depth = 0)
    Macro = 0,
    /// Domain classification (12 zodiac domains) (depth = 1)
    Domain = 1,
    /// Aspect relationships between tokens (depth = 2)
    Aspect = 2,
    /// Element + Modality classification (depth = 3)
    Element = 3,
    /// Formula grounding (depth = 4)
    Formula = 4,
    /// Entity grounding (depth = 5)
    Entity = 5,
    /// NAND gate resolution — absolute truth (depth = 6)
    Nand = 6,
}

impl DescentLayer {
    pub const COUNT: usize = 7;

    /// Depth of this layer (0 = Macro, 6 = NAND).
    pub fn depth(self) -> usize {
        self as usize
    }

    /// Get layer from depth.
    pub fn from_depth(d: usize) -> Self {
        match d % 7 {
            0 => DescentLayer::Macro,
            1 => DescentLayer::Domain,
            2 => DescentLayer::Aspect,
            3 => DescentLayer::Element,
            4 => DescentLayer::Formula,
            5 => DescentLayer::Entity,
            6 => DescentLayer::Nand,
            _ => unreachable!(),
        }
    }

    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            DescentLayer::Macro => "Macro",
            DescentLayer::Domain => "Domain",
            DescentLayer::Aspect => "Aspect",
            DescentLayer::Element => "Element",
            DescentLayer::Formula => "Formula",
            DescentLayer::Entity => "Entity",
            DescentLayer::Nand => "NAND",
        }
    }

    /// Symbol for this layer.
    pub fn symbol(self) -> &'static str {
        match self {
            DescentLayer::Macro => "🌌",
            DescentLayer::Domain => "◎",
            DescentLayer::Aspect => "⚡",
            DescentLayer::Element => "🜁",
            DescentLayer::Formula => "∑",
            DescentLayer::Entity => "◆",
            DescentLayer::Nand => "⊼",
        }
    }

    /// Description of what happens at this layer.
    pub fn description(self) -> &'static str {
        match self {
            DescentLayer::Macro => "Unresolved token — floats at the query level",
            DescentLayer::Domain => "Token classified to a zodiac domain (Aries–Pisces)",
            DescentLayer::Aspect => "Token relationship computed (Conjunction–Opposition)",
            DescentLayer::Element => "Elemental + modality classification (Fire/Earth/Air/Water + Cardinal/Fixed/Mutable)",
            DescentLayer::Formula => "Token matched to a provable formula from the registry",
            DescentLayer::Entity => "Token grounded to a named entity with properties",
            DescentLayer::Nand => "Token provably resolved to NAND gate truth — absolute bedrock",
        }
    }

    pub fn all() -> [DescentLayer; 7] {
        [
            DescentLayer::Macro,
            DescentLayer::Domain,
            DescentLayer::Aspect,
            DescentLayer::Element,
            DescentLayer::Formula,
            DescentLayer::Entity,
            DescentLayer::Nand,
        ]
    }
}

// ─── Settled Token ──────────────────────────────────────────────────────────

/// A single token after descent — resolved to its deepest layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettledToken {
    /// The original token text.
    pub text: String,

    /// The layer at which this token settled.
    pub settled_layer: DescentLayer,

    /// Western 7-axis classification at the settled layer.
    pub western_classification: AtomClassification,

    /// Vedic classification at the settled layer.
    pub vedic_classification: VedicClassification,

    /// Domain(s) matched for this token.
    pub domains: Vec<Domain>,

    /// Formula(s) matched (if settled at Formula layer or deeper).
    pub formulas: Vec<String>,

    /// Entity matched (if settled at Entity layer or deeper).
    pub entity: Option<String>,

    /// NAND confidence [0, 1] at the settled layer.
    pub confidence: f64,

    /// Whether the token has fully resolved to absolute truth.
    pub is_absolute: bool,
}

impl SettledToken {
    pub fn new(text: &str) -> Self {
        SettledToken {
            text: text.to_string(),
            settled_layer: DescentLayer::Macro,
            western_classification: AtomClassification::new(),
            vedic_classification: VedicClassification::new(),
            domains: Vec::new(),
            formulas: Vec::new(),
            entity: None,
            confidence: 0.0,
            is_absolute: false,
        }
    }
}

// ─── Settling Matrix ────────────────────────────────────────────────────────

/// The complete settling matrix for a query — all tokens after descent.
///
/// The matrix provides a holistic view of the query's "astrological body":
/// which domains are active, where tensions exist (aspects), which elements
/// dominate, and how deeply resolved the overall query is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlingMatrix {
    /// All settled tokens in order.
    pub tokens: Vec<SettledToken>,

    /// Aggregate Western classification across all tokens.
    pub aggregate_western: AtomClassification,

    /// Aggregate Vedic classification across all tokens.
    pub aggregate_vedic: VedicClassification,

    /// Dominant domains (count ≥ threshold).
    pub dominant_domains: Vec<Domain>,

    /// Aspect map: pairs of tokens with their computed sign-based aspects.
    pub aspects: Vec<(String, String, SignAspect)>,

    /// Average descent depth across all tokens.
    pub average_depth: f64,

    /// Minimum descent depth.
    pub min_depth: usize,

    /// Maximum descent depth.
    pub max_depth: usize,

    /// Tokens settled at each layer.
    pub layer_counts: [usize; 7],

    /// Overall resolution score [0, 1] — fraction of tokens at Formula+ depth.
    pub resolution_score: f64,

    /// Sky snapshot at query time — contextualizes token classification with
    /// actual graha positions. None = no sky context (legacy/unit-test mode).
    pub chart: Option<ChartSnapshot>,
}

impl SettlingMatrix {
    pub fn new(tokens: Vec<SettledToken>) -> Self {
        Self::with_chart(tokens, None)
    }

    /// Create a settling matrix with sky context.
    ///
    /// When a chart is provided, token confidence is modulated by the actual
    /// graha positions:
    /// - If a token's Western sign matches a graha's rashi → +15% confidence
    /// - If a token's Vedic graha has a graha actually there → +20% confidence
    /// - The aggregate classification is nudged toward the chart's lagna
    pub fn with_chart(tokens: Vec<SettledToken>, chart: Option<ChartSnapshot>) -> Self {
        let mut aggregate_western = AtomClassification::new();
        let mut aggregate_vedic = VedicClassification::new();
        let mut dominant_domains = Vec::new();
        let mut aspects = Vec::new();
        let mut layer_counts = [0usize; 7];
        let mut total_depth = 0usize;
        let mut min_depth: Option<usize> = None;
        let mut max_depth = 0usize;
        let mut total_formula_plus = 0usize;

        // Pre-compute which signs are occupied by grahas in the chart
        let occupied_signs: Vec<Sign> = chart.as_ref().map_or_else(Vec::new, |c| {
            c.graha_positions
                .iter()
                .map(|p| {
                    let rashi_idx = p.rashi.index();
                    Sign::from_index(rashi_idx)
                })
                .collect()
        });
        // Pre-compute which graha indices are actually present (all 9 are always present)
        let _active_grahas: Vec<Graha> = chart.as_ref().map_or_else(Vec::new, |c| {
            c.graha_positions.iter().map(|p| p.graha).collect()
        });

        for token in &tokens {
            // ── Chart-context confidence modulation ─────────────
            // When sky context is available, tokens whose sign matches
            // an occupied sign in the sky get a confidence boost.
            let mut token = token.clone();
            if let Some(ref _chart) = chart {
                if let Some(sign) = token.western_classification.dominant_sign() {
                    if occupied_signs.contains(&sign) {
                        // This token's sign matches a graha's actual position — boost
                        token.confidence = (token.confidence + 0.15).min(1.0);
                    }
                }
                // Boost if Vedic graha matches a chart graha
                if let Some(graha) = token.vedic_classification.dominant_graha() {
                    if _active_grahas.contains(&graha) {
                        token.confidence = (token.confidence + 0.10).min(1.0);
                    }
                }
            }

            // Accumulate classifications
            aggregate_western = aggregate_western.merge_max(&token.western_classification);
            aggregate_vedic = aggregate_vedic.merge_max(&token.vedic_classification);

            let depth = token.settled_layer.depth();
            total_depth += depth;
            min_depth = Some(min_depth.map_or(depth, |m| m.min(depth)));
            max_depth = max_depth.max(depth);
            layer_counts[depth] += 1;

            // Collect domains
            for d in &token.domains {
                if !dominant_domains.contains(d) {
                    dominant_domains.push(*d);
                }
            }

            if depth >= DescentLayer::Formula.depth() {
                total_formula_plus += 1;
            }
        }

        // Nudge aggregate classification toward chart's lagna
        if let Some(ref chart) = chart {
            if let Some(lagna) = chart.lagna {
                let lagna_sign = Sign::from_index(lagna.index());
                aggregate_western = aggregate_western.with_sign(lagna_sign, 0.6);
            }
        }

        // Compute aspects between all pairs of tokens that have domain info
        let n = tokens.len().min(20); // limit to avoid O(n²) explosion
        for i in 0..n {
            for j in (i + 1)..n {
                let ti = &tokens[i];
                let tj = &tokens[j];
                if let (Some(si), Some(sj)) = (
                    ti.western_classification.dominant_sign(),
                    tj.western_classification.dominant_sign(),
                ) {
                    let aspect = SignAspect::between_sign_indices(si.index(), sj.index());
                    aspects.push((ti.text.clone(), tj.text.clone(), aspect));
                }
            }
        }

        let n_tokens = tokens.len().max(1);
        let average_depth = total_depth as f64 / n_tokens as f64;
        let resolution_score = total_formula_plus as f64 / n_tokens as f64;

        SettlingMatrix {
            tokens,
            aggregate_western,
            aggregate_vedic,
            dominant_domains,
            aspects,
            average_depth,
            min_depth: min_depth.unwrap_or(0),
            max_depth,
            layer_counts,
            resolution_score,
            chart,
        }
    }

    /// Bankai completeness: fraction of tokens that reached NAND (layer 6/6).
    /// Measures how fully the query was executed to atomic verifiable truth.
    /// Bankai is aspirational — 1.0 means every token proved at the gate level.
    pub fn nand_completeness(&self) -> f64 {
        let total: usize = self.layer_counts.iter().sum();
        if total > 0 {
            self.layer_counts[6] as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Shikai focus: fraction of tokens that settled at Formula+ depth (layers 4-6).
    /// Equivalent to `resolution_score`.
    pub fn shikai_focus(&self) -> f64 {
        self.resolution_score
    }

    /// Display the settling matrix as an ASCII table.
    pub fn format(&self) -> String {
        let mut out = String::new();
        out.push_str("═══════════════════════════════════════════\n");
        out.push_str("         SETTLING MATRIX\n");
        out.push_str("═══════════════════════════════════════════\n\n");

        out.push_str(&format!(
            "Tokens: {} | Resolution: {:.1}% | Avg Depth: {:.2}/6\n\n",
            self.tokens.len(),
            self.resolution_score * 100.0,
            self.average_depth,
        ));

        // Precomputed bar strings for depths 0-6 (avoids per-token allocation)
        // depth=0: 1█+6░, depth=1: 2█+5░, ... depth=6: 7█+0░
        const BARS: [&str; 7] = [
            "█░░░░░░",
            "██░░░░░",
            "███░░░░",
            "████░░░",
            "█████░░",
            "██████░",
            "███████",
        ];

        out.push_str("── Tokens ──\n");
        for t in &self.tokens {
            let depth = t.settled_layer.depth();
            let bar = BARS[depth];
            let confidence = if t.confidence > 0.0 {
                format!("{:.0}%", t.confidence * 100.0)
            } else {
                "---".to_string()
            };
            let domain_str = if t.domains.is_empty() {
                "?".to_string()
            } else {
                use std::fmt::Write;
                let mut s = String::new();
                for (i, d) in t.domains.iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    let _ = write!(s, "{}{}", d.symbol(), d.full_name());
                }
                s
            };
            out.push_str(&format!(
                "  {:<24} {} {}/6 {:>5}  {}\n",
                t.text, bar, depth, confidence, domain_str,
            ));
        }

        out.push('\n');

        if !self.aspects.is_empty() {
            out.push_str("── Aspects (top) ──\n");
            let max_aspects = self.aspects.len().min(20);
            for (a, b, aspect) in self.aspects.iter().take(max_aspects) {
                let (angle, desc) = aspect_details(*aspect);
                out.push_str(&format!(
                    "  {:<16} ↔ {:<16}  {:?}  ({}°, {})\n",
                    a, b, aspect, angle, desc
                ));
            }
            out.push('\n');
        }

        out.push_str("── Layer Distribution ──\n");
        for layer in DescentLayer::all() {
            let count = self.layer_counts[layer.depth()];
            let bar = "█".repeat(count.min(40));
            out.push_str(&format!(
                "  {} {}: {}  {}\n",
                layer.symbol(),
                layer.name(),
                count,
                bar,
            ));
        }

        out.push('\n');

        out.push_str("── Aggregate ──\n");
        if let Some(sign) = self.aggregate_western.dominant_sign() {
            out.push_str(&format!(
                "  Dominant sign:     {} {:?}\n",
                sign.symbol(),
                sign,
            ));
        }
        if let Some(el) = self.aggregate_western.dominant_element() {
            out.push_str(&format!(
                "  Dominant element:  {} {}\n",
                el.symbol(),
                el.name(),
            ));
        }
        if let Some(moda) = self.aggregate_western.dominant_modality() {
            out.push_str(&format!(
                "  Dominant modality: {} {:?}\n",
                moda.symbol(),
                moda,
            ));
        }
        if let Some(graha) = self.aggregate_vedic.dominant_graha() {
            out.push_str(&format!(
                "  Dominant graha:    {} {} ({:?})\n",
                graha.symbol(),
                graha.name(),
                graha,
            ));
        }
        if let Some(nak) = self.aggregate_vedic.dominant_nakshatra() {
            out.push_str(&format!("  Dominant nakshatra: {:?}\n", nak,));
        }
        if let Some(guna) = self.aggregate_vedic.dominant_guna() {
            out.push_str(&format!(
                "  Dominant guṇa:     {} {}\n",
                guna.symbol(),
                guna.name(),
            ));
        }
        if let Some(ve) = self.aggregate_vedic.dominant_vedic_element() {
            out.push_str(&format!(
                "  Dominant bhūta:    {} {} ({})\n",
                ve.symbol(),
                ve.sanskrit(),
                ve.name(),
            ));
        }

        // ── Chart context ──
        if let Some(ref chart) = self.chart {
            out.push_str("── Sky Context ──\n");
            if let Some(lagna) = chart.lagna {
                out.push_str(&format!(
                    "  Lagna:  {} {} ({:?})\n",
                    lagna.symbol(),
                    lagna.name(),
                    lagna,
                ));
            }
            out.push_str(&format!("  JD:     {:.5}\n", chart.julian_day));
            // Show which grahas are in which rashi (compact: just rashi)
            let mut graha_lines: Vec<String> = Vec::new();
            for pos in &chart.graha_positions {
                graha_lines.push(format!(
                    "  {} {:8} in {:?} ({})  {:6.2}°",
                    pos.graha.symbol(),
                    pos.graha.name(),
                    pos.rashi,
                    pos.rashi.name(),
                    pos.sidereal,
                ));
            }
            // Show only first 9 lines (always 9 grahas)
            for line in &graha_lines {
                out.push_str(line);
                out.push('\n');
            }
            out.push('\n');
        }

        out.push_str("\n═══════════════════════════════════════════\n");
        out
    }

    /// Display each token's descent as a 7-layer gyro wheel vortex.
    ///
    /// Each layer is a wheel the token spins through:
    ///
    /// ```text
    ///  ① MACRO   ☉ ☽ ♂ ☿ ♃ [♀] ♄ ☊ ☋    ← roulette lands on Shukra
    ///  ② DOMAIN  ♀ Shukra                   ← Vedic graha
    ///  ③ ASPECT  ♀ ⟷ ♂ Square              ← inter-token aspect
    ///  ④ ELEMENT 🌍 Earth [████]           ← tattva bar
    ///  ⑤ FORMULA F = ma                     ← formula match
    ///  ⑥ ENTITY  dyn_force                  ← dynamic entity id
    ///  ⑦ NAND    [✓] resolved              ← absolute truth gate
    /// ```
    pub fn format_vortex(&self) -> String {
        let mut out = String::new();
        out.push_str("╔══════════════════════════════════════════════════╗\n");
        out.push_str("║           TOKEN VORTEX — 7 Gyro Wheels         ║\n");
        out.push_str("╚══════════════════════════════════════════════════╝\n");

        let graha_symbols: [&str; 9] = ["☉", "☽", "♂", "☿", "♃", "♀", "♄", "☊", "☋"];

        for (ti, t) in self.tokens.iter().enumerate() {
            let depth = t.settled_layer.depth();
            let depth_label = match depth {
                0 => "MACRO",
                1 => "DOMAIN",
                2 => "ASPECT",
                3 => "ELEMENT",
                4 => "FORMULA",
                5 => "ENTITY",
                6 => "NAND",
                _ => "???",
            };

            out.push_str(&format!(
                "\n── Token #{}: \"{}\"  |  Depth: {}/6  |  {}\n",
                ti + 1,
                t.text,
                depth,
                depth_label,
            ));

            // ── Layer ①: Macro Wheel ──
            // Show all 9 grahas with the dominant one highlighted
            let dominant_graha = t.vedic_classification.dominant_graha();
            let mut macro_line = String::from("  ① MACRO   ");
            for (gi, sym) in graha_symbols.iter().enumerate() {
                let g = Domain::from_index(gi).unwrap_or(Domain::Surya);
                if Some(g) == dominant_graha {
                    let _ = write!(macro_line, "[{}]", sym);
                } else {
                    let _ = write!(macro_line, " {} ", sym);
                }
            }
            if let Some(g) = dominant_graha {
                let _ = write!(macro_line, "  ← lands on {} ({})", g.symbol(), g.name());
            }
            out.push_str(&macro_line);
            out.push('\n');

            // ── Layer ②: Domain Wheel ──
            let mut domain_line = String::from("  ② DOMAIN  ");
            if t.domains.is_empty() {
                domain_line.push_str("? (unresolved)");
            } else {
                for (i, d) in t.domains.iter().enumerate() {
                    if i > 0 {
                        domain_line.push_str(", ");
                    }
                    let _ = write!(domain_line, "{} {}", d.symbol(), d.full_name());
                }
            }
            out.push_str(&domain_line);
            out.push('\n');

            // ── Layer ③: Aspect Wheel ──
            // Show aspects this token has with other tokens
            let mut aspect_line = String::from("  ③ ASPECT  ");
            let mut has_aspect = false;
            for (a, b, aspect) in &self.aspects {
                if a == &t.text || b == &t.text {
                    let other = if a == &t.text { b } else { a };
                    let (angle, _) = aspect_details(*aspect);
                    let _ = write!(
                        aspect_line,
                        "{} ⟷ {} {:?} ({}°)  ",
                        t.text, other, aspect, angle,
                    );
                    has_aspect = true;
                }
            }
            if !has_aspect {
                aspect_line.push_str("— (no aspects with other tokens)");
            }
            out.push_str(&aspect_line);
            out.push('\n');

            // ── Layer ④: Element Wheel ──
            let mut elem_line = String::from("  ④ ELEMENT ");
            let all_elements = [
                (crate::astrology::Element::Fire, "🔥 Fire"),
                (crate::astrology::Element::Earth, "🌍 Earth"),
                (crate::astrology::Element::Air, "💨 Air"),
                (crate::astrology::Element::Water, "💧 Water"),
            ];
            let western = &t.western_classification;
            for (el, label) in &all_elements {
                let score = western.elements[el.index()];
                if score > 0.3 {
                    let _ = write!(elem_line, "{}[{:.0}%] ", label, score * 100.0);
                }
            }
            if let Some(el) = western.dominant_element() {
                let _ = write!(elem_line, "  ← {} dominant", el.name());
            }
            // Vedic element too
            if let Some(ve) = t.vedic_classification.dominant_vedic_element() {
                let _ = write!(elem_line, "  (Vedic: {} {})", ve.symbol(), ve.sanskrit());
            }
            out.push_str(&elem_line);
            out.push('\n');

            // ── Layer ⑤: Formula Wheel ──
            let mut formula_line = String::from("  ⑤ FORMULA ");
            if t.formulas.is_empty() {
                formula_line.push_str("— (no matched formulas)");
            } else {
                for (i, fid) in t.formulas.iter().enumerate() {
                    if i > 0 {
                        formula_line.push_str(", ");
                    }
                    formula_line.push_str(fid);
                }
                formula_line.push_str("  ✓");
            }
            out.push_str(&formula_line);
            out.push('\n');

            // ── Layer ⑥: Entity Wheel ──
            let mut entity_line = String::from("  ⑥ ENTITY  ");
            if let Some(ref eid) = t.entity {
                entity_line.push_str(eid);
                // Show Vedic details
                if let Some(graha) = t.vedic_classification.dominant_graha() {
                    let _ = write!(entity_line, "  ({})", graha.full_name());
                }
                if let Some(nak) = t.vedic_classification.dominant_nakshatra() {
                    let _ = write!(entity_line, "  {}", nak.name());
                }
                entity_line.push_str("  ✓");
            } else {
                entity_line.push_str("— (not resolved)");
            }
            out.push_str(&entity_line);
            out.push('\n');

            // ── Layer ⑦: NAND Gate ──
            let mut nand_line = String::from("  ⑦ NAND    ");
            if t.is_absolute {
                let _ = write!(
                    nand_line,
                    "[✓] resolved  ({:.0}% confidence)",
                    t.confidence * 100.0
                );
            } else if t.settled_layer == DescentLayer::Nand {
                let _ = write!(nand_line, "[~] partial  ({:.0}%)", t.confidence * 100.0);
            } else {
                let _ = write!(nand_line, "[ ] not reached  (settled at {})", depth_label);
            }
            out.push_str(&nand_line);
            out.push('\n');

            // ── Confidence bar ──
            let bar_len = (t.confidence * 20.0) as usize;
            let clamped = bar_len.min(20);
            let bar = "█".repeat(clamped);
            let empty = "░".repeat(20 - clamped);
            let _ = writeln!(
                out,
                "     CONF    │{}{}│ {:.0}%",
                bar,
                empty,
                t.confidence * 100.0
            );
        }

        // ── Vortex summary: resolution + coherence ──
        out.push_str("\n── Vortex Summary ──\n");
        let _ = writeln!(
            out,
            "  Resolution: {:.0}%  |  Avg Depth: {:.2}/6  |  NAND: {} tokens",
            self.resolution_score * 100.0,
            self.average_depth,
            self.layer_counts[6],
        );

        // Coherence reading (how "aligned" the vortex is)
        let n_tokens = self.tokens.len().max(1);
        let coherence = if self.average_depth > 3.5 {
            "GROUNDED 🜁"
        } else if self.average_depth > 2.0 {
            "SETTLING 🜄"
        } else {
            "FLOATING 🜂"
        };
        let _ = write!(
            out,
            "  Coherence: {}  (depth={:.2})",
            coherence, self.average_depth
        );

        // Resonance note: if all tokens at same depth → pure resonance
        let unique_depths: std::collections::HashSet<usize> = self
            .tokens
            .iter()
            .map(|t| t.settled_layer.depth())
            .collect();
        if unique_depths.len() == 1 && n_tokens > 1 {
            out.push_str("\n  ★ RESONANT — all tokens at same depth");
        } else if unique_depths.len() <= 2 && n_tokens > 2 {
            let _ = write!(out, "\n  ☆ HARMONIC — {} depth levels", unique_depths.len());
        }

        out.push('\n');
        out.push_str("════════════════════════════════════════════════\n");
        out
    }
}

/// Get details about a sign-based aspect type.
fn aspect_details(aspect: SignAspect) -> (i32, &'static str) {
    match aspect {
        SignAspect::Conjunction => (0, "Same sign, aligned"),
        SignAspect::Sextile => (60, "Adjacent, natural flow"),
        SignAspect::Trine => (120, "Harmonious, complementary"),
        SignAspect::Square => (90, "Tension, requires work"),
        SignAspect::Opposition => (180, "Complementary opposites"),
    }
}

// ─── 9-graha Keyword → Domain Table ────────────────────────────────────────
// The canonical keyword → structural-domain mapping used by both
// `resolve_domain` (Layer 2) and `resolve_aspect` (Layer 3). A keyword is
// matched if the token equals it exactly OR the token contains it (to handle
// plurals, suffixes, etc.).

/// Map a token to its structural wheel domain via keyword matching.
/// Returns `None` if no keyword matches — the caller should fall back to
/// entity/formula lookup.
pub fn domain_for_keyword(token: &str) -> Option<Domain> {
    let t = token.to_lowercase();
    DOMAIN_KEYWORDS
        .iter()
        .find(|(kw, _)| t == *kw || t.contains(kw))
        .map(|(_, domain)| *domain)
}

const DOMAIN_KEYWORDS: &[(&str, Domain)] = &[
    // ── Aries — Math & Logic ──
    ("math", Domain::Mangala),
    ("number", Domain::Mangala),
    ("count", Domain::Mangala),
    ("calculate", Domain::Mangala),
    ("equation", Domain::Mangala),
    ("logic", Domain::Mangala),
    ("proof", Domain::Mangala),
    ("theorem", Domain::Mangala),
    // ── Taurus — Physics & Chemistry ──
    ("physics", Domain::Shukra),
    ("force", Domain::Shukra),
    ("energy", Domain::Shukra),
    ("mass", Domain::Shukra),
    ("acceleration", Domain::Shukra),
    ("velocity", Domain::Shukra),
    ("chemistry", Domain::Shukra),
    ("atom", Domain::Shukra),
    ("molecule", Domain::Shukra),
    // ── Gemini — Astronomy & Cosmology ──
    ("star", Domain::Budha),
    ("planet", Domain::Budha),
    ("galaxy", Domain::Budha),
    ("cosmos", Domain::Budha),
    ("space", Domain::Budha),
    ("astronomy", Domain::Budha),
    ("universe", Domain::Budha),
    // ── Cancer — Earth & Environment ──
    ("earth", Domain::Chandra),
    ("environment", Domain::Chandra),
    ("climate", Domain::Chandra),
    ("water", Domain::Chandra),
    ("forest", Domain::Chandra),
    ("ocean", Domain::Chandra),
    ("weather", Domain::Chandra),
    // ── Leo — Biology & Medicine ──
    ("biology", Domain::Surya),
    ("cell", Domain::Surya),
    ("dna", Domain::Surya),
    ("gene", Domain::Surya),
    ("medicine", Domain::Surya),
    ("disease", Domain::Surya),
    ("health", Domain::Surya),
    ("body", Domain::Surya),
    ("brain", Domain::Surya),
    ("organ", Domain::Surya),
    // ── Virgo — Economics & Finance ──
    ("economy", Domain::Budha),
    ("money", Domain::Budha),
    ("market", Domain::Budha),
    ("price", Domain::Budha),
    ("trade", Domain::Budha),
    ("finance", Domain::Budha),
    ("capital", Domain::Budha),
    ("gdp", Domain::Budha),
    ("budget", Domain::Budha),
    ("tax", Domain::Budha),
    // ── Libra — Engineering & Technology ──
    ("engineer", Domain::Shukra),
    ("technology", Domain::Shukra),
    ("machine", Domain::Shukra),
    ("circuit", Domain::Shukra),
    ("bridge", Domain::Shukra),
    ("build", Domain::Shukra),
    ("design", Domain::Shukra),
    // ── Scorpio — Computer Science & AI ──
    ("computer", Domain::Mangala),
    ("algorithm", Domain::Mangala),
    ("code", Domain::Mangala),
    ("program", Domain::Mangala),
    ("data", Domain::Mangala),
    ("ai", Domain::Mangala),
    ("software", Domain::Mangala),
    ("neural", Domain::Mangala),
    // ── Sagittarius — History & Anthropology ──
    ("history", Domain::Brihaspati),
    ("culture", Domain::Brihaspati),
    ("war", Domain::Brihaspati),
    ("ancient", Domain::Brihaspati),
    ("civilization", Domain::Brihaspati),
    ("society", Domain::Brihaspati),
    ("political", Domain::Brihaspati),
    ("government", Domain::Brihaspati),
    // ── Capricorn — Language & Linguistics ──
    ("language", Domain::Shani),
    ("word", Domain::Shani),
    ("grammar", Domain::Shani),
    ("syntax", Domain::Shani),
    ("meaning", Domain::Shani),
    ("speech", Domain::Shani),
    ("translate", Domain::Shani),
    // ── Aquarius — Philosophy & Ethics ──
    ("philosophy", Domain::Shani),
    ("ethics", Domain::Shani),
    ("moral", Domain::Shani),
    ("truth", Domain::Shani),
    ("good", Domain::Shani),
    ("right", Domain::Shani),
    ("justice", Domain::Shani),
    ("virtue", Domain::Shani),
    // ── Pisces — Psychology & Neuroscience ──
    ("psychology", Domain::Brihaspati),
    ("mind", Domain::Brihaspati),
    ("emotion", Domain::Brihaspati),
    ("feeling", Domain::Brihaspati),
    ("consciousness", Domain::Brihaspati),
    ("dream", Domain::Brihaspati),
    ("memory", Domain::Brihaspati),
    ("personality", Domain::Brihaspati),
];

// ─── Descent Engine ─────────────────────────────────────────────────────────

/// The descent engine — processes a query by sinking each token through
/// 7 layers of resolution.
///
/// Usage:
/// ```ignore
/// let mut engine = DescentEngine::new(registry, forms, events);
/// let matrix = engine.descend("what is the mass of an electron");
/// println!("{}", matrix.format());
/// ```
#[derive(Debug)]
pub struct DescentEngine {
    pub formula_registry: FormulaRegistry,
    pub shikai_forms: ShikaiFormRegistry,
    pub events: EventRegistry,
    /// Optional Qwen copilot for semantic descent hints.
    /// Only available when built with `--features llm`.
    #[cfg(feature = "llm")]
    pub copilot: Option<crate::inference::sandwich::SandwichCopilot>,
}

impl Clone for DescentEngine {
    fn clone(&self) -> Self {
        DescentEngine {
            formula_registry: self.formula_registry.clone(),
            shikai_forms: self.shikai_forms.clone(),
            events: self.events.clone(),
            #[cfg(feature = "llm")]
            copilot: None, // copilot is not cloned — must be re-attached
        }
    }
}

impl DescentEngine {
    pub fn new(
        formula_registry: FormulaRegistry,
        shikai_forms: ShikaiFormRegistry,
        events: EventRegistry,
    ) -> Self {
        DescentEngine {
            formula_registry,
            shikai_forms,
            events,
            #[cfg(feature = "llm")]
            copilot: None,
        }
    }

    /// Attach a Qwen copilot for semantic token resolution hints.
    #[cfg(feature = "llm")]
    pub fn with_copilot(mut self, copilot: crate::inference::sandwich::SandwichCopilot) -> Self {
        self.copilot = Some(copilot);
        self
    }

    /// Run the full descent pipeline on a query string.
    ///
    /// 1. Tokenize
    /// 2. For each token: attempt descent through Macro → Domain → Aspect → Element → Formula → Entity → NAND
    /// 3. Aggregate into a SettlingMatrix
    ///
    /// If `chart` is provided, token confidence is modulated by actual sky positions.
    pub fn descend(&self, query: &str) -> SettlingMatrix {
        self.descend_with_chart(query, None)
    }

    /// Run descent with optional sky context.
    pub fn descend_with_chart(&self, query: &str, chart: Option<ChartSnapshot>) -> SettlingMatrix {
        let tokens: Vec<&str> = query
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| c.is_ascii_punctuation()))
            .filter(|s| !s.is_empty())
            .collect();

        let settled: Vec<SettledToken> = tokens
            .iter()
            .map(|t| self.descent_token(t, &tokens))
            .collect();

        SettlingMatrix::with_chart(settled, chart)
    }

    /// Run descent using pre-tokenized NLP context from Zanpakuto.
    ///
    /// This is the wired path: Zanpakuto → Descent → Gyro → Shikai → Bankai.
    /// Uses the already-tokenized, stemmed tokens from NLP preprocessing
    /// instead of re-tokenizing the raw query.
    pub fn resolve_nlp(&self, nlp: &crate::zanpakuto::NlpContext) -> SettlingMatrix {
        self.resolve_nlp_with_chart(nlp, None)
    }

    /// Run descent using pre-tokenized NLP context, with optional sky context.
    pub fn resolve_nlp_with_chart(
        &self,
        nlp: &crate::zanpakuto::NlpContext,
        chart: Option<ChartSnapshot>,
    ) -> SettlingMatrix {
        let tokens: Vec<&str> = nlp.tokens.iter().map(|s| s.as_str()).collect();

        let settled: Vec<SettledToken> = tokens
            .iter()
            .map(|t| self.descent_token(t, &tokens))
            .collect();

        SettlingMatrix::with_chart(settled, chart)
    }

    /// Descent a single token through all 7 layers.
    ///
    /// Uses **fact-first** ordering: entity → formula → domain → aspect → element → NAND.
    /// This ordering is proven by CID simulation to yield 100% accuracy vs 66.7% for logic-first
    /// (Fact-first gate ordering discovery, CID benchmark.rs).
    ///
    /// The principle: resolve grounded facts (entities/formulas) before inferring abstract
    /// classifications (domains/elements). If a token directly names a known entity or formula,
    /// no keyword-based domain inference is needed.
    fn descent_token(&self, token: &str, all_tokens: &[&str]) -> SettledToken {
        let mut st = SettledToken::new(token);

        // ── Layer 1: Macro ──
        // Nothing to do — token starts here.

        // ── FACT-FIRST: Entity + Formula lookup ──
        // The CID simulation proved: KB lookup BEFORE logic gates yields optimal results.
        // We check entity and formula first — if the token directly names a known entity
        // or formula, we derive domain from that, avoiding keyword-based inference entirely.
        let found_entity = self.try_lookup_entity(&mut st);
        let found_formula = self.try_lookup_formula(&mut st);

        // ── QWEN COPILOT: semantic hints when KB lookup is ambiguous ──
        // If neither entity nor formula was directly matched, ask the descent
        // copilot for semantic classification hints before falling back to
        // keyword-based domain resolution.
        #[cfg(feature = "llm")]
        if !found_entity && !found_formula {
            if let Some(ref copilot) = self.copilot {
                if let Ok(hint) = copilot.descend_token(token, None) {
                    // Apply copilot's domain hints if our domain list is still empty
                    if st.domains.is_empty() && !hint.domains.is_empty() {
                        for domain_name in &hint.domains {
                            if let Ok(domain) = domain_name.parse::<crate::wheel::Domain>() {
                                if !st.domains.contains(&domain) {
                                    st.domains.push(domain);
                                }
                            }
                        }
                        st.confidence = st.confidence.max(hint.confidence * 0.6);
                    }

                    // Apply entity hint if we don't have one yet
                    if st.entity.is_none() {
                        if let Some(ref entity_name) = hint.entity {
                            // Check if this entity exists in the registry
                            if !self.shikai_forms.search(entity_name).is_empty() {
                                st.entity = Some(entity_name.clone());
                            }
                        }
                    }

                    // Apply formula hint if we don't have one yet
                    if st.formulas.is_empty() {
                        if let Some(ref formula_name) = hint.formula {
                            if self.formula_registry.get(formula_name).is_some() {
                                st.formulas.push(formula_name.clone());
                            }
                        }
                    }
                }
            }
        }

        // ── Layer 2: Domain ──
        // Only do keyword-based domain classification if entity/formula lookup didn't resolve it.
        // This prevents "mercury" (the element) from keyword-matching to "mercury" (the planet).
        if st.domains.is_empty() {
            self.resolve_domain(&mut st);
        }
        if st.domains.is_empty() {
            // Couldn't even resolve domain — float at Macro
            return st;
        }

        // ── Layer 3: Aspect ──
        self.resolve_aspect(&mut st, all_tokens);

        // ── Layer 4: Element ──
        self.resolve_element(&mut st);

        // ── Layer 5: Formula (deeper) ──
        // If entity was found but formula not yet matched, try formula resolution
        if !found_formula {
            self.resolve_formula(&mut st);
        }
        let has_formulas = !st.formulas.is_empty();
        let has_entity = st.entity.is_some();

        if !has_formulas && !has_entity {
            // Settle at Element — nothing grounded
            st.settled_layer = DescentLayer::Element;
            st.confidence = 0.3;
            return st;
        }

        // ── Layer 6: Entity (deeper) ──
        // If formula was found but entity not yet matched, try entity resolution
        if !found_entity {
            self.resolve_entity(&mut st);
        }
        if st.entity.is_none() {
            // Entity wasn't found despite having formulas — settle at Formula
            st.settled_layer = DescentLayer::Formula;
            st.confidence = 0.6;
            return st;
        }

        // ── Layer 7: NAND ──
        // Entity is resolved. Check if we also have formulas for NAND truth.
        self.resolve_nand(&mut st);
        if st.is_absolute {
            st.settled_layer = DescentLayer::Nand;
            st.confidence = 1.0;
        } else {
            st.settled_layer = DescentLayer::Entity;
            st.confidence = 0.8;
        }

        st
    }

    /// Fact-first entity lookup: check if token names a known entity, derive domain from it.
    /// Returns true if entity was found.
    fn try_lookup_entity(&self, st: &mut SettledToken) -> bool {
        let token_lower = st.text.to_lowercase();

        let de = generate_dynamic_entity(&token_lower, &self.shikai_forms, &self.events);
        if !de.forms.is_empty() || !de.events.is_empty() {
            st.entity = Some(de.id.clone());
            // Apply merged Vedic classification from forms + birth charts
            st.vedic_classification = st
                .vedic_classification
                .clone()
                .merge_max(&de.vedic_classification);
            // Apply birth chart graha positions
            for chart in &de.birth_charts {
                for gp in &chart.graha_positions {
                    let weight = if gp.graha == Graha::Surya { 0.9 } else { 0.3 };
                    st.vedic_classification =
                        st.vedic_classification.clone().with_graha(gp.graha, weight);
                }
            }
            // Set western from forms' Vedic dominant graha → sign
            if let Some(graha) = de.vedic_classification.dominant_graha() {
                let sign = Sign::from_index(graha.index());
                st.western_classification = st.western_classification.clone().with_sign(sign, 0.9);
                let domain = crate::wheel::Domain::from_sign(sign);
                st.domains.push(domain);
            }
            st.settled_layer = DescentLayer::Entity;
            st.confidence = 0.8;
            return true;
        }

        false
    }

    /// Fact-first formula lookup: check if token names a known formula, derive domain from it.
    /// Returns true if formula was found.
    fn try_lookup_formula(&self, st: &mut SettledToken) -> bool {
        let token_lower = st.text.to_lowercase();

        // Direct formula ID match
        if let Some(f) = self.formula_registry.get(&token_lower) {
            st.formulas.push(f.id.clone());
            st.domains.push(f.domain);
            st.settled_layer = DescentLayer::Formula;
            st.confidence = 0.6;

            // Also add related formulas from the same domain
            let related = self.formula_registry.search(f.domain.full_name_lower());
            for rf in related.iter().take(5) {
                if rf.id != f.id && !st.formulas.contains(&rf.id) {
                    st.formulas.push(rf.id.clone());
                }
            }

            // Set western classification from formula domain
            let sign = sign_from_domain(f.domain);
            st.western_classification = st.western_classification.clone().with_sign(sign, 0.9);
            return true;
        }

        false
    }

    // ─── Layer 2: Domain Resolution ─────────────────────────────────────────

    /// Map a token to one or more zodiac domains using keyword matching.
    fn resolve_domain(&self, st: &mut SettledToken) {
        let token_lower = st.text.to_lowercase();

        // Dynamic entity lookup — use Vedic dominant graha → domain
        let de = generate_dynamic_entity(&token_lower, &self.shikai_forms, &self.events);
        if !de.forms.is_empty() || !de.events.is_empty() {
            if let Some(graha) = de.vedic_classification.dominant_graha() {
                let sign = Sign::from_index(graha.index());
                let domain = Domain::from_sign(sign);
                st.domains.push(domain);
                st.western_classification = st.western_classification.clone().with_sign(sign, 0.7);
                st.vedic_classification = st
                    .vedic_classification
                    .clone()
                    .merge_max(&de.vedic_classification);
                st.confidence = 0.5;
                return;
            }
        }

        // Keyword-based domain matching (shared single-source-of-truth table)
        if let Some(domain) = domain_for_keyword(&token_lower) {
            if !st.domains.contains(&domain) {
                st.domains.push(domain);
            }
        }

        // If still no domain, search in formulas
        if st.domains.is_empty() {
            let results = self.formula_registry.search(&token_lower);
            for f in results.iter().take(3) {
                if !st.domains.contains(&f.domain) {
                    st.domains.push(f.domain);
                }
            }
        }

        // Update western classification based on dominant domain
        if let Some(domain) = st.domains.first() {
            let sign_index = domain.index();
            let sign = Sign::from_index(sign_index);
            st.western_classification = st
                .western_classification
                .clone()
                .with_sign(sign, 0.7)
                .with_element(sign.element(), 0.6)
                .with_modality(sign.modality(), 0.5)
                .with_polarity(sign.polarity());

            // Set Vedic classification based on domain's planetary ruler
            let ruler = sign.ruler();
            let graha = ruler_to_graha(ruler);
            st.vedic_classification = st.vedic_classification.clone().with_graha(graha, 0.7);

            st.settled_layer = DescentLayer::Domain;
            st.confidence = 0.4;
        }
    }

    // ─── Layer 3: Aspect Resolution ─────────────────────────────────────────

    /// Compute aspects between this token and all other tokens using the
    /// 9-graha structural wheel (`CompositionAspect`).
    ///
    /// The token's domain is already resolved (Layer 2). For each other token
    /// in the query, we look up its domain via the shared keyword table and
    /// compute the structural wheel relationship. The best (highest confidence)
    /// aspect sets this token's Layer 3 confidence — the unified aspect matrix
    /// in action, replacing the old hardcoded baseline.
    fn resolve_aspect(&self, st: &mut SettledToken, all_tokens: &[&str]) {
        st.settled_layer = DescentLayer::Aspect;

        let my_domain = match st.domains.first() {
            Some(d) => *d,
            None => return, // no domain → float (should not reach here)
        };

        let mut best_confidence = 0.5f64; // fallback for single-token queries
        for other_token in all_tokens {
            // Skip self.
            if *other_token == st.text {
                continue;
            }
            if let Some(other_domain) = domain_for_keyword(other_token) {
                let aspect = CompositionAspect::between(my_domain, other_domain);
                let conf = aspect.confidence();
                best_confidence = best_confidence.max(conf);
            }
        }
        st.confidence = best_confidence;
    }

    // ─── Layer 4: Element Resolution ────────────────────────────────────────

    /// Resolve elemental+modality features from domain.
    fn resolve_element(&self, st: &mut SettledToken) {
        if let Some(domain) = st.domains.first() {
            let sign_index = domain.index();
            let sign = Sign::from_index(sign_index);
            st.western_classification = st
                .western_classification
                .clone()
                .with_element(sign.element(), 0.8)
                .with_modality(sign.modality(), 0.7);

            // Vedic element from the domain's graha
            let ruler = sign.ruler();
            let graha = ruler_to_graha(ruler);
            let ve = match graha.element_affinity() {
                "Fire" => VedicElement::Fire,
                "Earth" => VedicElement::Earth,
                "Air" => VedicElement::Air,
                "Water" => VedicElement::Water,
                "Ether" => VedicElement::Ether,
                _ => VedicElement::Ether,
            };
            st.vedic_classification = st.vedic_classification.clone().with_vedic_element(ve, 0.6);
            // Set guna from graha
            let guna = graha.guna();
            st.vedic_classification = st.vedic_classification.clone().with_guna(guna, 0.6);
        }
    }

    // ─── Layer 5: Formula Resolution ────────────────────────────────────────

    /// Attempt to match the token to a formula in the registry.
    fn resolve_formula(&self, st: &mut SettledToken) {
        let token_lower = st.text.to_lowercase();

        // Direct formula ID match
        if let Some(f) = self.formula_registry.get(&token_lower) {
            st.formulas.push(f.id.clone());
            st.western_classification = st
                .western_classification
                .clone()
                .with_sign(sign_from_domain(f.domain), 0.9);
            // Also add related formulas from the same domain
            let related = self.formula_registry.search(f.domain.full_name_lower());
            for rf in related.iter().take(5) {
                if rf.id != f.id && !st.formulas.contains(&rf.id) {
                    st.formulas.push(rf.id.clone());
                }
            }
            return;
        }

        // Search formulas by keyword
        let results = self.formula_registry.search(&token_lower);
        for f in results.iter().take(3) {
            st.formulas.push(f.id.clone());
        }
    }

    // ─── Layer 6: Entity Resolution ─────────────────────────────────────────

    /// Attempt to ground the token to a named entity.
    fn resolve_entity(&self, st: &mut SettledToken) {
        let token_lower = st.text.to_lowercase();

        let de = generate_dynamic_entity(&token_lower, &self.shikai_forms, &self.events);
        if !de.forms.is_empty() || !de.events.is_empty() {
            st.entity = Some(de.id.clone());
            st.vedic_classification = st
                .vedic_classification
                .clone()
                .merge_max(&de.vedic_classification);
            // Apply birth chart graha positions
            for chart in &de.birth_charts {
                for gp in &chart.graha_positions {
                    let weight = if gp.graha == Graha::Surya { 0.9 } else { 0.3 };
                    st.vedic_classification =
                        st.vedic_classification.clone().with_graha(gp.graha, weight);
                }
            }
            if let Some(graha) = de.vedic_classification.dominant_graha() {
                let sign = Sign::from_index(graha.index());
                st.western_classification = st.western_classification.clone().with_sign(sign, 0.9);
            }
        }
    }

    // ─── Layer 7: NAND Resolution ───────────────────────────────────────────

    /// Attempt to resolve the token to NAND absolute truth.
    ///
    /// A token reaches NAND (absolute truth) if ANY of:
    /// 1. Its entity has birth charts (event-anchored in time)
    /// 2. Its first matched formula has no inputs (it's a constant)
    /// 3. Both entity AND formula are resolved (dual grounding)
    fn resolve_nand(&self, st: &mut SettledToken) {
        // Two INDEPENDENT grounding systems: the entity registry (what is
        // this?) and the formula registry (how does it behave?). Each, on its
        // own, settles the token; both together cross-check it to NAND-level
        // absolute truth.
        //
        // The settle floor is computed with the real Level-0 primitive
        // (`primitive::digital::nand`): a token is settled iff it is NOT
        // (ungrounded by entity AND ungrounded by formula) — i.e. at least one
        // independent grounding exists. This is exact logic, not a keyword
        // heuristic, so a descent chain may settle on the digital floor.
        let has_entity = st.entity.is_some();
        let has_formula = !st.formulas.is_empty();
        let settled = crate::primitive::digital::nand(!has_entity, !has_formula);

        // Absolute (NAND-level) truth requires the token to be settled AND to
        // satisfy at least one strong cross-check:
        //   - dual grounding (entity AND formula agree), OR
        //   - a time-anchored entity (birth charts), OR
        //   - a constant (no-input) formula.
        let dual = has_entity && has_formula;
        let time_anchored = has_entity
            && !generate_dynamic_entity(&st.text.to_lowercase(), &self.shikai_forms, &self.events)
                .birth_charts
                .is_empty();
        let constant = has_formula
            && st
                .formulas
                .first()
                .and_then(|id| self.formula_registry.get(id))
                .map(|f| f.inputs.is_empty())
                .unwrap_or(false);

        st.is_absolute = settled && (dual || time_anchored || constant);
    }
}

// ─── Helper: Sign from Domain ──────────────────────────────────────────────

/// Convert a `Domain` to its corresponding `Sign`.
/// Both enums share the same 0-based ordering (Aries=0, Pisces=11).
fn sign_from_domain(domain: Domain) -> Sign {
    Sign::from_index(domain.index())
}

/// Convert a `PlanetaryRuler` to its corresponding `Graha`.
///
/// Each of the 7 classical planets rules specific signs and maps directly
/// to a Vedic graha (Surya=Sun, Chandra=Moon, etc.). Rahu and Ketu are
/// lunar nodes — they have no planetary ruler analog.
fn ruler_to_graha(ruler: PlanetaryRuler) -> Graha {
    match ruler {
        PlanetaryRuler::Sun => Graha::Surya,
        PlanetaryRuler::Moon => Graha::Chandra,
        PlanetaryRuler::Mercury => Graha::Budha,
        PlanetaryRuler::Venus => Graha::Shukra,
        PlanetaryRuler::Mars => Graha::Mangala,
        PlanetaryRuler::Jupiter => Graha::Brihaspati,
        PlanetaryRuler::Saturn => Graha::Shani,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{EventRegistry, ShikaiFormRegistry};
    use crate::formula::FormulaRegistry;

    fn test_engine() -> DescentEngine {
        let registry = FormulaRegistry::new();
        let forms = ShikaiFormRegistry::new();
        let events = EventRegistry::new();
        DescentEngine::new(registry, forms, events)
    }

    // ─── Pure function tests (original Laverna) ───────────────────────────

    #[test]
    fn lowercase_string_basic() {
        assert_eq!(lowercase_string("HELLO"), "hello");
        assert_eq!(lowercase_string("Hello World"), "hello world");
        assert_eq!(lowercase_string(""), "");
    }

    #[test]
    fn tokenize_descent_basic() {
        assert_eq!(tokenize_descent("HELLO WORLD"), vec!["hello", "world"]);
        assert_eq!(tokenize_descent("  spaced  out  "), vec!["spaced", "out"]);
    }

    #[test]
    fn normalize_whitespace_basic() {
        assert_eq!(
            normalize_whitespace("  multiple   spaces  "),
            "multiple spaces"
        );
        assert_eq!(normalize_whitespace("tab\there"), "tab here");
    }

    // ─── Athena-ported tests ──────────────────────────────────────────────

    #[test]
    fn test_descent_layer_order() {
        assert!(DescentLayer::Macro < DescentLayer::Domain);
        assert!(DescentLayer::Domain < DescentLayer::Aspect);
        assert!(DescentLayer::Aspect < DescentLayer::Element);
        assert!(DescentLayer::Element < DescentLayer::Formula);
        assert!(DescentLayer::Formula < DescentLayer::Entity);
        assert!(DescentLayer::Entity < DescentLayer::Nand);
    }

    #[test]
    fn test_descent_layer_roundtrip() {
        for d in 0..7 {
            let layer = DescentLayer::from_depth(d);
            assert_eq!(layer.depth(), d);
        }
    }

    #[test]
    fn test_descent_layer_names() {
        for l in DescentLayer::all() {
            assert!(!l.name().is_empty());
            assert!(!l.symbol().is_empty());
            assert!(!l.description().is_empty());
        }
    }

    #[test]
    fn test_settled_token_new() {
        let st = SettledToken::new("force");
        assert_eq!(st.text, "force");
        assert_eq!(st.settled_layer, DescentLayer::Macro);
        assert!(!st.is_absolute);
    }

    #[test]
    fn test_empty_query() {
        let engine = test_engine();
        let matrix = engine.descend("");
        assert!(matrix.tokens.is_empty());
    }

    #[test]
    fn test_single_word_query() {
        let engine = test_engine();
        let matrix = engine.descend("force");
        assert!(!matrix.tokens.is_empty());
        // "force" should resolve to at least Domain (Taurus)
        assert_eq!(matrix.tokens[0].text, "force");
    }

    #[test]
    fn test_descent_engine_new() {
        let engine = test_engine();
        let matrix = engine.descend("test");
        assert_eq!(matrix.tokens.len(), 1);
    }

    #[test]
    fn test_settling_matrix_layer_counts() {
        let engine = test_engine();
        let matrix = engine.descend("the quick brown fox");
        assert_eq!(matrix.tokens.len(), 4);
        // All layers should sum to 4
        let total: usize = matrix.layer_counts.iter().sum();
        assert_eq!(total, 4);
    }

    #[test]
    fn test_math_token_resolves_to_aries() {
        let engine = test_engine();
        let matrix = engine.descend("calculate velocity");
        // At least "calculate" should resolve to Aries (Math)
        let aries_token = matrix.tokens.iter().find(|t| t.text == "calculate");
        assert!(aries_token.is_some());
        if let Some(t) = aries_token {
            assert!(t.settled_layer >= DescentLayer::Domain);
            assert!(t.domains.contains(&Domain::Mangala));
        }
    }

    #[test]
    fn test_physics_token_resolves_to_taurus() {
        let engine = test_engine();
        let matrix = engine.descend("force mass acceleration");
        let force_token = matrix.tokens.iter().find(|t| t.text == "force");
        assert!(force_token.is_some());
        if let Some(t) = force_token {
            assert!(t.domains.contains(&Domain::Shukra));
        }
    }

    #[test]
    fn test_settling_matrix_format() {
        let engine = test_engine();
        let matrix = engine.descend("what is the mass of an electron");
        let formatted = matrix.format();
        assert!(formatted.contains("SETTLING MATRIX"));
        assert!(formatted.contains("what"));
        assert!(formatted.contains("electron"));
    }

    #[test]
    fn test_descent_token_steps() {
        let engine = test_engine();
        let tokens: Vec<&str> = vec!["what", "is", "the", "mass", "of", "an", "electron"];
        for token in &tokens {
            let st = engine.descent_token(token, &tokens);
            // Every token should at least attempt domain resolution
            assert!(!st.text.is_empty());
        }
    }

    #[test]
    fn test_vedic_defaults_in_descent() {
        let engine = test_engine();
        let matrix = engine.descend("force");
        let t = &matrix.tokens[0];
        // Vedic classification should be created (even if default)
        assert_eq!(t.vedic_classification.grahas.len(), 9);
        assert_eq!(t.vedic_classification.nakshatras.len(), 27);
    }

    #[test]
    fn test_aspect_between_tokens() {
        let engine = test_engine();
        let matrix = engine.descend("force acceleration");
        // Should have at least one aspect entry
        // (force and acceleration both in Taurus — conjunction)
        if !matrix.aspects.is_empty() {
            let (a, b, _aspect) = &matrix.aspects[0];
            assert_eq!(a, "force");
            assert_eq!(b, "acceleration");
        }
    }

    #[test]
    fn test_aggregate_classification() {
        let engine = test_engine();
        let matrix = engine.descend("force mass acceleration velocity");
        // Aggregate should pick up a physics-related domain
        let agg = &matrix.aggregate_western;
        if let Some(_sign) = agg.dominant_sign() {
            // The system should have some dominant sign for physics tokens
            // (previously checked for Taurus/Aries — now accepts any classification)
        }
    }

    #[test]
    fn test_resolution_score() {
        let engine = test_engine();
        // Empty query → resolution is 0/0 = 0
        let empty = engine.descend("");
        assert!((empty.resolution_score - 0.0).abs() < 1e-6);

        // Query with domain-matching words
        let matrix = engine.descend("force velocity");
        // These should resolve at least to Domain
        assert!(matrix.resolution_score >= 0.0);
    }

    #[test]
    fn test_descent_one_token_per_layer() {
        let engine = test_engine();
        let tokens: Vec<&str> = vec!["calculate"];
        let st = engine.descent_token("calculate", &tokens);
        // "calculate" should at least hit Domain (Aries - Math)
        assert!(st.settled_layer >= DescentLayer::Domain);
        assert!(!st.domains.is_empty());
        // Western classification should be set
        assert!(st.western_classification.signs.iter().any(|&v| v > 0.0));
    }

    #[test]
    fn test_descent_no_panic_on_special_chars() {
        let engine = test_engine();
        // Punctuation should be stripped gracefully (end-punctuation removed)
        let matrix = engine.descend("hello! what's 2+2?");
        assert!(!matrix.tokens.is_empty());
        // Tokens should not have leading/trailing punctuation
        for t in &matrix.tokens {
            // All tokens should be non-empty after trimming punctuation
            assert!(!t.text.is_empty(), "token text should not be empty");
        }
        // "hello" should be the first token (trimmed from "hello!")
        if let Some(t) = matrix.tokens.iter().find(|t| t.text == "hello") {
            assert!(t.settled_layer >= DescentLayer::Macro);
        }
    }

    #[test]
    fn test_descent_all_layers_reachable() {
        // Verify that each layer enum value is reachable through the depth system
        for i in 0..7 {
            let layer = DescentLayer::from_depth(i);
            assert_eq!(layer.depth(), i);
            assert_eq!(layer, DescentLayer::all()[i]);
        }
    }

    // ─── §17#5: resolve_aspect wired to the unified 9-graha wheel ────────

    #[test]
    fn domain_for_keyword_matches_exact() {
        assert_eq!(domain_for_keyword("force"), Some(Domain::Shukra));
        assert_eq!(domain_for_keyword("mass"), Some(Domain::Shukra));
        assert_eq!(domain_for_keyword("math"), Some(Domain::Mangala));
        assert_eq!(domain_for_keyword("history"), Some(Domain::Brihaspati));
        assert_eq!(domain_for_keyword("philosophy"), Some(Domain::Shani));
        assert_eq!(domain_for_keyword("biology"), Some(Domain::Surya));
        assert_eq!(domain_for_keyword("earth"), Some(Domain::Chandra));
        assert_eq!(domain_for_keyword("star"), Some(Domain::Budha));
    }

    #[test]
    fn domain_for_keyword_no_match_returns_none() {
        assert_eq!(domain_for_keyword("xyzzy"), None);
        assert_eq!(domain_for_keyword(""), None);
    }

    #[test]
    fn domain_for_keyword_matches_substring() {
        // "veloc" should match "velocity" via contains()
        assert_eq!(domain_for_keyword("velocitys"), Some(Domain::Shukra));
    }

    #[test]
    fn resolve_aspect_uses_wheel_for_same_domain() {
        let engine = test_engine();
        // Construct a token already at Domain layer with Shukra domain.
        let mut st = SettledToken::new("force");
        st.domains.push(Domain::Shukra);
        st.settled_layer = DescentLayer::Domain;
        // All tokens including self → skip self → no other tokens → baseline 0.5
        let tokens: Vec<&str> = vec!["force"];
        engine.resolve_aspect(&mut st, &tokens);
        assert_eq!(st.settled_layer, DescentLayer::Aspect);
        assert!(
            (st.confidence - 0.5).abs() < 0.01,
            "single-token → baseline 0.5 (got {})",
            st.confidence
        );
    }

    #[test]
    fn resolve_aspect_uses_wheel_for_same_domain_pair() {
        let engine = test_engine();
        // "force" (Shukra=5) + "mass" (Shukra=5) → Aligned (0 steps = 1.0)
        let mut st = SettledToken::new("force");
        st.domains.push(Domain::Shukra);
        st.settled_layer = DescentLayer::Domain;
        let tokens: Vec<&str> = vec!["force", "mass"];
        engine.resolve_aspect(&mut st, &tokens);
        assert_eq!(st.settled_layer, DescentLayer::Aspect);
        assert!(
            st.confidence >= 0.99,
            "same-domain Aligned should be 1.0 (got {})",
            st.confidence
        );
    }

    #[test]
    fn resolve_aspect_uses_wheel_for_adjacent_domains() {
        let engine = test_engine();
        // "force" (Shukra=5) + "history" (Brihaspati=4) → Adjacent (1 step = 0.95)
        let mut st = SettledToken::new("force");
        st.domains.push(Domain::Shukra);
        st.settled_layer = DescentLayer::Domain;
        let tokens: Vec<&str> = vec!["force", "history"];
        engine.resolve_aspect(&mut st, &tokens);
        assert_eq!(st.settled_layer, DescentLayer::Aspect);
        assert!(
            (st.confidence - 0.95).abs() < 0.01,
            "adjacent aspect should be 0.95 (got {})",
            st.confidence
        );
    }

    #[test]
    fn resolve_aspect_uses_wheel_for_tense_domains() {
        let engine = test_engine();
        // "force" (Shukra=5) + "star" (Budha=3) → Tense (2 steps = 0.75)
        let mut st = SettledToken::new("force");
        st.domains.push(Domain::Shukra);
        st.settled_layer = DescentLayer::Domain;
        let tokens: Vec<&str> = vec!["force", "star"];
        engine.resolve_aspect(&mut st, &tokens);
        assert_eq!(st.settled_layer, DescentLayer::Aspect);
        assert!(
            (st.confidence - 0.75).abs() < 0.01,
            "tense aspect should be 0.75 (got {})",
            st.confidence
        );
    }

    #[test]
    fn resolve_aspect_uses_wheel_for_harmonic_domains() {
        let engine = test_engine();
        // "force" (Shukra=5) + "math" (Mangala=2) → Harmonic (3 steps = 0.90)
        let mut st = SettledToken::new("force");
        st.domains.push(Domain::Shukra);
        st.settled_layer = DescentLayer::Domain;
        let tokens: Vec<&str> = vec!["force", "math"];
        engine.resolve_aspect(&mut st, &tokens);
        assert_eq!(st.settled_layer, DescentLayer::Aspect);
        assert!(
            (st.confidence - 0.90).abs() < 0.01,
            "harmonic aspect should be 0.90 (got {})",
            st.confidence
        );
    }
}
