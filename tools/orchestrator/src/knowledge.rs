use crate::models::{ActiveMigration, ContinuationTask};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeBase {
    pub graph: HashMap<String, GraphNode>,
    pub migrations: Vec<MigrationEntry>,
    pub subsystem_owners: HashMap<String, String>,
    #[serde(default)]
    pub active_migrations: Vec<ActiveMigration>,
    #[serde(default)]
    pub seed_continuation_tasks: Vec<ContinuationTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub children: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationEntry {
    pub deprecated: Vec<String>,
    pub replacement: Vec<String>,
    pub owner: Option<String>,
    pub affected_systems: Vec<String>,
    pub blockers: Vec<String>,
    pub risk: String,
}

impl KnowledgeBase {
    pub fn load(knowledge_dir: &Path) -> Self {
        let mut merged = Self::default_seed();
        if !knowledge_dir.is_dir() {
            return merged;
        }
        let Ok(entries) = fs::read_dir(knowledge_dir) else {
            return merged;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(patch) = serde_json::from_str::<Self>(&text) else {
                continue;
            };
            merged.merge(patch);
        }
        merged
    }

    fn merge(&mut self, other: Self) {
        self.graph.extend(other.graph);
        if !other.migrations.is_empty() {
            self.migrations = other.migrations;
        }
        self.subsystem_owners.extend(other.subsystem_owners);
        if !other.active_migrations.is_empty() {
            self.active_migrations = other.active_migrations;
        }
        if !other.seed_continuation_tasks.is_empty() {
            self.seed_continuation_tasks = other.seed_continuation_tasks;
        }
    }

    pub fn default_seed() -> Self {
        let mut graph = HashMap::new();
        graph.insert(
            "viewport_authority".into(),
            GraphNode {
                id: "viewport_authority".into(),
                children: vec![
                    "semantic_viewport".into(),
                    "ui_measured_rect".into(),
                    "camera_viewport".into(),
                    "minimap_shell".into(),
                    "world_preview".into(),
                    "render_diagnostics".into(),
                    "drift_detection".into(),
                ],
                notes: "Single authoritative semantic viewport target (Stage 5)".into(),
            },
        );

        // Viewport authority migration complete (2026-05): canonical path in src/gui viewport
        // solver; witness debug_runs/viewport_authority_migration_witness.json + stage5_full_app_live.json.
        let migrations: Vec<MigrationEntry> = vec![];

        let mut subsystem_owners = HashMap::new();
        subsystem_owners.insert(
            "GUI / VIEWPORT_AUTHORITY / SEMANTIC_SOLVER".into(),
            "viewport_cleanup_agent".into(),
        );
        subsystem_owners.insert(
            "GUI / HUD / VIEWPORT_SYNC_DEBUG".into(),
            "viewport_cleanup_agent".into(),
        );

        let active_migrations: Vec<ActiveMigration> = vec![];
        let seed_continuation_tasks: Vec<ContinuationTask> = vec![];

        Self {
            graph,
            migrations,
            subsystem_owners,
            active_migrations,
            seed_continuation_tasks,
        }
    }

    pub fn is_deprecated_symbol(&self, symbol: &str) -> bool {
        self.migrations
            .iter()
            .any(|m| m.deprecated.iter().any(|d| d == symbol))
    }

    pub fn migration_for_symbol(&self, symbol: &str) -> Option<&MigrationEntry> {
        self.migrations
            .iter()
            .find(|m| m.deprecated.iter().any(|d| d == symbol))
    }

    pub fn owner_for_subsystem(&self, subsystem: &str) -> Option<String> {
        self.subsystem_owners.get(subsystem).cloned()
    }

    pub fn graph_node_for_file(&self, file: &str) -> Option<String> {
        if file.contains("viewport") || file.contains("sim_view_sync") {
            Some("viewport_authority".into())
        } else if file.contains("map_view") {
            Some("map_view_spine".into())
        } else if file.starts_with("src/render/") {
            Some("render_pipeline".into())
        } else {
            None
        }
    }

    pub fn knowledge_dir(repo_root: &Path) -> PathBuf {
        repo_root.join("tools/orchestrator/knowledge")
    }
}
