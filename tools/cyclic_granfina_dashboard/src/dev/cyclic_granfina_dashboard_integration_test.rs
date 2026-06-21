//! **CYCLIC-GRANFINA-INTEGRATION-001** — integration test for cyclic granfina dashboard with existing systems.
//!
//! This test verifies that the cyclic granfina dashboard integrates correctly with:
//! 1. Hash lock verification
//! 2. API access verification
//! 3. Witness functionality
//! 4. File I/O operations
//!
//! Plan: [`cyclic_granfina_dashboard_v1.md`](cyclic_granfina_dashboard_v1.md)

use super::cyclic_granfina_dashboard::{
    BlockStatus, CyclicGranfinaDashboard, GranfinaDashboardConfig, PriorityLevel,
};

#[test]
fn test_cyclic_granfina_dashboard_integration() {
    // Initialize the dashboard
    let config = GranfinaDashboardConfig {
        api_key: "granfina_api_key_2026".to_string(),
        hash_lock: "granfina_hash_lock_2026".to_string(),
        dashboard_path: "debug_runs/cyclic_granfina_dashboard_integration_test.json".to_string(),
        ignore_dcc_status: true,
        process_driven: true,
    };

    let mut dashboard = CyclicGranfinaDashboard::new(config);

    // Add blockers with different priorities and DCC components
    dashboard
        .add_blocker(
            "INTEGRATION-BLOCKER-001".to_string(),
            "DCC component integration issue".to_string(),
            PriorityLevel::High,
            vec!["DCC-COMPONENT-001".to_string(), "DCC-COMPONENT-002".to_string()],
        )
        .unwrap();

    dashboard
        .add_blocker(
            "INTEGRATION-BLOCKER-002".to_string(),
            "Process-driven workflow issue".to_string(),
            PriorityLevel::Medium,
            vec!["DCC-COMPONENT-003".to_string()],
        )
        .unwrap();

    dashboard
        .add_blocker(
            "INTEGRATION-BLOCKER-003".to_string(),
            "Hash lock verification issue".to_string(),
            PriorityLevel::Critical,
            vec!["DCC-COMPONENT-004".to_string()],
        )
        .unwrap();

    // Verify dashboard structure
    assert_eq!(dashboard.entries.len(), 3);
    assert_eq!(dashboard.cycle, 1);
    assert!(dashboard.config.ignore_dcc_status);
    assert!(dashboard.config.process_driven);

    // Verify API access
    assert!(dashboard.verify_api_access());
    assert!(dashboard.verify_hash_lock());

    // Verify DCC status is ignored
    for entry in &dashboard.entries {
        assert!(entry.ignore_dcc);
    }

    // Verify process-driven workflows
    for entry in &dashboard.entries {
        assert!(entry.status == BlockStatus::Open);
        assert!(!entry.id.is_empty());
        assert!(!entry.description.is_empty());
        assert!(!entry.dcc_components.is_empty());
    }

    // Update blockers
    dashboard
        .update_blocker(
            "INTEGRATION-BLOCKER-001",
            Some(BlockStatus::InProgress),
            Some(0),
        )
        .unwrap();

    dashboard
        .update_blocker(
            "INTEGRATION-BLOCKER-002",
            Some(BlockStatus::Resolved),
            Some(1),
        )
        .unwrap();

    // Verify updates
    let blocker1 = dashboard
        .entries
        .iter()
        .find(|e| e.id == "INTEGRATION-BLOCKER-001")
        .unwrap();
    assert_eq!(blocker1.status, BlockStatus::InProgress);
    assert_eq!(blocker1.exit_code, Some(0));

    let blocker2 = dashboard
        .entries
        .iter()
        .find(|e| e.id == "INTEGRATION-BLOCKER-002")
        .unwrap();
    assert_eq!(blocker2.status, BlockStatus::Resolved);
    assert_eq!(blocker2.exit_code, Some(1));

    // Verify integrity
    assert!(dashboard.validate_integrity().is_ok());

    // Save dashboard
    assert!(dashboard.save().is_ok());

    // Load dashboard
    let loaded_dashboard = CyclicGranfinaDashboard::load(
        "debug_runs/cyclic_granfina_dashboard_integration_test.json",
    )
    .unwrap();

    // Verify loaded dashboard
    assert_eq!(loaded_dashboard.entries.len(), 3);
    assert_eq!(loaded_dashboard.cycle, 1);
    assert!(loaded_dashboard.config.ignore_dcc_status);
    assert!(loaded_dashboard.config.process_driven);

    // Verify loaded entries
    let loaded_blocker1 = loaded_dashboard
        .entries
        .iter()
        .find(|e| e.id == "INTEGRATION-BLOCKER-001")
        .unwrap();
    assert_eq!(loaded_blocker1.status, BlockStatus::InProgress);
    assert_eq!(loaded_blocker1.exit_code, Some(0));

    // Clean up
    std::fs::remove_file("debug_runs/cyclic_granfina_dashboard_integration_test.json").unwrap();
}

#[test]
fn test_cyclic_granfina_dashboard_witness_integration() {
    // Initialize the dashboard
    let config = GranfinaDashboardConfig {
        api_key: "granfina_api_key_2026".to_string(),
        hash_lock: "granfina_hash_lock_2026".to_string(),
        dashboard_path: "debug_runs/cyclic_granfina_dashboard_witness_test.json".to_string(),
        ignore_dcc_status: true,
        process_driven: true,
    };

    let mut dashboard = CyclicGranfinaDashboard::new(config);

    // Add blockers
    dashboard
        .add_blocker(
            "WITNESS-BLOCKER-001".to_string(),
            "Witness integration issue".to_string(),
            PriorityLevel::High,
            vec!["DCC-COMPONENT-001".to_string()],
        )
        .unwrap();

    // Create witness
    dashboard.create_witness().unwrap();

    // Verify witness file exists
    assert!(std::path::Path::new(&dashboard.config.dashboard_path).exists());

    // Clean up
    std::fs::remove_file(&dashboard.config.dashboard_path).unwrap();
}
