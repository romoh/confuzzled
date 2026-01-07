//! # confuzzled_lib
//!
//! Structure-aware fuzzing library for container images.
//! Generate malformed OCI/Docker images to test runtimes, registries, and scanners.
//!
//! ## Philosophy
//!
//! > The fuzzed container is the input. The thing you plug it into is the target.
//!
//! ## Example
//!
//! ```rust,ignore
//! use confuzzled_lib::{Fuzzer, Seed, MutationSet};
//!
//! let fuzzer = Fuzzer::builder()
//!     .seed(Seed::from_oci_dir("./busybox"))
//!     .mutations(MutationSet::SEMANTIC | MutationSet::STRUCTURAL)
//!     .rng_seed(42)
//!     .build()?;
//!
//! for image in fuzzer.generate(100) {
//!     image.write_to_oci_dir("./corpus")?;
//! }
//! ```

pub mod mutations;
pub mod oci;
pub mod output;
pub mod pipeline;
pub mod registry;
pub mod seed;

use rand::prelude::*;
use rand::SeedableRng;
use thiserror::Error;

pub use mutations::{Mutation, MutationSet};
pub use oci::Image;
pub use output::OutputAdapter;
pub use pipeline::Pipeline;
pub use registry::pull_image;
pub use seed::Seed;

/// Errors that can occur in confuzzled
#[derive(Error, Debug)]
pub enum ConfuzzledError {
    #[error("Seed error: {0}")]
    SeedError(String),

    #[error("Mutation error: {0}")]
    MutationError(String),

    #[error("OCI format error: {0}")]
    OciError(String),

    #[error("Output error: {0}")]
    OutputError(String),

    #[error("Registry error: {0}")]
    RegistryError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ConfuzzledError>;

/// Main fuzzer builder and runner
pub struct Fuzzer {
    seed: Seed,
    mutations: MutationSet,
    rng: StdRng,
}

impl Fuzzer {
    /// Create a new Fuzzer builder
    pub fn builder() -> FuzzerBuilder {
        FuzzerBuilder::default()
    }

    /// Generate fuzzed images
    pub fn generate(&mut self, count: usize) -> Vec<Image> {
        let mut images = Vec::with_capacity(count);

        for i in 0..count {
            tracing::debug!("Generating fuzzed image {}/{}", i + 1, count);
            match self.generate_one() {
                Ok(image) => images.push(image),
                Err(e) => tracing::warn!("Failed to generate image: {}", e),
            }
        }

        images
    }

    /// Generate a single fuzzed image
    pub fn generate_one(&mut self) -> Result<Image> {
        let mut image = self.seed.load()?;

        // Select and apply random mutations based on the mutation set
        let mutations = self.mutations.select_random(&mut self.rng);

        for mutation in mutations {
            image = mutation.apply(image, &mut self.rng)?;
        }

        Ok(image)
    }
}

/// Builder for creating a Fuzzer
#[derive(Default)]
pub struct FuzzerBuilder {
    seed: Option<Seed>,
    mutations: MutationSet,
    rng_seed: Option<u64>,
}

impl FuzzerBuilder {
    /// Set the seed image source
    pub fn seed(mut self, seed: Seed) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set which mutation categories to apply
    pub fn mutations(mut self, mutations: MutationSet) -> Self {
        self.mutations = mutations;
        self
    }

    /// Set RNG seed for deterministic generation
    pub fn rng_seed(mut self, seed: u64) -> Self {
        self.rng_seed = Some(seed);
        self
    }

    /// Build the Fuzzer
    pub fn build(self) -> Result<Fuzzer> {
        let seed = self
            .seed
            .ok_or_else(|| ConfuzzledError::SeedError("No seed image provided".to_string()))?;

        let rng = match self.rng_seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };

        Ok(Fuzzer {
            seed,
            mutations: self.mutations,
            rng,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzer_builder() {
        let fuzzer = Fuzzer::builder()
            .seed(Seed::minimal())
            .mutations(MutationSet::STRUCTURAL)
            .rng_seed(42)
            .build();

        assert!(fuzzer.is_ok());
    }
}
