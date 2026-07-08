//! Edge-case explorer: identify edge cases the code may not handle.

use nexus_cog_core::antifragile::{EdgeCase, EdgeCaseReport};
use indexmap::IndexMap;

/// Configuration for the explorer.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExplorerConfig {
    /// Keywords to look for in code that suggest edge cases.
    pub keywords: Vec<String>,
    /// Whether to include common edge cases regardless of keywords.
    pub include_common: bool,
}

impl Default for ExplorerConfig {
    fn default() -> Self {
        Self {
            keywords: vec![
                "parse".into(),
                "read".into(),
                "fetch".into(),
                "open".into(),
                "new".into(),
                "split".into(),
                "index".into(),
                "nth".into(),
            ],
            include_common: true,
        }
    }
}

/// Edge-case explorer.
#[derive(Debug, Clone, Default)]
pub struct EdgeCaseExplorer {
    config: ExplorerConfig,
}

impl EdgeCaseExplorer {
    /// Construct a new explorer with default config.
    #[must_use]
    pub fn new() -> Self {
        Self { config: ExplorerConfig::default() }
    }

    /// Construct with custom config.
    #[must_use]
    pub fn with_config(config: ExplorerConfig) -> Self {
        Self { config }
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &ExplorerConfig {
        &self.config
    }

    /// Explore a piece of code.
    #[must_use]
    pub fn explore(&self, target: &str, code: &str) -> EdgeCaseReport {
        let cases = self.cases_for(code);
        let total = cases.len();
        let unhandled = cases.iter().filter(|c| !c.handled).count();
        let coverage = if total == 0 {
            1.0
        } else {
            (total - unhandled) as f32 / total as f32
        };
        EdgeCaseReport {
            id: format!("edge-{}", uuid::Uuid::new_v4()),
            target: target.to_string(),
            cases,
            coverage,
            unhandled,
            total,
            timestamp: chrono::Utc::now(),
        }
    }

    fn cases_for(&self, code: &str) -> Vec<EdgeCase> {
        let mut cases = Vec::new();
        let lower = code.to_lowercase();

        if self.config.keywords.iter().any(|k| lower.contains(k)) {
            if lower.contains("parse") {
                cases.push(EdgeCase {
                    id: format!("ec-parse-{}", uuid::Uuid::new_v4()),
                    description: "Empty input to parse".into(),
                    example: "\"\"".into(),
                    handled: code.contains("is_empty") || code.contains(".len()"),
                    recommendation: "Check for empty input before parsing".into(),
                    confidence: 0.9,
                    severity: nexus_cog_core::common::Severity::Warning,
                    tags: vec!["parsing".into()],
                });
                cases.push(EdgeCase {
                    id: format!("ec-parse-malformed-{}", uuid::Uuid::new_v4()),
                    description: "Malformed input to parse".into(),
                    example: "\"{ invalid json \"".into(),
                    handled: code.contains("Result") || code.contains("try ") || code.contains("catch"),
                    recommendation: "Return Result on parse errors".into(),
                    confidence: 0.95,
                    severity: nexus_cog_core::common::Severity::Warning,
                    tags: vec!["parsing".into(), "errors".into()],
                });
            }
            if lower.contains("read") || lower.contains("fetch") || lower.contains("open") {
                cases.push(EdgeCase {
                    id: format!("ec-io-{}", uuid::Uuid::new_v4()),
                    description: "I/O failure (network down, file missing)".into(),
                    example: "ConnectionRefused, NotFound, etc.".into(),
                    handled: code.contains("Result") || code.contains("?") || code.contains("try"),
                    recommendation: "Propagate I/O errors with context".into(),
                    confidence: 0.95,
                    severity: nexus_cog_core::common::Severity::Error,
                    tags: vec!["io".into()],
                });
                cases.push(EdgeCase {
                    id: format!("ec-timeout-{}", uuid::Uuid::new_v4()),
                    description: "Timeout".into(),
                    example: "operation takes longer than expected".into(),
                    handled: code.contains("timeout") || code.contains("Timeout") || code.contains("deadline"),
                    recommendation: "Add explicit timeout".into(),
                    confidence: 0.7,
                    severity: nexus_cog_core::common::Severity::Warning,
                    tags: vec!["io".into()],
                });
            }
            if lower.contains("nth") || lower.contains("[0]") {
                cases.push(EdgeCase {
                    id: format!("ec-index-{}", uuid::Uuid::new_v4()),
                    description: "Index out of bounds".into(),
                    example: "v[0] on empty v".into(),
                    handled: code.contains("get(") || code.contains("first()"),
                    recommendation: "Use safe indexing (`.get(n)` or `.first()`)".into(),
                    confidence: 0.8,
                    severity: nexus_cog_core::common::Severity::Error,
                    tags: vec!["indexing".into()],
                });
            }
            if lower.contains("split") {
                cases.push(EdgeCase {
                    id: format!("ec-split-{}", uuid::Uuid::new_v4()),
                    description: "Empty input to split".into(),
                    example: "\"a,b,c\".split(\",\") on \"\"".into(),
                    handled: code.contains("is_empty") || code.contains(".len() > 0"),
                    recommendation: "Handle empty separator results".into(),
                    confidence: 0.6,
                    severity: nexus_cog_core::common::Severity::Info,
                    tags: vec!["split".into()],
                });
            }
        }

        if self.config.include_common {
            cases.push(EdgeCase {
                id: format!("ec-null-{}", uuid::Uuid::new_v4()),
                description: "Null / None input".into(),
                example: "None, null, undefined".into(),
                handled: code.contains("Option") || code.contains("null check") || code.contains("is_none"),
                recommendation: "Handle nullability explicitly".into(),
                confidence: 0.7,
                severity: nexus_cog_core::common::Severity::Warning,
                tags: vec!["null".into()],
            });
            if lower.contains("+") || lower.contains("-") || lower.contains("*") {
                cases.push(EdgeCase {
                    id: format!("ec-overflow-{}", uuid::Uuid::new_v4()),
                    description: "Numeric overflow".into(),
                    example: "u8::MAX + 1".into(),
                    handled: lower.contains("checked_") || lower.contains("saturating_") || lower.contains("wrapping_"),
                    recommendation: "Use checked/saturating arithmetic".into(),
                    confidence: 0.6,
                    severity: nexus_cog_core::common::Severity::Warning,
                    tags: vec!["numeric".into()],
                });
            }
            if lower.contains("/") || lower.contains("div") || lower.contains("% ") || lower.contains("mod ") {
                let guarded = lower.contains("checked_div")
                    || lower.contains("checked_rem")
                    || lower.contains("checked_div_euclid")
                    || lower.contains(".checked_div(")
                    || lower.contains("nonzero")
                    || lower.contains("!= 0")
                    || lower.contains("> 0")
                    || lower.contains("< 0")
                    || lower.contains("is_zero")
                    || lower.contains("guard")
                    || lower.contains("if b == 0")
                    || lower.contains("if b != 0")
                    || lower.contains("if x == 0")
                    || lower.contains("denominator");
                cases.push(EdgeCase {
                    id: format!("ec-div-zero-{}", uuid::Uuid::new_v4()),
                    description: "Division by zero".into(),
                    example: "a / b where b == 0".into(),
                    handled: guarded,
                    recommendation: "Use checked_div or guard the divisor against zero.".into(),
                    confidence: 0.95,
                    severity: nexus_cog_core::common::Severity::Critical,
                    tags: vec!["numeric".into(), "division".into()],
                });
                cases.push(EdgeCase {
                    id: format!("ec-div-overflow-{}", uuid::Uuid::new_v4()),
                    description: "Division overflow (i32::MIN / -1)".into(),
                    example: "i32::MIN / -1".into(),
                    handled: lower.contains("checked_div") || lower.contains(".checked_div("),
                    recommendation: "Use checked_div to avoid panic on signed overflow.".into(),
                    confidence: 0.7,
                    severity: nexus_cog_core::common::Severity::Warning,
                    tags: vec!["numeric".into(), "division".into()],
                });
            }
        }

        cases
    }

    /// Build a quick lookup index by tag.
    #[must_use]
    pub fn index_by_tag<'a>(&self, cases: &'a [EdgeCase]) -> IndexMap<String, Vec<&'a EdgeCase>> {
        let mut out: IndexMap<String, Vec<&'a EdgeCase>> = IndexMap::new();
        for c in cases {
            for tag in &c.tags {
                out.entry(tag.clone()).or_default().push(c);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_code_yields_common_cases() {
        let e = EdgeCaseExplorer::new();
        let r = e.explore("f", "");
        assert!(r.total >= 1);
    }

    #[test]
    fn parse_code_triggers_parse_edge_cases() {
        let e = EdgeCaseExplorer::new();
        let r = e.explore("f", "fn parse(s: &str) -> Result<i32, E> { s.parse() }");
        assert!(r.cases.iter().any(|c| c.description.contains("Empty input")));
    }

    #[test]
    fn handled_code_detected() {
        let e = EdgeCaseExplorer::new();
        let r = e.explore("f", "if s.is_empty() { return Err(...) }; s.parse()?");
        let parse_cases: Vec<_> = r.cases.iter().filter(|c| c.description.contains("parse") || c.description.contains("Parse")).collect();
        assert!(parse_cases.iter().any(|c| c.handled));
    }

    #[test]
    fn coverage_reflects_handled() {
        let e = EdgeCaseExplorer::new();
        let r = e.explore("f", "fn f(s: &str) -> Result<i32, E> { if s.is_empty() { return Err(E::Empty) }; s.parse() }");
        assert!(r.coverage > 0.0);
    }

    #[test]
    fn divide_by_zero_surfaced() {
        let e = EdgeCaseExplorer::new();
        let r = e.explore("divide", "fn divide(a: i32, b: i32) -> i32 { a / b }");
        assert!(r.cases.iter().any(|c| c.description.contains("Division by zero")));
        assert!(!r.cases.iter().any(|c| c.description.contains("Division by zero") && c.handled));
    }

    #[test]
    fn divide_with_guard_marked_handled() {
        let e = EdgeCaseExplorer::new();
        let r = e.explore("divide", "fn divide(a: i32, b: i32) -> i32 { if b == 0 { return 0; } a / b }");
        let div_case = r.cases.iter().find(|c| c.description.contains("Division by zero")).unwrap();
        assert!(div_case.handled, "guard should mark division as handled");
    }

    #[test]
    fn checked_div_marked_handled() {
        let e = EdgeCaseExplorer::new();
        let r = e.explore("divide", "fn divide(a: i32, b: i32) -> Option<i32> { a.checked_div(b) }");
        let div_case = r.cases.iter().find(|c| c.description.contains("Division by zero")).unwrap();
        assert!(div_case.handled);
    }
}
