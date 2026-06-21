//! **CRASH-MONITOR-001** — Prometheus-style crash and file wire monitoring.
//!
//! This module simulates a monitoring system that:
//! 1. Watches for crashes or file changes
//! 2. Builds alerts in an alert command center
//! 3. Feeds data into the cyclic granfina dashboard graph system
//!
//! Plan: [`cyclic_granfina_dashboard_v1.md`](cyclic_granfina_dashboard_v1.md)

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::cyclic_granfina_dashboard::{BlockStatus, CyclicGranfinaDashboard, GranfinaDashboardConfig, PriorityLevel};

/// Alert from crash or file monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique alert ID
    pub id: String,
    /// Timestamp when alert was created
    pub created_at: u64,
    /// Alert type (crash, file_change, etc.)
    pub alert_type: AlertType,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Description of the alert
    pub description: String,
    /// Source of the alert (file path, process ID, etc.)
    pub source: String,
    /// Whether the alert has been processed
    pub processed: bool,
    /// Correlation ID for linking to dashboard
    pub correlation_id: Option<String>,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    /// Process crash detected
    Crash,
    /// File change detected
    FileChange,
    /// API failure
    ApiFailure,
    /// Hash lock violation
    HashLockViolation,
    /// Witness validation failure
    WitnessFailure,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Alert command center for processing alerts
pub struct AlertCommandCenter {
    /// Configuration for the dashboard
    config: GranfinaDashboardConfig,
    /// Dashboard instance
    dashboard: CyclicGranfinaDashboard,
    /// Pending alerts
    pending_alerts: Vec<Alert>,
    /// Processed alerts
    processed_alerts: Vec<Alert>,
}

impl AlertCommandCenter {
    /// Create a new alert command center
    pub fn new(config: GranfinaDashboardConfig) -> Self {
        let dashboard = CyclicGranfinaDashboard::new(config.clone());
        Self {
            config,
            dashboard,
            pending_alerts: Vec::new(),
            processed_alerts: Vec::new(),
        }
    }

    /// Add an alert to the pending queue
    pub fn add_alert(&mut self, alert: Alert) {
        self.pending_alerts.push(alert);
    }

    /// Process all pending alerts
    pub fn process_alerts(&mut self) -> Result<(), String> {
        // Collect alerts to process first to avoid borrow conflicts
        let alerts_to_process: Vec<Alert> = self.pending_alerts.drain(..).collect();

        for alert in alerts_to_process {
            // Create a blocker for this alert
            let blocker_id = format!("ALERT-{}", alert.id);
            let blocker_desc = format!(
                "[{}] {} - Source: {}",
                match alert.alert_type {
                    AlertType::Crash => "CRASH",
                    AlertType::FileChange => "FILE_CHANGE",
                    AlertType::ApiFailure => "API_FAILURE",
                    AlertType::HashLockViolation => "HASH_LOCK_VIOLATION",
                    AlertType::WitnessFailure => "WITNESS_FAILURE",
                },
                alert.description,
                alert.source
            );

            let priority = match alert.severity {
                AlertSeverity::Low => PriorityLevel::Low,
                AlertSeverity::Medium => PriorityLevel::Medium,
                AlertSeverity::High => PriorityLevel::High,
                AlertSeverity::Critical => PriorityLevel::Critical,
            };

            let dcc_components = self.extract_dcc_components(&alert.source);

            self.dashboard
                .add_blocker(blocker_id.clone(), blocker_desc, priority, dcc_components)
                .map_err(|e| e.to_string())?;

            // Update the alert with correlation ID
            let mut processed_alert = alert;
            processed_alert.correlation_id = Some(blocker_id);
            processed_alert.processed = true;

            self.processed_alerts.push(processed_alert);
        }

        Ok(())
    }

    /// Extract DCC components from source string
    fn extract_dcc_components(&self, source: &str) -> Vec<String> {
        // Simple extraction - in real implementation, this would parse the source
        // and extract DCC component names
        vec![format!("DCC-FROM-{}", source.trim().chars().take(10).collect::<String>())]
    }

    /// Get the dashboard
    pub fn dashboard(&self) -> &CyclicGranfinaDashboard {
        &self.dashboard
    }

    /// Get the dashboard mutably
    pub fn dashboard_mut(&mut self) -> &mut CyclicGranfinaDashboard {
        &mut self.dashboard
    }

    /// Get pending alerts
    pub fn pending_alerts(&self) -> &[Alert] {
        &self.pending_alerts
    }

    /// Get processed alerts
    pub fn processed_alerts(&self) -> &[Alert] {
        &self.processed_alerts
    }

    /// Save dashboard state
    pub fn save(&self) -> Result<(), String> {
        self.dashboard.save()
    }
}

/// Simulate crash detection
pub fn simulate_crash_detection() -> Alert {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Alert {
        id: format!("CRASH-{}", now),
        created_at: now,
        alert_type: AlertType::Crash,
        severity: AlertSeverity::Critical,
        description: "Process crashed - exit code non-zero".to_string(),
        source: "process_worker_001".to_string(),
        processed: false,
        correlation_id: None,
    }
}

/// Simulate file change detection
pub fn simulate_file_change_detection(path: &str) -> Alert {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Alert {
        id: format!("FILE-{}", now),
        created_at: now,
        alert_type: AlertType::FileChange,
        severity: AlertSeverity::Medium,
        description: format!("File changed: {}", path).to_string(),
        source: path.to_string(),
        processed: false,
        correlation_id: None,
    }
}

/// Simulate hash lock violation detection
pub fn simulate_hash_lock_violation(blocker_id: &str) -> Alert {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Alert {
        id: format!("HASH-{}", now),
        created_at: now,
        alert_type: AlertType::HashLockViolation,
        severity: AlertSeverity::High,
        description: format!("Hash lock violation detected for blocker: {}", blocker_id).to_string(),
        source: blocker_id.to_string(),
        processed: false,
        correlation_id: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_command_center() {
        let config = GranfinaDashboardConfig {
            api_key: "test_api_key".to_string(),
            hash_lock: "test_hash_lock".to_string(),
            dashboard_path: "debug_runs/test_alert_center.json".to_string(),
            ignore_dcc_status: true,
            process_driven: true,
        };

        let mut center = AlertCommandCenter::new(config);

        // Add some alerts
        let crash_alert = simulate_crash_detection();
        let file_alert = simulate_file_change_detection("/path/to/file.json");
        let hash_alert = simulate_hash_lock_violation("BLOCKER-001");

        center.add_alert(crash_alert);
        center.add_alert(file_alert);
        center.add_alert(hash_alert);

        // Process alerts
        assert!(center.process_alerts().is_ok());

        // Verify alerts were processed
        assert_eq!(center.pending_alerts().len(), 0);
        assert_eq!(center.processed_alerts().len(), 3);

        // Verify dashboard has blockers
        assert!(center.dashboard().get_blockers().len() >= 3);
    }

    #[test]
    fn test_crash_simulation() {
        let alert = simulate_crash_detection();
        assert_eq!(alert.alert_type, AlertType::Crash);
        assert_eq!(alert.severity, AlertSeverity::Critical);
        assert!(!alert.processed);
    }

    #[test]
    fn test_file_change_simulation() {
        let alert = simulate_file_change_detection("/test/path.json");
        assert_eq!(alert.alert_type, AlertType::FileChange);
        assert_eq!(alert.severity, AlertSeverity::Medium);
        assert!(!alert.processed);
    }
}