//! OCI image types and structures
//!
//! Represents OCI image layout for manipulation and fuzzing.

use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Represents an OCI container image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    /// Image manifest
    pub manifest: Option<Manifest>,

    /// Image index (for multi-platform images)
    pub index: Option<ImageIndex>,

    /// Image configuration
    pub config: Option<ImageConfig>,

    /// Layer blobs
    pub layers: Vec<Layer>,

    /// Raw blobs by digest
    #[serde(skip)]
    pub blobs: HashMap<String, Vec<u8>>,
}

impl Image {
    /// Create a minimal scratch-like image
    pub fn minimal() -> Self {
        Self {
            manifest: Some(Manifest::minimal()),
            index: Some(ImageIndex::minimal()),
            config: Some(ImageConfig::minimal()),
            layers: vec![Layer::empty()],
            blobs: HashMap::new(),
        }
    }

    /// Create a busybox-style image
    pub fn busybox() -> Self {
        let mut image = Self::minimal();
        if let Some(ref mut image_config) = image.config {
            if let Some(ref mut container_config) = image_config.config {
                container_config.cmd = Some(vec!["/bin/sh".to_string()]);
                container_config.entrypoint = None;
            }
        }
        image
    }

    /// Load from OCI directory layout
    ///
    /// # Not Yet Implemented
    /// This is a placeholder - currently returns a minimal image.
    pub fn from_oci_dir<P: AsRef<Path>>(_path: P) -> Result<Self> {
        // TODO: Implement OCI directory loading
        // - Read oci-layout file
        // - Parse index.json
        // - Load manifest and config blobs
        // - Load layer blobs
        Ok(Self::minimal())
    }

    /// Load from Docker tarball
    ///
    /// # Not Yet Implemented
    /// This is a placeholder - currently returns a minimal image.
    pub fn from_docker_tar<P: AsRef<Path>>(_path: P) -> Result<Self> {
        // TODO: Implement Docker tar loading
        // - Extract tar archive
        // - Parse manifest.json
        // - Load config and layer blobs
        Ok(Self::minimal())
    }

    /// Write to OCI directory layout
    pub fn write_to_oci_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        use sha2::{Digest, Sha256};

        let path = path.as_ref();
        std::fs::create_dir_all(path)?;

        // Write oci-layout
        let layout = OciLayout {
            image_layout_version: "1.0.0".to_string(),
        };
        std::fs::write(
            path.join("oci-layout"),
            serde_json::to_string_pretty(&layout)?,
        )?;

        // Create blobs directory
        let blobs_dir = path.join("blobs").join("sha256");
        std::fs::create_dir_all(&blobs_dir)?;

        // Helper to write blob and return descriptor
        let write_blob = |data: &[u8], media_type: &str| -> Result<Descriptor> {
            let mut hasher = Sha256::new();
            hasher.update(data);
            let hash = format!("{:x}", hasher.finalize());
            let digest = format!("sha256:{}", hash);

            std::fs::write(blobs_dir.join(&hash), data)?;

            Ok(Descriptor {
                media_type: media_type.to_string(),
                digest,
                size: data.len() as i64,
                platform: None,
                annotations: None,
            })
        };

        // Write config blob
        let config_descriptor = if let Some(ref config) = self.config {
            let config_json = serde_json::to_vec_pretty(config)?;
            Some(write_blob(
                &config_json,
                "application/vnd.oci.image.config.v1+json",
            )?)
        } else {
            None
        };

        // Write layer blobs - normalize Docker media types to OCI
        let mut layer_descriptors = Vec::new();
        for layer in &self.layers {
            if let Some(ref data) = layer.data {
                // Convert Docker layer media types to OCI
                let media_type = normalize_layer_media_type(&layer.descriptor.media_type);
                let desc = write_blob(data, &media_type)?;
                layer_descriptors.push(desc);
            }
        }

        // Write any additional blobs
        for (digest, data) in &self.blobs {
            let hash = digest.trim_start_matches("sha256:");
            let blob_path = blobs_dir.join(hash);
            if !blob_path.exists() {
                std::fs::write(blob_path, data)?;
            }
        }

        // Build and write manifest
        let manifest = if let Some(ref m) = self.manifest {
            Manifest {
                schema_version: m.schema_version,
                media_type: m.media_type.clone(),
                config: config_descriptor.clone().unwrap_or_else(|| Descriptor {
                    media_type: "application/vnd.oci.image.config.v1+json".to_string(),
                    digest: "sha256:empty".to_string(),
                    size: 0,
                    platform: None,
                    annotations: None,
                }),
                layers: layer_descriptors,
                annotations: m.annotations.clone(),
            }
        } else {
            Manifest {
                schema_version: 2,
                media_type: Some("application/vnd.oci.image.manifest.v1+json".to_string()),
                config: config_descriptor.clone().unwrap_or_else(|| Descriptor {
                    media_type: "application/vnd.oci.image.config.v1+json".to_string(),
                    digest: "sha256:empty".to_string(),
                    size: 0,
                    platform: None,
                    annotations: None,
                }),
                layers: vec![],
                annotations: None,
            }
        };

        let manifest_json = serde_json::to_vec_pretty(&manifest)?;
        let manifest_descriptor =
            write_blob(&manifest_json, "application/vnd.oci.image.manifest.v1+json")?;

        // Write index.json with manifest reference
        let index = ImageIndex {
            schema_version: 2,
            media_type: Some("application/vnd.oci.image.index.v1+json".to_string()),
            manifests: vec![manifest_descriptor],
            annotations: None,
        };
        std::fs::write(
            path.join("index.json"),
            serde_json::to_string_pretty(&index)?,
        )?;

        Ok(())
    }

    /// Write to Docker tarball
    pub fn write_to_docker_tar<P: AsRef<Path>>(&self, _path: P) -> Result<()> {
        // TODO: Implement Docker tar writing
        Ok(())
    }
}

/// Normalize Docker layer media types to OCI format
fn normalize_layer_media_type(media_type: &str) -> String {
    match media_type {
        // Docker compressed layer -> OCI compressed layer
        "application/vnd.docker.image.rootfs.diff.tar.gzip" => {
            "application/vnd.oci.image.layer.v1.tar+gzip".to_string()
        }
        // Docker uncompressed layer -> OCI uncompressed layer
        "application/vnd.docker.image.rootfs.diff.tar" => {
            "application/vnd.oci.image.layer.v1.tar".to_string()
        }
        // Already OCI or unknown, pass through
        _ => media_type.to_string(),
    }
}

/// OCI layout file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciLayout {
    #[serde(rename = "imageLayoutVersion")]
    pub image_layout_version: String,
}

/// OCI image index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageIndex {
    #[serde(rename = "schemaVersion")]
    pub schema_version: i32,

    #[serde(rename = "mediaType", skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,

    pub manifests: Vec<Descriptor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

impl ImageIndex {
    pub fn minimal() -> Self {
        Self {
            schema_version: 2,
            media_type: Some("application/vnd.oci.image.index.v1+json".to_string()),
            manifests: vec![],
            annotations: None,
        }
    }
}

/// OCI image manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: i32,

    #[serde(rename = "mediaType", skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,

    pub config: Descriptor,

    pub layers: Vec<Descriptor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

impl Manifest {
    pub fn minimal() -> Self {
        Self {
            schema_version: 2,
            media_type: Some("application/vnd.oci.image.manifest.v1+json".to_string()),
            config: Descriptor {
                media_type: "application/vnd.oci.image.config.v1+json".to_string(),
                digest: "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                    .to_string(),
                size: 0,
                platform: None,
                annotations: None,
            },
            layers: vec![],
            annotations: None,
        }
    }
}

/// OCI content descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Descriptor {
    #[serde(rename = "mediaType")]
    pub media_type: String,

    pub digest: String,

    pub size: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<Platform>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

/// Platform specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub architecture: String,
    pub os: String,

    #[serde(rename = "os.version", skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,

    #[serde(rename = "os.features", skip_serializing_if = "Option::is_none")]
    pub os_features: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

/// OCI image configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub architecture: String,
    pub os: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ContainerConfig>,

    pub rootfs: RootFs,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<History>>,
}

impl ImageConfig {
    pub fn minimal() -> Self {
        Self {
            architecture: "amd64".to_string(),
            os: "linux".to_string(),
            config: Some(ContainerConfig::default()),
            rootfs: RootFs {
                // Empty tar has this digest (uncompressed)
                // The hash below is for empty content
                diff_ids: vec![
                    "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                        .to_string(),
                ],
                rootfs_type: "layers".to_string(),
            },
            history: None,
        }
    }
}

/// Container runtime configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContainerConfig {
    #[serde(rename = "Cmd", skip_serializing_if = "Option::is_none")]
    pub cmd: Option<Vec<String>>,

    #[serde(rename = "Entrypoint", skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<Vec<String>>,

    #[serde(rename = "Env", skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<String>>,

    #[serde(rename = "WorkingDir", skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,

    #[serde(rename = "User", skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    #[serde(rename = "ExposedPorts", skip_serializing_if = "Option::is_none")]
    pub exposed_ports: Option<HashMap<String, serde_json::Value>>,

    #[serde(rename = "Volumes", skip_serializing_if = "Option::is_none")]
    pub volumes: Option<HashMap<String, serde_json::Value>>,

    #[serde(rename = "Labels", skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
}

/// Root filesystem specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootFs {
    #[serde(rename = "type")]
    pub rootfs_type: String,

    pub diff_ids: Vec<String>,
}

/// Image history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub empty_layer: Option<bool>,
}

/// Represents a layer in the image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Layer descriptor
    pub descriptor: Descriptor,

    /// Raw layer data (compressed)
    #[serde(skip)]
    pub data: Option<Vec<u8>>,
}

impl Layer {
    /// Create an empty layer
    pub fn empty() -> Self {
        Self {
            descriptor: Descriptor {
                media_type: "application/vnd.oci.image.layer.v1.tar+gzip".to_string(),
                digest: "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                    .to_string(),
                size: 0,
                platform: None,
                annotations: None,
            },
            data: Some(vec![]),
        }
    }
}
