//! Basic example of using confuzzled library

use confuzzled_lib::{Fuzzer, MutationSet, Seed};

fn main() -> anyhow::Result<()> {
    // Create a fuzzer with minimal seed and semantic mutations
    let fuzzer = Fuzzer::builder()
        .seed(Seed::Minimal)
        .mutations(MutationSet::SEMANTIC | MutationSet::STRUCTURAL)
        .rng_seed(42) // Fixed seed for reproducibility
        .build()?;

    // Generate 5 fuzzed images
    let images = fuzzer.generate(5);

    // Print generated images
    for (i, image) in images.iter().enumerate() {
        println!("Generated image {}", i);
        println!("  Manifests: {}", image.manifests.len());
        if let Some(ref config) = image.config {
            if let Some(ref container_config) = config.config {
                println!("  Entrypoint: {:?}", container_config.entrypoint);
                println!("  Cmd: {:?}", container_config.cmd);
            }
        }
        println!("  Layers: {}", image.layers.len());
        println!();
    }

    Ok(())
}
