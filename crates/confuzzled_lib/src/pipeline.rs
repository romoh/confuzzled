//! Mutation pipeline for composing mutations
//!
//! Allows chaining mutations together in a reproducible way.

use crate::mutations::Mutation;
use crate::oci::Image;
use crate::Result;
use rand::prelude::*;

/// A pipeline of mutations to apply sequentially
pub struct Pipeline {
    mutations: Vec<Box<dyn Mutation>>,
}

impl Pipeline {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Self {
            mutations: Vec::new(),
        }
    }

    /// Add a mutation to the pipeline
    pub fn then<M: Mutation + 'static>(mut self, mutation: M) -> Self {
        self.mutations.push(Box::new(mutation));
        self
    }

    /// Add a boxed mutation to the pipeline
    pub fn then_boxed(mut self, mutation: Box<dyn Mutation>) -> Self {
        self.mutations.push(mutation);
        self
    }

    /// Apply all mutations in sequence
    pub fn apply(&self, image: Image, rng: &mut StdRng) -> Result<Image> {
        let mut result = image;

        for mutation in &self.mutations {
            tracing::debug!("Applying mutation: {}", mutation.name());
            result = mutation.apply(result, rng)?;
        }

        Ok(result)
    }

    /// Get the list of mutation names in this pipeline
    pub fn mutation_names(&self) -> Vec<&'static str> {
        self.mutations.iter().map(|m| m.name()).collect()
    }

    /// Check if the pipeline is empty
    pub fn is_empty(&self) -> bool {
        self.mutations.is_empty()
    }

    /// Get the number of mutations in the pipeline
    pub fn len(&self) -> usize {
        self.mutations.len()
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipeline")
            .field("mutations", &self.mutation_names())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutations::{DigestMismatch, RemoveConfig, RemoveManifest};
    use crate::seed::Seed;
    use rand::SeedableRng;

    #[test]
    fn test_pipeline_basic() {
        let pipeline = Pipeline::new().then(RemoveConfig).then(RemoveManifest);

        assert_eq!(pipeline.len(), 2);
        assert_eq!(
            pipeline.mutation_names(),
            vec!["remove-config", "remove-manifest"]
        );
    }

    #[test]
    fn test_pipeline_apply() {
        let pipeline = Pipeline::new().then(RemoveConfig).then(DigestMismatch);

        let image = Seed::minimal().load().unwrap();
        let mut rng = StdRng::seed_from_u64(42);

        let result = pipeline.apply(image, &mut rng).unwrap();
        assert!(result.config.is_none());
    }
}
