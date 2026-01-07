//! Structural mutations
//!
//! Mutations that modify the structure of the image (remove/reorder/duplicate components).

use super::{Mutation, MutationCategory};
use crate::oci::Image;
use crate::Result;
use rand::prelude::*;

/// Remove the manifest from the image
#[derive(Debug)]
pub struct RemoveManifest;

impl Mutation for RemoveManifest {
    fn name(&self) -> &'static str {
        "remove-manifest"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Structural
    }

    fn apply(&self, mut image: Image, _rng: &mut StdRng) -> Result<Image> {
        image.manifest = None;
        Ok(image)
    }
}

/// Remove the config from the image
#[derive(Debug)]
pub struct RemoveConfig;

impl Mutation for RemoveConfig {
    fn name(&self) -> &'static str {
        "remove-config"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Structural
    }

    fn apply(&self, mut image: Image, _rng: &mut StdRng) -> Result<Image> {
        image.config = None;
        Ok(image)
    }
}

/// Remove the index.json from the image
#[derive(Debug)]
pub struct RemoveIndex;

impl Mutation for RemoveIndex {
    fn name(&self) -> &'static str {
        "remove-index"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Structural
    }

    fn apply(&self, mut image: Image, _rng: &mut StdRng) -> Result<Image> {
        image.index = None;
        Ok(image)
    }
}

/// Duplicate layers in the image
#[derive(Debug)]
pub struct DuplicateLayers;

impl Mutation for DuplicateLayers {
    fn name(&self) -> &'static str {
        "duplicate-layers"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Structural
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        if !image.layers.is_empty() {
            let idx = rng.gen_range(0..image.layers.len());
            let dup = image.layers[idx].clone();
            image.layers.push(dup);
        }
        Ok(image)
    }
}

/// Reorder layers randomly
#[derive(Debug)]
pub struct ReorderLayers;

impl Mutation for ReorderLayers {
    fn name(&self) -> &'static str {
        "reorder-layers"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Structural
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        image.layers.shuffle(rng);
        Ok(image)
    }
}

/// Add a blob that isn't referenced by any layer
#[derive(Debug)]
pub struct AddDanglingBlob;

impl Mutation for AddDanglingBlob {
    fn name(&self) -> &'static str {
        "add-dangling-blob"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Structural
    }

    fn apply(&self, mut image: Image, rng: &mut StdRng) -> Result<Image> {
        let fake_digest = format!("sha256:{:064x}", rng.gen::<u128>());
        let fake_data: Vec<u8> = (0..rng.gen_range(100..1000)).map(|_| rng.gen()).collect();
        image.blobs.insert(fake_digest, fake_data);
        Ok(image)
    }
}

/// Create an empty manifest
#[derive(Debug)]
pub struct EmptyManifest;

impl Mutation for EmptyManifest {
    fn name(&self) -> &'static str {
        "empty-manifest"
    }
    fn category(&self) -> MutationCategory {
        MutationCategory::Structural
    }

    fn apply(&self, mut image: Image, _rng: &mut StdRng) -> Result<Image> {
        if let Some(ref mut manifest) = image.manifest {
            manifest.layers.clear();
        }
        Ok(image)
    }
}
