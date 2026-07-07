//! Adversarial input collection helpers.

use nexus_cog_core::antifragile::{AdversarialCategory, AdversarialInput};
use indexmap::IndexMap;

/// A collection of adversarial inputs with categorization.
#[derive(Debug, Clone, Default)]
pub struct AdversarialInputs {
    inputs: Vec<AdversarialInput>,
}

impl AdversarialInputs {
    /// Construct an empty collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct from a vector.
    #[must_use]
    pub fn from_vec(inputs: Vec<AdversarialInput>) -> Self {
        Self { inputs }
    }

    /// Add an input.
    pub fn push(&mut self, input: AdversarialInput) {
        self.inputs.push(input);
    }

    /// Extend with more inputs.
    pub fn extend(&mut self, inputs: impl IntoIterator<Item = AdversarialInput>) {
        self.inputs.extend(inputs);
    }

    /// Number of inputs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inputs.len()
    }

    /// Returns `true` if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inputs.is_empty()
    }

    /// Get the inputs.
    #[must_use]
    pub fn inputs(&self) -> &[AdversarialInput] {
        &self.inputs
    }

    /// Group inputs by category.
    #[must_use]
    pub fn by_category(&self) -> IndexMap<&'static str, Vec<&AdversarialInput>> {
        let mut out: IndexMap<&'static str, Vec<&AdversarialInput>> = IndexMap::new();
        for i in &self.inputs {
            let key: &'static str = category_key(i.category);
            out.entry(key).or_default().push(i);
        }
        out
    }

    /// Inputs of a particular category.
    #[must_use]
    pub fn of_category(&self, category: AdversarialCategory) -> Vec<&AdversarialInput> {
        self.inputs.iter().filter(|i| i.category == category).collect()
    }

    /// Inputs that are most likely to break the code (top N by `break_likelihood`).
    #[must_use]
    pub fn most_dangerous(&self, n: usize) -> Vec<&AdversarialInput> {
        let mut all: Vec<&AdversarialInput> = self.inputs.iter().collect();
        all.sort_by(|a, b| b.break_likelihood.partial_cmp(&a.break_likelihood).unwrap_or(std::cmp::Ordering::Equal));
        all.into_iter().take(n).collect()
    }

    /// Total estimated break-likelihood across all inputs.
    #[must_use]
    pub fn total_break_likelihood(&self) -> f32 {
        self.inputs.iter().map(|i| i.break_likelihood).sum()
    }
}

fn category_key(c: AdversarialCategory) -> &'static str {
    match c {
        AdversarialCategory::Empty => "empty",
        AdversarialCategory::Boundary => "boundary",
        AdversarialCategory::SpecialCharacters => "special_characters",
        AdversarialCategory::Large => "large",
        AdversarialCategory::Malformed => "malformed",
        AdversarialCategory::Repetition => "repetition",
        AdversarialCategory::Injection => "injection",
        AdversarialCategory::NumericEdge => "numeric_edge",
        AdversarialCategory::TypeConfusion => "type_confusion",
        AdversarialCategory::Concurrency => "concurrency",
        AdversarialCategory::Fuzz => "fuzz",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(cat: AdversarialCategory) -> AdversarialInput {
        AdversarialInput::new(cat, "x", "v", "r", 0.5)
    }

    #[test]
    fn by_category_groups() {
        let mut c = AdversarialInputs::new();
        c.push(sample(AdversarialCategory::Empty));
        c.push(sample(AdversarialCategory::Empty));
        c.push(sample(AdversarialCategory::Injection));
        let g = c.by_category();
        assert_eq!(g.get("empty").map(|v| v.len()), Some(2));
        assert_eq!(g.get("injection").map(|v| v.len()), Some(1));
    }

    #[test]
    fn most_dangerous_ordered() {
        let mut c = AdversarialInputs::new();
        c.push(AdversarialInput::new(AdversarialCategory::Empty, "low", "v", "r", 0.1));
        c.push(AdversarialInput::new(AdversarialCategory::Empty, "high", "v", "r", 0.9));
        c.push(AdversarialInput::new(AdversarialCategory::Empty, "mid", "v", "r", 0.5));
        let top = c.most_dangerous(2);
        assert_eq!(top.len(), 2);
        assert!(top[0].break_likelihood > top[1].break_likelihood);
    }
}
