pub mod diagnostics;
pub mod protocol;
pub mod verifier;

/// Pure function: Compute confidence score from evidence count.
pub fn compute_confidence_score(evidence_count: u32, total_requirements: u32) -> f64 {
    if total_requirements == 0 {
        return 0.0;
    }
    evidence_count as f64 / total_requirements as f64
}

/// Pure function: Check if confidence meets threshold.
pub fn confidence_meets_threshold(score: f64, threshold: f64) -> bool {
    score >= threshold
}

/// Pure function: Aggregate multiple confidence scores.
pub fn aggregate_confidence_scores(scores: &[f64]) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    let sum: f64 = scores.iter().sum();
    sum / scores.len() as f64
}

/// Pure function: Compute bankai solve result hash for deduplication.
pub fn compute_solve_result_hash(query: &str, result: &str) -> u64 {
    let mut hash: u64 = 0;
    for byte in query.bytes().chain(result.bytes()) {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_confidence_score_basic() {
        assert_eq!(compute_confidence_score(3, 3), 1.0);
        assert_eq!(compute_confidence_score(0, 3), 0.0);
        assert_eq!(compute_confidence_score(1, 3), 1.0 / 3.0);
        assert_eq!(compute_confidence_score(0, 0), 0.0);
    }

    #[test]
    fn confidence_meets_threshold_basic() {
        assert!(confidence_meets_threshold(0.9, 0.8));
        assert!(!confidence_meets_threshold(0.5, 0.8));
    }

    #[test]
    fn aggregate_confidence_scores_basic() {
        assert_eq!(aggregate_confidence_scores(&[0.8, 0.9, 1.0]), 0.9);
        assert_eq!(aggregate_confidence_scores(&[]), 0.0);
    }

    #[test]
    fn compute_solve_result_hash_basic() {
        let hash_a = compute_solve_result_hash("query", "result");
        let hash_b = compute_solve_result_hash("query", "result");
        let hash_c = compute_solve_result_hash("query", "different");
        assert_eq!(hash_a, hash_b);
        assert_ne!(hash_a, hash_c);
    }
}
