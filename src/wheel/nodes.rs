use serde::{Deserialize, Serialize};
use std::fmt;

/// The 9 Vedic grahas — primary domain nodes on the wheel.
///
/// Each graha represents a fundamental cognitive/knowledge domain.
/// The wheel defines how knowledge composes across domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    Surya,
    Chandra,
    Mangala,
    Budha,
    Brihaspati,
    Shukra,
    Shani,
    Rahu,
    Ketu,
}

/// All 9 domains in wheel order (Surya → Ketu at 40° intervals).
pub const ALL_DOMAINS: [Domain; 9] = [
    Domain::Surya,
    Domain::Chandra,
    Domain::Mangala,
    Domain::Budha,
    Domain::Brihaspati,
    Domain::Shukra,
    Domain::Shani,
    Domain::Rahu,
    Domain::Ketu,
];

impl Domain {
    /// Arc index on the wheel (0–8).
    pub fn index(self) -> usize {
        match self {
            Domain::Surya => 0,
            Domain::Chandra => 1,
            Domain::Mangala => 2,
            Domain::Budha => 3,
            Domain::Brihaspati => 4,
            Domain::Shukra => 5,
            Domain::Shani => 6,
            Domain::Rahu => 7,
            Domain::Ketu => 8,
        }
    }

    /// Create a domain from its arc index (0–8).
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Domain::Surya),
            1 => Some(Domain::Chandra),
            2 => Some(Domain::Mangala),
            3 => Some(Domain::Budha),
            4 => Some(Domain::Brihaspati),
            5 => Some(Domain::Shukra),
            6 => Some(Domain::Shani),
            7 => Some(Domain::Rahu),
            8 => Some(Domain::Ketu),
            _ => None,
        }
    }

    /// Astrological symbol.
    pub fn symbol(self) -> &'static str {
        match self {
            Domain::Surya => "☉",
            Domain::Chandra => "☽",
            Domain::Mangala => "♂",
            Domain::Budha => "☿",
            Domain::Brihaspati => "♃",
            Domain::Shukra => "♀",
            Domain::Shani => "♄",
            Domain::Rahu => "☊",
            Domain::Ketu => "☋",
        }
    }

    /// Sanskrit name.
    pub fn name(self) -> &'static str {
        match self {
            Domain::Surya => "Surya",
            Domain::Chandra => "Chandra",
            Domain::Mangala => "Mangala",
            Domain::Budha => "Budha",
            Domain::Brihaspati => "Brihaspati",
            Domain::Shukra => "Shukra",
            Domain::Shani => "Shani",
            Domain::Rahu => "Rahu",
            Domain::Ketu => "Ketu",
        }
    }

    /// English name.
    pub fn full_name(self) -> &'static str {
        match self {
            Domain::Surya => "Sun",
            Domain::Chandra => "Moon",
            Domain::Mangala => "Mars",
            Domain::Budha => "Mercury",
            Domain::Brihaspati => "Jupiter",
            Domain::Shukra => "Venus",
            Domain::Shani => "Saturn",
            Domain::Rahu => "North Node",
            Domain::Ketu => "South Node",
        }
    }

    /// Knowledge domain archetype.
    pub fn archetype(self) -> &'static str {
        match self {
            Domain::Surya => "Self & Leadership",
            Domain::Chandra => "Mind & Emotion",
            Domain::Mangala => "Action & Engineering",
            Domain::Budha => "Logic & Communication",
            Domain::Brihaspati => "Wisdom & Law",
            Domain::Shukra => "Arts & Value",
            Domain::Shani => "Structure & Time",
            Domain::Rahu => "Innovation & Tech",
            Domain::Ketu => "Spirituality & Science",
        }
    }

    /// Offset by `delta` steps on the 9-node wheel (wrapping).
    pub fn offset(self, delta: isize) -> Self {
        let idx = ((self.index() as isize + delta).rem_euclid(9)) as usize;
        Self::from_index(idx).unwrap_or(Domain::Surya)
    }

    /// Opposite graha (index + 4) % 9 — 160° opposition.
    pub fn opposite(self) -> Self {
        self.offset(4)
    }

    /// Adjacent grahas (±1 step).
    pub fn adjacent(self) -> [Self; 2] {
        [self.offset(1), self.offset(-1)]
    }

    /// Trine grahas (index + 3) % 9 and (index + 6) % 9.
    pub fn trines(self) -> [Self; 2] {
        [self.offset(3), self.offset(6)]
    }

    /// Greek letter name (Phase 7 — Greek ontology).
    pub fn greek_name(self) -> &'static str {
        match self {
            Domain::Surya => "Alpha (α)",
            Domain::Chandra => "Beta (β)",
            Domain::Mangala => "Gamma (γ)",
            Domain::Budha => "Delta (δ)",
            Domain::Brihaspati => "Epsilon (ε)",
            Domain::Shukra => "Zeta (ζ)",
            Domain::Shani => "Eta (η)",
            Domain::Rahu => "Theta (θ)",
            Domain::Ketu => "Iota (ι)",
        }
    }

    /// Greek symbol.
    pub fn greek_symbol(self) -> &'static str {
        match self {
            Domain::Surya => "α",
            Domain::Chandra => "β",
            Domain::Mangala => "γ",
            Domain::Budha => "δ",
            Domain::Brihaspati => "ε",
            Domain::Shukra => "ζ",
            Domain::Shani => "η",
            Domain::Rahu => "θ",
            Domain::Ketu => "ι",
        }
    }

    /// Lowercased name — no allocation.
    pub fn name_lower(self) -> &'static str {
        match self {
            Domain::Surya => "surya",
            Domain::Chandra => "chandra",
            Domain::Mangala => "mangala",
            Domain::Budha => "budha",
            Domain::Brihaspati => "brihaspati",
            Domain::Shukra => "shukra",
            Domain::Shani => "shani",
            Domain::Rahu => "rahu",
            Domain::Ketu => "ketu",
        }
    }

    /// Western/English name for this graha.
    pub fn english_name(self) -> &'static str {
        match self {
            Domain::Surya => "sun",
            Domain::Chandra => "moon",
            Domain::Mangala => "mars",
            Domain::Budha => "mercury",
            Domain::Brihaspati => "jupiter",
            Domain::Shukra => "venus",
            Domain::Shani => "saturn",
            Domain::Rahu => "rahu",
            Domain::Ketu => "ketu",
        }
    }

    /// Sanskrit name.
    pub fn sanskrit(self) -> &'static str {
        match self {
            Domain::Surya => "सूर्य",
            Domain::Chandra => "चन्द्र",
            Domain::Mangala => "मङ्गल",
            Domain::Budha => "बुध",
            Domain::Brihaspati => "बृहस्पति",
            Domain::Shukra => "शुक्र",
            Domain::Shani => "शनि",
            Domain::Rahu => "राहु",
            Domain::Ketu => "केतु",
        }
    }

    /// Element affinity (Vedic tattva name as string).
    pub fn element_affinity(self) -> &'static str {
        match self {
            Domain::Surya => "Fire",
            Domain::Chandra => "Water",
            Domain::Mangala => "Fire",
            Domain::Budha => "Earth",
            Domain::Brihaspati => "Ether",
            Domain::Shukra => "Water",
            Domain::Shani => "Air",
            Domain::Rahu => "Air",
            Domain::Ketu => "Ether",
        }
    }

    /// All 9 domains in wheel order.
    pub fn all() -> [Self; 9] {
        ALL_DOMAINS
    }

    /// Alias for `name_lower()` — backward compatibility.
    #[inline]
    pub fn full_name_lower(self) -> &'static str {
        self.name_lower()
    }

    /// Map an astrology `Sign` to its ruling `Domain` via planetary rulership.
    pub fn from_sign(sign: crate::astrology::Sign) -> Self {
        use crate::astrology::Sign;
        match sign {
            Sign::Aries => Domain::Mangala,
            Sign::Taurus => Domain::Shukra,
            Sign::Gemini => Domain::Budha,
            Sign::Cancer => Domain::Chandra,
            Sign::Leo => Domain::Surya,
            Sign::Virgo => Domain::Budha,
            Sign::Libra => Domain::Shukra,
            Sign::Scorpio => Domain::Mangala,
            Sign::Sagittarius => Domain::Brihaspati,
            Sign::Capricorn => Domain::Shani,
            Sign::Aquarius => Domain::Shani,
            Sign::Pisces => Domain::Brihaspati,
        }
    }

    /// Inherent Guna of this graha.
    pub fn guna(self) -> crate::astrology::Guna {
        use crate::astrology::Guna;
        match self {
            Domain::Surya => Guna::Sattva,
            Domain::Chandra => Guna::Sattva,
            Domain::Mangala => Guna::Rajas,
            Domain::Budha => Guna::Rajas,
            Domain::Brihaspati => Guna::Sattva,
            Domain::Shukra => Guna::Rajas,
            Domain::Shani => Guna::Tamas,
            Domain::Rahu => Guna::Tamas,
            Domain::Ketu => Guna::Sattva,
        }
    }

    /// Parse a domain from a string (name, symbol, or alias).
    pub fn parse(s: &str) -> Option<Self> {
        let lower = s.to_ascii_lowercase();
        match lower.as_str() {
            "surya" | "☉" | "sun" | "alpha" | "α" => Some(Domain::Surya),
            "chandra" | "☽" | "moon" | "beta" | "β" => Some(Domain::Chandra),
            "mangala" | "♂" | "mars" | "gamma" | "γ" => Some(Domain::Mangala),
            "budha" | "☿" | "mercury" | "delta" | "δ" => Some(Domain::Budha),
            "brihaspati" | "♃" | "jupiter" | "epsilon" | "ε" => Some(Domain::Brihaspati),
            "shukra" | "♀" | "venus" | "zeta" | "ζ" => Some(Domain::Shukra),
            "shani" | "♄" | "saturn" | "eta" | "η" => Some(Domain::Shani),
            "rahu" | "☊" | "north node" | "theta" | "θ" => Some(Domain::Rahu),
            "ketu" | "☋" | "south node" | "iota" | "ι" => Some(Domain::Ketu),
            _ => None,
        }
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.symbol(), self.name())
    }
}

impl std::str::FromStr for Domain {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| format!("unknown domain: {s}"))
    }
}

/// A node on the wheel, combining a domain with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub domain: Domain,
    pub symbol: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub index: usize,
    pub opposite: Domain,
    pub trines: [Domain; 2],
}

/// All 9 nodes with metadata.
pub fn compute_all_nodes() -> Vec<Node> {
    ALL_DOMAINS
        .iter()
        .enumerate()
        .map(|(i, &domain)| Node {
            domain,
            symbol: domain.symbol(),
            name: domain.name(),
            description: domain.archetype(),
            index: i,
            opposite: domain.opposite(),
            trines: domain.trines(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_index_roundtrip() {
        for d in &ALL_DOMAINS {
            assert_eq!(Domain::from_index(d.index()), Some(*d));
        }
    }

    #[test]
    fn test_opposite_is_4_steps() {
        for d in &ALL_DOMAINS {
            let opp = d.opposite();
            let diff = (d.index() as isize - opp.index() as isize).abs();
            assert!(diff == 4 || diff == 5);
        }
    }

    #[test]
    fn test_parse_variants() {
        assert_eq!(Domain::parse("surya"), Some(Domain::Surya));
        assert_eq!(Domain::parse("☉"), Some(Domain::Surya));
        assert_eq!(Domain::parse("sun"), Some(Domain::Surya));
        assert_eq!(Domain::parse("mangala"), Some(Domain::Mangala));
        assert_eq!(Domain::parse("MARS"), Some(Domain::Mangala));
        assert_eq!(Domain::parse("rahu"), Some(Domain::Rahu));
        assert_eq!(Domain::parse("unknown"), None);
    }

    #[test]
    fn test_greek_aliases() {
        assert_eq!(Domain::parse("alpha"), Some(Domain::Surya));
        assert_eq!(Domain::parse("β"), Some(Domain::Chandra));
        assert_eq!(Domain::parse("gamma"), Some(Domain::Mangala));
        assert_eq!(Domain::parse("δ"), Some(Domain::Budha));
        assert_eq!(Domain::parse("epsilon"), Some(Domain::Brihaspati));
        assert_eq!(Domain::parse("zeta"), Some(Domain::Shukra));
        assert_eq!(Domain::parse("η"), Some(Domain::Shani));
        assert_eq!(Domain::parse("theta"), Some(Domain::Rahu));
        assert_eq!(Domain::parse("ι"), Some(Domain::Ketu));
    }

    #[test]
    fn test_greek_names() {
        assert_eq!(Domain::Surya.greek_name(), "Alpha (α)");
        assert_eq!(Domain::Chandra.greek_name(), "Beta (β)");
        assert_eq!(Domain::Mangala.greek_name(), "Gamma (γ)");
        assert_eq!(Domain::Budha.greek_name(), "Delta (δ)");
        assert_eq!(Domain::Brihaspati.greek_name(), "Epsilon (ε)");
        assert_eq!(Domain::Shukra.greek_name(), "Zeta (ζ)");
        assert_eq!(Domain::Shani.greek_name(), "Eta (η)");
        assert_eq!(Domain::Rahu.greek_name(), "Theta (θ)");
        assert_eq!(Domain::Ketu.greek_name(), "Iota (ι)");
    }

    #[test]
    fn test_domain_format() {
        let s = format!("{}", Domain::Surya);
        assert!(s.contains("☉"));
        assert!(s.contains("Surya"));
    }

    #[test]
    fn test_adjacent() {
        let adj = Domain::Surya.adjacent();
        assert_eq!(adj[0], Domain::Chandra);
        assert_eq!(adj[1], Domain::Ketu);
    }

    #[test]
    fn test_trines() {
        let trines = Domain::Surya.trines();
        assert_eq!(trines[0], Domain::Budha);
        assert_eq!(trines[1], Domain::Shani);
    }

    #[test]
    fn test_node_metadata() {
        let nodes = compute_all_nodes();
        assert_eq!(nodes.len(), 9);
        for n in &nodes {
            assert!(!n.symbol.is_empty());
            assert!(!n.name.is_empty());
            assert!(!n.description.is_empty());
        }
    }
}
