//! Simple fuzz-style test helpers.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// A simple fuzzer that generates random inputs.
#[derive(Debug, Clone)]
pub struct Fuzzer {
    seed: Option<u64>,
    iterations: usize,
}

impl Default for Fuzzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Fuzzer {
    /// Construct a new fuzzer with default settings (1000 iterations, non-deterministic).
    #[must_use]
    pub fn new() -> Self {
        Self { seed: None, iterations: 1000 }
    }

    /// Set a seed for reproducible fuzzing.
    #[must_use]
    pub const fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set the number of iterations.
    #[must_use]
    pub const fn with_iterations(mut self, n: usize) -> Self {
        self.iterations = n;
        self
    }

    /// Run the fuzzer, calling `f` for each generated input. Returns the number of panics observed.
    pub fn run<F>(&self, mut f: F) -> usize
    where
        F: FnMut(&str),
    {
        let mut rng = self.seed.map_or_else(StdRng::from_entropy, StdRng::seed_from_u64);
        let mut panics = 0;
        for _ in 0..self.iterations {
            let len = rng.gen_range(0..200);
            let bytes: Vec<u8> = (0..len).map(|_| rng.r#gen()).collect();
            let s = String::from_utf8_lossy(&bytes);
            let prev_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(|_| {}));
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&s)));
            std::panic::set_hook(prev_hook);
            if result.is_err() {
                panics += 1;
            }
        }
        panics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzer_handles_inputs() {
        let f = Fuzzer::new().with_seed(42).with_iterations(100);
        let mut ok_count = 0_usize;
        let panics = f.run(|_| ok_count += 1);
        assert_eq!(panics, 0);
        assert_eq!(ok_count, 100);
    }

    #[test]
    fn fuzzer_detects_panics() {
        let f = Fuzzer::new().with_seed(7).with_iterations(50);
        let panics = f.run(|s| {
            if s.contains('a') {
                panic!("oh no");
            }
        });
        assert!(panics > 0);
    }
}
