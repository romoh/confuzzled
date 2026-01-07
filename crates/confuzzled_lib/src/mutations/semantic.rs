//! Semantic mutations
//!
//! Mutations that produce valid format but invalid semantics.

use super::{Mutation, MutationCategory};
use crate::oci::Image;
use crate::Result;
use rand::prelude::*;

/// Make digest not match actual content
#[derive(Debug)]
pub struct DigestMismatch;

impl Mutation for DigestMismatch {
    fn name(&self) -> &'static str {
        "digest-mismatch"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Semantic
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        if !image.layers.is_empty() {
            let idx = rng.gen_range(0..image.layers.len());
            // Replace digest with a fake one
            image.layers[idx].descriptor.digest = format!("sha256:{:064x}", rng.gen::<u128>());
        }
        Ok(image)
    }
}

/// Make size field not match actual content size
#[derive(Debug)]
pub struct SizeMismatch;

impl Mutation for SizeMismatch {
    fn name(&self) -> &'static str {
        "size-mismatch"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Semantic
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        if !image.layers.is_empty() {
            let idx = rng.gen_range(0..image.layers.len());
            // Set wrong size
            image.layers[idx].descriptor.size = rng.gen_range(0..1000000);
        }
        Ok(image)
    }
}

/// Use incorrect media types
#[derive(Debug)]
pub struct WrongMediaType;

impl Mutation for WrongMediaType {
    fn name(&self) -> &'static str {
        "wrong-media-type"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Semantic
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        let wrong_types = [
            "application/octet-stream",
            "text/plain",
            "application/vnd.docker.distribution.manifest.v1+json",
            "application/vnd.oci.image.layer.v1.tar",
            "application/vnd.oci.image.layer.v1.tar+zstd",
            "image/png",
        ];

        if !image.layers.is_empty() {
            let idx = rng.gen_range(0..image.layers.len());
            image.layers[idx].descriptor.media_type =
                wrong_types[rng.gen_range(0..wrong_types.len())].to_string();
        }
        Ok(image)
    }
}

/// Use mismatched platform specifications
#[derive(Debug)]
pub struct PlatformMismatch;

impl Mutation for PlatformMismatch {
    fn name(&self) -> &'static str {
        "platform-mismatch"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Semantic
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        let platforms = [
            ("linux", "amd64"),
            ("linux", "arm64"),
            ("windows", "amd64"),
            ("darwin", "arm64"),
            ("freebsd", "amd64"),
        ];

        if let Some(ref mut config) = image.config {
            let (os, arch) = platforms[rng.gen_range(0..platforms.len())];
            // Set config to different platform than layers claim
            config.os = os.to_string();
            config.architecture = arch.to_string();
        }
        Ok(image)
    }
}

/// Add invalid or problematic annotations
#[derive(Debug)]
pub struct InvalidAnnotations;

impl Mutation for InvalidAnnotations {
    fn name(&self) -> &'static str {
        "invalid-annotations"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Semantic
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        use std::collections::HashMap;

        let mut annotations = HashMap::new();

        // Add problematic annotations
        annotations.insert("".to_string(), "empty key".to_string());
        annotations.insert("key\0with\0nulls".to_string(), "value".to_string());
        annotations.insert(
            "org.opencontainers.image.created".to_string(),
            "not-a-date".to_string(),
        );
        annotations.insert("a".repeat(10000), "huge key".to_string());

        if let Some(ref mut manifest) = image.manifest {
            manifest.annotations = Some(annotations.clone());
        }

        if let Some(ref mut index) = image.index {
            index.annotations = Some(annotations);
        }

        let _ = rng; // Silence unused warning
        Ok(image)
    }
}

/// Reference layers that don't exist
#[derive(Debug)]
pub struct MissingLayerReference;

impl Mutation for MissingLayerReference {
    fn name(&self) -> &'static str {
        "missing-layer-reference"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Semantic
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        use crate::oci::{Descriptor, Layer};

        // Add a layer descriptor that references a non-existent blob
        let fake_layer = Layer {
            descriptor: Descriptor {
                media_type: "application/vnd.oci.image.layer.v1.tar+gzip".to_string(),
                digest: format!("sha256:{:064x}", rng.gen::<u128>()),
                size: rng.gen_range(1000..100000),
                platform: None,
                annotations: None,
            },
            data: None, // No actual data!
        };

        image.layers.push(fake_layer);
        Ok(image)
    }
}
