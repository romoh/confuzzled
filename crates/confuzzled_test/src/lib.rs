//! Integration tests and fuzz runners for confuzzled
//!
//! This crate provides utilities for running integration tests
//! with generated container images.

use confuzzled_lib::oci::Image;
use confuzzled_lib::Result;

/// Test runner for executing fuzzed images against targets
pub struct TestRunner {
    images: Vec<Image>,
    timeout_seconds: u64,
}

impl TestRunner {
    /// Create a new test runner with the given images
    pub fn new(images: Vec<Image>) -> Self {
        Self {
            images,
            timeout_seconds: 300,
        }
    }

    /// Set the timeout for each test run
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Run all images against a target and collect results
    pub async fn run_all(&self) -> Result<Vec<TestResult>> {
        let mut results = Vec::with_capacity(self.images.len());

        for (i, image) in self.images.iter().enumerate() {
            let result = self.run_single(image, i).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Run a single image
    async fn run_single(&self, _image: &Image, index: usize) -> Result<TestResult> {
        tracing::info!("Running fuzzed image: {}", index);

        // TODO: Implement actual image testing against targets
        // This is a placeholder that simulates success

        Ok(TestResult {
            image_index: index,
            success: true,
            duration_ms: 0,
            error: None,
        })
    }
}

/// Result of a fuzzed image test run
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Image index
    pub image_index: usize,

    /// Whether the test succeeded
    pub success: bool,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Error message if failed
    pub error: Option<String>,
}

impl TestResult {
    /// Check if this result represents a successful test
    pub fn is_success(&self) -> bool {
        self.success
    }
}

/// Summary of test results
#[derive(Debug)]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration_ms: u64,
}

impl TestSummary {
    /// Create a summary from a list of test results
    pub fn from_results(results: &[TestResult]) -> Self {
        let total = results.len();
        let passed = results.iter().filter(|r| r.success).count();
        let failed = total - passed;
        let duration_ms = results.iter().map(|r| r.duration_ms).sum();

        Self {
            total,
            passed,
            failed,
            duration_ms,
        }
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use confuzzled_lib::{Fuzzer, MutationSet, Seed};

    #[tokio::test]
    async fn test_runner_basic() {
        let mut fuzzer = Fuzzer::builder()
            .seed(Seed::minimal())
            .mutations(MutationSet::STRUCTURAL)
            .rng_seed(42)
            .build()
            .unwrap();

        let images = fuzzer.generate(3);
        let runner = TestRunner::new(images);
        let results = runner.run_all().await.unwrap();

        assert_eq!(results.len(), 3);

        let summary = TestSummary::from_results(&results);
        assert!(summary.all_passed());
    }
}
