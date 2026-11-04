//! Extreme mutations
//!
//! Edge case mutations with extreme values, deep nesting, etc.

use super::{Mutation, MutationCategory};
use crate::oci::{Descriptor, Image, Layer};
use crate::Result;
use rand::prelude::*;

/// Add a huge number of layers
#[derive(Debug)]
pub struct HugeLayerCount;

impl Mutation for HugeLayerCount {
    fn name(&self) -> &'static str {
        "huge-layer-count"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Extreme
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        let count = rng.gen_range(100..1000);

        for _ in 0..count {
            let layer = Layer {
                descriptor: Descriptor {
                    media_type: "application/vnd.oci.image.layer.v1.tar+gzip".to_string(),
                    digest: format!("sha256:{:064x}", rng.gen::<u128>()),
                    size: 0,
                    platform: None,
                    annotations: None,
                },
                data: Some(vec![]),
            };
            image.layers.push(layer);
        }

        Ok(image)
    }
}

/// Create deeply nested paths in layer content
#[derive(Debug)]
pub struct DeepPathNesting;

impl Mutation for DeepPathNesting {
    fn name(&self) -> &'static str {
        "deep-path-nesting"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Extreme
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        // Add annotations with deeply nested paths
        use std::collections::HashMap;

        let depth = rng.gen_range(100..500);
        let deep_path: String = (0..depth).map(|i| format!("dir{}/", i)).collect();

        let mut annotations = HashMap::new();
        annotations.insert("deep.path.example".to_string(), deep_path);

        if let Some(ref mut manifest) = image.manifest {
            manifest.annotations = Some(annotations);
        }

        Ok(image)
    }
}

/// Use extreme/boundary values
#[derive(Debug)]
pub struct ExtremeValues;

impl Mutation for ExtremeValues {
    fn name(&self) -> &'static str {
        "extreme-values"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Extreme
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        let extremes: Vec<i64> = vec![0, -1, i64::MAX, i64::MIN, i32::MAX as i64, i32::MIN as i64];

        if !image.layers.is_empty() {
            let idx = rng.gen_range(0..image.layers.len());
            image.layers[idx].descriptor.size = extremes[rng.gen_range(0..extremes.len())];
        }

        Ok(image)
    }
}

/// Make everything empty
#[derive(Debug)]
pub struct EmptyEverything;

impl Mutation for EmptyEverything {
    fn name(&self) -> &'static str {
        "empty-everything"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Extreme
    }

    fn apply(&self, mut image: Image, _rng: &mut StdRng) -> Result<Image> {
        image.manifest = None;
        image.index = None;
        image.config = None;
        image.layers.clear();
        image.blobs.clear();
        Ok(image)
    }
}

/// Empty layers with null diff_ids
#[derive(Debug)]
pub struct EmptyLayersNullDiffIds;

impl Mutation for EmptyLayersNullDiffIds {
    fn name(&self) -> &'static str {
        "empty-layers-null-diffids"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Extreme
    }

    fn apply(&self, mut image: Image, _rng: &mut StdRng) -> Result<Image> {
        // Clear all layers from manifest
        image.layers.clear();

        // Set diff_ids to None (serializes as null or omitted)
        if let Some(ref mut config) = image.config {
            config.rootfs.diff_ids = None;
        }

        Ok(image)
    }
}

/// Null diff_ids with layers intact
///
/// Sets diff_ids to null while keeping the actual layers.
/// This creates a mismatch where layers exist but config claims none.
#[derive(Debug)]
pub struct NullDiffIdsOnly;

impl Mutation for NullDiffIdsOnly {
    fn name(&self) -> &'static str {
        "null-diffids-only"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Extreme
    }

    fn apply(&self, mut image: Image, _rng: &mut StdRng) -> Result<Image> {
        // Keep layers but null out diff_ids
        if let Some(ref mut config) = image.config {
            config.rootfs.diff_ids = None;
        }
        Ok(image)
    }
}

/// Empty diff_ids array with layers intact
///
/// Sets diff_ids to an empty array while keeping actual layers.
/// This creates a count mismatch (N layers, 0 diff_ids).
#[derive(Debug)]
pub struct EmptyDiffIds;

impl Mutation for EmptyDiffIds {
    fn name(&self) -> &'static str {
        "empty-diffids"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Extreme
    }

    fn apply(&self, mut image: Image, _rng: &mut StdRng) -> Result<Image> {
        if let Some(ref mut config) = image.config {
            config.rootfs.diff_ids = Some(vec![]);
        }
        Ok(image)
    }
}

/// Mismatched diff_ids count
///
/// Adds extra fake diff_ids or removes some, creating a count mismatch
/// between manifest layers and config diff_ids.
#[derive(Debug)]
pub struct MismatchedDiffIdsCount;

impl Mutation for MismatchedDiffIdsCount {
    fn name(&self) -> &'static str {
        "mismatched-diffids-count"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Extreme
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        if let Some(ref mut config) = image.config {
            if let Some(ref mut diff_ids) = config.rootfs.diff_ids {
                // Randomly add extra or remove entries
                if rng.gen_bool(0.5) && !diff_ids.is_empty() {
                    // Remove half of the diff_ids
                    diff_ids.truncate(diff_ids.len() / 2);
                } else {
                    // Add fake diff_ids
                    for _ in 0..rng.gen_range(1..5) {
                        diff_ids.push(format!("sha256:{:064x}", rng.gen::<u128>()));
                    }
                }
            }
        }
        Ok(image)
    }
}

/// Create extremely long strings
#[derive(Debug)]
pub struct HugeStrings;

impl Mutation for HugeStrings {
    fn name(&self) -> &'static str {
        "huge-strings"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Extreme
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        let huge_string = "x".repeat(rng.gen_range(100_000..1_000_000));

        if let Some(ref mut image_config) = image.config {
            if let Some(ref mut container_config) = image_config.config {
                container_config.working_dir = Some(huge_string);
            }
        }

        Ok(image)
    }
}
