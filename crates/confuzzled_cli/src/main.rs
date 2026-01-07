//! confuzzled CLI - Structure-aware container image fuzzer
//!
//! Generate malformed container images to test runtimes, registries, and scanners.

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use confuzzled_lib::output::{OciDirOutput, OutputAdapter};
use confuzzled_lib::{Fuzzer, MutationSet, Seed};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser)]
#[command(name = "confuzzled")]
#[command(author, version, about = "Structure-aware container image fuzzer", long_about = None)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate fuzzed container images
    Generate {
        /// Seed image source (path to OCI dir, Docker tar, or "minimal"/"busybox")
        #[arg(short, long, default_value = "minimal")]
        seed: String,

        /// Mutation categories to apply (structural,format,semantic,extreme,all)
        #[arg(short, long, default_value = "semantic,structural")]
        mutations: String,

        /// Number of images to generate
        #[arg(short, long, default_value = "10")]
        count: usize,

        /// RNG seed for deterministic generation
        #[arg(long)]
        seed_rng: Option<u64>,

        /// Output directory
        #[arg(short, long, default_value = "./corpus")]
        out: String,

        /// Output format
        #[arg(short, long, default_value = "oci")]
        format: OutputFormat,
    },

    /// List available mutations
    Mutations {
        /// List mutations for specific category
        #[arg(short, long)]
        category: Option<String>,

        /// Show detailed descriptions
        #[arg(long)]
        list: bool,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Oci,
    DockerTar,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging
    let log_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level)))
        .init();

    match cli.command {
        Commands::Generate {
            seed,
            mutations,
            count,
            seed_rng,
            out,
            format,
        } => {
            tracing::info!("Generating {} fuzzed images...", count);

            // Parse seed
            let seed_source = match seed.as_str() {
                "minimal" => Seed::minimal(),
                "busybox" => Seed::busybox(),
                s if s.ends_with(".tar") => Seed::from_docker_tar(s),
                s if s.contains('/') || s.contains(':') || s.contains('.') => {
                    // Looks like a registry reference (e.g., busybox:1.36, docker.io/library/alpine)
                    Seed::from_registry(s)
                }
                s if std::path::Path::new(s).exists() => Seed::from_oci_dir(s),
                other => {
                    // Try as registry reference
                    Seed::from_registry(other)
                }
            };

            // Parse mutations
            let mutation_set: MutationSet =
                mutations.parse().map_err(|e: String| anyhow::anyhow!(e))?;

            // Build fuzzer
            let mut builder = Fuzzer::builder().seed(seed_source).mutations(mutation_set);

            if let Some(rng_seed) = seed_rng {
                builder = builder.rng_seed(rng_seed);
            }

            let mut fuzzer = builder.build()?;

            // Generate images
            let images = fuzzer.generate(count);

            // Write output
            std::fs::create_dir_all(&out)?;
            let output: Box<dyn OutputAdapter> = match format {
                OutputFormat::Oci => Box::new(OciDirOutput::new(&out)),
                OutputFormat::DockerTar => {
                    Box::new(confuzzled_lib::output::DockerTarOutput::new(&out))
                }
            };

            for (i, image) in images.iter().enumerate() {
                let name = format!("fuzzed-{:04}", i);
                output.write(image, &name)?;
            }

            tracing::info!(
                "Successfully generated {} fuzzed images to {}",
                images.len(),
                out
            );
        }

        Commands::Mutations { category, list } => {
            println!("Available mutation categories:\n");

            let categories = [
                (
                    "structural",
                    vec![
                        ("remove-manifest", "Remove the manifest from the image"),
                        ("remove-config", "Remove the config from the image"),
                        ("remove-index", "Remove index.json from the image"),
                        ("duplicate-layers", "Duplicate random layers"),
                        ("reorder-layers", "Randomly reorder layers"),
                        ("add-dangling-blob", "Add unreferenced blob"),
                        ("empty-manifest", "Create empty manifest"),
                    ],
                ),
                (
                    "format",
                    vec![
                        ("invalid-json", "Produce invalid JSON"),
                        ("trailing-garbage", "Add trailing bytes after JSON"),
                        ("utf8-violation", "Introduce UTF-8 encoding errors"),
                        ("truncate-blob", "Truncate layer blobs"),
                        ("corrupt-compression", "Corrupt compression headers"),
                    ],
                ),
                (
                    "semantic",
                    vec![
                        ("digest-mismatch", "Digest doesn't match content"),
                        ("size-mismatch", "Size field doesn't match"),
                        ("wrong-media-type", "Use incorrect media types"),
                        ("platform-mismatch", "Mismatched platform specs"),
                        ("invalid-annotations", "Add problematic annotations"),
                        ("missing-layer-reference", "Reference non-existent layers"),
                    ],
                ),
                (
                    "extreme",
                    vec![
                        ("huge-layer-count", "Add hundreds of layers"),
                        ("deep-path-nesting", "Deeply nested paths"),
                        ("extreme-values", "Boundary/extreme values"),
                        ("empty-everything", "Remove all content"),
                    ],
                ),
            ];

            for (cat_name, mutations_list) in &categories {
                if category.as_ref().is_none_or(|c| c == *cat_name) {
                    println!("{}:", cat_name.to_uppercase());
                    for (name, desc) in mutations_list {
                        if list {
                            println!("  {:<25} {}", name, desc);
                        } else {
                            println!("  {}", name);
                        }
                    }
                    println!();
                }
            }
        }
    }

    Ok(())
}
