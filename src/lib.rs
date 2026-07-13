pub mod asauchi;
pub mod astrology;
pub mod bankai;
pub mod chart;
pub mod descent;
pub mod economy;
pub mod entity;
pub mod ephemeris;
pub mod formula;
pub mod gyro;
pub mod pachinko;
pub mod primitive;
pub mod shikai;
pub mod tanto;
pub mod validation;
pub mod wheel;
pub mod zanpakuto;

#[cfg(feature = "mcp")]
pub mod mcp;

pub mod cli;

pub mod prelude {
    pub use crate::asauchi::{compute_aspect, is_adjacent, is_conjunction, validate_graha_index};
    pub use crate::astrology::barnum::{analyze_barnum, passes_barnum_filter};
    pub use crate::astrology::{
        AtomClassification, ChangeSorter, Element, Graha, Guna, House, Modality, Nakshatra,
        PlanetaryRuler, Rashi, Sign, SignAspect, VedicClassification, VedicElement,
    };
    pub use crate::bankai::diagnostics::{Diagnostic, DiagnosticGate, DiagnosticReport, Severity};
    pub use crate::bankai::protocol::RefinementSession;
    pub use crate::bankai::verifier::{
        verify_expression, verify_proposal, LlmProposal, ProposalKind,
    };
    pub use crate::bankai::{aggregate_confidence_scores, compute_confidence_score};
    pub use crate::chart::{compute_planetary_aspect, determine_planet_house, ChartSnapshot};
    pub use crate::descent::{
        lowercase_string, tokenize_descent, DescentEngine, DescentLayer, SettledToken,
        SettlingMatrix,
    };
    pub use crate::economy::budget::Budget;
    pub use crate::economy::conversation::ConversationTracker;
    pub use crate::economy::cost::CostTracker;
    pub use crate::economy::tray::BallEconomy;
    pub use crate::entity::{
        compute_entity_hash, validate_entity_id, DynamicEntity, Entity, EntityRegistry,
        EventRegistry, ShikaiFormRegistry,
    };
    pub use crate::ephemeris::{compute_vsop87_approximation, julian_day_to_date, GrahaPosition};
    pub use crate::formula::{
        extract_formula_domain, validate_formula_id, Formula, FormulaRegistry, FormulaType,
    };
    pub use crate::gyro::{compute_next_position, map_graha_to_position};
    pub use crate::primitive::arithmetic::{add_unsigned_8, full_adder, half_adder};
    pub use crate::primitive::nand::{and_gate, nand_gate, not_gate, or_gate, xor_gate};
    pub use crate::shikai::{extract_numerical_values, parse_query_intent};
    pub use crate::tanto::{
        compute_formula, create_env, evaluate_expr, evaluate_nl, evaluate_pipeline, solve_problem,
    };
    pub use crate::wheel::{
        CompositionAspect, Domain, UnderstandingAxis, WheelError, WheelGraph, ALL_DOMAINS,
    };
    pub use crate::zanpakuto::{extract_keywords, normalize_query_text, NlpContext};
}
