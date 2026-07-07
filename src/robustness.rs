//! Robustness scoring.

use nexus_cog_core::antifragile::{AdversarialResult, RobustnessReport};

/// Computes robustness scores.
#[derive(Debug, Clone, Default)]
pub struct RobustnessScorer;

impl RobustnessScorer {
    /// Construct a new scorer.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Score a set of results.
    #[must_use]
    pub fn score(&self, target: &str, results: Vec<AdversarialResult>) -> RobustnessReport {
        let total = results.len();
        let failures = results.iter().filter(|r| !r.handled).count();
        let score = if total == 0 {
            1.0
        } else {
            (total - failures) as f32 / total as f32
        };

        let recommendations = self.recommendations(&results);
        let inputs: Vec<_> = results.iter().map(|r| r.input.clone()).collect();

        RobustnessReport {
            id: format!("rb-{}", uuid::Uuid::new_v4()),
            target: target.to_string(),
            inputs,
            results,
            score,
            failures,
            total,
            recommendations,
            timestamp: chrono::Utc::now(),
        }
    }

    fn recommendations(&self, results: &[AdversarialResult]) -> Vec<String> {
        let mut recs = Vec::new();
        let by_category_failed = self.failures_by_category(results);
        for (category, count) in by_category_failed {
            if count == 0 {
                continue;
            }
            let cat_rec = match category.as_str() {
                "empty" => "Add explicit empty-input guards",
                "boundary" => "Add boundary checks at limits",
                "special_characters" => "Validate and normalize unicode input",
                "large" => "Enforce input size limits",
                "malformed" => "Use a strict parser; reject malformed input early",
                "repetition" => "Bound loop iterations / use streaming",
                "injection" => "Use parameterized queries / sanitization",
                "numeric_edge" => "Use checked arithmetic / handle NaN and infinity",
                "type_confusion" => "Validate types at the boundary",
                "concurrency" => "Use atomic operations and locks",
                "fuzz" => "Fuzz-test with cargo-fuzz / afl",
                _ => "Review failure modes",
            };
            recs.push(format!("[{category}] ({count} failures): {cat_rec}"));
        }
        if results.is_empty() {
            recs.push("No adversarial inputs were run. Generate some to assess robustness.".into());
        }
        recs
    }

    fn failures_by_category(&self, results: &[AdversarialResult]) -> indexmap::IndexMap<String, usize> {
        let mut out: indexmap::IndexMap<String, usize> = indexmap::IndexMap::new();
        for r in results.iter().filter(|r| !r.handled) {
            let key = r.input.category.id().to_string();
            *out.entry(key).or_insert(0) += 1;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_cog_core::antifragile::{AdversarialCategory, AdversarialInput};

    fn result(cat: AdversarialCategory, handled: bool) -> AdversarialResult {
        AdversarialResult {
            input: AdversarialInput::new(cat, "x", "v", "r", 0.5),
            handled,
            output: None,
            error: if handled { None } else { Some("oops".into()) },
            severity: None,
            elapsed_ms: 1,
        }
    }

    #[test]
    fn perfect_score_when_all_handled() {
        let s = RobustnessScorer::new();
        let r = s.score("f", vec![result(AdversarialCategory::Empty, true), result(AdversarialCategory::Boundary, true)]);
        assert_eq!(r.score, 1.0);
        assert_eq!(r.failures, 0);
    }

    #[test]
    fn partial_score_with_some_failures() {
        let s = RobustnessScorer::new();
        let r = s.score("f", vec![result(AdversarialCategory::Empty, true), result(AdversarialCategory::Empty, false)]);
        assert_eq!(r.score, 0.5);
        assert_eq!(r.failures, 1);
    }

    #[test]
    fn recommendations_for_failures() {
        let s = RobustnessScorer::new();
        let r = s.score("f", vec![result(AdversarialCategory::Empty, false)]);
        assert!(!r.recommendations.is_empty());
    }
}
