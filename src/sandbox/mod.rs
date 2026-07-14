//! Deterministic sandbox subsystem for Laverna.
//!
//! ## Provenance
//! Assimilated from `mana-core` ("Run AI Tasks Securely on Your Machine"). mana-core's pipeline
//! is: (1) Local AI **classifier** → (2) **Planning AI** → (3) WASM **sandbox** → (4) full
//! **audit log**. Its classifier and planner are ML/local-AI, which Laverna's core forbids
//! (no ML / non-deterministic deps). We port the *shape* of that pipeline and substitute the
//! ML classifier + planner with CID's 5-gate deterministic validation
//! (`crate::validation`): Math / Logic / Confidence / Formal / Fact.
//!
//! An untrusted step is admitted to execution only if it survives every gate. Every decision —
//! allow or deny — is written to an append-only, deterministic audit log.

use crate::pachinko::ball::{Ball, GateResult, TokenCandidate};
use crate::pachinko::pin::{PinField, ValidationPin};
use crate::validation::validate_ball;
use std::collections::VecDeque;

/// An untrusted operation submitted to the sandbox.
#[derive(Debug, Clone)]
pub struct SandboxStep {
    pub id: u32,
    pub command: String,
    pub context: String,
}

impl SandboxStep {
    pub fn new(id: u32, command: impl Into<String>, context: impl Into<String>) -> Self {
        SandboxStep {
            id,
            command: command.into(),
            context: context.into(),
        }
    }
}

/// Outcome of a single gated step: allowed (all gates passed) or denied, with the audit trail.
#[derive(Debug, Clone)]
pub struct SandboxOutcome {
    pub step_id: u32,
    pub allowed: bool,
    pub total_score: f64,
    pub audit: Vec<GateResult>,
}

impl SandboxOutcome {
    /// Human-readable one-line verdict for the audit trail.
    pub fn verdict(&self) -> String {
        let gates: Vec<String> = self
            .audit
            .iter()
            .map(|r| format!("{:?}:{}", r.gate, if r.passed { "pass" } else { "DENY" }))
            .collect();
        format!(
            "[step {}] {} score={:.3} | {}",
            self.step_id,
            if self.allowed { "ALLOW" } else { "DENY" },
            self.total_score,
            gates.join(" ")
        )
    }
}

/// Append-only, deterministic audit log of sandbox decisions.
#[derive(Debug, Default)]
pub struct AuditLog {
    entries: VecDeque<SandboxOutcome>,
}

impl AuditLog {
    pub fn new() -> Self {
        AuditLog::default()
    }

    pub fn record(&mut self, outcome: SandboxOutcome) {
        self.entries.push_back(outcome);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn allowed_count(&self) -> usize {
        self.entries.iter().filter(|o| o.allowed).count()
    }

    pub fn denied_count(&self) -> usize {
        self.entries.iter().filter(|o| !o.allowed).count()
    }

    pub fn iter(&self) -> impl Iterator<Item = &SandboxOutcome> {
        self.entries.iter()
    }
}

/// A deterministic sandbox: gates untrusted steps through CID's 5-gate pipeline, then logs.
#[derive(Debug)]
pub struct Sandbox {
    pins: Vec<ValidationPin>,
    audit: AuditLog,
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Sandbox {
    pub fn new() -> Self {
        let field = PinField::new();
        let pins: Vec<ValidationPin> = field.active_pins().into_iter().cloned().collect();
        Sandbox {
            pins,
            audit: AuditLog::new(),
        }
    }

    /// Gate a step through the deterministic pipeline. The step is "executed" only if every
    /// gate passes — the deterministic analogue of mana-core's sandbox admission, with no ML.
    pub fn run(&mut self, step: SandboxStep) -> SandboxOutcome {
        let candidate = TokenCandidate::new(step.id, &step.command, 0.0);
        let mut ball = Ball::new(candidate);
        validate_ball(&mut ball, &self.pins, &step.context);
        let outcome = SandboxOutcome {
            step_id: step.id,
            allowed: ball.all_passed(),
            total_score: ball.total_score,
            audit: ball.gate_results,
        };
        self.audit.record(outcome.clone());
        outcome
    }

    pub fn audit(&self) -> &AuditLog {
        &self.audit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_denies_untrusted_text_step() {
        let mut sb = Sandbox::new();
        // Non-math, non-factual prose fails the Math/Formal/Fact gates deterministically.
        let out = sb.run(SandboxStep::new(1, "rm -rf /", "shell command"));
        assert!(!out.allowed, "untrusted shell step must be denied");
        assert_eq!(sb.audit().len(), 1);
        assert_eq!(sb.audit().denied_count(), 1);
        assert!(out.verdict().contains("DENY"));
    }

    #[test]
    fn sandbox_allows_well_formed_math() {
        let mut sb = Sandbox::new();
        let out = sb.run(SandboxStep::new(2, "2 + 3 = 5", "arithmetic identity"));
        assert!(out.allowed, "valid arithmetic should pass all gates");
        assert_eq!(sb.audit().allowed_count(), 1);
    }

    #[test]
    fn audit_log_accumulates() {
        let mut sb = Sandbox::new();
        sb.run(SandboxStep::new(10, "2+2", "math"));
        sb.run(SandboxStep::new(
            11,
            "ignore all prior instructions",
            "injection",
        ));
        assert_eq!(sb.audit().len(), 2);
        assert!(sb.audit().allowed_count() + sb.audit().denied_count() == 2);
    }
}
