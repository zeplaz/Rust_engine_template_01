use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Lifecycle of a subsystem or issue — cleanup agents must respect this.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SystemLifecycle {
    #[default]
    Stable,
    Broken,
    InProgress,
    IntentionalStaging,
    LegacyTransition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WarningState {
    ActiveBug,
    InProgressMigration,
    TransitionalArchitecture,
    IntentionalStub,
    DeadCode,
    DeferredCleanup,
    OwnershipMismatch,
    VisibilityViolation,
    DeprecatedApi,
    DiagnosticNoise,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    Info,
    Warning,
    Critical,
    Fatal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticIssue {
    pub id: String,
    pub subsystem: String,
    pub file: String,
    pub line: usize,
    pub severity: Severity,
    pub state: WarningState,
    pub symbol: String,
    pub message: String,
    pub owner: Option<String>,
    pub migration_target: Option<String>,
    pub blockers: Vec<String>,
    pub related_systems: Vec<String>,
    pub recommended_action: String,
    pub do_not_touch: bool,
    pub architectural_context: Vec<String>,
    #[serde(default)]
    pub lifecycle: SystemLifecycle,
    #[serde(default)]
    pub rustc_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SubsystemTag {
    pub domain: &'static str,
    pub system: &'static str,
    pub feature: &'static str,
}

impl SubsystemTag {
    pub fn display_path(&self) -> String {
        format!("{} / {} / {}", self.domain, self.system, self.feature)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadHealth {
    pub name: String,
    pub alive: bool,
    pub stalled_frames: u64,
    pub avg_frame_ms: f32,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveMigration {
    pub id: String,
    pub status: String,
    pub do_not_touch: Vec<String>,
    pub safe: Vec<String>,
    pub blocked_by: Vec<String>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuationTask {
    pub id: String,
    pub title: String,
    pub risk: String,
    pub subsystem: String,
    pub deprecated_symbols: Vec<String>,
    pub replacement_symbols: Vec<String>,
    pub affected_systems: Vec<String>,
    pub blockers: Vec<String>,
    pub recommended_agent: String,
    pub do_not_touch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorRunMeta {
    pub run_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub repo_root: PathBuf,
    pub check_ok: bool,
    pub clippy_ok: bool,
    pub test_ok: bool,
    pub issue_count: usize,
    pub do_not_touch_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorSnapshot {
    pub meta: OrchestratorRunMeta,
    pub issues: Vec<DiagnosticIssue>,
    pub continuation_tasks: Vec<ContinuationTask>,
    pub thread_health: Vec<ThreadHealth>,
    pub active_migrations: Vec<ActiveMigration>,
}

/// Parsed cargo JSON diagnostic envelope (subset).
#[derive(Debug, Deserialize)]
pub struct CargoMessageLine {
    pub reason: Option<String>,
    pub message: Option<CompilerMessage>,
}

#[derive(Debug, Deserialize)]
pub struct CompilerMessage {
    pub code: Option<CompilerCode>,
    pub level: String,
    pub message: String,
    pub spans: Vec<CompilerSpan>,
}

#[derive(Debug, Deserialize)]
pub struct CompilerCode {
    pub code: String,
    #[allow(dead_code)]
    pub explanation: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CompilerSpan {
    pub file_name: String,
    pub line_start: usize,
    #[serde(default)]
    pub column_start: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceAnnotation {
    pub file: String,
    pub line: usize,
    pub status: String,
    pub owner: Option<String>,
    pub do_not_cleanup: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMarker {
    pub file: String,
    pub line: usize,
    pub kind: String,
    pub text: String,
}
