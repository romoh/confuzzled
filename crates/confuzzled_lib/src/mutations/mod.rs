//! Mutation engine for container image fuzzing
//!
//! Provides composable mutation primitives organized by category.

pub mod extreme;
pub mod format;
pub mod semantic;
pub mod structural;

use crate::oci::Image;
use crate::Result;
use rand::prelude::*;
use std::fmt::Debug;

pub use extreme::*;
pub use format::*;
pub use semantic::*;
pub use structural::*;

/// A mutation that can be applied to an image
pub trait Mutation: Debug + Send + Sync {
    /// Name of the mutation
    fn name(&self) -> &'static str;

    /// Category of the mutation
    fn category(&self) -> MutationCategory;

    /// Apply the mutation to an image
    fn apply(&self, image: Image, rng: &mut StdRng) -> Result<Image>;
}

/// Categories of mutations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MutationCategory {
    /// Structural mutations (remove/reorder/duplicate components)
    Structural,

    /// Format mutations (corrupt JSON, compression, encoding)
    Format,

    /// Semantic mutations (valid format but invalid semantics)
    Semantic,

    /// Extreme edge cases (huge values, deep nesting, etc.)
    Extreme,
}

/// Set of enabled mutation categories
#[derive(Debug, Clone, Copy, Default)]
pub struct MutationSet {
    pub structural: bool,
    pub format: bool,
    pub semantic: bool,
    pub extreme: bool,
}

impl MutationSet {
    /// All structural mutations
    pub const STRUCTURAL: Self = Self {
        structural: true,
        format: false,
        semantic: false,
        extreme: false,
    };

    /// All format mutations
    pub const FORMAT: Self = Self {
        structural: false,
        format: true,
        semantic: false,
        extreme: false,
    };

    /// All semantic mutations
    pub const SEMANTIC: Self = Self {
        structural: false,
        format: false,
        semantic: true,
        extreme: false,
    };

    /// All extreme mutations
    pub const EXTREME: Self = Self {
        structural: false,
        format: false,
        semantic: false,
        extreme: true,
    };

    /// All mutations enabled
    pub const ALL: Self = Self {
        structural: true,
        format: true,
        semantic: true,
        extreme: true,
    };

    /// No mutations (identity transform)
    pub const NONE: Self = Self {
        structural: false,
        format: false,
        semantic: false,
        extreme: false,
    };

    /// Select random mutations based on enabled categories
    pub fn select_random(&self, rng: &mut StdRng) -> Vec<Box<dyn Mutation>> {
        let mut mutations: Vec<Box<dyn Mutation>> = Vec::new();

        if self.structural {
            mutations.extend(self.random_structural(rng));
        }
        if self.format {
            mutations.extend(self.random_format(rng));
        }
        if self.semantic {
            mutations.extend(self.random_semantic(rng));
        }
        if self.extreme {
            mutations.extend(self.random_extreme(rng));
        }

        // Shuffle and take a subset
        mutations.shuffle(rng);
        let count = rng.gen_range(1..=mutations.len().max(1));
        mutations.truncate(count);

        mutations
    }

    fn random_structural(&self, rng: &mut StdRng) -> Vec<Box<dyn Mutation>> {
        let all: Vec<Box<dyn Mutation>> = vec![
            Box::new(RemoveManifest),
            Box::new(RemoveConfig),
            Box::new(RemoveIndex),
            Box::new(DuplicateLayers),
            Box::new(ReorderLayers),
            Box::new(AddDanglingBlob),
            Box::new(EmptyManifest),
        ];
        self.pick_random(all, rng)
    }

    fn random_format(&self, rng: &mut StdRng) -> Vec<Box<dyn Mutation>> {
        let all: Vec<Box<dyn Mutation>> = vec![
            Box::new(InvalidJson),
            Box::new(TrailingGarbage),
            Box::new(Utf8Violation),
            Box::new(TruncateBlob),
            Box::new(CorruptCompression),
        ];
        self.pick_random(all, rng)
    }

    fn random_semantic(&self, rng: &mut StdRng) -> Vec<Box<dyn Mutation>> {
        let all: Vec<Box<dyn Mutation>> = vec![
            Box::new(DigestMismatch),
            Box::new(SizeMismatch),
            Box::new(WrongMediaType),
            Box::new(PlatformMismatch),
            Box::new(InvalidAnnotations),
            Box::new(MissingLayerReference),
        ];
        self.pick_random(all, rng)
    }

    fn random_extreme(&self, rng: &mut StdRng) -> Vec<Box<dyn Mutation>> {
        let all: Vec<Box<dyn Mutation>> = vec![
            Box::new(HugeLayerCount),
            Box::new(DeepPathNesting),
            Box::new(ExtremeValues),
            Box::new(EmptyEverything),
            Box::new(EmptyLayersNullDiffIds),
            Box::new(NullDiffIdsOnly),
            Box::new(EmptyDiffIds),
            Box::new(MismatchedDiffIdsCount),
        ];
        self.pick_random(all, rng)
    }

    fn pick_random(
        &self,
        mut mutations: Vec<Box<dyn Mutation>>,
        rng: &mut StdRng,
    ) -> Vec<Box<dyn Mutation>> {
        mutations.shuffle(rng);
        let count = rng.gen_range(0..=2);
        mutations.truncate(count);
        mutations
    }
}

impl std::ops::BitOr for MutationSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            structural: self.structural || rhs.structural,
            format: self.format || rhs.format,
            semantic: self.semantic || rhs.semantic,
            extreme: self.extreme || rhs.extreme,
        }
    }
}

/// Parse mutation set from string (e.g., "structural,semantic")
impl std::str::FromStr for MutationSet {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut set = MutationSet::NONE;

        for part in s.split(',') {
            match part.trim().to_lowercase().as_str() {
                "structural" => set.structural = true,
                "format" => set.format = true,
                "semantic" => set.semantic = true,
                "extreme" => set.extreme = true,
                "all" => return Ok(MutationSet::ALL),
                "none" => {}
                other => return Err(format!("Unknown mutation category: {}", other)),
            }
        }

        Ok(set)
    }
}
