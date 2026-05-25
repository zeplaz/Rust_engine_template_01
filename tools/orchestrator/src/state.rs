use crate::models::OrchestratorSnapshot;
use std::fs;
use std::path::{Path, PathBuf};

pub struct OrchestratorPaths {
    pub root: PathBuf,
    pub reports: PathBuf,
    pub runbooks: PathBuf,
    pub history: PathBuf,
    pub agents: PathBuf,
    pub state: PathBuf,
    pub knowledge: PathBuf,
    pub queues: PathBuf,
}

impl OrchestratorPaths {
    pub fn from_repo(repo_root: &Path) -> Self {
        let root = repo_root.join("tools/orchestrator");
        Self {
            reports: root.join("reports"),
            runbooks: root.join("runbooks"),
            history: root.join("history"),
            agents: root.join("agents"),
            state: root.join("state"),
            knowledge: root.join("knowledge"),
            queues: root.join("queues"),
            root,
        }
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        for dir in [
            &self.reports,
            &self.runbooks,
            &self.history,
            &self.agents,
            &self.state,
            &self.knowledge,
            &self.queues,
        ] {
            fs::create_dir_all(dir)?;
        }
        Ok(())
    }
}

pub fn persist_agent_state(paths: &OrchestratorPaths, snapshot: &OrchestratorSnapshot) -> std::io::Result<()> {
    let latest = paths.state.join("last_run.json");
    let history_file = paths
        .history
        .join(format!("run_{}.json", snapshot.meta.run_id));
    let json = serde_json::to_string_pretty(snapshot)?;
    fs::write(&latest, &json)?;
    fs::write(history_file, json)?;
    Ok(())
}

pub fn load_last_snapshot(paths: &OrchestratorPaths) -> Option<OrchestratorSnapshot> {
    let path = paths.state.join("last_run.json");
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}
