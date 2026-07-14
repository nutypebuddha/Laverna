//! # Laverna Hands-On: Energy Efficiency in Code
//!
//! Integration test exercising ALL Laverna subsystems against the query
//! `"energy efficiency in code"`. Each phase exercises a different module.
//!
//! Run: `cargo test --test energy_efficiency_integration -- --nocapture`

use laverna::prelude::*;
use laverna::*;
use std::collections::HashMap;
use std::path::Path;

const QUERY: &str = "energy efficiency in code";

fn load_formula_registry() -> FormulaRegistry {
    let mut reg = FormulaRegistry::new();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("formulas");
    if dir.exists() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().map_or(false, |e| e == "toml") {
                reg.load_from_file(&path)
                    .unwrap_or_else(|e| panic!("Failed to load {}: {}", path.display(), e));
            }
        }
    }
    reg
}

fn load_entity_registries() -> (EntityRegistry, ShikaiFormRegistry, EventRegistry) {
    let mut entities = EntityRegistry::new();
    let forms = ShikaiFormRegistry::new();
    let events = EventRegistry::new();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("entities");
    if dir.exists() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().map_or(false, |e| e == "toml") {
                entities
                    .load_seeds_from_file(&path)
                    .unwrap_or_else(|e| panic!("Failed to load {}: {}", path.display(), e));
            }
        }
    }
    (entities, forms, events)
}

#[test]
fn energy_efficiency_in_code_full_analysis() {
    let mut report = String::new();
    report.push_str("═══════════════════════════════════════════════════════════\n");
    report.push_str("  LAVERNA: Energy Efficiency in Code — Full Analysis\n");
    report.push_str("═══════════════════════════════════════════════════════════\n\n");
    report.push_str(&format!("Query: \"{}\"\n\n", QUERY));

    // ── Phase 1: Data Loading ─────────────────────────────────────────────
    report.push_str("─── Phase 1: Data Loading ───────────────────────────────\n");
    let formula_reg = load_formula_registry();
    let (entity_reg, _forms, _events) = load_entity_registries();
    let formula_count = formula_reg.len();
    let seed_count = entity_reg.seeds().count();
    report.push_str(&format!("  Formulas loaded: {}\n", formula_count));
    report.push_str(&format!("  Seed entities loaded: {}\n\n", seed_count));
    assert!(formula_count > 100, "Should have loaded 100+ formulas");
    assert!(seed_count > 100, "Should have loaded 100+ seed entities");

    // ── Phase 2: Token Classification ─────────────────────────────────────
    report.push_str("─── Phase 2: Token Classification (ChangeSorter) ────────\n");
    let sorter = ChangeSorter::new();
    let test_tokens = vec!["energy", "efficiency", "code", "function", "loop"];
    let mut classifications = Vec::new();
    for token in &test_tokens {
        let class = sorter.classify_token(token);
        let dominant_sign = class
            .dominant_sign()
            .map(|s| format!("{:?}", s))
            .unwrap_or_else(|| "none".to_string());
        let dominant_element = class
            .dominant_element()
            .map(|e| e.name().to_string())
            .unwrap_or_else(|| "none".to_string());
        let dominant_graha = class
            .vedic
            .dominant_graha()
            .map(|g| format!("{}", g.name()))
            .unwrap_or_else(|| "none".to_string());
        let dominant_guna = class
            .vedic
            .dominant_guna()
            .map(|g| g.name().to_string())
            .unwrap_or_else(|| "none".to_string());
        report.push_str(&format!(
            "  \"{}\" → sign: {}, element: {}, graha: {}, guṇa: {}\n",
            token, dominant_sign, dominant_element, dominant_graha, dominant_guna
        ));
        classifications.push((token.to_string(), class));
    }
    report.push_str("\n");

    // ── Phase 3: Descent Engine ───────────────────────────────────────────
    report.push_str("─── Phase 3: Descent Engine (7-Layer Pipeline) ──────────\n");
    let descent_engine = DescentEngine::new(
        formula_reg.clone(),
        entity_reg.clone(),
        _forms.clone(),
        _events.clone(),
    );
    let matrix = descent_engine.descend(QUERY);
    report.push_str(&format!(
        "  Tokens: {} | Resolution: {:.1}% | Avg Depth: {:.2}/6\n",
        matrix.tokens.len(),
        matrix.resolution_score * 100.0,
        matrix.average_depth
    ));
    report.push_str(&format!(
        "  NAND completeness: {:.1}% | Shikai focus: {:.1}%\n",
        matrix.nand_completeness() * 100.0,
        matrix.shikai_focus() * 100.0
    ));
    for token in &matrix.tokens {
        report.push_str(&format!(
            "    \"{}\" → layer: {} (depth {}), confidence: {:.2}, domains: {:?}, formulas: {}, entity: {:?}",
            token.text,
            token.settled_layer.name(),
            token.settled_layer.depth(),
            token.confidence,
            token.domains,
            token.formulas.len(),
            token.entity
        ));
        if let Some(ref expr) = token.formal_expression {
            report.push_str(&format!(", formal: \"{}\"", expr));
        }
        if !token.provenance.is_empty() {
            report.push_str(&format!(", provenance: {} steps", token.provenance.len()));
        }
        report.push_str("\n");
    }
    report.push_str(&format!(
        "  Dominant domains: {:?}\n",
        matrix.dominant_domains
    ));
    report.push_str(&format!("  Layer histogram: {:?}\n", matrix.layer_counts));
    report.push_str("\n");

    // ── Phase 4: Formula Search + Tanto Math ──────────────────────────────
    report.push_str("─── Phase 4: Formula Search + Tanto Math Engine ─────────\n");
    let energy_formulas: Vec<_> = formula_reg
        .all()
        .into_iter()
        .filter(|f| {
            f.zodiac.iter().any(|t| t.contains("energy"))
                || f.id.contains("energy")
                || f.expression.contains("energy")
        })
        .collect();
    report.push_str(&format!(
        "  Formulas matching 'energy': {}\n",
        energy_formulas.len()
    ));
    for f in energy_formulas.iter().take(5) {
        report.push_str(&format!(
            "    {} [{}]: {} → {} = {}\n",
            f.id,
            f.domain.name(),
            f.inputs.join(", "),
            f.output,
            f.expression
        ));
    }

    let env = tanto::create_env();
    if let Some(result) = tanto::evaluate_expr("2 + 2", &env) {
        report.push_str(&format!("  Tanto: 2 + 2 = {}\n", result));
    }
    if let Some(result) = tanto::evaluate_expr("0.85 * 100", &env) {
        report.push_str(&format!(
            "  Tanto: 0.85 * 100 = {} (efficiency %)\n",
            result
        ));
    }
    if let Some(result) = tanto::evaluate_expr("1000 / 0.85", &env) {
        report.push_str(&format!(
            "  Tanto: 1000 / 0.85 = {:.2} (total input for 1000 useful output)\n",
            result
        ));
    }

    if let Some(result) = tanto::compute_formula(
        "conservation_energy",
        "ke_initial=50 pe_initial=30 ke_final=40 pe_final=40",
    ) {
        report.push_str(&format!(
            "  conservation_energy: {} = {} (args: {:?})\n",
            result.name, result.result, result.args
        ));
    }
    report.push_str("\n");

    // ── Phase 5: Entity Lookup ────────────────────────────────────────────
    report.push_str("─── Phase 5: Entity Registry Lookup ─────────────────────\n");
    let energy_seeds: Vec<_> = entity_reg
        .seeds()
        .filter(|s| {
            s.tags
                .iter()
                .any(|t| t.contains("energy") || t.contains("efficiency"))
                || s.id.contains("energy")
        })
        .collect();
    report.push_str(&format!(
        "  Seed entities matching 'energy': {}\n",
        energy_seeds.len()
    ));
    for seed in energy_seeds.iter().take(5) {
        report.push_str(&format!(
            "    {} [{}]: {} (graha: {:?})\n",
            seed.id,
            seed.tags.join(", "),
            seed.description.chars().take(60).collect::<String>(),
            seed.dominant_graha()
        ));
    }

    let mangala_entities = entity_reg.seeds_by_graha(Domain::Mangala);
    report.push_str(&format!(
        "  Entities ruled by Mangala (CS/Math): {}\n",
        mangala_entities.len()
    ));
    for seed in mangala_entities.iter().take(3) {
        report.push_str(&format!("    {}\n", seed.id));
    }
    report.push_str("\n");

    // ── Phase 6: Chart + Personality ──────────────────────────────────────
    report.push_str("─── Phase 6: Chart Snapshot + Personality ───────────────\n");
    let jd = ephemeris::julian_day(2026, 7, 13, 12.0);
    let chart = ChartSnapshot::new(jd)
        .with_location(28.6139, 77.2090)
        .with_label("energy_analysis");
    report.push_str(&format!("  Julian Day: {:.4}\n", jd));
    report.push_str(&format!(
        "  Graha positions: {}\n",
        chart.graha_positions.len()
    ));
    for pos in &chart.graha_positions {
        report.push_str(&format!(
            "    {} ({}) → {:.2}° (rashi: {})\n",
            pos.graha.name(),
            pos.graha.symbol(),
            pos.tropical,
            pos.rashi.name()
        ));
    }
    if let Some(lagna) = chart.lagna {
        report.push_str(&format!("  Lagna (Ascendant): {}\n", lagna.name()));
    }

    report.push_str(&format!("  Aspect pairs: {}\n", chart.aspect_matrix.len()));

    let personality = derive_personality(&chart);
    report.push_str(&format!(
        "  Dominant pillar: {} ({})\n",
        personality.dominant.name(),
        personality.dominant.description()
    ));
    report.push_str("  All pillar weights:\n");
    for i in 0..7 {
        let pillar = Pillar::from_index(i);
        let w = personality.pillar_weights[pillar.index()];
        let bar = "#".repeat((w * 40.0) as usize);
        report.push_str(&format!("    {:12} {:5.2} {}\n", pillar.name(), w, bar));
    }

    let watch = personality.archetype;
    report.push_str(&format!(
        "  Watch archetype: {} ({})\n\n",
        watch.name(),
        watch.solver_hint()
    ));

    // ── Phase 7: Wheel Graph ──────────────────────────────────────────────
    report.push_str("─── Phase 7: Wheel Graph Pathfinding ────────────────────\n");
    let graph = WheelGraph::new();
    let path = graph
        .shortest_path(Domain::Mangala, Domain::Shukra)
        .unwrap_or_default();
    report.push_str(&format!(
        "  Shortest path Mangala→Shukra: {}\n",
        path.iter()
            .map(|d| d.name())
            .collect::<Vec<_>>()
            .join(" → ")
    ));
    report.push_str(&format!(
        "  Relationship: {}\n",
        graph.describe_relationship(Domain::Mangala, Domain::Shukra)
    ));
    report.push_str(&format!(
        "  Mangala→Budha: {}\n",
        graph.describe_relationship(Domain::Mangala, Domain::Budha)
    ));
    report.push_str(&format!(
        "  Mangala→Surya: {}\n",
        graph.describe_relationship(Domain::Mangala, Domain::Surya)
    ));
    report.push_str("\n");

    // ── Phase 8: Gyro Router ──────────────────────────────────────────────
    report.push_str("─── Phase 8: Gyro Spinning Wheel Router ────────────────\n");
    let mut gyro_state = GyroState::new();
    let mut gyro_dynamics = GyroDynamics::new();
    let gyro_router = GyroRouter::new();
    let route_tokens = ["energy", "efficiency", "code", "optimization", "power"];
    for token in &route_tokens {
        let result = gyro_router.process_token(token, &mut gyro_state, &mut gyro_dynamics);
        report.push_str(&format!(
            "  \"{}\" → sign: {}, alignment: {:.3}, fired: [{}], orientation: {:.1}°\n",
            token,
            result.dominant_sign.symbol(),
            result.alignment,
            result
                .fired
                .iter()
                .map(|f| f.primitive)
                .collect::<Vec<_>>()
                .join(", "),
            result.orientation_deg
        ));
    }
    report.push_str(&format!(
        "  Final dominant: {:?} (weight: {:.3})\n",
        gyro_state.dominant_graha_info().0,
        gyro_state.dominant_graha_info().1
    ));
    report.push_str(&format!(
        "  Total tokens processed: {}\n\n",
        gyro_state.token_count
    ));

    // ── Phase 9: NAND Primitives + Continuous Logic ────────────────────────
    report.push_str("─── Phase 9: NAND Primitives + Continuous Logic ─────────\n");
    report.push_str("  Boolean truth table (NAND):\n");
    for a in [false, true] {
        for b in [false, true] {
            report.push_str(&format!("    NAND({}, {}) = {}\n", a, b, nand_gate(a, b)));
        }
    }

    let efficiency = 0.85_f64;
    let code_quality = 0.70_f64;
    let energy_saved = primitive::nand_continuous::and(efficiency, code_quality);
    report.push_str(&format!(
        "  Continuous AND(efficiency={}, quality={}) = {:.4}\n",
        efficiency, code_quality, energy_saved
    ));
    let bottleneck = primitive::nand_continuous::nand(efficiency, code_quality);
    report.push_str(&format!(
        "  Continuous NAND(eff, qual) = {:.4} (bottleneck)\n",
        bottleneck
    ));
    let combined = primitive::nand_continuous::or(efficiency, code_quality);
    report.push_str(&format!("  Continuous OR(eff, qual) = {:.4}\n", combined));

    let (sum, carry) = primitive::nand_continuous::half_adder(0.7, 0.3);
    report.push_str(&format!(
        "  Half adder(0.7, 0.3) = sum={:.4}, carry={:.4}\n\n",
        sum, carry
    ));

    let expr = NandExpression::compile("nand(a, not(b))").expect("should compile");
    let mut inputs = HashMap::new();
    inputs.insert("a".to_string(), 1.0);
    inputs.insert("b".to_string(), 0.0);
    let result = expr.evaluate(&inputs).expect("should evaluate");
    report.push_str(&format!(
        "  NandExpression \"nand(a, not(b))\" with a=1, b=0 = {:.4}\n",
        result
    ));
    report.push_str(&format!(
        "  NAND count: {}, node count: {}\n",
        expr.nand_count(),
        expr.node_count()
    ));

    let mut dag = NandDag::new();
    let a = dag.add_input("a");
    let b = dag.add_input("b");
    let nand_ab = dag.add_nand(a, b);
    let not_nand = dag.add_not(nand_ab);
    let _and_result = not_nand;
    report.push_str(&format!(
        "  NandDag: {} nodes, {} NAND gates\n",
        dag.node_count(),
        dag.nand_count()
    ));
    let mut dag_inputs = HashMap::new();
    dag_inputs.insert("a".to_string(), 1.0);
    dag_inputs.insert("b".to_string(), 1.0);
    if let Some(val) = dag.evaluate(&dag_inputs) {
        report.push_str(&format!("  NandDag AND(1,1) = {:.4}\n", val));
    }
    report.push_str("\n");

    // ── Phase 10: Bankai Verification + Diagnostics ───────────────────────
    report.push_str("─── Phase 10: Bankai Verification + Diagnostics ─────────\n");
    let report1 = verify_expression("2 + 3 = 5");
    report.push_str(&format!(
        "  verify \"2 + 3 = 5\": passed={}, errors={}, warnings={}\n",
        report1.passed, report1.error_count, report1.warning_count
    ));

    let report2 = verify_expression("2 + 2 = 5");
    report.push_str(&format!(
        "  verify \"2 + 2 = 5\": passed={}, errors={}, warnings={}\n",
        report2.passed, report2.error_count, report2.warning_count
    ));

    let vague = "This person has a tendency to sometimes feel uncertain about things.";
    let specific =
        "Mangala in Aries at 15.3° indicates martial energy channeled through cardinal fire.";
    let barnum_vague = analyze_barnum(vague);
    let barnum_specific = analyze_barnum(specific);
    report.push_str(&format!(
        "  Barnum check vague: score={:.2}, discriminating={}\n",
        barnum_vague.barnum_score, barnum_vague.is_discriminating
    ));
    report.push_str(&format!(
        "  Barnum check specific: score={:.2}, discriminating={}\n",
        barnum_specific.barnum_score, barnum_specific.is_discriminating
    ));

    let mut session = RefinementSession::new(QUERY, 3);
    let verdict1 = session.begin_round(
        "efficiency = useful_output / total_input * 100",
        ProposalKind::Arithmetic,
    );
    report.push_str(&format!(
        "  Refinement round 1: passed={}, errors={}\n",
        verdict1.passed, verdict1.error_count
    ));
    session.accept_verdict();
    session.finalize();
    let summary = session.summarize();
    report.push_str(&format!(
        "  Session: rounds={}, converged={}, confidence={:.2}\n\n",
        summary.total_rounds, summary.converged, summary.final_confidence
    ));

    // ── Phase 11: Pachinko Validation ─────────────────────────────────────
    report.push_str("─── Phase 11: Pachinko Validation Ball ──────────────────\n");
    let candidate = pachinko::ball::TokenCandidate::new(1, "energy_efficiency", 0.85);
    let mut ball = pachinko::ball::Ball::new(candidate);

    let pin_field = pachinko::pin::PinField::new();
    let active_pins = pin_field.active_pins();
    let pins: Vec<_> = active_pins.into_iter().cloned().collect();
    let pin_count = pins.len();
    report.push_str(&format!("  Pin field: {} active pins\n", pin_count));

    let context = "energy efficiency = useful_work / total_energy";
    validation::validate_ball(&mut ball, &pins, context);
    report.push_str(&format!(
        "  Ball score: {:.4}, all_passed: {}\n",
        ball.total_score,
        ball.all_passed()
    ));

    let pocket = pachinko::pocket::Pocket::new(ball);
    report.push_str(&format!(
        "  Pocket kakuhen: {}\n\n",
        pocket.should_trigger_kakuhen()
    ));

    // ── Phase 12: Ephemeris ───────────────────────────────────────────────
    report.push_str("─── Phase 12: Ephemeris (Astronomical Positions) ────────\n");
    let jd_now = ephemeris::julian_day(2026, 7, 13, 12.0);
    let (y, m, d) = ephemeris::julian_day_to_date(jd_now);
    report.push_str(&format!(
        "  Julian Day: {:.4} → {}-{:02}-{:02}\n",
        jd_now, y, m, d
    ));

    let ayanamsa = ephemeris::lahiri_ayanamsa(jd_now);
    report.push_str(&format!("  Lahiri Ayanamsa: {:.4}°\n", ayanamsa));

    let sun_tropical = ephemeris::tropical_longitude(Graha::Surya, jd_now);
    let sun_sidereal = (sun_tropical - ayanamsa).rem_euclid(360.0);
    report.push_str(&format!(
        "  Sun tropical: {:.4}° → sidereal: {:.4}°\n",
        sun_tropical, sun_sidereal
    ));

    let all_positions = ephemeris::all_graha_positions(jd_now);
    for pos in &all_positions {
        report.push_str(&format!(
            "  {} ({}) → tropical {:.2}°, rashi {} ({:.2}°)\n",
            pos.graha.name(),
            pos.graha.symbol(),
            pos.tropical,
            pos.rashi.name(),
            pos.sidereal
        ));
    }
    report.push_str("\n");

    // ── Phase 13: Economy Tracking ────────────────────────────────────────
    report.push_str("─── Phase 13: Economy & Budget Tracking ─────────────────\n");
    let mut budget = Budget::new(10_000, 0.50);
    report.push_str(&format!(
        "  Budget: {} tokens, ${:.2}\n",
        budget.remaining_tokens(),
        budget.remaining_cost()
    ));
    budget.spend_tokens(2500);
    budget.spend_cost(0.12);
    report.push_str(&format!(
        "  After spend: {} tokens ({:.1}% used), ${:.3} remaining\n",
        budget.remaining_tokens(),
        budget.token_usage_percent(),
        budget.remaining_cost()
    ));

    let mut cost_tracker = CostTracker::new();
    cost_tracker.record("descent", 0.02);
    cost_tracker.record("chart", 0.01);
    cost_tracker.record("verifier", 0.03);
    report.push_str(&format!(
        "  Cost tracker: avg={:.4}, min={:.4}, max={:.4}\n",
        cost_tracker.average_cost(),
        cost_tracker.min_cost(),
        cost_tracker.max_cost()
    ));

    let mut ball_econ = BallEconomy::new(100);
    ball_econ.spend(3);
    ball_econ.win(10);
    report.push_str(&format!(
        "  Ball economy: balance={}, spent={}, won={}, ROI={:.2}\n",
        ball_econ.balance(),
        ball_econ.total_spent(),
        ball_econ.total_won(),
        ball_econ.roi()
    ));
    report.push_str("\n");

    // ── Phase 14: Assemble & Determinism Check ────────────────────────────
    report.push_str("═══════════════════════════════════════════════════════════\n");
    report.push_str("  SUMMARY\n");
    report.push_str("═══════════════════════════════════════════════════════════\n");
    report.push_str(&format!(
        "  Formulas: {} | Entities: {} | Descent depth: {:.2}/6\n",
        formula_count, seed_count, matrix.average_depth
    ));
    report.push_str(&format!(
        "  Dominant domain: {:?} | Dominant pillar: {}\n",
        matrix.dominant_domains.first(),
        personality.dominant.name()
    ));
    report.push_str(&format!(
        "  Lagna: {} | Watch: {}\n",
        chart
            .lagna
            .map(|l| l.name().to_string())
            .unwrap_or_else(|| "n/a".to_string()),
        watch.name()
    ));
    report.push_str(&format!(
        "  Budget remaining: {} tokens\n",
        budget.remaining_tokens()
    ));
    report.push_str("═══════════════════════════════════════════════════════════\n");

    println!("\n{}", report);

    // ── Determinism Check ─────────────────────────────────────────────────
    let matrix2 = descent_engine.descend(QUERY);
    assert_eq!(
        matrix.tokens.len(),
        matrix2.tokens.len(),
        "Determinism: same token count"
    );
    for (a, b) in matrix.tokens.iter().zip(matrix2.tokens.iter()) {
        assert_eq!(a.text, b.text, "Determinism: same token text");
        assert_eq!(
            a.settled_layer.depth(),
            b.settled_layer.depth(),
            "Determinism: same settled layer for '{}'",
            a.text
        );
        assert!(
            (a.confidence - b.confidence).abs() < 1e-10,
            "Determinism: same confidence for '{}'",
            a.text
        );
    }
    assert!(
        (matrix.resolution_score - matrix2.resolution_score).abs() < 1e-10,
        "Determinism: same resolution score"
    );

    // Verify all 14 subsystems exercised
    assert!(formula_count > 0);
    assert!(!classifications.is_empty());
    assert!(!matrix.tokens.is_empty());
    assert!(!energy_formulas.is_empty());
    assert!(seed_count > 0);
    assert!(!chart.graha_positions.is_empty());
    assert!(graph.shortest_path(Domain::Mangala, Domain::Shukra).is_ok());
    assert!(gyro_state.token_count > 0);
    assert!(expr.nand_count() > 0);
    assert!(report1.passed || report2.passed);
    assert!(pin_count > 0);
    assert_eq!(all_positions.len(), 9);
    assert!(budget.remaining_tokens() < 10_000);
}
