//! # Laverna Self-Classification Test
//!
//! Meta-test: Laverna classifies queries about itself.
//! Proves the classifier can reason about its own operation
//! without recursion issues — describing the classifier ≠ running it.
//!
//! Based on Athena's self-classification analysis (8 findings).
//!
//! Run: `cargo test --test self_classification -- --nocapture`

use laverna::prelude::*;

const SELF_QUERIES: &[(&str, &str, &str)] = &[
    // Core philosophy
    ("energy efficiency in code", "general", "Shukra"),
    ("NAND gate minimalism", "general", ""),
    ("deterministic code", "general", "Mangala"),
    // Technical descriptions
    ("binary size 605KB", "general", ""),
    ("memory footprint minimal", "general", "Brihaspati"),
    ("startup time < 1ms", "general", "Budha"),
    // Classification about classification
    ("validate this formula", "validation", "Mangala"),
    ("compute the answer", "computation", "Mangala"),
    // Self-referential (classifier describing itself)
    ("pattern matching intent routing", "general", "Mangala"),
    ("keyword extraction pipeline", "general", "Shani"),
    // Advanced syntax
    ("validate(memory < 1MB)", "validation", "Brihaspati"),
    ("search(optimization)", "search", "Mangala"),
    // Philosophical
    ("no invented scalars", "general", ""),
    ("errors corrected on record", "general", "Mangala"),
];

#[test]
fn self_classification_all_queries() {
    let formula_reg = load_formula_registry();
    let entity_reg = load_entity_registry();
    let _forms = ShikaiFormRegistry::new();
    let _events = EventRegistry::new();

    let descent_engine = DescentEngine::new(
        formula_reg.clone(),
        entity_reg.clone(),
        _forms.clone(),
        _events.clone(),
    );

    let mut passed = 0;
    let mut failed = 0;

    println!("\n═══ Self-Classification Test ═══════════════════════════════");
    println!(
        "  Testing {} queries about Laverna itself",
        SELF_QUERIES.len()
    );
    println!("═══════════════════════════════════════════════════════════════\n");

    for (query, expected_intent, expected_domain) in SELF_QUERIES {
        let intent = laverna::shikai::parse_query_intent(query);
        let domain = laverna::shikai::determine_query_domain(query);
        let matrix = descent_engine.descend(query);

        let intent_ok = intent == *expected_intent;
        let _domain_str = domain;
        let domain_ok = if expected_domain.is_empty() {
            matrix.dominant_domains.is_empty()
        } else {
            matrix
                .dominant_domains
                .first()
                .map(|d| format!("{:?}", d) == *expected_domain)
                .unwrap_or(false)
        };

        let status = if intent_ok && domain_ok {
            passed += 1;
            "✅"
        } else {
            failed += 1;
            "❌"
        };

        println!("  {} \"{}\"", status, query);
        if !intent_ok {
            println!(
                "       intent: got '{}' expected '{}'",
                intent, expected_intent
            );
        }
        if !domain_ok {
            println!(
                "       domain: got {:?} expected '{}'",
                matrix.dominant_domains, expected_domain
            );
        }
    }

    println!("\n═══════════════════════════════════════════════════════════════");
    println!(
        "  Results: {}/{} passed, {} failed",
        passed,
        SELF_QUERIES.len(),
        failed
    );
    println!("═══════════════════════════════════════════════════════════════\n");

    assert!(failed == 0, "{} self-classification queries failed", failed);
}

#[test]
fn self_classification_determinism() {
    let formula_reg = load_formula_registry();
    let entity_reg = load_entity_registry();
    let _forms = ShikaiFormRegistry::new();
    let _events = EventRegistry::new();

    let descent_engine = DescentEngine::new(
        formula_reg.clone(),
        entity_reg.clone(),
        _forms.clone(),
        _events.clone(),
    );

    // Run the same query 3 times — must produce identical results
    let query = "energy efficiency in code";
    let runs: Vec<_> = (0..3)
        .map(|_| {
            let matrix = descent_engine.descend(query);
            matrix
                .tokens
                .iter()
                .map(|t| {
                    (
                        t.text.clone(),
                        t.settled_layer.depth(),
                        t.domains.clone(),
                        t.entity.clone(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();

    for i in 1..runs.len() {
        assert_eq!(
            runs[0], runs[i],
            "Run 0 and run {} differ — determinism broken!",
            i
        );
    }

    println!("✅ Determinism verified: 3 runs of \"{}\" identical", query);
}

#[test]
fn self_classification_provenance_chain() {
    let formula_reg = load_formula_registry();
    let entity_reg = load_entity_registry();
    let _forms = ShikaiFormRegistry::new();
    let _events = EventRegistry::new();

    let descent_engine = DescentEngine::new(
        formula_reg.clone(),
        entity_reg.clone(),
        _forms.clone(),
        _events.clone(),
    );

    let matrix = descent_engine.descend("energy efficiency in code");

    println!("\n═══ Provenance Chains ══════════════════════════════════════\n");

    for token in &matrix.tokens {
        println!(
            "  \"{}\" → {} provenance steps:",
            token.text,
            token.provenance.len()
        );
        for (i, step) in token.provenance.iter().enumerate() {
            match step {
                ProvenanceStep::DomainClassification {
                    domain,
                    keyword,
                    confidence,
                } => {
                    println!(
                        "    {}. DomainClassification: domain={}, keyword={}, confidence={:.1}",
                        i + 1,
                        domain,
                        keyword,
                        confidence
                    );
                }
                ProvenanceStep::FormulaMatch {
                    formula_id,
                    domain,
                    inputs,
                    output,
                } => {
                    println!(
                        "    {}. FormulaMatch: id={}, domain={}, inputs={:?} → {}",
                        i + 1,
                        formula_id,
                        domain,
                        inputs,
                        output
                    );
                }
                ProvenanceStep::EntityMatch { entity_id, domain } => {
                    println!(
                        "    {}. EntityMatch: id={}, domain={}",
                        i + 1,
                        entity_id,
                        domain
                    );
                }
                ProvenanceStep::Unification {
                    formula_id,
                    entity_id,
                    bound_inputs,
                    output_value,
                } => {
                    println!(
                        "    {}. Unification: formula={}, entity={}, bound={:?}, output={:?}",
                        i + 1,
                        formula_id,
                        entity_id,
                        bound_inputs,
                        output_value
                    );
                }
                ProvenanceStep::FormalExpression { token, expression } => {
                    println!(
                        "    {}. FormalExpression: token={}, expr=\"{}\"",
                        i + 1,
                        token,
                        expression
                    );
                }
                ProvenanceStep::NandEvaluation {
                    gate,
                    inputs,
                    output,
                } => {
                    println!(
                        "    {}. NandEvaluation: gate={}, inputs={:?}, output={}",
                        i + 1,
                        gate,
                        inputs,
                        output
                    );
                }
            }
        }
        println!();
    }

    // Verify all tokens have provenance
    for token in &matrix.tokens {
        assert!(
            !token.provenance.is_empty(),
            "Token '{}' has no provenance — traceability broken!",
            token.text
        );
    }

    println!("✅ All tokens have provenance chains — traceability verified");
}

fn load_formula_registry() -> FormulaRegistry {
    let mut reg = FormulaRegistry::new();
    let paths = [
        "formulas/astrology.toml",
        "formulas/computation.toml",
        "formulas/science.toml",
        "formulas/psychology.toml",
        "formulas/economics.toml",
        "formulas/technology.toml",
    ];
    for path in &paths {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
        if reg.load_from_file(&p).is_ok() {
            println!("  Loaded formulas from {}", path);
        }
    }
    reg
}

fn load_entity_registry() -> EntityRegistry {
    let mut reg = EntityRegistry::new();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("entities");
    if dir.exists() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().map_or(false, |e| e == "toml") {
                let _ = reg.load_seeds_from_file(&path);
            }
        }
    }
    reg
}
