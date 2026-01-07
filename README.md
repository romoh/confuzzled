# confuzzled (CONtainer FUZZing LED)

A structure-aware fuzzing tool for container images that can generate malformed
container images.

## Overview

The goal of confuzzled is to help platform developers, security researchers,
platform providers... and really anyone working with container images to test
the robustness of their container runtimes, image registries, scanners, and
admission controllers against malformed or unexpected container images.

Confuzzled focuses on *generating* fuzzed container images, not running them.
This keeps it reusable across:
- Container runtimes (runc, crun, kata, gVisor)
- Image registries (Docker Hub, Harbor, Zot)
- Image scanners (Trivy, Grype, Snyk)
- Admission controllers (OPA, Kyverno)
- Snapshotters and storage drivers

## Fuzzing Levels

### A. Artifact Level (file/bytes)
- Missing headers
- Malformed JSON manifests
- Truncated layer blobs
- Invalid checksums
- Corrupted compression streams

### B. Semantic Level (container invariants)
- Valid JSON but invalid semantics
- Layer order mismatches
- Config references non-existent layers
- Wrong mediaTypes
- Size/digest mismatches
- Entrypoint/env/resource edge cases

## Installation

```bash
# Build from source
cargo build --release

# Install CLI
cargo install --path crates/confuzzled_cli
```

## Quick Start

```bash
# Generate 3 fuzzed images from Azure Linux (pulls from registry)
confuzzled generate \
  --seed "mcr.microsoft.com/azurelinux/base/core:3.0" \
  --mutations all \
  --count 3 \
  --out ./corpus

# Generate from minimal scratch image
confuzzled generate --seed minimal --count 10 --out ./corpus

# Generate from busybox-like image
confuzzled generate --seed busybox --count 10 --out ./corpus

# Load and test a generated image
skopeo copy oci:./corpus/fuzzed-0000 docker-daemon:test:latest
docker run --rm test:latest echo "It works!"
```

## Usage

### CLI

```bash
# Generate fuzzed images from a registry image
confuzzled generate \
  --seed "docker.io/library/alpine:latest" \
  --mutations semantic,structural \
  --count 100 \
  --out ./corpus

# Use all mutation categories
confuzzled generate \
  --seed busybox \
  --mutations all \
  --count 50 \
  --out ./corpus

# Deterministic mode for reproducibility
confuzzled generate \
  --seed minimal \
  --seed-rng 42 \
  --count 100 \
  --out ./corpus

# List available mutations
confuzzled mutations --list

# List mutations by category
confuzzled mutations --category structural
```

### Seed Sources

| Seed Type | Example | Description |
|-----------|---------|-------------|
| **Registry** | `mcr.microsoft.com/azurelinux/base/core:3.0` | Pull from container registry |
| **Registry** | `docker.io/library/busybox:latest` | Docker Hub images |
| **Registry** | `ghcr.io/owner/repo:tag` | GitHub Container Registry |
| **Built-in** | `minimal` | Empty scratch-like image |
| **Built-in** | `busybox` | Minimal image with /bin/sh |
| **Local** | `./path/to/oci-dir` | OCI image layout directory |

### Mutation Categories

| Category | Examples |
|----------|----------|
| **Structural** | Remove layers, duplicate descriptors, reorder layers, remove config |
| **Format** | Invalid media types, malformed JSON fields, text/plain layers |
| **Semantic** | Digest mismatches, wrong architecture, invalid timestamps |
| **Extreme** | 100+ empty layers, huge values, empty everything |

### Library Usage

```rust
use confuzzled::{Fuzzer, Seed, MutationCategory};

fn main() -> anyhow::Result<()> {
    // Pull image from registry and fuzz it
    let fuzzer = Fuzzer::builder()
        .seed(Seed::from_registry("mcr.microsoft.com/azurelinux/base/core:3.0"))
        .mutations(vec![MutationCategory::Structural, MutationCategory::Semantic])
        .rng_seed(Some(42))  // deterministic
        .build()?;

    for (i, image) in fuzzer.generate(10).enumerate() {
        let path = format!("./corpus/fuzzed-{:04}", i);
        image.write_to_oci_dir(&path)?;
    }

    Ok(())
}
```

### Mutation Pipeline (Composable)

```rust
use confuzzled::mutations::{Pipeline, Mutation};
use confuzzled::mutations::structural::RemoveLayer;
use confuzzled::mutations::extreme::AddManyLayers;

let pipeline = Pipeline::new()
    .add(Box::new(RemoveLayer))
    .add(Box::new(AddManyLayers));

let fuzzed = pipeline.apply(seed_image)?;
```

## Architecture

```
┌───────────────────────┐
│     Seed Sources      │
│ (Registry/OCI/Minimal)│
└───────────┬───────────┘
            │
┌───────────▼───────────┐
│    Mutation Engine    │
│  (structural/format/  │
│   semantic/extreme)   │
└───────────┬───────────┘
            │
┌───────────▼───────────┐
│     OCI Image Writer  │
│  (blobs + manifest)   │
└───────────┬───────────┘
            │
┌───────────▼───────────┐
│   Output Directory    │
│   (OCI layout format) │
└───────────────────────┘
```

## Project Structure

```
confuzzled/
├── crates/
│   ├── confuzzled_lib/        # Core library
│   ├── confuzzled_cli/        # CLI binary
│   └── confuzzled_test/       # Test utilities
├── examples/
└── tests/
```

## Seed Images

Confuzzled can pull seed images from container registries or use built-in
minimal images:

### Registry Images (Recommended)
```bash
# Microsoft Azure Linux
confuzzled generate --seed "mcr.microsoft.com/azurelinux/base/core:3.0" ...

# Docker Hub
confuzzled generate --seed "docker.io/library/alpine:3.19" ...
confuzzled generate --seed "busybox:latest" ...

# GitHub Container Registry  
confuzzled generate --seed "ghcr.io/owner/repo:tag" ...
```

### Built-in Images
- `minimal` - Empty scratch image (default)
- `busybox` - Minimal image with /bin/sh entrypoint

## Example: Fuzzing a Container Runtime

```bash
# 1. Generate mutated images
confuzzled generate \
  --seed "mcr.microsoft.com/azurelinux/base/core:3.0" \
  --mutations all \
  --count 100 \
  --out ./corpus

# 2. Test each image against container runtime
for dir in ./corpus/fuzzed-*; do
  echo "Testing $dir..."
  skopeo copy "oci:$dir" docker-daemon:test:fuzz 2>&1 || echo "REJECTED by skopeo"
  docker run --rm test:fuzz echo ok 2>&1 || echo "REJECTED by docker"
done
```

## License

MIT License - see [LICENSE](LICENSE) for details
