//! Development planning: witness digest + triage/markdown boards → recommended implementation slices.
//!
//! Writes `tools/orchestrator/reports/plan_slice.md` and `tools/orchestrator/queues/continuation_queue.json`.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const KNOWN_WITNESSES: &[(&str, &str)] = &[
    ("debug_runs/stage5_full_app_live.json", "stage5"),
    ("debug_runs/infrastructure_view_isolation_live.json", "view_isolation"),
    ("debug_runs/construction_stage_live.json", "construction"),
    ("debug_runs/industrial_activation_live.json", "industrial"),
    ("debug_runs/logistics_throughput_live.json", "logistics"),
    ("debug_runs/fire_ecology_live.json", "fire_ecology"),
    ("debug_runs/replay_editor_parity_live.json", "replay"),
    ("debug_runs/main_thread_orchestrator_live.json", "orchestrator_shift"),
];

const TRIAGE_DOC: &str = "src/dev/stage5_triage_backlog.md";
const STAGE55_DOC: &str = "docs/archive/2026-06-src-dev/plans/stage5_5_open.md";
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuationTask {
    pub id: String,
    pub priority: u32,
    pub title: String,
    pub lane: String,
    pub agent: String,
    pub track: String,
    pub witness: String,
    pub commands: Vec<String>,
    pub playbook: String,
    pub docs: Vec<String>,
    pub source: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct WitnessDigest {
    path: String,
    present: bool,
    age_hours: Option<f64>,
    green: Option<bool>,
    note: String,
}

#[derive(Debug, Clone)]
pub struct TriageRow {
    id: String,
    work: String,
    stage: String,
    worker: String,
    source: String,
}

#[derive(Debug, Clone)]
pub struct MarkdownTodo {
    id: String,
    text: String,
    doc: String,
}

#[derive(Debug)]
pub struct PlanSliceReport {
    pub generated_at_epoch: u64,
    pub witnesses: Vec<WitnessDigest>,
    pub triage_open: Vec<TriageRow>,
    pub markdown_open: Vec<MarkdownTodo>,
    pub recommended: Vec<ContinuationTask>,
    pub health_notes: Vec<String>,
}

pub fn run_plan_slice(repo_root: &Path, top_n: usize, write_queue: bool) -> PlanSliceReport {
    let witnesses = digest_witnesses(repo_root);
    let triage_open = parse_triage_backlog(&repo_root.join(TRIAGE_DOC));
    let markdown_open = parse_open_markdown_todos(repo_root);
    let health_notes = reconcile_health(&witnesses, &triage_open);
    let recommended = recommend_slices(&witnesses, &triage_open, &markdown_open, top_n);
    if write_queue {
        let queue_path = repo_root.join("tools/orchestrator/queues/continuation_queue.json");
        if let Ok(json) = serde_json::to_string_pretty(&recommended) {
            let _ = fs::write(&queue_path, json);
        }
    }
    let report = PlanSliceReport {
        generated_at_epoch: now_epoch(),
        witnesses,
        triage_open,
        markdown_open,
        recommended: recommended.clone(),
        health_notes,
    };
    write_plan_markdown(repo_root, &report);
    report
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn digest_witnesses(repo_root: &Path) -> Vec<WitnessDigest> {
    KNOWN_WITNESSES
        .iter()
        .map(|(rel, kind)| digest_one(repo_root, rel, kind))
        .collect()
}

fn digest_one(repo_root: &Path, rel: &str, kind: &str) -> WitnessDigest {
    let path = repo_root.join(rel);
    if !path.is_file() {
        return WitnessDigest {
            path: rel.to_string(),
            present: false,
            age_hours: None,
            green: None,
            note: "missing — run proof command for this lane".into(),
        };
    }
    let text = fs::read_to_string(&path).unwrap_or_default();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
    let age_hours = value
        .pointer("/_agent_meta/written_at_epoch_secs")
        .and_then(|v| v.as_u64())
        .map(|ts| (now_epoch().saturating_sub(ts)) as f64 / 3600.0);
    let (green, note) = witness_green_note(kind, &value);
    WitnessDigest {
        path: rel.to_string(),
        present: true,
        age_hours,
        green,
        note,
    }
}

fn witness_green_note(kind: &str, v: &serde_json::Value) -> (Option<bool>, String) {
    match kind {
        "stage5" => {
            let passes = v
                .pointer("/readiness/passes")
                .or_else(|| v.pointer("/passes"))
                .and_then(|x| x.as_bool());
            let viol = v
                .pointer("/readiness/violations/len")
                .or_else(|| v.pointer("/violations_len"))
                .and_then(|x| x.as_u64())
                .unwrap_or(0);
            (
                passes,
                format!(
                    "FULL_APP passes={:?} violations={viol}",
                    passes.unwrap_or(false)
                ),
            )
        }
        "construction" => (
            v.pointer("/construction_operational_green")
                .and_then(|x| x.as_bool()),
            "construction boards".into(),
        ),
        "industrial" => (
            v.pointer("/activation_green").and_then(|x| x.as_bool()),
            "industrial activation".into(),
        ),
        "logistics" => (
            v.pointer("/throughput_green").and_then(|x| x.as_bool()),
            format!(
                "open_todos={}",
                v.get("open_todos").and_then(|x| x.as_u64()).unwrap_or(0)
            ),
        ),
        "fire_ecology" => (
            v.pointer("/f1_green").and_then(|x| x.as_bool()),
            v.pointer("/witness/mean_heat")
                .and_then(|x| x.as_f64())
                .map(|h| format!("mean_heat={h:.3}"))
                .unwrap_or_else(|| "fire F1 witness".into()),
        ),
        "view_isolation" => (
            v.pointer("/isolation_green").and_then(|x| x.as_bool()),
            "per-view isolation".into(),
        ),
        _ => (None, kind.to_string()),
    }
}

fn parse_triage_backlog(path: &Path) -> Vec<TriageRow> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with("| TRIAGE-") {
            continue;
        }
        if line.contains("**Done**") || line.contains(" fixed ") && line.contains("verify") {
            // Keep verify rows — still open until witness proves
        }
        let parts: Vec<&str> = line.split('|').map(str::trim).collect();
        if parts.len() < 6 {
            continue;
        }
        let id = parts[1].to_string();
        if id == "ID" {
            continue;
        }
        let work = parts[2].to_string();
        if work.to_ascii_lowercase().contains("done") && !work.contains("verify") {
            continue;
        }
        rows.push(TriageRow {
            id,
            work,
            stage: parts[3].to_string(),
            worker: parts[4].to_string(),
            source: parts[5].to_string(),
        });
    }
    rows
}

fn parse_open_markdown_todos(repo_root: &Path) -> Vec<MarkdownTodo> {
    let mut out = Vec::new();
    for rel in [
        "src/dev/fire_ecology_f1_todos.md",
        "src/dev/visual_run_blockers.md",
        "src/dev/COMPILE_WARNINGS_TODOS.md",
    ] {
        let path = repo_root.join(rel);
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("- [ ] **") {
                if let Some((id, text)) = rest.split_once("**") {
                    out.push(MarkdownTodo {
                        id: id.trim().to_string(),
                        text: text.trim().trim_start_matches('—').trim().to_string(),
                        doc: rel.to_string(),
                    });
                }
            } else if let Some(text) = trimmed.strip_prefix("- [ ] ") {
                out.push(MarkdownTodo {
                    id: String::new(),
                    text: text.to_string(),
                    doc: rel.to_string(),
                });
            }
        }
    }
    out
}

fn reconcile_health(witnesses: &[WitnessDigest], triage: &[TriageRow]) -> Vec<String> {
    let mut notes = Vec::new();
    if let Some(s5) = witnesses.iter().find(|w| w.path.contains("stage5_full_app")) {
        if !s5.present {
            notes.push("Stage 5 proof missing — run: cargo run -p proc_A_dine01 -- --test visual".into());
        } else if s5.green == Some(false) {
            notes.push("Stage 5 FULL_APP not green — fix readiness violations before infra tracks".into());
        } else if let Some(h) = s5.age_hours {
            if h > 72.0 {
                notes.push(format!(
                    "stage5_full_app_live.json is {:.0}h old — refresh with --test visual",
                    h
                ));
            }
        }
    }
    if let Some(log) = witnesses.iter().find(|w| w.path.contains("logistics")) {
        if log.present && log.green == Some(false) {
            notes.push("Logistics witness not green — see logistics_throughput_live.json open_todos".into());
        }
    }
    if triage.iter().any(|r| r.id.starts_with("TRIAGE-VM-06")) {
        notes.push("Default infra track: 5.5-A — start TRIAGE-VM-06 (view runtime sole writer)".into());
    }
    notes
}

fn recommend_slices(
    witnesses: &[WitnessDigest],
    triage: &[TriageRow],
    markdown: &[MarkdownTodo],
    top_n: usize,
) -> Vec<ContinuationTask> {
    let mut tasks = Vec::new();
    let stage5_green = witnesses
        .iter()
        .find(|w| w.path.contains("stage5_full_app"))
        .and_then(|w| w.green)
        .unwrap_or(false);

    if !stage5_green {
        tasks.push(ContinuationTask {
            id: "SLICE-STAGE5-REGRESS".into(),
            priority: 1,
            title: "Restore FULL_APP green (visual test + readiness)".into(),
            lane: "Stage5".into(),
            agent: "sim-steward".into(),
            track: "operational".into(),
            witness: "debug_runs/stage5_full_app_live.json".into(),
            commands: vec![
                "cargo test -p proc_A_dine01 --lib stage5".into(),
                "cargo run -p proc_A_dine01 --release -- --test visual".into(),
            ],
            playbook: "tools/orchestrator/agents/stage5_readiness_agent.md".into(),
            docs: vec![
                "prompts/guides/stage5_convergence_directive_v1.md".into(),
                "src/dev/visual_run_blockers.md".into(),
            ],
            source: "witness:stage5".into(),
            status: "ready".into(),
        });
    }

    if let Some(row) = triage.iter().find(|r| r.id == "TRIAGE-VM-06") {
        tasks.push(slice_from_triage(row, 2, "5.5-A"));
    } else if let Some(row) = triage.first() {
        tasks.push(slice_from_triage(row, 2, "5.5-A"));
    }

    if let Some(row) = triage.iter().find(|r| r.id == "TRIAGE-FIRE-STREAM") {
        let fire_ok = witnesses
            .iter()
            .find(|w| w.path.contains("fire_ecology"))
            .and_then(|w| w.green)
            .unwrap_or(false);
        if fire_ok {
            tasks.push(slice_from_triage(row, 3, "5.5-E"));
        }
    }

    for md in markdown.iter().take(3) {
        if md.text.is_empty() {
            continue;
        }
        let lane = if md.doc.contains("fire") {
            "Fire"
        } else if md.doc.contains("COMPILE") {
            "Ops"
        } else {
            "Stage5"
        };
        tasks.push(ContinuationTask {
            id: format!(
                "SLICE-MD-{}",
                if md.id.is_empty() {
                    md.text.chars().take(12).collect::<String>()
                } else {
                    md.id.clone()
                }
            ),
            priority: 4,
            title: md.text.clone(),
            lane: lane.into(),
            agent: "coder".into(),
            track: "5.5-E".into(),
            witness: "debug_runs/fire_ecology_live.json".into(),
            commands: vec![
                "cargo test -p proc_A_dine01 fire:: --lib".into(),
                "cargo run -p proc_A_dine01 --release -- --test visual".into(),
            ],
            playbook: "tools/orchestrator/agents/render_pipeline_agent.md".into(),
            docs: vec![md.doc.clone()],
            source: format!("markdown:{}", md.doc),
            status: "ready".into(),
        });
    }

    tasks.sort_by_key(|t| t.priority);
    tasks.truncate(top_n.max(1));
    tasks
}

fn slice_from_triage(row: &TriageRow, priority: u32, track: &str) -> ContinuationTask {
    let lane = if row.id.contains("VM") || row.id.contains("PROJ") {
        "VM"
    } else if row.id.contains("FIRE") {
        "Fire"
    } else if row.id.contains("VISUAL") || row.id.contains("GPU") {
        "Stage5"
    } else {
        "Other"
    };
    let agent = if row.worker.contains("designer") {
        "designer"
    } else if row.worker.contains("sim-steward") {
        "sim-steward"
    } else if row.worker.contains("planner") {
        "planner"
    } else {
        "coder"
    };
    let playbook = if row.id.contains("VM") {
        "tools/orchestrator/agents/viewport_cleanup_agent.md"
    } else if row.id.contains("FIRE") || row.id.contains("GPU") {
        "tools/orchestrator/agents/render_pipeline_agent.md"
    } else {
        "tools/orchestrator/agents/stage5_readiness_agent.md"
    };
    ContinuationTask {
        id: format!("SLICE-{}", row.id),
        priority,
        title: row.work.clone(),
        lane: lane.into(),
        agent: agent.into(),
        track: track.into(),
        witness: "debug_runs/infrastructure_view_isolation_live.json".into(),
        commands: vec![
            "cargo test -p proc_A_dine01 --lib".into(),
            "cargo orchestrate --plan-slice".into(),
        ],
        playbook: playbook.into(),
        docs: vec![
            TRIAGE_DOC.into(),
            STAGE55_DOC.into(),
            row.source.clone(),
        ],
        source: row.id.clone(),
        status: "ready".into(),
    }
}

fn write_plan_markdown(repo_root: &Path, report: &PlanSliceReport) {
    let path = repo_root.join("tools/orchestrator/reports/plan_slice.md");
    let mut md = String::new();
    md.push_str("# Plan slice report\n\n");
    md.push_str(&format!(
        "Generated: epoch {} (orchestrator `--plan-slice`)\n\n",
        report.generated_at_epoch
    ));
    if !report.health_notes.is_empty() {
        md.push_str("## Health\n\n");
        for n in &report.health_notes {
            md.push_str(&format!("- {n}\n"));
        }
        md.push('\n');
    }
    md.push_str("## Witness digest\n\n");
    md.push_str("| Proof | Present | Green | Age (h) | Note |\n");
    md.push_str("|-------|---------|-------|---------|------|\n");
    for w in &report.witnesses {
        md.push_str(&format!(
            "| `{}` | {} | {:?} | {:?} | {} |\n",
            w.path,
            w.present,
            w.green,
            w.age_hours.map(|h| format!("{h:.1}")).unwrap_or_else(|| "—".into()),
            w.note
        ));
    }
    md.push_str("\n## Recommended slices (continuation queue)\n\n");
    for t in &report.recommended {
        md.push_str(&format!("### {} (P{})\n\n", t.id, t.priority));
        md.push_str(&format!("- **Title:** {}\n", t.title));
        md.push_str(&format!("- **Track:** {} · **Lane:** {} · **Agent:** @{}\n", t.track, t.lane, t.agent));
        md.push_str(&format!("- **Source:** {}\n", t.source));
        md.push_str(&format!("- **Witness:** `{}`\n", t.witness));
        md.push_str(&format!("- **Playbook:** `{}`\n", t.playbook));
        md.push_str("- **Commands:**\n");
        for c in &t.commands {
            md.push_str(&format!("  ```powershell\n  {c}\n  ```\n"));
        }
        md.push('\n');
    }
    md.push_str(&format!(
        "\nOpen triage rows parsed: **{}** · Open markdown todos: **{}**\n",
        report.triage_open.len(),
        report.markdown_open.len()
    ));
    let _ = fs::write(path, md);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_triage_table_row() {
        let rows = parse_triage_backlog(Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join("src/dev/stage5_triage_backlog.md")
            .as_path());
        assert!(rows.iter().any(|r| r.id == "TRIAGE-VM-06"));
    }
}
