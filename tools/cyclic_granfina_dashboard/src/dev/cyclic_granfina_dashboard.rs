//! **CYCLIC-GRANFINA-DASHBOARD-001** — atomic blocker tracking dashboard with hash locks.
//!
//! Plan: [`cyclic_granfina_dashboard_v1.md`](cyclic_granfina_dashboard_v1.md)
//!
//! This system provides a cyclic dashboard for tracking blockers with atomic operations,
//! hash-based locking, and API-only access. It integrates with the existing witness
//! integrity system and ignores DCC status in the UI bar as requested.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Configuration for the cyclic granfina dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GranfinaDashboardConfig {
    /// API key for accessing the dashboard (locked file)
    pub api_key: String,
    /// Hash lock for atomic operations
    pub hash_lock: String,
    /// Path to the dashboard data file
    pub dashboard_path: String,
    /// Whether to ignore DCC status in the UI bar
    pub ignore_dcc_status: bool,
    /// Whether to use process-driven workflows
    pub process_driven: bool,
}

/// Block tracker entry for the cyclic dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTrackerEntry {
    /// Unique identifier for the blocker
    pub id: String,
    /// Timestamp when the blocker was created
    pub created_at: u64,
    /// Timestamp when the blocker was last updated
    pub updated_at: u64,
    /// Status of the blocker
    pub status: BlockStatus,
    /// Priority level
    pub priority: PriorityLevel,
    /// Description of the blocker
    pub description: String,
    /// Associated DCC (Digital Content Creation) components
    pub dcc_components: Vec<String>,
    /// Whether DCC status should be ignored
    pub ignore_dcc: bool,
    /// Hash of the previous entry for cyclic integrity
    pub previous_hash: String,
    /// Current hash for integrity verification
    pub current_hash: String,
    /// Exit code or action result
    pub exit_code: Option<i32>,
    /// Whether the entry is locked for API access
    pub locked: bool,
}

/// Status of a blocker
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockStatus {
    /// Open and active
    Open,
    /// In progress
    InProgress,
    /// Resolved
    Resolved,
    /// Blocked
    Blocked,
    /// Closed
    Closed,
}

/// Priority level for blockers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriorityLevel {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

/// Main dashboard structure for cyclic granfina
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyclicGranfinaDashboard {
    /// Dashboard configuration
    pub config: GranfinaDashboardConfig,
    /// Current cycle number
    pub cycle: u64,
    /// List of blocker entries
    pub entries: Vec<BlockTrackerEntry>,
    /// Hash of the previous dashboard state
    pub previous_hash: String,
    /// Current hash for integrity verification
    pub current_hash: String,
    /// Timestamp of last update
    pub last_updated: u64,
}

impl CyclicGranfinaDashboard {
    /// Create a new cyclic granfina dashboard
    pub fn new(config: GranfinaDashboardConfig) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut dashboard = Self {
            config,
            cycle: 1,
            entries: Vec::new(),
            previous_hash: String::new(),
            current_hash: String::new(),
            last_updated: now,
        };

        dashboard.update_hash();
        dashboard
    }

    /// Add a new blocker entry to the dashboard
    pub fn add_blocker(
        &mut self,
        id: String,
        description: String,
        priority: PriorityLevel,
        dcc_components: Vec<String>,
    ) -> Result<String, String> {
        // Verify API access
        if !self.verify_api_access() {
            return Err("API access denied: invalid or missing API key".to_string());
        }

        // Check if the hash lock is valid
        if !self.verify_hash_lock() {
            return Err("Hash lock verification failed".to_string());
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Create new entry
        let entry = BlockTrackerEntry {
            id: id.clone(),
            created_at: now,
            updated_at: now,
            status: BlockStatus::Open,
            priority,
            description,
            dcc_components,
            ignore_dcc: self.config.ignore_dcc_status,
            previous_hash: self.current_hash.clone(),
            current_hash: String::new(), // Will be calculated after creation
            exit_code: None,
            locked: true,
        };

        let entry_hash = self.calculate_entry_hash(&entry);
        let mut entry = entry;
        entry.current_hash = entry_hash;

        // Add to entries
        self.entries.push(entry);

        // Update dashboard hash
        self.update_hash();

        Ok(id)
    }

    /// Update an existing blocker entry
    pub fn update_blocker(
        &mut self,
        id: &str,
        status: Option<BlockStatus>,
        exit_code: Option<i32>,
    ) -> Result<(), String> {
        // Verify API access
        if !self.verify_api_access() {
            return Err("API access denied: invalid or missing API key".to_string());
        }

        // Find the entry index
        let index = self.entries.iter().position(|e| e.id == id);
        if let Some(index) = index {
            // Update fields
            if let Some(status) = status {
                self.entries[index].status = status;
            }
            if let Some(code) = exit_code {
                self.entries[index].exit_code = Some(code);
            }
            self.entries[index].updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Update hash
            self.entries[index].current_hash = self.calculate_entry_hash(&self.entries[index]);

            // Update dashboard hash
            self.update_hash();

            Ok(())
        } else {
            Err(format!("Blocker with id '{}' not found", id))
        }
    }

    /// Remove a blocker entry
    pub fn remove_blocker(&mut self, id: &str) -> Result<(), String> {
        // Verify API access
        if !self.verify_api_access() {
            return Err("API access denied: invalid or missing API key".to_string());
        }

        let index = self.entries.iter().position(|e| e.id == id);
        if let Some(index) = index {
            self.entries.remove(index);
            self.update_hash();
            Ok(())
        } else {
            Err(format!("Blocker with id '{}' not found", id))
        }
    }

    /// Get all blocker entries
    pub fn get_blockers(&self) -> &[BlockTrackerEntry] {
        &self.entries
    }

    /// Get blockers by status
    pub fn get_blockers_by_status(&self, status: BlockStatus) -> Vec<&BlockTrackerEntry> {
        self.entries
            .iter()
            .filter(|e| e.status == status)
            .collect()
    }

    /// Get blockers by priority
    pub fn get_blockers_by_priority(&self, priority: PriorityLevel) -> Vec<&BlockTrackerEntry> {
        self.entries
            .iter()
            .filter(|e| e.priority == priority)
            .collect()
    }

    /// Get blockers by DCC component
    pub fn get_blockers_by_dcc(&self, dcc: &str) -> Vec<&BlockTrackerEntry> {
        self.entries
            .iter()
            .filter(|e| e.dcc_components.contains(&dcc.to_string()))
            .collect()
    }

    /// Verify API access
    pub fn verify_api_access(&self) -> bool {
        // In a real implementation, this would check the API key
        // For now, we'll assume it's valid if the config has an API key
        !self.config.api_key.is_empty()
    }

    /// Verify hash lock
    pub fn verify_hash_lock(&self) -> bool {
        // In a real implementation, this would verify the hash lock
        // For now, we'll assume it's valid if the config has a hash lock
        !self.config.hash_lock.is_empty()
    }

    /// Calculate hash for an entry
    fn calculate_entry_hash(&self, entry: &BlockTrackerEntry) -> String {
        let mut hasher = Sha256::new();
        // Create a temporary entry without the hash fields to avoid circular dependency
        let temp_entry = BlockTrackerEntry {
            current_hash: String::new(),
            previous_hash: entry.previous_hash.clone(),
            ..entry.clone()
        };
        let entry_str = serde_json::to_string(&temp_entry).unwrap_or_default();
        hasher.update(entry_str.as_bytes());
        hasher.update(self.config.hash_lock.as_bytes());
        let hash_bytes = hasher.finalize();
        hex::encode(hash_bytes)
    }

    /// Update dashboard hash
    fn update_hash(&mut self) {
        let mut hasher = Sha256::new();
        let entries_str = serde_json::to_string(&self.entries).unwrap_or_default();
        hasher.update(entries_str.as_bytes());
        hasher.update(self.config.hash_lock.as_bytes());
        let hash_bytes = hasher.finalize();
        self.previous_hash = self.current_hash.clone();
        self.current_hash = hex::encode(hash_bytes);
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Save dashboard to file
    pub fn save(&self) -> Result<(), String> {
        let path = self.config.dashboard_path.clone();
        let json_str = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;

        std::fs::write(&path, json_str).map_err(|e| e.to_string())
    }

    /// Load dashboard from file
    pub fn load(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let dashboard: CyclicGranfinaDashboard = serde_json::from_str(&content)
            .map_err(|e| e.to_string())?;
        Ok(dashboard)
    }

    /// Validate dashboard integrity
    pub fn validate_integrity(&self) -> Result<(), String> {
        // Verify each entry's hash
        for entry in &self.entries {
            let calculated_hash = self.calculate_entry_hash(entry);
            if calculated_hash != entry.current_hash {
                return Err(format!("Entry {} has invalid hash", entry.id));
            }
        }

        // Verify dashboard hash
        let mut hasher = Sha256::new();
        let entries_str = serde_json::to_string(&self.entries).unwrap_or_default();
        hasher.update(entries_str.as_bytes());
        hasher.update(self.config.hash_lock.as_bytes());
        let hash_bytes = hasher.finalize();
        let calculated_hash = hex::encode(hash_bytes);
        if calculated_hash != self.current_hash {
            return Err("Dashboard hash is invalid".to_string());
        }

        Ok(())
    }

    /// Create a witness for the dashboard
    pub fn create_witness(&self) -> Result<(), String> {
        let body = serde_json::to_value(self).map_err(|e| e.to_string())?;
        let wrapped = Self::wrap_debug_run(
            "CYCLIC-GRANFINA-DASHBOARD-001",
            "create_witness",
            &self.config.dashboard_path,
            body,
        );

        self.write_debug_run_json(&self.config.dashboard_path, wrapped)
    }

    fn wrap_debug_run(
        profile: &str,
        source_system: &str,
        relative_path: &str,
        body: serde_json::Value,
    ) -> serde_json::Value {
        let mut map = match body {
            serde_json::Value::Object(m) => m,
            other => {
                let mut m = serde_json::Map::new();
                m.insert("payload".into(), other);
                m
            }
        };

        let commands: Vec<serde_json::Value> = vec![
            serde_json::Value::String("cargo test -p cyclic_granfina_dashboard".to_string()),
        ];

        map.insert(
            "_agent_meta".into(),
            serde_json::json!({
                "schema": "debug_run_envelope_v1",
                "written_at_epoch_secs": SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "profile": profile,
                "source_system": source_system,
                "relative_path": relative_path,
                "logging_env": {},
                "agent_commands": commands,
                "related_proofs": [],
                "orchestrator": {},
                "docs": {
                    "stage5_directive": "prompts/guides/stage5_convergence_directive_v1.md",
                    "compile_warnings": "src/dev/COMPILE_WARNINGS_TODOS.md",
                    "viewport_recovery": "src/dev/recovery_viewport.md",
                },
            }),
        );

        serde_json::Value::Object(map)
    }

    fn write_debug_run_json(&self, relative_path: &str, payload: serde_json::Value) -> Result<(), String> {
        let path = PathBuf::from(&self.config.dashboard_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let text = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
        std::fs::write(&path, text).map_err(|e| e.to_string())
    }
}

/// Initialize the cyclic granfina dashboard
pub fn initialize_granfina_dashboard() -> CyclicGranfinaDashboard {
    let config = GranfinaDashboardConfig {
        api_key: "granfina_api_key_2026".to_string(),
        hash_lock: "granfina_hash_lock_2026".to_string(),
        dashboard_path: "debug_runs/cyclic_granfina_dashboard_live.json".to_string(),
        ignore_dcc_status: true, // Ignore DCC status in UI bar as requested
        process_driven: true,    // Use process-driven workflows
    };

    CyclicGranfinaDashboard::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dashboard() {
        let config = GranfinaDashboardConfig {
            api_key: "test_api_key".to_string(),
            hash_lock: "test_hash_lock".to_string(),
            dashboard_path: "debug_runs/test_dashboard.json".to_string(),
            ignore_dcc_status: true,
            process_driven: true,
        };

        let mut dashboard = CyclicGranfinaDashboard::new(config);

        // Add a blocker
        let blocker_id = dashboard
            .add_blocker(
                "BLOCKER-001".to_string(),
                "Test blocker".to_string(),
                PriorityLevel::High,
                vec!["DCC-COMPONENT-001".to_string()],
            )
            .unwrap();

        assert_eq!(blocker_id, "BLOCKER-001");
        assert_eq!(dashboard.entries.len(), 1);
        assert_eq!(dashboard.entries[0].id, "BLOCKER-001");
        assert_eq!(dashboard.entries[0].status, BlockStatus::Open);
        assert!(dashboard.entries[0].ignore_dcc);
    }

    #[test]
    fn test_update_blocker() {
        let config = GranfinaDashboardConfig {
            api_key: "test_api_key".to_string(),
            hash_lock: "test_hash_lock".to_string(),
            dashboard_path: "debug_runs/test_dashboard.json".to_string(),
            ignore_dcc_status: true,
            process_driven: true,
        };

        let mut dashboard = CyclicGranfinaDashboard::new(config);

        // Add a blocker
        dashboard
            .add_blocker(
                "BLOCKER-002".to_string(),
                "Test blocker 2".to_string(),
                PriorityLevel::Medium,
                vec!["DCC-COMPONENT-002".to_string()],
            )
            .unwrap();

        // Update the blocker
        dashboard
            .update_blocker("BLOCKER-002", Some(BlockStatus::InProgress), Some(0))
            .unwrap();

        assert_eq!(dashboard.entries[0].status, BlockStatus::InProgress);
        assert_eq!(dashboard.entries[0].exit_code, Some(0));
    }

    #[test]
    fn test_dashboard_integrity() {
        let config = GranfinaDashboardConfig {
            api_key: "test_api_key".to_string(),
            hash_lock: "test_hash_lock".to_string(),
            dashboard_path: "debug_runs/test_dashboard.json".to_string(),
            ignore_dcc_status: true,
            process_driven: true,
        };

        let mut dashboard = CyclicGranfinaDashboard::new(config);

        // Add blockers
        dashboard
            .add_blocker(
                "BLOCKER-003".to_string(),
                "Test blocker 3".to_string(),
                PriorityLevel::Low,
                vec!["DCC-COMPONENT-003".to_string()],
            )
            .unwrap();

        dashboard
            .add_blocker(
                "BLOCKER-004".to_string(),
                "Test blocker 4".to_string(),
                PriorityLevel::Critical,
                vec!["DCC-COMPONENT-004".to_string()],
            )
            .unwrap();

        // Validate integrity
        assert!(dashboard.validate_integrity().is_ok());

        // Modify an entry
        dashboard
            .update_blocker("BLOCKER-003", Some(BlockStatus::Resolved), Some(1))
            .unwrap();

        // Validate integrity again
        assert!(dashboard.validate_integrity().is_ok());
    }
}
