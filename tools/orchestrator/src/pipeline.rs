use crate::architectural::analyze_architectural_state;
use crate::cargo_collect::{
    collect_compiler_output, compiler_messages_to_issues, run_cargo_check, run_cargo_clippy,
    run_cargo_test, CargoPhaseResult,
};
use crate::classify::classify_warnings;
use crate::knowledge::KnowledgeBase;
use crate::models::{OrchestratorRunMeta, OrchestratorSnapshot, ThreadHealth};
use crate::ownership::{merge_annotation_owners, resolve_ownership};
use crate::drift::write_drift_summary;
use crate::reports::generate_reports;
use crate::scanner::{repo_src_root, scan_source_tree};
use crate::state::{persist_agent_state, OrchestratorPaths};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PipelineOptions {
    pub repo_root: PathBuf,
    pub skip_clippy: bool,
    pub skip_test: bool,
    pub skip_cargo: bool,
    pub runtime_snapshot: Option<PathBuf>,
}

impl PipelineOptions {
    pub fn from_repo_root(repo_root: PathBuf) -> Self {
        Self {
            repo_root,
            skip_clippy: false,
            skip_test: false,
            skip_cargo: false,
            runtime_snapshot: None,
        }
    }
}

#[derive(serde::Deserialize)]
struct RuntimeHealthFile {
    threads: Vec<ThreadHealth>,
}

fn load_runtime_thread_health(path: &Path) -> Option<Vec<ThreadHealth>> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str::<RuntimeHealthFile>(&text)
        .ok()
        .map(|f| f.threads)
}

pub fn run_build_pipeline(opts: &PipelineOptions) -> OrchestratorSnapshot {
    let paths = OrchestratorPaths::from_repo(&opts.repo_root);
    paths.ensure_dirs().expect("create orchestrator dirs");

    let started = Utc::now().to_rfc3339();
    let run_id = Utc::now().format("%Y%m%d_%H%M%S").to_string();

    let mut phases: Vec<CargoPhaseResult> = Vec::new();
    if !opts.skip_cargo {
        phases.push(run_cargo_check(&opts.repo_root));
        if !opts.skip_clippy {
            phases.push(run_cargo_clippy(&opts.repo_root));
        }
        if !opts.skip_test {
            phases.push(run_cargo_test(&opts.repo_root));
        }
    }

    let messages = collect_compiler_output(&phases, &opts.repo_root);
    let mut issues = compiler_messages_to_issues(&messages, &opts.repo_root);

    let src_root = repo_src_root(&opts.repo_root);
    let (markers, annotations) = scan_source_tree(&src_root);

    let knowledge = KnowledgeBase::load(&paths.knowledge);

    classify_warnings(&mut issues, &annotations, &knowledge);
    merge_annotation_owners(&mut issues, &annotations);
    resolve_ownership(&mut issues, &knowledge);

    let analysis = analyze_architectural_state(&issues, &markers, &knowledge);

    let thread_health = opts
        .runtime_snapshot
        .as_ref()
        .and_then(|p| load_runtime_thread_health(p))
        .unwrap_or_else(default_thread_health_placeholders);
    let do_not_touch_count = issues.iter().filter(|i| i.do_not_touch).count();

    let finished = Utc::now().to_rfc3339();
    let snapshot = OrchestratorSnapshot {
        meta: OrchestratorRunMeta {
            run_id: run_id.clone(),
            started_at: started,
            finished_at: finished,
            repo_root: opts.repo_root.clone(),
            check_ok: phases
                .iter()
                .find(|p| p.name == "check")
                .map(|p| p.ok)
                .unwrap_or(true),
            clippy_ok: phases
                .iter()
                .find(|p| p.name == "clippy")
                .map(|p| p.ok)
                .unwrap_or(true),
            test_ok: phases
                .iter()
                .find(|p| p.name == "test")
                .map(|p| p.ok)
                .unwrap_or(true),
            issue_count: issues.len(),
            do_not_touch_count,
        },
        issues,
        continuation_tasks: analysis.continuation_tasks.clone(),
        thread_health,
        active_migrations: analysis.active_migrations.clone(),
    };

    generate_reports(
        &paths,
        &snapshot,
        &phases,
        &analysis,
        &opts.repo_root,
        &markers,
    )
    .expect("generate reports");
    write_drift_summary(&paths, &snapshot);
    persist_agent_state(&paths, &snapshot).expect("persist state");

    snapshot
}

fn default_thread_health_placeholders() -> Vec<ThreadHealth> {
    vec![
        ThreadHealth {
            name: "main_bevy".into(),
            alive: true,
            stalled_frames: 0,
            avg_frame_ms: 0.0,
            notes: "Wire from FrameTime diagnostics".into(),
        },
        ThreadHealth {
            name: "render_thread".into(),
            alive: true,
            stalled_frames: 0,
            avg_frame_ms: 0.0,
            notes: "Placeholder until runtime hook".into(),
        },
    ]
}

pub fn find_repo_root(start: &Path) -> PathBuf {
    let mut cur = start.to_path_buf();
    loop {
        if cur.join("Cargo.toml").exists() && cur.join("src").exists() {
            return cur;
        }
        if !cur.pop() {
            return start.to_path_buf();
        }
    }
}
