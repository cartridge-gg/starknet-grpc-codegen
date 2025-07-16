pub mod json_generator;
pub mod round_trip_tester;
pub mod test_runner;

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

use crate::spec::{Schema, Specification};

/// Configuration for the testing system
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Maximum depth for nested object generation
    pub max_depth: usize,
    /// Maximum number of items in arrays
    pub max_array_items: usize,
    /// Whether to generate optional fields
    pub generate_optional: bool,
    /// Seed for reproducible random generation
    pub seed: Option<u64>,
    /// Output directory for test artifacts
    pub output_dir: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            max_depth: 3,
            max_array_items: 5,
            generate_optional: true,
            seed: None,
            output_dir: "test_output".to_string(),
        }
    }
}

/// Result of a round-trip test
#[derive(Debug, Clone)]
pub struct RoundTripTestResult {
    /// Name of the type being tested
    pub type_name: String,
    /// Whether the test passed
    pub passed: bool,
    /// Input JSON that was used
    pub input_json: Value,
    /// Output JSON after round-trip
    pub output_json: Value,
    /// Error message if test failed
    pub error: Option<String>,
    /// Time taken for the test in milliseconds
    pub duration_ms: u64,
}

/// Summary of all test results
#[derive(Debug, Clone)]
pub struct TestSummary {
    /// Total number of tests run
    pub total_tests: usize,
    /// Number of tests that passed
    pub passed_tests: usize,
    /// Number of tests that failed
    pub failed_tests: usize,
    /// Detailed results for each test
    pub results: Vec<RoundTripTestResult>,
    /// Overall success rate
    pub success_rate: f64,
}

impl TestSummary {
    pub fn new() -> Self {
        Self {
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            results: Vec::new(),
            success_rate: 0.0,
        }
    }

    pub fn add_result(&mut self, result: RoundTripTestResult) {
        self.total_tests += 1;
        if result.passed {
            self.passed_tests += 1;
        } else {
            self.failed_tests += 1;
        }
        self.results.push(result);
        self.success_rate = self.passed_tests as f64 / self.total_tests as f64;
    }

    pub fn print_summary(&self) {
        println!("=== Round-Trip Test Summary ===");
        println!("Total tests: {}", self.total_tests);
        println!("Passed: {}", self.passed_tests);
        println!("Failed: {}", self.failed_tests);
        println!("Success rate: {:.2}%", self.success_rate * 100.0);

        if self.failed_tests > 0 {
            println!("\nFailed tests:");
            for result in &self.results {
                if !result.passed {
                    println!("  - {}: {}", result.type_name, result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
                }
            }
        }
    }
}

/// Main testing interface
pub struct RoundTripTester {
    config: TestConfig,
    spec: Specification,
    generated_protos: HashMap<String, String>,
}

impl RoundTripTester {
    pub fn new(spec: Specification, config: TestConfig) -> Self {
        Self {
            config,
            spec,
            generated_protos: HashMap::new(),
        }
    }

    /// Run comprehensive round-trip tests for all types in the specification
    pub fn run_all_tests(&mut self) -> Result<TestSummary> {
        let mut summary = TestSummary::new();
        
        println!("Starting round-trip tests for {} types...", self.spec.components.schemas.len());
        
        for (type_name, schema) in &self.spec.components.schemas {
            println!("Testing type: {}", type_name);
            
            let start_time = std::time::Instant::now();
            let result = self.test_type_round_trip(type_name, schema)?;
            let duration = start_time.elapsed();
            
            let test_result = RoundTripTestResult {
                type_name: type_name.clone(),
                passed: result.is_ok(),
                input_json: result.as_ref().map(|r| r.input_json.clone()).unwrap_or_default(),
                output_json: result.as_ref().map(|r| r.output_json.clone()).unwrap_or_default(),
                error: result.err().map(|e| e.to_string()),
                duration_ms: duration.as_millis() as u64,
            };
            
            summary.add_result(test_result);
            
            if result.is_ok() {
                println!("  ✅ Passed");
            } else {
                println!("  ❌ Failed: {}", result.unwrap_err());
            }
        }
        
        summary.print_summary();
        Ok(summary)
    }

    /// Test round-trip serialization for a specific type
    fn test_type_round_trip(&self, type_name: &str, schema: &Schema) -> Result<RoundTripTestResult> {
        // This will be implemented to use the json_generator and actual protobuf compilation
        // For now, return a placeholder
        Ok(RoundTripTestResult {
            type_name: type_name.to_string(),
            passed: true,
            input_json: serde_json::json!({}),
            output_json: serde_json::json!({}),
            error: None,
            duration_ms: 0,
        })
    }
}