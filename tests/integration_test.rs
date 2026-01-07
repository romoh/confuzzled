//! Integration tests for confuzzled

use confuzzled_lib::{Fuzzer, MutationSet, Seed};

#[test]
fn test_basic_generation() {
    let fuzzer = Fuzzer::builder()
        .seed(Seed::Minimal)
        .mutations(MutationSet::STRUCTURAL)
        .rng_seed(12345)
        .build()
        .unwrap();

    let images = fuzzer.generate(10);
    assert_eq!(images.len(), 10);
}

#[test]
fn test_reproducible_generation() {
    let fuzzer1 = Fuzzer::builder()
        .seed(Seed::Minimal)
        .mutations(MutationSet::SEMANTIC)
        .rng_seed(999)
        .build()
        .unwrap();

    let fuzzer2 = Fuzzer::builder()
        .seed(Seed::Minimal)
        .mutations(MutationSet::SEMANTIC)
        .rng_seed(999)
        .build()
        .unwrap();

    let images1 = fuzzer1.generate(5);
    let images2 = fuzzer2.generate(5);

    // Same seed should produce same number of manifests
    for (i1, i2) in images1.iter().zip(images2.iter()) {
        assert_eq!(i1.manifests.len(), i2.manifests.len());
        assert_eq!(i1.layers.len(), i2.layers.len());
    }
}

#[test]
fn test_serialization() {
    let fuzzer = Fuzzer::builder()
        .seed(Seed::Minimal)
        .mutations(MutationSet::FORMAT)
        .rng_seed(42)
        .build()
        .unwrap();

    let images = fuzzer.generate(3);

    // Serialize to JSON
    let json = serde_json::to_string(&images).unwrap();
    assert!(!json.is_empty());

    // Deserialize back
    let deserialized: Vec<confuzzled_lib::oci::Image> =
        serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.len(), 3);
}
