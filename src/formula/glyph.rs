//! # Glyph Grammar — Sign/Blend/Transmute/Bind operators
//!
//! Assimilated from **RUNE-Core** (unified-game `spellcasting/grammar.rs`),
//! which derived its 4-operator system from Ifá divination, Pratt parsing,
//! and the Tibetan Three Mysteries (Body/Speech/Mind) synchronization model.
//!
//! These operators provide a deterministic formula composition system:
//!
//! | Operator | Binding Power | Behavior |
//! |----------|--------------|----------|
//! | **Sign** | 10 | Identity — declares a single glyph/formula |
//! | **Blend** | 20 | Combine two glyphs into a compound (a*16 + b) |
//! | **Transmute** | 30 | Transform via XOR catalyst ((a ^ b) % 16) |
//! | **Bind** | 40 | Persistent linked pair (0x8000 \| (a << 8) \| b) |
//!
//! ## Two-Speed Combinatorics
//!
//! Athena operates at two simultaneous speeds:
//!
//! ### Speed 1: Open (Generative) — Sign / Blend / Transmute
//!
//! These operators produce **unbounded vocabulary**:
//! - `Sign(a)` = a (identity, always succeeds)
//! - `Blend(a, b)` = a×16 + b (exponential combinations — 16² = 256 blends)
//! - `Transmute(a, b)` = (a ^ b) % 16 (XOR catalyst, 16×16 = 256 transmutes)
//!
//! **Open speed** is the creative engine — it generates novel compound glyphs
//! from primitive elements, enabling formula discovery and chaining.
//! The same inputs to Sign/Blend/Transmute always produce the same output,
//! but the possible *combinations* of formulas are unbounded.
//!
//! ### Speed 2: Closed (Relational) — Bind
//!
//! `Bind` constructs a **fixed, balanceable set of links**:
//! - `Bind(a, b)` = 0x8000 | (a << 8) | b (persistent pair)
//! - Once bound, the pair is deterministic and immutable
//! - The entity registry (86 entities) provides the closed relational speed:
//!   entity lookups always return the same properties; entity counts never grow
//!   at runtime.
//!
//! **Closed speed** is the grounding mechanism — it anchors the generative
//! system to fixed reference points (entities, constants, registered formulas).
//!
//! ### Why Two Speeds?
//!
//! Pure open systems (unbounded generation) lose grounding — they produce
//! unverifiable claims. Pure closed systems (only lookups) cannot discover
//! novel relationships. The two-speed design keeps both: generative chains
//! can explore new territory while always being able to "fall back" to
//! deterministic entity/formula lookups.
//!
//! All operations are deterministic, use fixed-size arrays, and avoid HashMap.

use serde::{Deserialize, Serialize};

/// A glyph is a 4-bit value (0–15) representing a primitive symbolic unit.
/// 16 primitives × 4 operators = 256 theoretical compounds.
pub type Glyph = u8;

/// The maximum glyph value (4-bit range).
pub const MAX_GLYPH: Glyph = 15;

/// Number of primitive glyphs.
pub const GLYPH_COUNT: usize = 16;

/// The four glyph grammar operators, ordered by binding power (ascending).
///
/// This matches the Pratt parser operator table pattern from RUNE-Core,
/// where higher binding power means tighter binding (evaluated first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GlyphOperator {
    /// Sign — identity, returns input glyph unchanged.
    /// Binding power: 10 (lowest — evaluated last in chain)
    Sign,
    /// Blend — combines two glyphs into a compound: left * 16 + right.
    /// Binding power: 20
    Blend,
    /// Transmute — transforms via XOR catalyst: (left ^ right) % 16.
    /// Binding power: 30
    Transmute,
    /// Bind — creates a persistent linked pair: 0x8000 | (left << 8) | right.
    /// Binding power: 40 (highest — evaluated first)
    Bind,
}

impl GlyphOperator {
    /// All 4 operators in order of increasing binding power.
    pub const ALL: [GlyphOperator; 4] = [
        GlyphOperator::Sign,
        GlyphOperator::Blend,
        GlyphOperator::Transmute,
        GlyphOperator::Bind,
    ];

    /// Binding power of this operator (10/20/30/40).
    ///
    /// Used for Pratt-style precedence: higher = tighter binding.
    /// Sign (10) is the loosest, Bind (40) is the tightest.
    pub fn binding_power(self) -> u8 {
        match self {
            GlyphOperator::Sign => 10,
            GlyphOperator::Blend => 20,
            GlyphOperator::Transmute => 30,
            GlyphOperator::Bind => 40,
        }
    }

    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            GlyphOperator::Sign => "Sign",
            GlyphOperator::Blend => "Blend",
            GlyphOperator::Transmute => "Transmute",
            GlyphOperator::Bind => "Bind",
        }
    }

    /// One-line description of the operator's behavior.
    pub fn description(self) -> &'static str {
        match self {
            GlyphOperator::Sign => "Identity — returns the glyph/formula unchanged",
            GlyphOperator::Blend => "Combine two glyphs into a compound (a×16+b)",
            GlyphOperator::Transmute => "Transform via XOR catalyst ((a^b)%16)",
            GlyphOperator::Bind => "Persistent linked pair (0x8000|(a<<8)|b)",
        }
    }

    /// Parse an operator from its name string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sign" => Some(GlyphOperator::Sign),
            "blend" => Some(GlyphOperator::Blend),
            "transmute" => Some(GlyphOperator::Transmute),
            "bind" => Some(GlyphOperator::Bind),
            _ => None,
        }
    }
}

/// A compound glyph result — the output of applying an operator to one or two glyphs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GlyphResult {
    /// A single primitive glyph (0–15).
    Primitive(Glyph),
    /// A compound glyph: left * 16 + right (0–255).
    Compound(u16),
    /// A bound pair: 0x8000 | (left << 8) | right.
    Bound(u16),
}

impl GlyphResult {
    /// Get the raw u16 value of this result.
    pub fn raw(self) -> u16 {
        match self {
            GlyphResult::Primitive(g) => g as u16,
            GlyphResult::Compound(v) => v,
            GlyphResult::Bound(v) => v,
        }
    }

    /// Human-readable representation.
    pub fn format(self) -> String {
        match self {
            GlyphResult::Primitive(g) => format!("Glyph({})", g),
            GlyphResult::Compound(v) => {
                let left = v / 16;
                let right = v % 16;
                format!("Compound({}←{}|{})", v, left, right)
            }
            GlyphResult::Bound(v) => {
                let left = ((v >> 4) & 0x0F) as u8;
                let right = (v & 0x0F) as u8;
                format!("Bound(0x{:04X} ← {}↔{})", v, left, right)
            }
        }
    }

    /// Decompose a compound or bound result back into left/right glyphs.
    /// Returns None for primitive glyphs.
    pub fn decompose(self) -> Option<(Glyph, Glyph)> {
        match self {
            GlyphResult::Primitive(_) => None,
            GlyphResult::Compound(v) => {
                let left = (v / 16) as Glyph;
                let right = (v % 16) as Glyph;
                Some((left, right))
            }
            GlyphResult::Bound(v) => {
                // Encoding: bits 4–7 = left, bits 0–3 = right
                let left = ((v >> 4) & 0x0F) as Glyph;
                let right = (v & 0x0F) as Glyph;
                Some((left, right))
            }
        }
    }
}

/// Apply a glyph operator to one or two glyphs.
///
/// Returns `None` if the glyph value is out of range (> 15) for Sign/Blend/Transmute.
/// Bind accepts any u8 value for left/right (up to 255 each).
pub fn apply_operator(op: GlyphOperator, left: Glyph, right: Option<Glyph>) -> Option<GlyphResult> {
    match op {
        GlyphOperator::Sign => {
            // Sign = identity — returns the glyph unchanged
            if left > MAX_GLYPH {
                return None;
            }
            Some(GlyphResult::Primitive(left))
        }
        GlyphOperator::Blend => {
            // Blend = combine: left * 16 + right (two 4-bit values into one byte)
            let r = right?;
            if left > MAX_GLYPH || r > MAX_GLYPH {
                return None;
            }
            let value = (left as u16) * 16 + r as u16;
            Some(GlyphResult::Compound(value))
        }
        GlyphOperator::Transmute => {
            // Transmute = XOR catalyst: (left ^ right) % 16
            let r = right?;
            if left > MAX_GLYPH || r > MAX_GLYPH {
                return None;
            }
            let value = ((left ^ r) % 16) as Glyph;
            Some(GlyphResult::Primitive(value))
        }
        GlyphOperator::Bind => {
            // Bind = persistent linked pair: 0x8000 | (left << 4) | right
            // Each glyph is 4 bits (0–15). The 0x8000 flag is bit 15, separate from data bits.
            // Data layout: bits 4–7 = left, bits 0–3 = right, bit 15 = bound flag.
            let r = right?;
            if left > MAX_GLYPH || r > MAX_GLYPH {
                return None;
            }
            let value = 0x8000u16 | ((left as u16) << 4) | r as u16;
            Some(GlyphResult::Bound(value))
        }
    }
}

/// Operator binding-power table as a const array (no HashMap).
///
/// This is the Pratt parser operator table — a fixed-size array that
/// can be linearly scanned to find an operator's precedence. The pattern
/// avoids all runtime allocation and is fully deterministic.
pub const OPERATOR_TABLE: [(GlyphOperator, u8); 4] = [
    (GlyphOperator::Sign, 10),
    (GlyphOperator::Blend, 20),
    (GlyphOperator::Transmute, 30),
    (GlyphOperator::Bind, 40),
];

/// Look up binding power for an operator via linear scan.
pub fn binding_power(op: GlyphOperator) -> u8 {
    for &(o, bp) in &OPERATOR_TABLE {
        if o == op {
            return bp;
        }
    }
    0
}

/// Check if a bound value is a valid Bind result (has the 0x8000 flag).
pub fn is_bound(value: u16) -> bool {
    value & 0x8000 != 0
}

/// Decompose a bound value into its left and right glyphs.
/// Returns None if the value is not a valid bound pair.
///
/// Encoding: bits 4–7 = left glyph, bits 0–3 = right glyph, bit 15 = bound flag.
pub fn decompose_bound(value: u16) -> Option<(Glyph, Glyph)> {
    if !is_bound(value) {
        return None;
    }
    let left = ((value >> 4) & 0x0F) as Glyph;
    let right = (value & 0x0F) as Glyph;
    Some((left, right))
}

// ─── Named Glyphs ───────────────────────────────────────────────────────────

/// Named primitive glyphs with semantic meaning.
///
/// These 13 glyphs map onto the RUNE-Core system's primitive set.
/// The remaining 3 slots (13–15) are reserved for domain-specific extensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NamedGlyph {
    // Core elements
    Fire = 0,
    Water = 1,
    Earth = 2,
    Air = 3,
    Ether = 4,
    // Forces
    Light = 5,
    Shadow = 6,
    Time = 7,
    Space = 8,
    // Structures
    Mind = 9,
    Body = 10,
    Spirit = 11,
    Void = 12,
    // Reserved slots 13-15
    Reserved1 = 13,
    Reserved2 = 14,
    Reserved3 = 15,
}

impl NamedGlyph {
    /// All named glyphs.
    pub const ALL: [NamedGlyph; 13] = [
        NamedGlyph::Fire,
        NamedGlyph::Water,
        NamedGlyph::Earth,
        NamedGlyph::Air,
        NamedGlyph::Ether,
        NamedGlyph::Light,
        NamedGlyph::Shadow,
        NamedGlyph::Time,
        NamedGlyph::Space,
        NamedGlyph::Mind,
        NamedGlyph::Body,
        NamedGlyph::Spirit,
        NamedGlyph::Void,
    ];

    /// Get the numeric glyph value (0–15).
    pub fn glyph(self) -> Glyph {
        self as u8
    }

    /// Get the name of this glyph.
    pub fn name(self) -> &'static str {
        match self {
            NamedGlyph::Fire => "Fire",
            NamedGlyph::Water => "Water",
            NamedGlyph::Earth => "Earth",
            NamedGlyph::Air => "Air",
            NamedGlyph::Ether => "Ether",
            NamedGlyph::Light => "Light",
            NamedGlyph::Shadow => "Shadow",
            NamedGlyph::Time => "Time",
            NamedGlyph::Space => "Space",
            NamedGlyph::Mind => "Mind",
            NamedGlyph::Body => "Body",
            NamedGlyph::Spirit => "Spirit",
            NamedGlyph::Void => "Void",
            NamedGlyph::Reserved1 => "Reserved-13",
            NamedGlyph::Reserved2 => "Reserved-14",
            NamedGlyph::Reserved3 => "Reserved-15",
        }
    }

    /// Get the symbol for this glyph.
    pub fn symbol(self) -> &'static str {
        match self {
            NamedGlyph::Fire => "▲",
            NamedGlyph::Water => "▼",
            NamedGlyph::Earth => "◆",
            NamedGlyph::Air => "◇",
            NamedGlyph::Ether => "★",
            NamedGlyph::Light => "☀",
            NamedGlyph::Shadow => "☽",
            NamedGlyph::Time => "⌛",
            NamedGlyph::Space => "◎",
            NamedGlyph::Mind => "◉",
            NamedGlyph::Body => "⊕",
            NamedGlyph::Spirit => "∞",
            NamedGlyph::Void => "○",
            NamedGlyph::Reserved1 => "?",
            NamedGlyph::Reserved2 => "?",
            NamedGlyph::Reserved3 => "?",
        }
    }

    /// Parse a glyph from its name string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fire" => Some(NamedGlyph::Fire),
            "water" => Some(NamedGlyph::Water),
            "earth" => Some(NamedGlyph::Earth),
            "air" => Some(NamedGlyph::Air),
            "ether" => Some(NamedGlyph::Ether),
            "light" => Some(NamedGlyph::Light),
            "shadow" => Some(NamedGlyph::Shadow),
            "time" => Some(NamedGlyph::Time),
            "space" => Some(NamedGlyph::Space),
            "mind" => Some(NamedGlyph::Mind),
            "body" => Some(NamedGlyph::Body),
            "spirit" => Some(NamedGlyph::Spirit),
            "void" => Some(NamedGlyph::Void),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_identity() {
        let result = apply_operator(GlyphOperator::Sign, 5, None).unwrap();
        assert_eq!(result, GlyphResult::Primitive(5));
        assert_eq!(result.raw(), 5);
    }

    #[test]
    fn test_blend_compound() {
        let result = apply_operator(GlyphOperator::Blend, 3, Some(7)).unwrap();
        // 3 * 16 + 7 = 55
        assert_eq!(result, GlyphResult::Compound(55));
        assert_eq!(result.raw(), 55);
        let (l, r) = result.decompose().unwrap();
        assert_eq!(l, 3);
        assert_eq!(r, 7);
    }

    #[test]
    fn test_transmute_xor() {
        let result = apply_operator(GlyphOperator::Transmute, 12, Some(7)).unwrap();
        // (12 ^ 7) % 16 = 11 % 16 = 11
        assert_eq!(result, GlyphResult::Primitive(11));
    }

    #[test]
    fn test_bind_linked_pair() {
        let result = apply_operator(GlyphOperator::Bind, 3, Some(7)).unwrap();
        let raw = result.raw();
        assert!(is_bound(raw));
        let (l, r) = decompose_bound(raw).unwrap();
        assert_eq!(l, 3);
        assert_eq!(r, 7);
    }

    #[test]
    fn test_binding_powers() {
        assert_eq!(GlyphOperator::Sign.binding_power(), 10);
        assert_eq!(GlyphOperator::Blend.binding_power(), 20);
        assert_eq!(GlyphOperator::Transmute.binding_power(), 30);
        assert_eq!(GlyphOperator::Bind.binding_power(), 40);
    }

    #[test]
    fn test_binding_power_table() {
        assert_eq!(binding_power(GlyphOperator::Sign), 10);
        assert_eq!(binding_power(GlyphOperator::Blend), 20);
        assert_eq!(binding_power(GlyphOperator::Transmute), 30);
        assert_eq!(binding_power(GlyphOperator::Bind), 40);
    }

    #[test]
    fn test_operator_table_order() {
        // Verify table is sorted by binding power ascending
        for i in 1..OPERATOR_TABLE.len() {
            assert!(
                OPERATOR_TABLE[i - 1].1 <= OPERATOR_TABLE[i].1,
                "Operator table must be sorted by binding power"
            );
        }
    }

    #[test]
    fn test_operator_parse() {
        assert_eq!(GlyphOperator::parse("sign"), Some(GlyphOperator::Sign));
        assert_eq!(GlyphOperator::parse("BLEND"), Some(GlyphOperator::Blend));
        assert_eq!(
            GlyphOperator::parse("Transmute"),
            Some(GlyphOperator::Transmute)
        );
        assert_eq!(GlyphOperator::parse("bind"), Some(GlyphOperator::Bind));
        assert_eq!(GlyphOperator::parse("unknown"), None);
    }

    #[test]
    fn test_named_glyph_values() {
        assert_eq!(NamedGlyph::Fire.glyph(), 0);
        assert_eq!(NamedGlyph::Void.glyph(), 12);
        assert_eq!(NamedGlyph::Reserved3.glyph(), 15);
    }

    #[test]
    fn test_named_glyph_parse() {
        assert_eq!(NamedGlyph::parse("fire"), Some(NamedGlyph::Fire));
        assert_eq!(NamedGlyph::parse("void"), Some(NamedGlyph::Void));
        assert_eq!(NamedGlyph::parse("unknown"), None);
    }

    #[test]
    fn test_glyph_out_of_range() {
        // Values > 15 should be rejected for all operators (4-bit glyphs)
        assert!(apply_operator(GlyphOperator::Sign, 16, None).is_none());
        assert!(apply_operator(GlyphOperator::Blend, 16, Some(5)).is_none());
        assert!(apply_operator(GlyphOperator::Transmute, 5, Some(16)).is_none());
        assert!(apply_operator(GlyphOperator::Bind, 16, Some(5)).is_none());
        assert!(apply_operator(GlyphOperator::Bind, 5, Some(16)).is_none());
    }

    #[test]
    fn test_decompose_primitive_returns_none() {
        let r = GlyphResult::Primitive(7);
        assert!(r.decompose().is_none());
    }

    #[test]
    fn test_is_bound_flag() {
        assert!(is_bound(0x8305)); // has 0x8000 flag
        assert!(!is_bound(0x0055)); // no 0x8000 flag
    }

    #[test]
    fn test_all_operators_have_names() {
        for op in &GlyphOperator::ALL {
            assert!(!op.name().is_empty());
            assert!(!op.description().is_empty());
        }
    }

    #[test]
    fn test_all_named_glyphs_have_names_and_symbols() {
        for g in &NamedGlyph::ALL {
            assert!(!g.name().is_empty());
            assert!(!g.symbol().is_empty());
        }
    }

    // ─── Two-Speed Combinatorics Demo (Phase 6) ────────────────────

    /// Helper: apply a binary operator and unwrap the raw u16 value.
    fn apply_bin(op: GlyphOperator, a: u8, b: u8) -> u16 {
        apply_operator(op, a, Some(b)).unwrap().raw()
    }

    /// Helper: apply a unary operator and unwrap the raw u16 value.
    fn apply_un(op: GlyphOperator, a: u8) -> u16 {
        apply_operator(op, a, None).unwrap().raw()
    }

    #[test]
    fn test_two_speed_open_generative() {
        // Open speed: Sign/Blend/Transmute produce unbounded combinations.
        // Same glyph inputs can produce MANY different outputs via different operators.
        // NamedGlyph values: Light=5, Earth=2
        let a = NamedGlyph::Light.glyph(); // 5
        let b = NamedGlyph::Earth.glyph(); // 2

        // Speed 1 (open): same inputs, different operators → different results
        let sign = apply_un(GlyphOperator::Sign, a); // identity → 5
        let blend = apply_bin(GlyphOperator::Blend, a, b); // a*16 + b = 82
        let transmute = apply_bin(GlyphOperator::Transmute, a, b); // (5 ^ 2) % 16 = 7

        assert_eq!(sign, a as u16, "Sign returns identity");
        assert_eq!(blend, 82, "Blend compounds a*16 + b");
        assert_eq!(transmute, 7, "Transmute XORs then wraps");

        // All three produce different values from the same inputs
        assert_ne!(sign, blend);
        assert_ne!(blend, transmute);
        assert_ne!(sign, transmute);

        // Blend exceeds single-glyph range (0-15) — open speed generates novel values
        assert!(
            blend >= 16,
            "Blend produces compound values beyond glyph range"
        );
    }

    #[test]
    fn test_two_speed_closed_relational() {
        // Closed speed: Bind creates a persistent, deterministic link.
        // Same input always produces the same output.
        let a = NamedGlyph::Light.glyph(); // 5
        let b = NamedGlyph::Air.glyph(); // 3

        let bind1 = apply_bin(GlyphOperator::Bind, a, b);
        let bind2 = apply_bin(GlyphOperator::Bind, a, b);
        assert_eq!(
            bind1, bind2,
            "Bind is deterministic — same input → same output"
        );

        // Bind encodes left in bits 4-7, right in bits 0-3, flag in bit 15
        // value = 0x8000 | (5 << 4) | 3 = 0x8053
        let encoded_left = ((bind1 >> 4) & 0x0F) as u8;
        let encoded_right = (bind1 & 0x0F) as u8;
        assert_eq!(encoded_left, a, "Bind preserves left operand in bits 4-7");
        assert_eq!(encoded_right, b, "Bind preserves right operand in bits 0-3");
        assert!(
            bind1 & 0x8000 != 0,
            "Bind result has persistence flag (bit 15)"
        );

        // Decompose recovers the original pair
        let decomposed = GlyphResult::Bound(bind1).decompose().unwrap();
        assert_eq!(decomposed, (a, b), "Decompose recovers original pair");
    }

    #[test]
    fn test_two_speed_combined() {
        // The two speeds work together: open operators generate novel glyphs,
        // which are then bound into the closed relational system.
        let a = NamedGlyph::Time.glyph(); // 7
        let b = NamedGlyph::Earth.glyph(); // 2

        // Open: transmute creates a novel glyph from existing ones
        let novel = apply_bin(GlyphOperator::Transmute, a, b);
        let expected = (a ^ b) % 16; // (7 ^ 2) % 16 = 5
        assert_eq!(
            novel, expected as u16,
            "Transmute generates novel glyph (7→5)"
        );

        // Closed: bind the novel glyph to its origin
        let bound = apply_bin(GlyphOperator::Bind, a, novel as u8);
        let (left, right) = GlyphResult::Bound(bound).decompose().unwrap();
        assert_eq!(left, a, "Origin preserved in bind");
        assert_eq!(right, novel as u8, "Novel glyph preserved in bind");
    }
}
