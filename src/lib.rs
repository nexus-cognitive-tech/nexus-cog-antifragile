//! Antifragile verification: don't just statically check code — try to break it.
//!
//! Generates adversarial inputs and scores the code's robustness. Inspired by
//! Taleb's antifragility concept: the goal is not merely to survive shocks, but
//! to *improve* under them — adversarial inputs reveal the limits of the code
//! so the developer can add guards.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod edge_cases;
pub mod error;
pub mod fuzz;
pub mod generators;
pub mod inputs;
pub mod robustness;

pub use edge_cases::{EdgeCaseExplorer, ExplorerConfig};
pub use error::{AntifragileError, AntifragileResult};
pub use fuzz::Fuzzer;
pub use generators::{AdversarialGenerator, GeneratorConfig};
pub use inputs::AdversarialInputs;
pub use robustness::RobustnessScorer;
