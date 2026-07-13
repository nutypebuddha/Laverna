/// Pure function: Validate a formula ID string.
pub fn validate_formula_id(formula_id: &str) -> bool {
    !formula_id.is_empty() && formula_id.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Pure function: Extract domain from a formula ID.
pub fn extract_formula_domain(formula_id: &str) -> &str {
    formula_id.split('_').next().unwrap_or("")
}

/// Pure function: Check if formula ID is atomic type.
pub fn is_atomic_formula(formula_id: &str) -> bool {
    extract_formula_domain(formula_id) == "atomic"
}

/// Pure function: Check if formula ID is bridging type.
pub fn is_bridging_formula(formula_id: &str) -> bool {
    extract_formula_domain(formula_id) == "bridging"
}

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::wheel::Domain;

mod glyph;
pub mod nonmath;
mod registry;

pub use glyph::{
    apply_operator, binding_power, decompose_bound, is_bound, Glyph, GlyphOperator, GlyphResult,
    NamedGlyph, GLYPH_COUNT, MAX_GLYPH, OPERATOR_TABLE,
};
pub use registry::FormulaRegistry;

/// Errors from formula operations.
#[derive(Error, Debug)]
pub enum FormulaError {
    #[error("formula not found: {0}")]
    NotFound(String),

    #[error("argument error: {0}")]
    ArgError(String),

    #[error("evaluation error: {0}")]
    EvalError(String),

    #[error("serialization error: {0}")]
    SerdeError(String),
}

/// The type of a formula — determines how it's evaluated and verified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormulaType {
    /// Math gate: evaluable arithmetic expression over f64.
    /// Example: `a + b`, `sqrt(x)`
    /// Proven by: Peano arithmetic / real field axioms
    Math,

    /// Logic gate: boolean operation over {0, 1}.
    /// Example: `1 - a*b` (NAND), `a*b` (AND)
    /// Proven by: truth table
    Logic,

    /// LLM gate: the capstone model estimates the output from a prompt
    /// template; deterministic gates validate the raw generation before a
    /// value is admitted. `expression` holds the prompt with `{input}`
    /// placeholders, not evaluable arithmetic.
    /// Proven by: nothing — gated, never trusted, confidence-penalized.
    Llm,
}

impl FormulaType {
    /// Human-readable description.
    pub fn description(self) -> &'static str {
        match self {
            FormulaType::Math => "Math gate — arithmetic operation",
            FormulaType::Logic => "Logic gate — boolean operation",
            FormulaType::Llm => "LLM gate — capstone estimation, gated",
        }
    }
}

/// A single primitive formula definition.
///
/// Every formula is a pure primitive operation with typed inputs,
/// a single output, and an evaluable expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Formula {
    /// Unique identifier (e.g. "add", "nand", "sqrt")
    pub id: String,

    /// The type of formula (math gate or logic gate)
    pub formula_type: FormulaType,

    /// The domain this formula belongs to.
    /// Math gates → Aries (Math & Logic).
    /// Logic gates → Scorpio (CS & AI).
    pub domain: Domain,

    /// Input parameter names.
    pub inputs: Vec<String>,

    /// Output parameter name.
    pub output: String,

    /// The evaluable expression using Azauchi syntax (src/asauchi).
    /// For math: arithmetic expression like "a + b", "sqrt(x)"
    /// For logic: boolean expression like "1 - a*b" (NAND)
    pub expression: String,

    /// Human-readable description.
    pub description: String,

    /// Zodiac symbol associations for the wheel classifier.
    /// These replace the old string tags and are used by Zanpakuto
    /// to match tokens to domains on the wheel.
    #[serde(default)]
    pub zodiac: Vec<String>,

    /// Optional evidence or source for this formula's provability.
    pub evidence: Option<String>,

    /// Additional domains this formula is also registered under.
    ///
    /// Populated when a later-loaded formula file declares the same `id`
    /// with a different `domain` (e.g. `aries_math.toml`'s "add" vs.
    /// `mangala_math.toml`'s "add" — same content, dual Western/Vedic
    /// cosmology). Without this, `FormulaRegistry::register` used to
    /// silently overwrite the earlier domain tag on collision.
    #[serde(default)]
    pub also_domains: Vec<Domain>,

    /// Source domain for bridging formulas (parsed from TOML `from` field).
    /// Bridging formulas connect `from_domain` → `domain`.
    #[serde(default)]
    pub from_domain: Option<Domain>,

    /// Target domain for bridging formulas (parsed from TOML `to` field).
    #[serde(default)]
    pub to_domain: Option<Domain>,

    /// Aspect between domains for bridging formulas (parsed from TOML `aspect` field).
    #[serde(default)]
    pub bridge_aspect: Option<String>,
}

impl Formula {
    /// Create a new primitive formula.
    pub fn new(
        id: &str,
        formula_type: FormulaType,
        domain: Domain,
        inputs: Vec<&str>,
        output: &str,
        expression: &str,
        description: &str,
    ) -> Self {
        Formula {
            id: id.to_string(),
            formula_type,
            domain,
            inputs: inputs.into_iter().map(String::from).collect(),
            output: output.to_string(),
            expression: expression.to_string(),
            description: description.to_string(),
            zodiac: Vec::new(),
            evidence: None,
            also_domains: Vec::new(),
            from_domain: None,
            to_domain: None,
            bridge_aspect: None,
        }
    }

    /// Create a math gate formula (convenience constructor).
    pub fn math(
        id: &str,
        inputs: Vec<&str>,
        output: &str,
        expression: &str,
        description: &str,
    ) -> Self {
        Formula::new(
            id,
            FormulaType::Math,
            Domain::Mangala,
            inputs,
            output,
            expression,
            description,
        )
    }

    /// Create a logic gate formula (convenience constructor).
    pub fn logic(
        id: &str,
        inputs: Vec<&str>,
        output: &str,
        expression: &str,
        description: &str,
    ) -> Self {
        Formula::new(
            id,
            FormulaType::Logic,
            Domain::Budha,
            inputs,
            output,
            expression,
            description,
        )
    }

    /// Create an atomic formula (convenience, equivalent to `math` with explicit domain).
    /// This preserves the API used throughout the codebase and tests.
    pub fn atomic(
        id: &str,
        domain: Domain,
        inputs: Vec<&str>,
        output: &str,
        expression: &str,
        description: &str,
    ) -> Self {
        Formula::new(
            id,
            FormulaType::Math,
            domain,
            inputs,
            output,
            expression,
            description,
        )
    }

    /// Create a math gate with zodiac associations.
    pub fn with_zodiac(mut self, zodiac: Vec<&str>) -> Self {
        self.zodiac = zodiac.into_iter().map(String::from).collect();
        self
    }

    /// Attach evidence to this formula.
    pub fn with_evidence(mut self, evidence: &str) -> Self {
        self.evidence = Some(evidence.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_formula_id_basic() {
        assert!(validate_formula_id("atomic_graha"));
        assert!(validate_formula_id("bridging_001"));
        assert!(!validate_formula_id(""));
        assert!(!validate_formula_id("has space"));
    }

    #[test]
    fn extract_formula_domain_basic() {
        assert_eq!(extract_formula_domain("atomic_graha"), "atomic");
        assert_eq!(extract_formula_domain("bridging_001"), "bridging");
    }

    #[test]
    fn is_atomic_formula_basic() {
        assert!(is_atomic_formula("atomic_graha"));
        assert!(!is_atomic_formula("bridging_001"));
    }

    #[test]
    fn is_bridging_formula_basic() {
        assert!(is_bridging_formula("bridging_001"));
        assert!(!is_bridging_formula("atomic_graha"));
    }

    #[test]
    fn test_create_math_formula() {
        let f = Formula::math("add", vec!["a", "b"], "sum", "a + b", "Addition");
        assert_eq!(f.formula_type, FormulaType::Math);
        assert_eq!(f.domain, Domain::Mangala);
        assert_eq!(f.id, "add");
        assert_eq!(f.inputs, vec!["a", "b"]);
        assert_eq!(f.output, "sum");
    }

    #[test]
    fn test_create_logic_formula() {
        let f = Formula::logic("nand", vec!["a", "b"], "out", "1 - a*b", "NAND gate");
        assert_eq!(f.formula_type, FormulaType::Logic);
        assert_eq!(f.domain, Domain::Budha);
        assert_eq!(f.output, "out");
    }

    #[test]
    fn test_formula_with_zodiac() {
        let f = Formula::math("add", vec!["a", "b"], "sum", "a + b", "Addition")
            .with_zodiac(vec!["♈", "aries", "math"]);
        assert_eq!(f.zodiac.len(), 3);
        assert!(f.zodiac.contains(&"♈".to_string()));
    }

    #[test]
    fn test_formula_with_evidence() {
        let f = Formula::math("add", vec!["a", "b"], "sum", "a + b", "Addition")
            .with_evidence("Peano arithmetic: S(a) + b = S(a + b)");
        assert!(f.evidence.is_some());
    }

    #[test]
    fn test_formula_type_description() {
        assert!(FormulaType::Math.description().contains("Math"));
        assert!(FormulaType::Logic.description().contains("Logic"));
    }
}
