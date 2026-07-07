//! Error types for `cog-antifragile`.

use thiserror::Error;

/// Result alias for `cog-antifragile`.
pub type AntifragileResult<T> = Result<T, AntifragileError>;

/// Errors produced by the antifragile engines.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AntifragileError {
    /// Target not found.
    #[error("target `{0}` not found")]
    TargetNotFound(String),

    /// Generation failed.
    #[error("generation failed: {0}")]
    GenerationFailed(String),

    /// Evaluation failed.
    #[error("evaluation failed: {0}")]
    EvaluationFailed(String),
}
