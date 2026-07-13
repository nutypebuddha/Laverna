/// Severity of a diagnostic message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    /// Hard failure: the expression is invalid, unsound, or contradicts itself.
    Error,
    /// Soft failure: the expression is questionable but not strictly invalid.
    Warning,
    /// Informational: passes validation but the verifier notes a concern.
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

/// Which validation gate produced this diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DiagnosticGate {
    Math,
    Logic,
    Formal,
    Fallacy,
    Confidence,
    Structural,
    Domain,
}

impl std::fmt::Display for DiagnosticGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticGate::Math => write!(f, "math"),
            DiagnosticGate::Logic => write!(f, "logic"),
            DiagnosticGate::Formal => write!(f, "formal"),
            DiagnosticGate::Fallacy => write!(f, "fallacy"),
            DiagnosticGate::Confidence => write!(f, "confidence"),
            DiagnosticGate::Structural => write!(f, "structural"),
            DiagnosticGate::Domain => write!(f, "domain"),
        }
    }
}

/// A single diagnostic message from a validation gate.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub gate: DiagnosticGate,
    pub severity: Severity,
    /// What went wrong (or was noted).
    pub message: String,
    /// Byte offset within the input where the issue was detected (if applicable).
    pub position: Option<usize>,
    /// What the gate expected (if applicable).
    pub expected: Option<String>,
    /// What the gate actually received.
    pub got: Option<String>,
    /// Machine-readable constraint identifier (e.g. "math.balanced_parens").
    pub constraint_id: Option<String>,
    /// Actionable fix suggestion for the LLM to try next.
    pub fix_suggestion: Option<String>,
}

impl Diagnostic {
    pub fn error(gate: DiagnosticGate, message: impl Into<String>) -> Self {
        Diagnostic {
            gate,
            severity: Severity::Error,
            message: message.into(),
            position: None,
            expected: None,
            got: None,
            constraint_id: None,
            fix_suggestion: None,
        }
    }

    pub fn warning(gate: DiagnosticGate, message: impl Into<String>) -> Self {
        Diagnostic {
            gate,
            severity: Severity::Warning,
            message: message.into(),
            position: None,
            expected: None,
            got: None,
            constraint_id: None,
            fix_suggestion: None,
        }
    }

    pub fn info(gate: DiagnosticGate, message: impl Into<String>) -> Self {
        Diagnostic {
            gate,
            severity: Severity::Info,
            message: message.into(),
            position: None,
            expected: None,
            got: None,
            constraint_id: None,
            fix_suggestion: None,
        }
    }

    pub fn with_position(mut self, position: usize) -> Self {
        self.position = Some(position);
        self
    }

    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }

    pub fn with_got(mut self, got: impl Into<String>) -> Self {
        self.got = Some(got.into());
        self
    }

    pub fn with_constraint_id(mut self, id: impl Into<String>) -> Self {
        self.constraint_id = Some(id.into());
        self
    }

    pub fn with_fix_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.fix_suggestion = Some(suggestion.into());
        self
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    pub fn is_warning(&self) -> bool {
        self.severity == Severity::Warning
    }
}

/// Aggregated result from the verifier across all gates.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticReport {
    /// The original input that was verified.
    pub input: String,
    /// All diagnostics produced, in gate-evaluation order.
    pub diagnostics: Vec<Diagnostic>,
    /// Whether every gate passed (no Error-level diagnostics).
    pub passed: bool,
    /// Aggregate confidence score [0.0, 1.0].
    pub confidence: f64,
    /// Number of error-level diagnostics.
    pub error_count: usize,
    /// Number of warning-level diagnostics.
    pub warning_count: usize,
}

impl DiagnosticReport {
    pub fn new(input: impl Into<String>) -> Self {
        DiagnosticReport {
            input: input.into(),
            diagnostics: Vec::new(),
            passed: true,
            confidence: 1.0,
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        if diagnostic.is_error() {
            self.passed = false;
            self.error_count += 1;
        }
        if diagnostic.is_warning() {
            self.warning_count += 1;
        }
        self.diagnostics.push(diagnostic);
    }

    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.is_error())
    }

    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.is_warning())
    }

    /// Compute aggregate confidence from gate scores (0.0–1.0).
    pub fn compute_confidence(&mut self) {
        if self.diagnostics.is_empty() {
            self.confidence = 1.0;
            return;
        }
        let error_penalty: f64 = self.errors().count() as f64 * 0.25;
        let warning_penalty: f64 = self.warnings().count() as f64 * 0.10;
        self.confidence = (1.0 - error_penalty - warning_penalty).max(0.0);
    }

    /// Format a human-readable summary for LLM consumption.
    pub fn format_for_llm(&self) -> String {
        if self.passed {
            return format!(
                "VERDICT: PASS (confidence: {:.0}%)\nNo errors detected.",
                self.confidence * 100.0
            );
        }

        let mut out = format!(
            "VERDICT: FAIL (confidence: {:.0}%)\nErrors: {}, Warnings: {}\n\n",
            self.confidence * 100.0,
            self.error_count,
            self.warning_count,
        );

        for (i, diag) in self.errors().enumerate() {
            out.push_str(&format!("{}. [{}] {}\n", i + 1, diag.gate, diag.message,));
            if let Some(ref expected) = diag.expected {
                out.push_str(&format!("   Expected: {}\n", expected));
            }
            if let Some(ref got) = diag.got {
                out.push_str(&format!("   Got: {}\n", got));
            }
            if let Some(ref fix) = diag.fix_suggestion {
                out.push_str(&format!("   Fix: {}\n", fix));
            }
            out.push('\n');
        }

        for diag in self.warnings() {
            out.push_str(&format!("[warning] [{}] {}\n", diag.gate, diag.message));
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_display() {
        assert_eq!(Severity::Error.to_string(), "error");
        assert_eq!(Severity::Warning.to_string(), "warning");
        assert_eq!(Severity::Info.to_string(), "info");
    }

    #[test]
    fn test_diagnostic_gate_display() {
        assert_eq!(DiagnosticGate::Math.to_string(), "math");
        assert_eq!(DiagnosticGate::Logic.to_string(), "logic");
        assert_eq!(DiagnosticGate::Formal.to_string(), "formal");
    }

    #[test]
    fn test_diagnostic_error_chain() {
        let d = Diagnostic::error(DiagnosticGate::Math, "unbalanced parens")
            .with_position(5)
            .with_expected("'(' closed before pos 5")
            .with_got("unclosed '('")
            .with_constraint_id("math.balanced_parens")
            .with_fix_suggestion("Add a closing ')' before position 5");

        assert!(d.is_error());
        assert!(!d.is_warning());
        assert_eq!(d.position, Some(5));
        assert_eq!(d.constraint_id.as_deref(), Some("math.balanced_parens"));
        assert!(d.fix_suggestion.is_some());
    }

    #[test]
    fn test_diagnostic_report_pass() {
        let mut report = DiagnosticReport::new("2 + 3");
        report.push(Diagnostic::info(DiagnosticGate::Math, "evaluates to 5"));
        report.compute_confidence();

        assert!(report.passed);
        assert_eq!(report.confidence, 1.0);
        assert_eq!(report.error_count, 0);
    }

    #[test]
    fn test_diagnostic_report_fail() {
        let mut report = DiagnosticReport::new("(2 + 3");
        report.push(
            Diagnostic::error(DiagnosticGate::Math, "unbalanced parens")
                .with_constraint_id("math.balanced_parens"),
        );
        report.push(Diagnostic::warning(DiagnosticGate::Logic, "no conclusion"));
        report.compute_confidence();

        assert!(!report.passed);
        assert_eq!(report.error_count, 1);
        assert_eq!(report.warning_count, 1);
        assert!(report.confidence < 1.0);
    }

    #[test]
    fn test_format_for_llm_pass() {
        let mut report = DiagnosticReport::new("2 + 3 = 5");
        report.compute_confidence();
        let formatted = report.format_for_llm();
        assert!(formatted.contains("PASS"));
    }

    #[test]
    fn test_format_for_llm_fail() {
        let mut report = DiagnosticReport::new("(2 + 3");
        report.push(Diagnostic::error(DiagnosticGate::Math, "unbalanced parens"));
        report.compute_confidence();
        let formatted = report.format_for_llm();
        assert!(formatted.contains("FAIL"));
        assert!(formatted.contains("unbalanced parens"));
    }

    #[test]
    fn test_diagnostic_serialization_roundtrip() {
        let d = Diagnostic::error(DiagnosticGate::Formal, "circular reasoning")
            .with_constraint_id("formal.no_circular");
        let json = serde_json::to_string(&d).unwrap();
        let back: Diagnostic = serde_json::from_str(&json).unwrap();
        assert_eq!(back.message, "circular reasoning");
        assert_eq!(back.gate, DiagnosticGate::Formal);
    }

    #[test]
    fn test_report_errors_and_warnings_iterators() {
        let mut report = DiagnosticReport::new("test");
        report.push(Diagnostic::error(DiagnosticGate::Math, "err1"));
        report.push(Diagnostic::warning(DiagnosticGate::Logic, "warn1"));
        report.push(Diagnostic::error(DiagnosticGate::Formal, "err2"));

        assert_eq!(report.errors().count(), 2);
        assert_eq!(report.warnings().count(), 1);
    }
}
