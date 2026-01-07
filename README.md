# confuzzled 😵‍💫

[![Build & Test](https://github.com/romoh/confuzzled/actions/workflows/build.yml/badge.svg?label=Build%20%26%20Test)](https://github.com/romoh/confuzzled/actions/workflows/build.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A structure-aware CONtainer FUZZing tool for container images that can generate
malformed container images.

## Overview

The goal of confuzzled is to help platform developers, security researchers,
platform providers... and really anyone working with container images to test
the robustness of their container runtimes, image registries, scanners, and
admission controllers against malformed or unexpected container images.

Confuzzled focuses on *generating* fuzzed container images, not running them.
This keeps it reusable across different scenarios.

## Mutation Categories

| Category | Description | Examples |
|----------|-------------|----------|
| **Structural** | Remove/reorder/duplicate components | Remove manifest, reorder layers, dangling blobs |
| **Format** | Corrupt encoding/format | Truncated blobs, corrupted compression |
| **Semantic** | Valid format, invalid semantics | Digest mismatch, wrong size, bad media type |
| **Extreme** | Edge cases with extreme values | 100+ layers, deep nesting, empty everything |

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
| **Registry** | `docker.io/library/busybox:latest` | Container registry images |
| **Built-in** | `minimal` | Empty scratch-like image |
| **Built-in** | `busybox` | Minimal image with /bin/sh |

### Mutation Categories

| Category | Examples |
|----------|----------|
| **Structural** | Remove layers, duplicate descriptors, reorder layers, remove config |
| **Format** | Invalid media types, malformed JSON fields, text/plain layers |
| **Semantic** | Digest mismatches, wrong architecture, invalid timestamps |
| **Extreme** | 100+ empty layers, huge values, empty everything |

### Library Usage

```rust
use confuzzled_lib::{Seed, Pipeline};
use confuzzled_lib::mutations::structural::RemoveConfig;
use rand::SeedableRng;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Pull image from registry
    let image = Seed::Registry("mcr.microsoft.com/azurelinux/base/core:3.0".into())
        .load()
        .await?;

    // Apply mutations via pipeline
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let pipeline = Pipeline::new().then(RemoveConfig);
    let fuzzed = pipeline.apply(image, &mut rng)?;

    // Write to OCI directory
    fuzzed.write_to_oci_dir("./corpus/fuzzed-0000")?;
    Ok(())
}
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
│   ├── confuzzled_lib/        # Core library (includes examples/)
│   ├── confuzzled_cli/        # CLI binary
│   └── confuzzled_test/       # Test utilities
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
  if skopeo copy "oci:$dir" docker-daemon:test:fuzz 2>&1; then
    docker run --rm test:fuzz echo ok 2>&1 || echo "REJECTED by docker"
  else
    echo "REJECTED by skopeo"
  fi
done
```

## License

MIT License - see [LICENSE](LICENSE) for details
