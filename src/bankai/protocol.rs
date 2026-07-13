//! Feedback protocol wire format for the LLM ↔ Laverna self-refinement loop.
//!
//! Research basis:
//! - Logic-LM: feeding solver error messages back to the LLM substantially
//!   outperforms pass/fail alone.
//! - LLM-Modulo: bidirectional generator-critic loop.
//! - Laverna design principle: the LLM is the untrusted proposer, Laverna
//!   is the trusted checker. This protocol is the interface between them.

use crate::bankai::diagnostics::DiagnosticReport;
use crate::bankai::verifier::{LlmProposal, ProposalKind};

/// A single round in the self-refinement loop.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RefinementRound {
    /// Monotonically increasing round number (0-based).
    pub round_index: u32,
    /// What the LLM proposed this round.
    pub proposal: LlmProposal,
    /// What Laverna found.
    pub verdict: DiagnosticReport,
    /// Whether the LLM accepted the verdict and revised.
    pub accepted: bool,
}

/// The full refinement session: a sequence of proposal → verdict → revise.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RefinementSession {
    /// The original natural language query that started the session.
    pub original_query: String,
    /// All rounds in order.
    pub rounds: Vec<RefinementRound>,
    /// Maximum allowed refinement rounds before giving up.
    pub max_rounds: u32,
    /// Whether the session converged to a passing result.
    pub converged: bool,
    /// Final output if converged (the last passing formalized expression).
    pub final_output: Option<String>,
}

impl RefinementSession {
    pub fn new(original_query: impl Into<String>, max_rounds: u32) -> Self {
        RefinementSession {
            original_query: original_query.into(),
            rounds: Vec::new(),
            max_rounds,
            converged: false,
            final_output: None,
        }
    }

    /// Start a new refinement round with the LLM's proposal.
    pub fn begin_round(&mut self, formalized: &str, kind: ProposalKind) -> &DiagnosticReport {
        let round_index = self.rounds.len() as u32;

        let proposal = LlmProposal {
            original_query: self.original_query.clone(),
            formalized: formalized.to_string(),
            kind,
            claimed_confidence: None,
        };

        let verdict = crate::bankai::verifier::verify_proposal(&proposal);

        self.rounds.push(RefinementRound {
            round_index,
            proposal,
            verdict: verdict.clone(),
            accepted: false,
        });

        &self.rounds.last().unwrap().verdict
    }

    /// Accept the verdict (LLM acknowledges the feedback).
    pub fn accept_verdict(&mut self) {
        if let Some(round) = self.rounds.last_mut() {
            round.accepted = true;
        }
    }

    /// Check if the latest round passed.
    pub fn last_round_passed(&self) -> bool {
        self.rounds.last().is_some_and(|r| r.verdict.passed)
    }

    /// Check if we've hit the round limit.
    pub fn at_limit(&self) -> bool {
        self.rounds.len() as u32 >= self.max_rounds
    }

    /// Finalize: if the last round passed, set converged and extract output.
    pub fn finalize(&mut self) {
        if self.last_round_passed() {
            self.converged = true;
            self.final_output = self.rounds.last().map(|r| r.proposal.formalized.clone());
        }
    }

    /// Generate a prompt-ready feedback message for the LLM.
    ///
    /// This is what the LLM receives after a failed verification round.
    /// It includes the structured diagnostics and a clear instruction to revise.
    pub fn feedback_for_llm(&self) -> Option<String> {
        let last = self.rounds.last()?;
        if last.verdict.passed {
            return Some(format!(
                "ROUND {} PASS: Expression verified.\n{}",
                last.round_index,
                last.verdict.format_for_llm(),
            ));
        }

        let remaining = self.max_rounds.saturating_sub(self.rounds.len() as u32);
        Some(format!(
            "ROUND {} FAIL: {} error(s), {} warning(s). {} rounds remaining.\n\
             Revise the expression to fix the errors below.\n\n{}\n\
             CONSTRAINT IDs for programmatic filtering: [{}]",
            last.round_index,
            last.verdict.error_count,
            last.verdict.warning_count,
            remaining,
            last.verdict.format_for_llm(),
            last.verdict
                .errors()
                .filter_map(|d| d.constraint_id.as_deref())
                .collect::<Vec<_>>()
                .join(", "),
        ))
    }
}

/// Compact summary of a refinement session for logging / telemetry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionSummary {
    pub total_rounds: u32,
    pub converged: bool,
    pub final_confidence: f64,
    pub total_errors_seen: usize,
    pub total_warnings_seen: usize,
    pub unique_constraint_ids: Vec<String>,
}

impl RefinementSession {
    pub fn summarize(&self) -> SessionSummary {
        let mut constraint_ids = std::collections::HashSet::new();
        let mut total_errors = 0usize;
        let mut total_warnings = 0usize;

        for round in &self.rounds {
            total_errors += round.verdict.error_count;
            total_warnings += round.verdict.warning_count;
            for diag in &round.verdict.diagnostics {
                if let Some(ref id) = diag.constraint_id {
                    constraint_ids.insert(id.clone());
                }
            }
        }

        let final_confidence = self.rounds.last().map_or(0.0, |r| r.verdict.confidence);

        SessionSummary {
            total_rounds: self.rounds.len() as u32,
            converged: self.converged,
            final_confidence,
            total_errors_seen: total_errors,
            total_warnings_seen: total_warnings,
            unique_constraint_ids: {
                let mut v: Vec<_> = constraint_ids.into_iter().collect();
                v.sort();
                v
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refinement_session_converges() {
        let mut session = RefinementSession::new("what is 2+3", 5);

        // Round 1: wrong.
        let verdict = session.begin_round("(2 + 3", ProposalKind::Arithmetic);
        assert!(!verdict.passed);
        session.accept_verdict();
        assert!(!session.last_round_passed());

        // Round 2: correct.
        let verdict = session.begin_round("add 2 3", ProposalKind::Arithmetic);
        assert!(verdict.passed);
        session.accept_verdict();
        session.finalize();

        assert!(session.converged);
        assert_eq!(session.final_output.as_deref(), Some("add 2 3"));
        assert_eq!(session.rounds.len(), 2);
    }

    #[test]
    fn test_refinement_session_hits_limit() {
        let mut session = RefinementSession::new("hard problem", 2);

        let _ = session.begin_round("(bad1", ProposalKind::Arithmetic);
        session.accept_verdict();

        let _ = session.begin_round("(bad2", ProposalKind::Arithmetic);
        session.accept_verdict();

        assert!(session.at_limit());
        session.finalize();
        assert!(!session.converged);
        assert!(session.final_output.is_none());
    }

    #[test]
    fn test_feedback_for_llm_pass() {
        let mut session = RefinementSession::new("q", 5);
        let _ = session.begin_round("add 2 3", ProposalKind::Arithmetic);
        let feedback = session.feedback_for_llm().unwrap();
        assert!(feedback.contains("PASS"));
    }

    #[test]
    fn test_feedback_for_llm_fail() {
        let mut session = RefinementSession::new("q", 5);
        let _ = session.begin_round("(2 + 3", ProposalKind::Arithmetic);
        let feedback = session.feedback_for_llm().unwrap();
        assert!(feedback.contains("FAIL"));
        assert!(feedback.contains("CONSTRAINT IDs"));
        assert!(feedback.contains("rounds remaining"));
    }

    #[test]
    fn test_session_summary() {
        let mut session = RefinementSession::new("q", 5);
        let _ = session.begin_round("(bad", ProposalKind::Arithmetic);
        session.accept_verdict();
        let _ = session.begin_round("add 2 3", ProposalKind::Arithmetic);
        session.accept_verdict();
        session.finalize();

        let summary = session.summarize();
        assert_eq!(summary.total_rounds, 2);
        assert!(summary.converged);
        assert!(summary.total_errors_seen > 0);
        assert!(!summary.unique_constraint_ids.is_empty());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut session = RefinementSession::new("test query", 3);
        let _ = session.begin_round("2 + 3", ProposalKind::Arithmetic);
        session.accept_verdict();
        session.finalize();

        let json = serde_json::to_string(&session).unwrap();
        let back: RefinementSession = serde_json::from_str(&json).unwrap();
        assert!(back.converged);
        assert_eq!(back.rounds.len(), 1);
    }
}
