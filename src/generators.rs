//! Adversarial input generators.

use nexus_cog_core::antifragile::{AdversarialCategory, AdversarialInput};

/// Configuration for the adversarial generator.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GeneratorConfig {
    /// Categories to include.
    pub categories: Vec<AdversarialCategory>,
    /// Maximum inputs to generate.
    pub max_inputs: usize,
    /// Whether to include fuzz-style random inputs.
    pub include_fuzz: bool,
    /// Random seed (None = non-deterministic).
    pub seed: Option<u64>,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            categories: AdversarialCategory::all().to_vec(),
            max_inputs: 50,
            include_fuzz: false,
            seed: None,
        }
    }
}

/// Generates adversarial inputs.
#[derive(Debug, Clone)]
pub struct AdversarialGenerator {
    config: GeneratorConfig,
}

impl Default for AdversarialGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl AdversarialGenerator {
    /// Construct with default config.
    #[must_use]
    pub fn new() -> Self {
        Self { config: GeneratorConfig::default() }
    }

    /// Construct with custom config.
    #[must_use]
    pub fn with_config(config: GeneratorConfig) -> Self {
        Self { config }
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &GeneratorConfig {
        &self.config
    }

    /// Generate a vector of adversarial inputs.
    #[must_use]
    pub fn generate(&self) -> Vec<AdversarialInput> {
        let mut out = Vec::new();
        for cat in &self.config.categories {
            match cat {
                AdversarialCategory::Empty => out.extend(empty_inputs()),
                AdversarialCategory::Boundary => out.extend(boundary_inputs()),
                AdversarialCategory::SpecialCharacters => out.extend(special_char_inputs()),
                AdversarialCategory::Large => out.extend(large_inputs()),
                AdversarialCategory::Malformed => out.extend(malformed_inputs()),
                AdversarialCategory::Repetition => out.extend(repetition_inputs()),
                AdversarialCategory::Injection => out.extend(injection_inputs()),
                AdversarialCategory::NumericEdge => out.extend(numeric_edge_inputs()),
                AdversarialCategory::TypeConfusion => out.extend(type_confusion_inputs()),
                AdversarialCategory::Concurrency => out.extend(concurrency_inputs()),
                AdversarialCategory::Fuzz => {
                    if self.config.include_fuzz {
                        out.extend(fuzz_inputs(self.config.seed));
                    }
                }
            }
            if out.len() >= self.config.max_inputs {
                break;
            }
        }
        out.truncate(self.config.max_inputs);
        out
    }

    /// Generate inputs targeting string handling.
    #[must_use]
    pub fn for_string_handler(&self) -> Vec<AdversarialInput> {
        let mut g = self.clone();
        g.config.categories = vec![
            AdversarialCategory::Empty,
            AdversarialCategory::SpecialCharacters,
            AdversarialCategory::Large,
            AdversarialCategory::Injection,
            AdversarialCategory::Malformed,
        ];
        g.config.max_inputs = self.config.max_inputs.min(20);
        g.generate()
    }

    /// Generate inputs targeting numeric handling.
    #[must_use]
    pub fn for_numeric_handler(&self) -> Vec<AdversarialInput> {
        let mut g = self.clone();
        g.config.categories = vec![
            AdversarialCategory::NumericEdge,
            AdversarialCategory::Boundary,
            AdversarialCategory::TypeConfusion,
        ];
        g.config.max_inputs = self.config.max_inputs.min(15);
        g.generate()
    }
}

fn empty_inputs() -> Vec<AdversarialInput> {
    vec![
        AdversarialInput::new(AdversarialCategory::Empty, "empty string", "", "empty string is a common edge case", 0.4),
        AdversarialInput::new(AdversarialCategory::Empty, "empty array", "[]", "empty array", 0.3),
        AdversarialInput::new(AdversarialCategory::Empty, "empty object", "{}", "empty object", 0.3),
    ]
}

fn boundary_inputs() -> Vec<AdversarialInput> {
    vec![
        AdversarialInput::new(AdversarialCategory::Boundary, "single character", "a", "single character is the smallest non-empty input", 0.2),
        AdversarialInput::new(AdversarialCategory::Boundary, "single whitespace", " ", "single whitespace", 0.4),
        AdversarialInput::new(AdversarialCategory::Boundary, "newline only", "\n", "string containing only a newline", 0.3),
        AdversarialInput::new(AdversarialCategory::Boundary, "two-char", "ab", "two-character string", 0.1),
    ]
}

fn special_char_inputs() -> Vec<AdversarialInput> {
    vec![
        AdversarialInput::new(AdversarialCategory::SpecialCharacters, "unicode emoji", "hello \u{1F600}", "emoji codepoint", 0.3),
        AdversarialInput::new(AdversarialCategory::SpecialCharacters, "RTL", "abc\u{202E}def", "RTL override can break naive rendering", 0.6),
        AdversarialInput::new(AdversarialCategory::SpecialCharacters, "null byte", "abc\0def", "null byte inside a string", 0.7),
        AdversarialInput::new(AdversarialCategory::SpecialCharacters, "combining marks", "a\u{0301}", "combining marks can break naive length counting", 0.5),
        AdversarialInput::new(AdversarialCategory::SpecialCharacters, "zero-width joiner", "a\u{200D}b", "zero-width joiner", 0.6),
    ]
}

fn large_inputs() -> Vec<AdversarialInput> {
    let big = "x".repeat(1_000_000);
    let bigger = "x".repeat(100_000_000);
    vec![
        AdversarialInput::new(AdversarialCategory::Large, "1 MB string", &big, "1 MB string may stress memory and allocations", 0.6),
        AdversarialInput::new(AdversarialCategory::Large, "100 MB string", &bigger, "100 MB string will likely OOM most systems", 0.9),
    ]
}

fn malformed_inputs() -> Vec<AdversarialInput> {
    vec![
        AdversarialInput::new(AdversarialCategory::Malformed, "unclosed brace", "{\"x\":1", "unclosed JSON object", 0.5),
        AdversarialInput::new(AdversarialCategory::Malformed, "trailing comma", "[1,2,3,]", "trailing comma in array", 0.4),
        AdversarialInput::new(AdversarialCategory::Malformed, "garbage", "\u{FFFF}\u{FFFE}???", "invalid byte sequence", 0.6),
    ]
}

fn repetition_inputs() -> Vec<AdversarialInput> {
    vec![
        AdversarialInput::new(AdversarialCategory::Repetition, "10k repeats", "a".repeat(10_000), "10,000 repetitions of the same char", 0.4),
        AdversarialInput::new(AdversarialCategory::Repetition, "100k repeats", "ab".repeat(50_000), "100,000 chars of alternating pattern", 0.5),
    ]
}

fn injection_inputs() -> Vec<AdversarialInput> {
    vec![
        AdversarialInput::new(AdversarialCategory::Injection, "sql injection", "'; DROP TABLE users; --", "SQL injection payload", 0.7),
        AdversarialInput::new(AdversarialCategory::Injection, "shell injection", "; rm -rf /", "shell metacharacters", 0.7),
        AdversarialInput::new(AdversarialCategory::Injection, "path traversal", "../../../../etc/passwd", "path traversal", 0.6),
        AdversarialInput::new(AdversarialCategory::Injection, "html xss", "<script>alert(1)</script>", "XSS payload", 0.5),
        AdversarialInput::new(AdversarialCategory::Injection, "format string", "%s%s%s%s%s%s%s%s%s%s", "format string vulnerability", 0.5),
    ]
}

fn numeric_edge_inputs() -> Vec<AdversarialInput> {
    vec![
        AdversarialInput::new(AdversarialCategory::NumericEdge, "zero", "0", "zero value", 0.1),
        AdversarialInput::new(AdversarialCategory::NumericEdge, "negative", "-1", "negative one", 0.2),
        numeric_input("i32::MAX", "2147483647", "maximum i32"),
        numeric_input("i32::MIN", "-2147483648", "minimum i32"),
        numeric_input("u32::MAX", "4294967295", "maximum u32"),
        numeric_input("i64::MAX", "9223372036854775807", "maximum i64"),
        numeric_input("NaN", "NaN", "Not a Number"),
        numeric_input("infinity", "inf", "positive infinity"),
        numeric_input("negative zero", "-0", "negative zero"),
    ]
}

fn numeric_input(description: &'static str, value: &'static str, rationale: &'static str) -> AdversarialInput {
    AdversarialInput::new(AdversarialCategory::NumericEdge, description, value, rationale, 0.5)
}

fn type_confusion_inputs() -> Vec<AdversarialInput> {
    vec![
        AdversarialInput::new(AdversarialCategory::TypeConfusion, "string for number", "\"42\"", "string \"42\" when expecting a number", 0.4),
        AdversarialInput::new(AdversarialCategory::TypeConfusion, "boolean for number", "true", "boolean true when expecting a number", 0.4),
        AdversarialInput::new(AdversarialCategory::TypeConfusion, "null for number", "null", "null when expecting a number", 0.5),
        AdversarialInput::new(AdversarialCategory::TypeConfusion, "array for object", "[1,2,3]", "array when expecting object", 0.6),
    ]
}

fn concurrency_inputs() -> Vec<AdversarialInput> {
    vec![
        AdversarialInput::new(AdversarialCategory::Concurrency, "rapid succession", "fire 1000x in 1ms", "rapid-fire requests may expose race conditions", 0.7),
        AdversarialInput::new(AdversarialCategory::Concurrency, "concurrent identical", "concurrent identical calls", "concurrent identical calls may expose non-atomic updates", 0.6),
    ]
}

fn fuzz_inputs(seed: Option<u64>) -> Vec<AdversarialInput> {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    let mut rng = seed.map_or_else(StdRng::from_entropy, StdRng::seed_from_u64);
    let mut out = Vec::new();
    for _ in 0..20 {
        let len = rng.gen_range(1..200);
        let bytes: Vec<u8> = (0..len).map(|_| rng.r#gen()).collect();
        let value = String::from_utf8_lossy(&bytes).to_string();
        out.push(AdversarialInput::new(
            AdversarialCategory::Fuzz,
            "random bytes",
            value,
            "fuzz-style random input",
            0.5,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_inputs() {
        let g = AdversarialGenerator::new();
        let inputs = g.generate();
        assert!(!inputs.is_empty());
    }

    #[test]
    fn for_string_handler_skips_numeric() {
        let g = AdversarialGenerator::new();
        let inputs = g.for_string_handler();
        assert!(inputs.iter().all(|i| !matches!(i.category, AdversarialCategory::NumericEdge)));
    }

    #[test]
    fn max_inputs_respected() {
        let cfg = GeneratorConfig { max_inputs: 3, ..GeneratorConfig::default() };
        let g = AdversarialGenerator::with_config(cfg);
        let inputs = g.generate();
        assert!(inputs.len() <= 3);
    }

    #[test]
    fn deterministic_with_seed() {
        let cfg1 = GeneratorConfig { seed: Some(42), include_fuzz: true, ..GeneratorConfig::default() };
        let cfg2 = GeneratorConfig { seed: Some(42), include_fuzz: true, ..GeneratorConfig::default() };
        let g1 = AdversarialGenerator::with_config(cfg1);
        let g2 = AdversarialGenerator::with_config(cfg2);
        let inputs1 = g1.generate();
        let inputs2 = g2.generate();
        assert_eq!(inputs1.len(), inputs2.len());
    }
}
