use std::collections::HashMap;

use super::{AtomClassification, Element, Modality, PlanetaryRuler, Sign};

/// The Change Sorter — classifies tokens across all 7 astrology axes.
#[derive(Debug, Clone)]
pub struct ChangeSorter {
    sign_keywords: HashMap<String, Vec<(Sign, f64)>>,
    element_keywords: HashMap<String, Vec<(Element, f64)>>,
    modality_keywords: HashMap<String, Vec<(Modality, f64)>>,
    ruler_keywords: HashMap<String, Vec<(PlanetaryRuler, f64)>>,
    /// Maps tokens to Tanto-evaluable formal expressions.
    /// When a token maps to a domain, also generate a formal expression for deeper descent.
    formal_expressions: HashMap<String, String>,
}

impl Default for ChangeSorter {
    fn default() -> Self {
        Self::new()
    }
}

impl ChangeSorter {
    pub fn new() -> Self {
        let mut sorter = ChangeSorter {
            sign_keywords: HashMap::new(),
            element_keywords: HashMap::new(),
            modality_keywords: HashMap::new(),
            ruler_keywords: HashMap::new(),
            formal_expressions: HashMap::new(),
        };
        sorter.init_default_mappings();
        sorter
    }

    fn init_default_mappings(&mut self) {
        use Element::*;
        use Modality::*;
        use PlanetaryRuler::*;
        use Sign::*;

        let sign_mappings: [(Sign, &str, f64); 272] = [
            (Aries, "add", 0.8),
            (Aries, "subtract", 0.8),
            (Aries, "multiply", 0.8),
            (Aries, "divide", 0.8),
            (Aries, "math", 0.9),
            (Aries, "logic", 0.8),
            (Aries, "number", 0.7),
            (Aries, "calculate", 0.7),
            (Aries, "prove", 0.6),
            (Aries, "theorem", 0.7),
            (Aries, "axiom", 0.6),
            (Taurus, "force", 0.8),
            (Taurus, "mass", 0.8),
            (Taurus, "energy", 0.8),
            (Taurus, "velocity", 0.7),
            (Taurus, "acceleration", 0.8),
            (Taurus, "physics", 0.9),
            (Taurus, "chemistry", 0.8),
            (Taurus, "atom", 0.6),
            (Taurus, "molecule", 0.6),
            (Taurus, "gravity", 0.7),
            (Taurus, "motion", 0.7),
            (Gemini, "star", 0.8),
            (Gemini, "planet", 0.8),
            (Gemini, "galaxy", 0.8),
            (Gemini, "cosmos", 0.8),
            (Gemini, "astronomy", 0.9),
            (Gemini, "orbit", 0.7),
            (Gemini, "space", 0.7),
            (Gemini, "universe", 0.7),
            (Gemini, "light", 0.6),
            (Cancer, "earth", 0.8),
            (Cancer, "water", 0.7),
            (Cancer, "air", 0.6),
            (Cancer, "climate", 0.8),
            (Cancer, "weather", 0.7),
            (Cancer, "ocean", 0.7),
            (Cancer, "environment", 0.8),
            (Cancer, "ecosystem", 0.7),
            (Cancer, "nature", 0.6),
            (Leo, "cell", 0.7),
            (Leo, "dna", 0.8),
            (Leo, "gene", 0.7),
            (Leo, "protein", 0.7),
            (Leo, "biology", 0.9),
            (Leo, "medicine", 0.8),
            (Leo, "organ", 0.6),
            (Leo, "tissue", 0.6),
            (Leo, "evolution", 0.7),
            (Virgo, "price", 0.7),
            (Virgo, "cost", 0.7),
            (Virgo, "market", 0.8),
            (Virgo, "economy", 0.9),
            (Virgo, "finance", 0.8),
            (Virgo, "money", 0.7),
            (Virgo, "trade", 0.7),
            (Virgo, "budget", 0.6),
            (Virgo, "tax", 0.6),
            (Libra, "engineer", 0.8),
            (Libra, "design", 0.7),
            (Libra, "machine", 0.7),
            (Libra, "circuit", 0.7),
            (Libra, "bridge", 0.6),
            (Libra, "structure", 0.7),
            (Libra, "technology", 0.8),
            (Libra, "system", 0.7),
            (Libra, "mechanics", 0.7),
            (Scorpio, "computer", 0.8),
            (Scorpio, "algorithm", 0.8),
            (Scorpio, "data", 0.7),
            (Scorpio, "code", 0.7),
            (Scorpio, "program", 0.7),
            (Scorpio, "software", 0.7),
            (Scorpio, "ai", 0.8),
            (Scorpio, "intelligence", 0.7),
            (Scorpio, "neural", 0.6),
            (Sagittarius, "history", 0.9),
            (Sagittarius, "culture", 0.7),
            (Sagittarius, "ancient", 0.7),
            (Sagittarius, "civilization", 0.7),
            (Sagittarius, "anthropology", 0.8),
            (Sagittarius, "society", 0.6),
            (Sagittarius, "tradition", 0.6),
            (Sagittarius, "origin", 0.6),
            (Capricorn, "word", 0.7),
            (Capricorn, "language", 0.9),
            (Capricorn, "grammar", 0.8),
            (Capricorn, "linguistics", 0.8),
            (Capricorn, "syntax", 0.7),
            (Capricorn, "semantics", 0.7),
            (Capricorn, "speech", 0.6),
            (Capricorn, "text", 0.6),
            (Aquarius, "truth", 0.7),
            (Aquarius, "ethics", 0.8),
            (Aquarius, "philosophy", 0.9),
            (Aquarius, "moral", 0.7),
            (Aquarius, "justice", 0.7),
            (Aquarius, "virtue", 0.6),
            (Aquarius, "reason", 0.7),
            (Aquarius, "knowledge", 0.7),
            (Pisces, "mind", 0.7),
            (Pisces, "brain", 0.8),
            (Pisces, "psychology", 0.9),
            (Pisces, "neuron", 0.7),
            (Pisces, "consciousness", 0.7),
            (Pisces, "emotion", 0.7),
            (Pisces, "behavior", 0.6),
            (Pisces, "cognition", 0.7),
            // ── Cross-domain: efficiency & performance ──
            (Taurus, "efficiency", 0.8),
            (Scorpio, "performance", 0.8),
            (Scorpio, "optimize", 0.7),
            (Scorpio, "speed", 0.7),
            (Scorpio, "fast", 0.6),
            (Scorpio, "latency", 0.7),
            (Scorpio, "throughput", 0.7),
            (Scorpio, "bottleneck", 0.6),
            (Scorpio, "cache", 0.7),
            (Scorpio, "memory", 0.6),
            (Scorpio, "cpu", 0.7),
            (Scorpio, "disk", 0.6),
            (Scorpio, "io", 0.6),
            // ── Cross-domain: programming terms ──
            (Scorpio, "function", 0.8),
            (Scorpio, "method", 0.7),
            (Scorpio, "class", 0.7),
            (Scorpio, "object", 0.6),
            (Scorpio, "struct", 0.7),
            (Scorpio, "enum", 0.6),
            (Scorpio, "trait", 0.7),
            (Scorpio, "module", 0.7),
            (Scorpio, "package", 0.6),
            (Scorpio, "crate", 0.7),
            (Scorpio, "import", 0.6),
            (Scorpio, "export", 0.6),
            (Scorpio, "return", 0.6),
            (Scorpio, "yield", 0.5),
            (Scorpio, "break", 0.6),
            (Scorpio, "continue", 0.5),
            (Scorpio, "match", 0.7),
            (Scorpio, "if", 0.5),
            (Scorpio, "else", 0.5),
            (Scorpio, "while", 0.6),
            (Scorpio, "for", 0.5),
            (Scorpio, "loop", 0.8),
            (Scorpio, "iterate", 0.7),
            (Scorpio, "recursion", 0.8),
            (Scorpio, "recursive", 0.7),
            (Scorpio, "stack", 0.6),
            (Scorpio, "queue", 0.6),
            (Scorpio, "hash", 0.7),
            (Scorpio, "tree", 0.6),
            (Scorpio, "graph", 0.7),
            (Scorpio, "node", 0.6),
            (Scorpio, "edge", 0.5),
            (Scorpio, "sort", 0.7),
            (Scorpio, "search", 0.7),
            (Scorpio, "traverse", 0.6),
            (Scorpio, "insert", 0.5),
            (Scorpio, "delete", 0.5),
            (Scorpio, "update", 0.5),
            (Scorpio, "query", 0.7),
            (Scorpio, "index", 0.6),
            (Scorpio, "buffer", 0.6),
            (Scorpio, "stream", 0.6),
            (Scorpio, "pipe", 0.5),
            (Scorpio, "filter", 0.6),
            (Scorpio, "reduce", 0.6),
            (Scorpio, "fold", 0.5),
            (Scorpio, "compose", 0.6),
            (Scorpio, "pipeline", 0.7),
            (Scorpio, "closure", 0.7),
            (Scorpio, "callback", 0.6),
            (Scorpio, "async", 0.7),
            (Scorpio, "thread", 0.7),
            (Scorpio, "process", 0.6),
            (Scorpio, "spawn", 0.5),
            (Scorpio, "mutex", 0.6),
            (Scorpio, "lock", 0.5),
            (Scorpio, "atomic", 0.7),
            (Scorpio, "channel", 0.6),
            (Scorpio, "error", 0.6),
            (Scorpio, "panic", 0.5),
            (Scorpio, "result", 0.6),
            (Scorpio, "option", 0.6),
            // ── Cross-domain: science terms ──
            (Taurus, "experiment", 0.7),
            (Aries, "hypothesis", 0.7),
            (Sagittarius, "theory", 0.7),
            (Virgo, "observation", 0.6),
            (Taurus, "measurement", 0.7),
            (Aries, "analysis", 0.7),
            (Taurus, "synthesis", 0.6),
            (Scorpio, "model", 0.7),
            (Scorpio, "simulation", 0.7),
            (Sagittarius, "prediction", 0.6),
            (Scorpio, "validation", 0.7),
            (Taurus, "calibration", 0.6),
            // ── Cross-domain: general terms ──
            (Libra, "system", 0.7),
            (Libra, "process", 0.6),
            (Sagittarius, "approach", 0.6),
            (Libra, "technique", 0.6),
            (Sagittarius, "strategy", 0.6),
            (Scorpio, "pattern", 0.7),
            (Libra, "behavior", 0.6),
            (Scorpio, "state", 0.6),
            (Libra, "transition", 0.5),
            (Sagittarius, "event", 0.6),
            (Aries, "action", 0.6),
            (Taurus, "reaction", 0.5),
            (Sagittarius, "cause", 0.6),
            (Taurus, "effect", 0.5),
            (Scorpio, "result", 0.6),
            (Sagittarius, "outcome", 0.5),
            (Taurus, "impact", 0.5),
            (Taurus, "change", 0.6),
            (Taurus, "improve", 0.6),
            (Taurus, "increase", 0.5),
            (Taurus, "decrease", 0.5),
            (Taurus, "grow", 0.5),
            (Libra, "scale", 0.6),
            (Taurus, "adapt", 0.5),
            (Taurus, "maintain", 0.5),
            (Cancer, "preserve", 0.5),
            (Taurus, "destroy", 0.5),
            (Libra, "create", 0.6),
            (Libra, "build", 0.7),
            (Scorpio, "break", 0.5),
            (Scorpio, "fix", 0.6),
            (Scorpio, "debug", 0.7),
            (Scorpio, "test", 0.7),
            (Libra, "deploy", 0.6),
            (Libra, "monitor", 0.6),
            (Scorpio, "log", 0.5),
            (Scorpio, "trace", 0.6),
            (Scorpio, "profile", 0.6),
            (Scorpio, "benchmark", 0.7),
            (Taurus, "measure", 0.6),
            (Aries, "compare", 0.6),
            (Aries, "evaluate", 0.7),
            (Aries, "assess", 0.6),
            (Sagittarius, "judge", 0.6),
            (Sagittarius, "decide", 0.6),
            (Sagittarius, "choose", 0.5),
            (Scorpio, "select", 0.6),
            (Sagittarius, "want", 0.5),
            (Sagittarius, "need", 0.5),
            (Sagittarius, "require", 0.5),
            (Taurus, "demand", 0.5),
            (Taurus, "supply", 0.5),
            (Taurus, "provide", 0.5),
            (Taurus, "use", 0.5),
            (Taurus, "consume", 0.5),
            (Taurus, "produce", 0.6),
            (Scorpio, "generate", 0.6),
            (Scorpio, "compute", 0.8),
            (Scorpio, "calculate", 0.8),
            (Scorpio, "solve", 0.8),
            (Scorpio, "find", 0.6),
            (Sagittarius, "discover", 0.6),
            (Sagittarius, "learn", 0.7),
            (Sagittarius, "understand", 0.7),
            (Sagittarius, "know", 0.6),
            (Sagittarius, "think", 0.7),
            (Sagittarius, "reason", 0.8),
            (Sagittarius, "argue", 0.6),
            (Aries, "prove", 0.8),
            (Scorpio, "verify", 0.7),
            (Scorpio, "validate", 0.7),
            (Scorpio, "confirm", 0.6),
            (Scorpio, "deny", 0.5),
            (Scorpio, "reject", 0.5),
            (Sagittarius, "accept", 0.5),
            (Sagittarius, "agree", 0.5),
            (Sagittarius, "disagree", 0.5),
        ];

        for (sign, keyword, weight) in &sign_mappings {
            self.sign_keywords
                .entry(keyword.to_string())
                .or_default()
                .push((*sign, *weight));
        }

        let element_mappings: [(Element, &str, f64); 16] = [
            (Fire, "action", 0.7),
            (Fire, "energy", 0.6),
            (Fire, "force", 0.6),
            (Fire, "heat", 0.7),
            (Earth, "structure", 0.7),
            (Earth, "solid", 0.6),
            (Earth, "material", 0.7),
            (Earth, "ground", 0.6),
            (Air, "thought", 0.6),
            (Air, "idea", 0.6),
            (Air, "communication", 0.7),
            (Air, "information", 0.6),
            (Water, "feeling", 0.7),
            (Water, "emotion", 0.7),
            (Water, "flow", 0.6),
            (Water, "intuition", 0.6),
        ];

        for (elem, keyword, weight) in &element_mappings {
            self.element_keywords
                .entry(keyword.to_string())
                .or_default()
                .push((*elem, *weight));
        }

        let modality_mappings: [(Modality, &str, f64); 12] = [
            (Cardinal, "initiate", 0.8),
            (Cardinal, "start", 0.7),
            (Cardinal, "lead", 0.7),
            (Cardinal, "begin", 0.6),
            (Fixed, "stable", 0.7),
            (Fixed, "endure", 0.7),
            (Fixed, "persist", 0.7),
            (Fixed, "steady", 0.6),
            (Mutable, "change", 0.7),
            (Mutable, "adapt", 0.7),
            (Mutable, "flexible", 0.7),
            (Mutable, "transform", 0.6),
        ];

        for (modality, keyword, weight) in &modality_mappings {
            self.modality_keywords
                .entry(keyword.to_string())
                .or_default()
                .push((*modality, *weight));
        }

        let ruler_mappings: [(PlanetaryRuler, &str, f64); 21] = [
            (Sun, "sun", 0.9),
            (Sun, "light", 0.6),
            (Sun, "shine", 0.5),
            (Moon, "moon", 0.9),
            (Moon, "lunar", 0.7),
            (Moon, "night", 0.5),
            (Mercury, "mercury", 0.9),
            (Mercury, "message", 0.5),
            (Mercury, "communicate", 0.6),
            (Venus, "venus", 0.9),
            (Venus, "beauty", 0.6),
            (Venus, "love", 0.5),
            (Mars, "mars", 0.9),
            (Mars, "war", 0.6),
            (Mars, "battle", 0.5),
            (Jupiter, "jupiter", 0.9),
            (Jupiter, "luck", 0.5),
            (Jupiter, "expand", 0.6),
            (Saturn, "saturn", 0.9),
            (Saturn, "time", 0.6),
            (Saturn, "structure", 0.6),
        ];

        for (ruler, keyword, weight) in &ruler_mappings {
            self.ruler_keywords
                .entry(keyword.to_string())
                .or_default()
                .push((*ruler, *weight));
        }

        // ── Formal expressions: token → Tanto-evaluable expression ──
        // These enable descent to reach Formula/NAND layers by providing
        // evaluable expressions for natural language tokens.
        let formal_exprs: [(&str, &str); 40] = [
            ("efficiency", "output / input"),
            ("energy", "mass * velocity^2"),
            ("power", "work / time"),
            ("force", "mass * acceleration"),
            ("velocity", "displacement / time"),
            ("acceleration", "velocity / time"),
            ("momentum", "mass * velocity"),
            ("torque", "force * distance"),
            ("pressure", "force / area"),
            ("density", "mass / volume"),
            ("temperature", "kelvin"),
            ("voltage", "current * resistance"),
            ("current", "voltage / resistance"),
            ("resistance", "voltage / current"),
            ("capacitance", "charge / voltage"),
            ("inductance", "flux / current"),
            ("frequency", "1 / period"),
            ("wavelength", "speed / frequency"),
            ("amplitude", "max_displacement"),
            ("mass", "kg"),
            ("weight", "mass * gravity"),
            ("work", "force * distance"),
            ("kinetic_energy", "0.5 * mass * velocity^2"),
            ("potential_energy", "mass * gravity * height"),
            ("heat", "mass * specific_heat * delta_temperature"),
            ("entropy", "boltzmann * ln(微观状态数)"),
            ("enthalpy", "internal_energy + pressure * volume"),
            ("function", "f(x) = y"),
            ("loop", "while(condition) { body }"),
            ("recursion", "f(n) = f(n-1) + f(n-2)"),
            ("algorithm", "O(f(n))"),
            ("complexity", "O(f(n))"),
            ("memory", "address_space"),
            ("cpu", "clock_cycles"),
            ("network", "bandwidth * latency"),
            ("cache", "hit_rate * total_accesses"),
            ("throughput", "items / time"),
            ("latency", "end_time - start_time"),
            ("bottleneck", "min(throughput_i)"),
            ("cpu_usage", "active_cycles / total_cycles"),
        ];

        for (token, expr) in &formal_exprs {
            self.formal_expressions
                .entry(token.to_string())
                .or_insert_with(|| expr.to_string());
        }
    }

    /// Look up a formal expression for a token.
    /// Returns Tanto-evaluable expression if the token has a known formalization.
    pub fn formal_expression(&self, token: &str) -> Option<&str> {
        self.formal_expressions
            .get(&token.to_lowercase())
            .map(|s| s.as_str())
    }

    pub fn classify_token(&self, token: &str) -> AtomClassification {
        let lower = token.to_lowercase();
        let mut result = AtomClassification::new();

        if let Some(activations) = self.sign_keywords.get(&lower) {
            for &(sign, weight) in activations {
                result.signs[sign.index()] = result.signs[sign.index()].max(weight);
            }
        }

        if let Some(activations) = self.element_keywords.get(&lower) {
            for &(elem, weight) in activations {
                result.elements[elem.index()] = result.elements[elem.index()].max(weight);
            }
        }

        if let Some(activations) = self.modality_keywords.get(&lower) {
            for &(modality, weight) in activations {
                result.modalities[modality.index()] =
                    result.modalities[modality.index()].max(weight);
            }
        }

        if let Some(activations) = self.ruler_keywords.get(&lower) {
            for &(ruler, weight) in activations {
                result.rulers[ruler.index()] = result.rulers[ruler.index()].max(weight);
            }
        }

        result
    }

    pub fn classify_query(&self, query: &str) -> AtomClassification {
        let mut result = AtomClassification::new();
        let tokens: Vec<&str> = query
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty())
            .collect();

        for token in &tokens {
            let token_class = self.classify_token(token);
            result = result.merge_sum(&token_class);
        }

        result
    }

    pub fn is_empty(&self) -> bool {
        self.sign_keywords.is_empty()
            && self.element_keywords.is_empty()
            && self.modality_keywords.is_empty()
            && self.ruler_keywords.is_empty()
    }

    pub fn mapping_count(&self) -> usize {
        self.sign_keywords.len()
            + self.element_keywords.len()
            + self.modality_keywords.len()
            + self.ruler_keywords.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astrology::Sign;

    #[test]
    fn test_classify_math_token() {
        let sorter = ChangeSorter::new();
        let result = sorter.classify_token("math");
        assert!(
            result.signs[Sign::Aries.index()] > 0.5,
            "math should activate Aries"
        );
    }

    #[test]
    fn test_classify_physics_token() {
        let sorter = ChangeSorter::new();
        let result = sorter.classify_token("force");
        assert!(
            result.signs[Sign::Taurus.index()] > 0.5,
            "force should activate Taurus"
        );
    }

    #[test]
    fn test_classify_query() {
        let sorter = ChangeSorter::new();
        let result = sorter.classify_query("calculate kinetic energy");
        assert!(result.signs.iter().any(|&v| v > 0.0));
    }

    #[test]
    fn test_unknown_token() {
        let sorter = ChangeSorter::new();
        let result = sorter.classify_token("xyznonexistent");
        assert_eq!(result, AtomClassification::new());
    }

    #[test]
    fn test_case_insensitive() {
        let sorter = ChangeSorter::new();
        let a = sorter.classify_token("MATH");
        let b = sorter.classify_token("math");
        assert_eq!(a, b);
    }

    #[test]
    fn test_is_empty_after_new() {
        let sorter = ChangeSorter::new();
        assert!(!sorter.is_empty());
    }

    #[test]
    fn test_element_classification() {
        let sorter = ChangeSorter::new();
        let result = sorter.classify_token("action");
        assert!(result.elements[Element::Fire.index()] > 0.5);
    }

    #[test]
    fn test_modality_classification() {
        let sorter = ChangeSorter::new();
        let result = sorter.classify_token("initiate");
        assert!(result.modalities[Modality::Cardinal.index()] > 0.5);
    }

    #[test]
    fn test_ruler_classification() {
        let sorter = ChangeSorter::new();
        let result = sorter.classify_token("mars");
        assert!(result.rulers[PlanetaryRuler::Mars.index()] > 0.5);
    }

    #[test]
    fn test_mapping_count() {
        let sorter = ChangeSorter::new();
        assert!(sorter.mapping_count() > 20);
    }
}
