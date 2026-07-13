//! # Non-math formulas
//!
//! Athena treats grammar rules, code patterns, and logical structures as
//! first-class formulas alongside mathematical expressions.
//!
//! These formulas don't evaluate to numbers — they evaluate to structural
//! validity judgments, transformations, or pattern matches.
//!
//! ## Hierarchy
//!
//! Grammar is the foundation — language is the medium through which math is
//! expressed. Code is executable math expressed in programming languages.
//! Logic is formal reasoning expressed through language.
//!
//! ```text
//!         Code
//!          ↑
//!         Math
//!          ↑
//!        Logic
//!          ↑
//!      Grammar  ← foundation
//! ```

use serde::{Deserialize, Serialize};

/// A grammar rule: a structural pattern in natural language.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRule {
    /// Rule identifier (e.g. "subject_verb_agreement")
    pub id: String,

    /// The pattern this rule matches (regex or structural description).
    pub pattern: String,

    /// The transformation to apply when the pattern is matched.
    pub transform: String,

    /// Human-readable description.
    pub description: String,

    /// Whether this rule is prescriptive or descriptive.
    pub rule_type: GrammarRuleType,

    /// Evidence or source citation.
    pub evidence: Option<String>,

    /// K-12 level on the understanding axis (0 = K, 12 = Grade 12).
    #[serde(default)]
    pub level: u8,

    /// Spiral cycle (0 = first pass, 1 = deeper).
    #[serde(default)]
    pub cycle: u8,
}

/// Types of grammar rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GrammarRuleType {
    /// Prescriptive: what "should" be (e.g., "don't end with preposition")
    Prescriptive,
    /// Descriptive: what is observed in usage
    Descriptive,
    /// Structural: constituency, dependency, phrase structure
    Structural,
}

/// A code pattern: a structural pattern in programming languages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    /// Pattern identifier (e.g. "early_return", "map_vs_for")
    pub id: String,

    /// The language this pattern applies to (or "general").
    pub language: String,

    /// The pattern description or code template.
    pub template: String,

    /// The recommended alternative (if this pattern is an anti-pattern).
    pub alternative: Option<String>,

    /// Complexity score (1-10).
    pub complexity: u8,

    /// Whether this is an anti-pattern.
    pub is_anti_pattern: bool,

    /// K-12 level on the understanding axis (0 = K, 12 = Grade 12).
    #[serde(default)]
    pub level: u8,

    /// Spiral cycle (0 = first pass, 1 = deeper).
    #[serde(default)]
    pub cycle: u8,
}

/// A logical structure: a valid inference pattern or transformation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicPattern {
    /// Pattern identifier (e.g. "modus_ponens", "contrapositive")
    pub id: String,

    /// The logical form expressed as a pattern.
    pub form: String,

    /// Premises for this logical form.
    pub premises: Vec<String>,

    /// Conclusion of this logical form.
    pub conclusion: String,

    /// Whether this is a valid inference.
    pub is_valid: bool,

    /// Counterexample for invalid forms (if any).
    pub counterexample: Option<String>,

    /// K-12 level on the understanding axis (0 = K, 12 = Grade 12).
    #[serde(default)]
    pub level: u8,

    /// Spiral cycle (0 = first pass, 1 = deeper).
    #[serde(default)]
    pub cycle: u8,
}

/// A non-math formula, wrapping grammar/code/logic patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NonMathFormula {
    Grammar(GrammarRule),
    Code(CodePattern),
    Logic(LogicPattern),
}

impl NonMathFormula {
    /// Get the formula identifier.
    pub fn id(&self) -> &str {
        match self {
            NonMathFormula::Grammar(g) => &g.id,
            NonMathFormula::Code(c) => &c.id,
            NonMathFormula::Logic(l) => &l.id,
        }
    }

    /// Get a human-readable description.
    pub fn description(&self) -> &str {
        match self {
            NonMathFormula::Grammar(g) => &g.description,
            NonMathFormula::Code(c) => &c.id,
            NonMathFormula::Logic(l) => &l.form,
        }
    }
}

// ─── NonMathRegistry ────────────────────────────────────────────────────────

/// A registry for non-math formulas: grammar rules, code patterns, and logic patterns.
///
/// Unlike `FormulaRegistry` (which stores evaluable math formulas), this registry
/// stores structural and linguistic patterns. Grammar is the foundation — language
/// is the medium through which all knowledge is expressed.
#[derive(Debug, Clone)]
pub struct NonMathRegistry {
    /// Grammar rules — linguistic structure patterns.
    grammar_rules: Vec<GrammarRule>,
    /// Code patterns — programming language structure patterns.
    code_patterns: Vec<CodePattern>,
    /// Logic patterns — formal inference patterns.
    logic_patterns: Vec<LogicPattern>,
}

impl Default for NonMathRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NonMathRegistry {
    /// Create an empty non-math registry.
    pub fn new() -> Self {
        NonMathRegistry {
            grammar_rules: Vec::new(),
            code_patterns: Vec::new(),
            logic_patterns: Vec::new(),
        }
    }

    // ─── Add ──────────────────────────────────────────────────────────────

    /// Add a grammar rule.
    pub fn add_grammar(&mut self, rule: GrammarRule) {
        self.grammar_rules.push(rule);
    }

    /// Add multiple grammar rules.
    pub fn add_grammar_all(&mut self, rules: Vec<GrammarRule>) {
        self.grammar_rules.extend(rules);
    }

    /// Add a code pattern.
    pub fn add_code(&mut self, pattern: CodePattern) {
        self.code_patterns.push(pattern);
    }

    /// Add multiple code patterns.
    pub fn add_code_all(&mut self, patterns: Vec<CodePattern>) {
        self.code_patterns.extend(patterns);
    }

    /// Add a logic pattern.
    pub fn add_logic(&mut self, pattern: LogicPattern) {
        self.logic_patterns.push(pattern);
    }

    /// Add multiple logic patterns.
    pub fn add_logic_all(&mut self, patterns: Vec<LogicPattern>) {
        self.logic_patterns.extend(patterns);
    }

    // ─── Accessors ────────────────────────────────────────────────────────

    /// Get all grammar rules.
    pub fn grammar_rules(&self) -> &[GrammarRule] {
        &self.grammar_rules
    }

    /// Get all code patterns.
    pub fn code_patterns(&self) -> &[CodePattern] {
        &self.code_patterns
    }

    /// Get all logic patterns.
    pub fn logic_patterns(&self) -> &[LogicPattern] {
        &self.logic_patterns
    }

    /// Total number of non-math formulas.
    pub fn len(&self) -> usize {
        self.grammar_rules.len() + self.code_patterns.len() + self.logic_patterns.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.grammar_rules.is_empty()
            && self.code_patterns.is_empty()
            && self.logic_patterns.is_empty()
    }

    // ─── Search ───────────────────────────────────────────────────────────

    /// Search grammar rules by keyword in ID, pattern, or description.
    pub fn search_grammar(&self, keyword: &str) -> Vec<&GrammarRule> {
        let kw = keyword.to_lowercase();
        self.grammar_rules
            .iter()
            .filter(|r| {
                r.id.to_lowercase().contains(&kw)
                    || r.pattern.to_lowercase().contains(&kw)
                    || r.description.to_lowercase().contains(&kw)
            })
            .collect()
    }

    /// Search code patterns by keyword in ID, template, or language.
    pub fn search_code(&self, keyword: &str) -> Vec<&CodePattern> {
        let kw = keyword.to_lowercase();
        self.code_patterns
            .iter()
            .filter(|p| {
                p.id.to_lowercase().contains(&kw)
                    || p.template.to_lowercase().contains(&kw)
                    || p.language.to_lowercase().contains(&kw)
                    || p.alternative
                        .as_ref()
                        .is_some_and(|a| a.to_lowercase().contains(&kw))
            })
            .collect()
    }

    /// Search logic patterns by keyword in ID, form, or premises.
    pub fn search_logic(&self, keyword: &str) -> Vec<&LogicPattern> {
        let kw = keyword.to_lowercase();
        self.logic_patterns
            .iter()
            .filter(|p| {
                p.id.to_lowercase().contains(&kw)
                    || p.form.to_lowercase().contains(&kw)
                    || p.premises.iter().any(|pr| pr.to_lowercase().contains(&kw))
            })
            .collect()
    }

    /// Search all non-math formulas by keyword.
    pub fn search(&self, keyword: &str) -> Vec<NonMathFormula> {
        let mut results = Vec::new();
        for r in self.search_grammar(keyword) {
            results.push(NonMathFormula::Grammar(r.clone()));
        }
        for p in self.search_code(keyword) {
            results.push(NonMathFormula::Code(p.clone()));
        }
        for p in self.search_logic(keyword) {
            results.push(NonMathFormula::Logic(p.clone()));
        }
        results
    }

    // ─── Filtered queries ─────────────────────────────────────────────────

    /// Get grammar rules filtered by type.
    pub fn grammar_by_type(&self, rule_type: GrammarRuleType) -> Vec<&GrammarRule> {
        self.grammar_rules
            .iter()
            .filter(|r| r.rule_type == rule_type)
            .collect()
    }

    /// Get code patterns filtered by language.
    pub fn code_by_language(&self, language: &str) -> Vec<&CodePattern> {
        let lang = language.to_lowercase();
        self.code_patterns
            .iter()
            .filter(|p| p.language.to_lowercase() == lang || p.language == "general")
            .collect()
    }

    /// Get all anti-patterns.
    pub fn anti_patterns(&self) -> Vec<&CodePattern> {
        self.code_patterns
            .iter()
            .filter(|p| p.is_anti_pattern)
            .collect()
    }

    /// Get all valid logic patterns.
    pub fn valid_logic_patterns(&self) -> Vec<&LogicPattern> {
        self.logic_patterns.iter().filter(|p| p.is_valid).collect()
    }

    /// Get all fallacious logic patterns.
    pub fn fallacious_patterns(&self) -> Vec<&LogicPattern> {
        self.logic_patterns.iter().filter(|p| !p.is_valid).collect()
    }

    /// Load non-math formulas from TOML content.
    ///
    /// Expected TOML sections:
    /// - `[[grammar]]` — GrammarRule entries
    /// - `[[code]]` — CodePattern entries
    /// - `[[logic]]` — LogicPattern entries
    pub fn load_from_toml_str(&mut self, toml_str: &str) -> Result<(), String> {
        #[derive(serde::Deserialize)]
        struct NonMathToml {
            #[allow(dead_code)]
            grammar: Option<Vec<GrammarRule>>,
            #[allow(dead_code)]
            code: Option<Vec<CodePattern>>,
            #[allow(dead_code)]
            logic: Option<Vec<LogicPattern>>,
        }

        let parsed: NonMathToml =
            toml::from_str(toml_str).map_err(|e| format!("TOML parse error: {e}"))?;

        if let Some(rules) = parsed.grammar {
            self.add_grammar_all(rules);
        }
        if let Some(patterns) = parsed.code {
            self.add_code_all(patterns);
        }
        if let Some(patterns) = parsed.logic {
            self.add_logic_all(patterns);
        }

        Ok(())
    }

    /// List all grammar rule IDs.
    pub fn list_grammar_ids(&self) -> Vec<&str> {
        self.grammar_rules.iter().map(|r| r.id.as_str()).collect()
    }

    /// List all code pattern IDs.
    pub fn list_code_ids(&self) -> Vec<&str> {
        self.code_patterns.iter().map(|p| p.id.as_str()).collect()
    }

    /// List all logic pattern IDs.
    pub fn list_logic_ids(&self) -> Vec<&str> {
        self.logic_patterns.iter().map(|p| p.id.as_str()).collect()
    }
}

/// Pre-defined logical patterns.
pub mod standard_patterns {
    use super::LogicPattern;

    pub fn modus_ponens() -> LogicPattern {
        LogicPattern {
            id: "modus_ponens".to_string(),
            form: "If P then Q. P. Therefore Q.".to_string(),
            premises: vec!["If P then Q".to_string(), "P".to_string()],
            conclusion: "Q".to_string(),
            is_valid: true,
            counterexample: None,
            level: 0,
            cycle: 0,
        }
    }

    pub fn modus_tollens() -> LogicPattern {
        LogicPattern {
            id: "modus_tollens".to_string(),
            form: "If P then Q. Not Q. Therefore not P.".to_string(),
            premises: vec!["If P then Q".to_string(), "Not Q".to_string()],
            conclusion: "Not P".to_string(),
            is_valid: true,
            counterexample: None,
            level: 0,
            cycle: 0,
        }
    }

    pub fn affirming_consequent() -> LogicPattern {
        LogicPattern {
            id: "affirming_consequent".to_string(),
            form: "If P then Q. Q. Therefore P.".to_string(),
            premises: vec!["If P then Q".to_string(), "Q".to_string()],
            conclusion: "P".to_string(),
            is_valid: false,
            counterexample: Some("If it rains, the ground is wet. The ground is wet. Therefore it rained. (Could be from sprinklers)".to_string()),
            level: 0,
            cycle: 0,
        }
    }

    pub fn syllogism() -> LogicPattern {
        LogicPattern {
            id: "syllogism".to_string(),
            form: "All A are B. All B are C. Therefore all A are C.".to_string(),
            premises: vec!["All A are B".to_string(), "All B are C".to_string()],
            conclusion: "All A are C".to_string(),
            is_valid: true,
            counterexample: None,
            level: 0,
            cycle: 0,
        }
    }

    pub fn all_standard() -> Vec<LogicPattern> {
        vec![
            modus_ponens(),
            modus_tollens(),
            affirming_consequent(),
            syllogism(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::standard_patterns::*;
    use super::*;

    #[test]
    fn test_modus_ponens_valid() {
        let mp = modus_ponens();
        assert!(mp.is_valid);
        assert_eq!(mp.premises.len(), 2);
    }

    #[test]
    fn test_affirming_consequent_invalid() {
        let ac = affirming_consequent();
        assert!(!ac.is_valid);
        assert!(ac.counterexample.is_some());
    }

    #[test]
    fn test_non_math_formula_wrapper() {
        let mp = modus_ponens();
        let nf = NonMathFormula::Logic(mp);
        assert_eq!(nf.id(), "modus_ponens");
    }

    // ─── NonMathRegistry tests ───────────────────────────────────────

    fn sample_grammar() -> GrammarRule {
        GrammarRule {
            id: "test_rule".to_string(),
            pattern: "\\btest\\b".to_string(),
            transform: "affirm test".to_string(),
            description: "A test rule".to_string(),
            rule_type: GrammarRuleType::Descriptive,
            evidence: None,
            level: 0,
            cycle: 0,
        }
    }

    fn sample_code() -> CodePattern {
        CodePattern {
            id: "test_pattern".to_string(),
            language: "rust".to_string(),
            template: "let x = y.clone();".to_string(),
            alternative: Some("use &y".to_string()),
            complexity: 3,
            is_anti_pattern: true,
            level: 0,
            cycle: 0,
        }
    }

    #[test]
    fn test_nonmath_registry_empty() {
        let r = NonMathRegistry::new();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn test_nonmath_registry_add_grammar() {
        let mut r = NonMathRegistry::new();
        r.add_grammar(sample_grammar());
        assert!(!r.is_empty());
        assert_eq!(r.grammar_rules().len(), 1);
        assert_eq!(r.len(), 1);
        assert_eq!(r.list_grammar_ids(), vec!["test_rule"]);
    }

    #[test]
    fn test_nonmath_registry_add_code() {
        let mut r = NonMathRegistry::new();
        r.add_code(sample_code());
        assert_eq!(r.code_patterns().len(), 1);
        assert_eq!(r.list_code_ids(), vec!["test_pattern"]);
    }

    #[test]
    fn test_nonmath_registry_add_logic() {
        let mut r = NonMathRegistry::new();
        r.add_logic(modus_ponens());
        assert_eq!(r.logic_patterns().len(), 1);
        assert_eq!(r.list_logic_ids(), vec!["modus_ponens"]);
    }

    #[test]
    fn test_nonmath_registry_add_all() {
        let mut r = NonMathRegistry::new();
        r.add_grammar_all(vec![
            GrammarRule {
                id: "rule_a".to_string(),
                pattern: "a".to_string(),
                transform: "a".to_string(),
                description: "A".to_string(),
                rule_type: GrammarRuleType::Prescriptive,
                evidence: None,
                level: 0,
                cycle: 0,
            },
            GrammarRule {
                id: "rule_b".to_string(),
                pattern: "b".to_string(),
                transform: "b".to_string(),
                description: "B".to_string(),
                rule_type: GrammarRuleType::Descriptive,
                evidence: None,
                level: 0,
                cycle: 0,
            },
        ]);
        assert_eq!(r.grammar_rules().len(), 2);
    }

    #[test]
    fn test_nonmath_registry_search_grammar() {
        let mut r = NonMathRegistry::new();
        r.add_grammar(sample_grammar());
        let found = r.search_grammar("test");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, "test_rule");
        let none = r.search_grammar("nonexistent");
        assert!(none.is_empty());
    }

    #[test]
    fn test_nonmath_registry_search_code() {
        let mut r = NonMathRegistry::new();
        r.add_code(sample_code());
        let found = r.search_code("clone");
        assert_eq!(found.len(), 1);
        let found_lang = r.search_code("rust");
        assert_eq!(found_lang.len(), 1);
        let found_alt = r.search_code("&y");
        assert_eq!(found_alt.len(), 1);
    }

    #[test]
    fn test_nonmath_registry_search_logic() {
        let mut r = NonMathRegistry::new();
        r.add_logic(modus_ponens());
        let found = r.search_logic("modus_ponens");
        assert_eq!(found.len(), 1);
        let found_form = r.search_logic("Therefore Q");
        assert_eq!(found_form.len(), 1);
    }

    #[test]
    fn test_nonmath_registry_unified_search() {
        let mut r = NonMathRegistry::new();
        r.add_grammar(sample_grammar()); // id="test_rule", pattern matches "test"
        r.add_code(sample_code()); // id="test_pattern" matches "test"
        r.add_logic(modus_ponens()); // no match
        let results = r.search("test");
        assert_eq!(results.len(), 2); // grammar + code match "test"
        let all = r.search("");
        assert_eq!(all.len(), 3); // empty keyword matches everything
    }

    #[test]
    fn test_nonmath_registry_grammar_by_type() {
        let mut r = NonMathRegistry::new();
        r.add_grammar(GrammarRule {
            id: "presc".to_string(),
            pattern: "".to_string(),
            transform: "".to_string(),
            description: "".to_string(),
            rule_type: GrammarRuleType::Prescriptive,
            evidence: None,
            level: 0,
            cycle: 0,
        });
        r.add_grammar(GrammarRule {
            id: "desc".to_string(),
            pattern: "".to_string(),
            transform: "".to_string(),
            description: "".to_string(),
            rule_type: GrammarRuleType::Descriptive,
            evidence: None,
            level: 0,
            cycle: 0,
        });
        r.add_grammar(GrammarRule {
            id: "struct".to_string(),
            pattern: "".to_string(),
            transform: "".to_string(),
            description: "".to_string(),
            rule_type: GrammarRuleType::Structural,
            evidence: None,
            level: 0,
            cycle: 0,
        });
        assert_eq!(r.grammar_by_type(GrammarRuleType::Prescriptive).len(), 1);
        assert_eq!(r.grammar_by_type(GrammarRuleType::Descriptive).len(), 1);
        assert_eq!(r.grammar_by_type(GrammarRuleType::Structural).len(), 1);
        assert_eq!(
            r.grammar_by_type(GrammarRuleType::Prescriptive)[0].id,
            "presc"
        );
    }

    #[test]
    fn test_nonmath_registry_code_filters() {
        let mut r = NonMathRegistry::new();
        let good = CodePattern {
            id: "good".to_string(),
            language: "rust".to_string(),
            template: "".to_string(),
            alternative: None,
            complexity: 1,
            is_anti_pattern: false,
            level: 0,
            cycle: 0,
        };
        let bad = CodePattern {
            id: "bad".to_string(),
            language: "python".to_string(),
            template: "".to_string(),
            alternative: Some("fix".to_string()),
            complexity: 5,
            is_anti_pattern: true,
            level: 0,
            cycle: 0,
        };
        r.add_code_all(vec![good, bad]);
        let anti = r.anti_patterns();
        assert_eq!(anti.len(), 1);
        assert_eq!(anti[0].id, "bad");
        let rust_patterns = r.code_by_language("rust");
        assert_eq!(rust_patterns.len(), 1);
    }

    #[test]
    fn test_nonmath_registry_logic_filters() {
        let mut r = NonMathRegistry::new();
        r.add_logic(modus_ponens());
        r.add_logic(affirming_consequent());
        assert_eq!(r.valid_logic_patterns().len(), 1);
        assert_eq!(r.fallacious_patterns().len(), 1);
    }

    #[test]
    fn test_nonmath_registry_load_from_toml() {
        let toml_str = r#"
        [[grammar]]
        id = "test_grammar"
        pattern = "\\btest\\b"
        transform = "affirm"
        description = "Test grammar"
        rule_type = "descriptive"

        [[code]]
        id = "test_code"
        language = "rust"
        template = "x.clone()"
        alternative = "use &x"
        complexity = 2
        is_anti_pattern = true

        [[logic]]
        id = "test_logic"
        form = "If A then B. A. Therefore B."
        premises = ["If A then B", "A"]
        conclusion = "B"
        is_valid = true
        "#;
        let mut r = NonMathRegistry::new();
        r.load_from_toml_str(toml_str).unwrap();
        assert_eq!(r.grammar_rules().len(), 1);
        assert_eq!(r.code_patterns().len(), 1);
        assert_eq!(r.logic_patterns().len(), 1);
        assert_eq!(r.len(), 3);
        assert_eq!(r.list_grammar_ids(), vec!["test_grammar"]);
    }
}
