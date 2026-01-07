//! Output adapters for writing fuzzed images
//!
//! Supports various output formats: OCI directory, Docker tarball, registry push.

use crate::oci::Image;
use crate::Result;
use std::path::Path;

/// Trait for output destinations
pub trait OutputAdapter: Send + Sync {
    /// Write an image to the output
    fn write(&self, image: &Image, name: &str) -> Result<()>;
}

/// Write images to OCI directory layout
pub struct OciDirOutput {
    base_path: std::path::PathBuf,
}

impl OciDirOutput {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            base_path: path.as_ref().to_path_buf(),
        }
    }
}

impl OutputAdapter for OciDirOutput {
    fn write(&self, image: &Image, name: &str) -> Result<()> {
        let path = self.base_path.join(name);
        image.write_to_oci_dir(&path)?;
        tracing::info!("Wrote OCI image to: {}", path.display());
        Ok(())
    }
}

/// Write images to Docker tarballs
pub struct DockerTarOutput {
    base_path: std::path::PathBuf,
}

impl DockerTarOutput {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            base_path: path.as_ref().to_path_buf(),
        }
    }
}

impl OutputAdapter for DockerTarOutput {
    fn write(&self, image: &Image, name: &str) -> Result<()> {
        let path = self.base_path.join(format!("{}.tar", name));
        image.write_to_docker_tar(&path)?;
        tracing::info!("Wrote Docker tar to: {}", path.display());
        Ok(())
    }
}

/// Write images to a container registry
pub struct RegistryOutput {
    registry_url: String,
    repository: String,
}

impl RegistryOutput {
    pub fn new(registry_url: &str, repository: &str) -> Self {
        Self {
            registry_url: registry_url.to_string(),
            repository: repository.to_string(),
        }
    }
}

impl OutputAdapter for RegistryOutput {
    fn write(&self, _image: &Image, name: &str) -> Result<()> {
        // TODO: Implement registry push
        tracing::info!(
            "Would push to {}/{}/{}",
            self.registry_url,
            self.repository,
            name
        );
        Ok(())
    }
}

/// In-memory output for testing
pub struct InMemoryOutput {
    images: std::sync::Mutex<Vec<(String, Image)>>,
}

impl InMemoryOutput {
    pub fn new() -> Self {
        Self {
            images: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn images(&self) -> Vec<(String, Image)> {
        self.images.lock().unwrap().clone()
    }
}

impl Default for InMemoryOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputAdapter for InMemoryOutput {
    fn write(&self, image: &Image, name: &str) -> Result<()> {
        self.images
            .lock()
            .unwrap()
            .push((name.to_string(), image.clone()));
        Ok(())
    }
}
