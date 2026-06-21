#!/usr/bin/env rustc

// Simple test to verify cyclic granfina dashboard functionality

use std::fs;
use std::path::Path;

fn main() {
    println!("Testing Cyclic Granfina Dashboard...");

    // Check if the dashboard file exists
    let dashboard_path = "tools/cyclic_granfina_dashboard/debug_runs/cyclic_granfina_dashboard_live.json";
    if Path::new(dashboard_path).exists() {
        println!("✓ Dashboard file exists: {}", dashboard_path);

        // Try to read the file
        match fs::read_to_string(dashboard_path) {
            Ok(content) => {
                println!("✓ Dashboard file is readable");
                println!("  File size: {} bytes", content.len());

                // Try to parse as JSON
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json) => {
                        println!("✓ Dashboard file is valid JSON");
                        println!("  Keys: {:?}", json.as_object().unwrap().keys().collect::<Vec<_>>());
                    }
                    Err(e) => {
                        println!("✗ Dashboard file is not valid JSON: {}", e);
                        return;
                    }
                }
            }
            Err(e) => {
                println!("✗ Failed to read dashboard file: {}", e);
                return;
            }
        }
    } else {
        println!("✗ Dashboard file does not exist: {}", dashboard_path);
        return;
    }

    // Check if the schema file exists
    let schema_path = "tools/cyclic_granfina_dashboard/src/dev/schemas/cyclic_granfina_dashboard_v1.schema.json";
    if Path::new(schema_path).exists() {
        println!("✓ Schema file exists: {}", schema_path);
    } else {
        println!("✗ Schema file does not exist: {}", schema_path);
        return;
    }

    // Check if the documentation file exists
    let doc_path = "tools/cyclic_granfina_dashboard/src/dev/cyclic_granfina_dashboard_v1.md";
    if Path::new(doc_path).exists() {
        println!("✓ Documentation file exists: {}", doc_path);
    } else {
        println!("✗ Documentation file does not exist: {}", doc_path);
        return;
    }

    // Check if the proof file exists
    let proof_path = "tools/cyclic_granfina_dashboard/src/dev/cyclic_granfina_dashboard_live_proof.rs";
    if Path::new(proof_path).exists() {
        println!("✓ Proof file exists: {}", proof_path);
    } else {
        println!("✗ Proof file does not exist: {}", proof_path);
        return;
    }

    // Check if the integration test file exists
    let test_path = "tools/cyclic_granfina_dashboard/src/dev/cyclic_granfina_dashboard_integration_test.rs";
    if Path::new(test_path).exists() {
        println!("✓ Integration test file exists: {}", test_path);
    } else {
        println!("✗ Integration test file does not exist: {}", test_path);
        return;
    }

    println!("\n✓ All checks passed! Cyclic Granfina Dashboard is properly set up.");
    println!("\nTo run tests:")
    println!("  cd tools/cyclic_granfina_dashboard && cargo test")
}
