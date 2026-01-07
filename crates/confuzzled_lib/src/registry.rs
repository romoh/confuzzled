//! OCI Distribution API client for pulling images from registries
//!
//! Uses the oci-distribution crate to pull images from OCI-compliant registries.

use crate::oci::{Descriptor, Image, ImageConfig, Layer, Manifest};
use crate::{ConfuzzledError, Result};
use oci_distribution::client::{Client, ClientConfig, ClientProtocol};
use oci_distribution::manifest::OciManifest;
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::Reference;
use std::collections::HashMap;

/// Pull an image from a container registry
///
/// # Arguments
/// * `reference` - Image reference like "busybox:latest", "docker.io/library/alpine:3.19"
///
/// # Examples
/// ```ignore
/// let image = pull_image("busybox:latest").await?;
/// let image = pull_image("ghcr.io/owner/repo:tag").await?;
/// ```
pub async fn pull_image(reference: &str) -> Result<Image> {
    // Parse the reference
    let reference: Reference = reference
        .parse()
        .map_err(|e| ConfuzzledError::RegistryError(format!("Invalid image reference: {}", e)))?;

    tracing::info!("Pulling image: {}", reference);

    // Create client with default config
    let client = Client::new(ClientConfig {
        protocol: ClientProtocol::Https,
        ..Default::default()
    });

    // Use anonymous auth for public images
    let auth = RegistryAuth::Anonymous;

    // Pull the manifest
    let (manifest, _digest) = client
        .pull_manifest(&reference, &auth)
        .await
        .map_err(|e| ConfuzzledError::RegistryError(format!("Failed to pull manifest: {}", e)))?;

    // Handle manifest type
    match manifest {
        OciManifest::Image(image_manifest) => {
            pull_single_image(&client, &reference, &auth, image_manifest).await
        }
        OciManifest::ImageIndex(index) => {
            // Multi-platform image - select linux/amd64
            let platform_manifest = index
                .manifests
                .iter()
                .find(|m| {
                    m.platform
                        .as_ref()
                        .is_some_and(|p| p.os == "linux" && p.architecture == "amd64")
                })
                .or_else(|| index.manifests.first())
                .ok_or_else(|| {
                    ConfuzzledError::RegistryError("No suitable manifest found".to_string())
                })?;

            // Create reference for specific digest
            let digest_ref = Reference::with_digest(
                reference.registry().to_string(),
                reference.repository().to_string(),
                platform_manifest.digest.clone(),
            );

            // Pull the specific manifest
            let (manifest, _) = client
                .pull_manifest(&digest_ref, &auth)
                .await
                .map_err(|e| {
                    ConfuzzledError::RegistryError(format!(
                        "Failed to pull platform manifest: {}",
                        e
                    ))
                })?;

            match manifest {
                OciManifest::Image(image_manifest) => {
                    pull_single_image(&client, &digest_ref, &auth, image_manifest).await
                }
                _ => Err(ConfuzzledError::RegistryError(
                    "Expected image manifest, got index".to_string(),
                )),
            }
        }
    }
}

/// Pull a single image given its manifest
async fn pull_single_image(
    client: &Client,
    reference: &Reference,
    _auth: &RegistryAuth,
    manifest: oci_distribution::manifest::OciImageManifest,
) -> Result<Image> {
    let mut blobs = HashMap::new();

    // Convert to our manifest type
    let config_descriptor = Descriptor {
        media_type: manifest.config.media_type.clone(),
        digest: manifest.config.digest.clone(),
        size: manifest.config.size,
        platform: None,
        annotations: None,
    };

    let layer_descriptors: Vec<Descriptor> = manifest
        .layers
        .iter()
        .map(|l| Descriptor {
            media_type: l.media_type.clone(),
            digest: l.digest.clone(),
            size: l.size,
            platform: None,
            annotations: None,
        })
        .collect();

    let our_manifest = Manifest {
        schema_version: 2,
        media_type: Some("application/vnd.oci.image.manifest.v1+json".to_string()),
        config: config_descriptor.clone(),
        layers: layer_descriptors.clone(),
        annotations: manifest.annotations.clone(),
    };

    // Fetch config - pull_blob writes to a Vec<u8>
    tracing::debug!("Fetching config: {}", manifest.config.digest);
    let mut config_bytes = Vec::new();
    client
        .pull_blob(reference, &manifest.config, &mut config_bytes)
        .await
        .map_err(|e| ConfuzzledError::RegistryError(format!("Failed to pull config: {}", e)))?;

    let config: ImageConfig = serde_json::from_slice(&config_bytes)
        .map_err(|e| ConfuzzledError::RegistryError(format!("Failed to parse config: {}", e)))?;

    blobs.insert(manifest.config.digest.clone(), config_bytes);

    // Fetch layers
    let mut layers = Vec::new();
    for (idx, layer) in manifest.layers.iter().enumerate() {
        tracing::debug!(
            "Fetching layer {}/{}: {} ({} bytes)",
            idx + 1,
            manifest.layers.len(),
            layer.digest,
            layer.size
        );

        let mut layer_bytes = Vec::new();
        client
            .pull_blob(reference, layer, &mut layer_bytes)
            .await
            .map_err(|e| {
                ConfuzzledError::RegistryError(format!(
                    "Failed to pull layer {}: {}",
                    layer.digest, e
                ))
            })?;

        blobs.insert(layer.digest.clone(), layer_bytes.clone());

        layers.push(Layer {
            descriptor: layer_descriptors[idx].clone(),
            data: Some(layer_bytes),
        });
    }

    tracing::info!("Successfully pulled image with {} layers", layers.len());

    Ok(Image {
        manifest: Some(our_manifest),
        index: None,
        config: Some(config),
        layers,
        blobs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_parsing() {
        // Simple name
        let r: Reference = "busybox".parse().unwrap();
        assert_eq!(r.repository(), "library/busybox");

        // With tag
        let r: Reference = "busybox:1.36".parse().unwrap();
        assert_eq!(r.repository(), "library/busybox");
        assert_eq!(r.tag(), Some("1.36"));

        // Full reference
        let r: Reference = "ghcr.io/owner/repo:v1.0".parse().unwrap();
        assert_eq!(r.registry(), "ghcr.io");
        assert_eq!(r.repository(), "owner/repo");
        assert_eq!(r.tag(), Some("v1.0"));
    }
}
