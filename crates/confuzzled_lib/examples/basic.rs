//! Basic example of using confuzzled library

use confuzzled_lib::{Fuzzer, MutationSet, Seed};

fn main() -> anyhow::Result<()> {
    // Create a fuzzer with minimal seed and semantic mutations
    let mut fuzzer = Fuzzer::builder()
        .seed(Seed::minimal())
        .mutations(MutationSet::SEMANTIC | MutationSet::STRUCTURAL)
        .rng_seed(42) // Fixed seed for reproducibility
        .build()?;

    // Generate 5 fuzzed images
    let images = fuzzer.generate(5);

    // Print generated images
    for (i, image) in images.iter().enumerate() {
        println!("Generated image {}", i);
        println!("  Has manifest: {}", image.manifest.is_some());
        println!("  Has config: {}", image.config.is_some());
        println!("  Layers: {}", image.layers.len());
        println!();
    }

    Ok(())
}
