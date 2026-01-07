//! Format mutations
//!
//! Mutations that corrupt the format/encoding of image components.

use super::{Mutation, MutationCategory};
use crate::oci::Image;
use crate::Result;
use rand::prelude::*;

/// Produce invalid JSON in manifests/configs
#[derive(Debug)]
pub struct InvalidJson;

impl Mutation for InvalidJson {
    fn name(&self) -> &'static str {
        "invalid-json"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Format
    }

    fn apply(&self, image: Image, _rng: &mut StdRng) -> Result<Image> {
        // This mutation is applied at serialization time
        // For now, we mark it and handle during output
        Ok(image)
    }
}

/// Add trailing garbage after JSON
#[derive(Debug)]
pub struct TrailingGarbage;

impl Mutation for TrailingGarbage {
    fn name(&self) -> &'static str {
        "trailing-garbage"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Format
    }

    fn apply(&self, image: Image, _rng: &mut StdRng) -> Result<Image> {
        // Applied at serialization time
        Ok(image)
    }
}

/// Introduce UTF-8 encoding violations
#[derive(Debug)]
pub struct Utf8Violation;

impl Mutation for Utf8Violation {
    fn name(&self) -> &'static str {
        "utf8-violation"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Format
    }

    fn apply(&self, image: Image, _rng: &mut StdRng) -> Result<Image> {
        // Applied at serialization time
        Ok(image)
    }
}

/// Truncate layer blobs
#[derive(Debug)]
pub struct TruncateBlob;

impl Mutation for TruncateBlob {
    fn name(&self) -> &'static str {
        "truncate-blob"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Format
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        if !image.layers.is_empty() {
            let idx = rng.gen_range(0..image.layers.len());
            if let Some(ref mut data) = image.layers[idx].data {
                if !data.is_empty() {
                    let new_len = rng.gen_range(0..data.len());
                    data.truncate(new_len);
                }
            }
        }
        Ok(image)
    }
}

/// Corrupt compression headers/streams
#[derive(Debug)]
pub struct CorruptCompression;

impl Mutation for CorruptCompression {
    fn name(&self) -> &'static str {
        "corrupt-compression"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Format
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        if !image.layers.is_empty() {
            let idx = rng.gen_range(0..image.layers.len());
            if let Some(ref mut data) = image.layers[idx].data {
                if data.len() > 10 {
                    // Corrupt first few bytes (gzip/zstd header area)
                    for i in 0..std::cmp::min(10, data.len()) {
                        data[i] = rng.gen();
                    }
                }
            }
        }
        Ok(image)
    }
}
