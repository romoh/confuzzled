//! Seed image sources for fuzzing
//!
//! Seeds are known-good container images used as the starting point for fuzzing.

use crate::oci::Image;
use crate::registry::pull_image;
use crate::Result;
use std::path::PathBuf;

/// Source for seed images
#[derive(Debug, Clone)]
pub enum Seed {
    /// Load from OCI image layout directory (placeholder)
    OciDir(PathBuf),

    /// Load from Docker tarball (placeholder)
    DockerTar(PathBuf),

    /// Pull from container registry
    Registry(String),

    /// Minimal scratch-like image (in-memory)
    Minimal,

    /// Busybox-style image
    Busybox,
}

impl Seed {
    /// Create seed from OCI directory layout
    pub fn from_oci_dir<P: Into<PathBuf>>(path: P) -> Self {
        Self::OciDir(path.into())
    }

    /// Create seed from Docker tarball
    pub fn from_docker_tar<P: Into<PathBuf>>(path: P) -> Self {
        Self::DockerTar(path.into())
    }

    /// Create seed from container registry
    ///
    /// # Examples
    /// ```ignore
    /// Seed::from_registry("busybox")
    /// Seed::from_registry("busybox:1.36")
    /// Seed::from_registry("docker.io/library/alpine:latest")
    /// Seed::from_registry("ghcr.io/owner/repo:tag")
    /// ```
    pub fn from_registry<S: Into<String>>(reference: S) -> Self {
        Self::Registry(reference.into())
    }

    /// Create a minimal scratch-like seed
    pub fn minimal() -> Self {
        Self::Minimal
    }

    /// Create a busybox seed
    pub fn busybox() -> Self {
        Self::Busybox
    }

    /// Load the seed image (blocking)
    pub fn load(&self) -> Result<Image> {
        match self {
            Self::OciDir(path) => Image::from_oci_dir(path),
            Self::DockerTar(path) => Image::from_docker_tar(path),
            Self::Registry(reference) => {
                // Use tokio runtime to run async code
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| crate::ConfuzzledError::RegistryError(e.to_string()))?;
                rt.block_on(pull_image(reference))
            }
            Self::Minimal => Ok(Image::minimal()),
            Self::Busybox => Ok(Image::busybox()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_seed() {
        let seed = Seed::minimal();
        let image = seed.load().unwrap();
        assert!(!image.layers.is_empty() || image.config.is_some());
    }
}
