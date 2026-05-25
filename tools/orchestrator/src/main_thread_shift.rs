//! Main-thread orchestrator shift (Shift A→B) — executable debug + cleanup + sim-grade checks.
//!
//! Writes `debug_runs/main_thread_orchestrator_live.json` for `@main-thread-orchestrator` agents.

use crate::authority_scan::{authority_alerts, scan_authority_writes};
use crate::models::SemanticMarker;
use crate::scanner::{repo_src_root, scan_source_tree};
use chrono::Utc;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub const LIVE_PROOF_PATH: &str = "debug_runs/main_thread_orchestrator_live.json";

const WITNESS_PATHS: &[&str] = &[
    "debug_runs/stage5_full_app_live.json",
    "debug_runs/viewport_drift.json",
    "debug_runs/viewport_authority_migration_witness.json",
    "debug_runs/infrastructure_view_isolation_live.json",
    "debug_runs/orchestrator_thread_health.json",
];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WitnessDigest {
    pub path: String,
    pub exists: bool,
    pub summary: Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CleanupClassification {
    pub file: String,
    pub line: usize,
    pub marker_kind: String,
    pub classification: String,
    pub decision: String,
    pub note: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DebugRouting {
    pub id: String,
    pub severity: String,
    pub root_cause: Vec<String>,
    pub affected: Vec<String>,
    pub evidence: Vec<String>,
    pub recommendation: Vec<String>,
    pub owner: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MainThreadShiftReport {
    pub run_id: String,
    pub shift_a: Value,
    pub shift_b_debug: DebugRouting,
    pub shift_b_cleanup: Vec<CleanupClassification>,
    pub simulation_grade: Value,
    pub fail_cycle_ledger: Value,
    pub ok: bool,
    pub highest_severity: String,
}

pub fn run_main_thread_shift(repo_root: &Path) -> MainThreadShiftReport {
    let run_id = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let witnesses = digest_witnesses(repo_root);
    let src_root = repo_src_root(repo_root);
    let (markers, annotations) = scan_source_tree(&src_root);
    let authority_sites = scan_authority_writes(&src_root);
    let alerts = authority_alerts(&authority_sites);

    let cleanup = classify_cleanup_markers(&markers, &annotations);
    let debug = build_debug_routing(&witnesses, &alerts, &cleanup);
    let sim_grade = json!({
        "principle": "single authority per resource",
        "authority_write_sites": authority_sites,
        "authority_alerts": alerts,
        "resolved_viewports_canonical": ["src/render/viewport_pipeline.rs"],
        "view_manager_canonical": ["src/gui/view_authority.rs"],
    });

    let shift_a = json!({
        "lane": "MAIN_THREAD_SHIFT",
        "witnesses": witnesses,
        "marker_count": markers.len(),
        "orchestrator_annotation_count": annotations.len(),
        "authorities_observed": authority_sites
            .iter()
            .map(|s| &s.resource)
            .collect::<std::collections::BTreeSet<_>>(),
    });

    let highest = if debug.severity == "HIGH" {
        "HIGH"
    } else if alerts.iter().any(|a| a.severity == "MED") {
        "MED"
    } else {
        "LOW"
    };

    let cleanup_blocks = cleanup.iter().any(|c| c.decision == "remove" && c.classification == "A_obsolete");
    let ok = debug.severity != "HIGH" && !cleanup_blocks;

    let fail_cycle = json!({
        "slice_id": "MAIN_THREAD_SHIFT",
        "attempts": [
            {
                "cycle": 2,
                "channel": "tools/orchestrator/main_thread_shift",
                "outcome": if ok { "yaml_emitted" } else { "action_required" },
                "note": "executable Shift A→B (no Task)"
            }
        ],
        "next_cycle": if ok { Value::Null } else { json!(3) },
    });

    MainThreadShiftReport {
        run_id: run_id.clone(),
        shift_a,
        shift_b_debug: debug,
        shift_b_cleanup: cleanup,
        simulation_grade: sim_grade,
        fail_cycle_ledger: fail_cycle,
        ok,
        highest_severity: highest.to_string(),
    }
}

pub fn write_live_proof(repo_root: &Path, report: &MainThreadShiftReport) -> PathBuf {
    let out_path = repo_root.join(LIVE_PROOF_PATH);
    if let Some(parent) = out_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let body = json!({
        "profile": "MAIN_THREAD_SHIFT",
        "run_id": report.run_id,
        "ok": report.ok,
        "highest_severity": report.highest_severity,
        "shift": {
            "A_observe": report.shift_a,
            "B_debug": report.shift_b_debug,
            "B_cleanup": report.shift_b_cleanup,
        },
        "debug_routing": report.shift_b_debug,
        "cleanup_classifications": report.shift_b_cleanup,
        "simulation_grade": report.simulation_grade,
        "fail_cycle_ledger": report.fail_cycle_ledger,
    });

    let wrapped = wrap_agent_meta(body);
    let text = serde_json::to_string_pretty(&wrapped).expect("serialize main thread proof");
    fs::write(&out_path, text).expect("write main thread proof");
    out_path
}

fn wrap_agent_meta(mut body: Value) -> Value {
    let map = body.as_object_mut().expect("object body");
    let commands = vec![
        "cargo orchestrate --main-thread-shift --skip-cargo",
        "cargo orchestrate --main-thread-shift",
        ".\\tools\\orchestrator\\scripts\\main_thread_shift.ps1",
        "cargo test -p proc_A_dine01 --lib stage5",
    ];
    map.insert(
        "_agent_meta".to_string(),
        json!({
            "schema": "debug_run_envelope_v1",
            "profile": "MAIN_THREAD_SHIFT",
            "source_system": "rust_engine_orchestrator::main_thread_shift",
            "relative_path": LIVE_PROOF_PATH,
            "agent_commands": commands,
            "related_proofs": WITNESS_PATHS,
            "agents": {
                "main_thread_orchestrator": ".cursor/agents/main-thread-orchestrator.md",
                "sim_steward": ".cursor/agents/sim-steward.md",
            },
            "skills": [
                "bevy-simulation-grade",
                "debug-intelligence",
                "cleanup-completion-intelligence",
            ],
        }),
    );
    body
}

fn digest_witnesses(repo_root: &Path) -> Vec<WitnessDigest> {
    WITNESS_PATHS
        .iter()
        .map(|rel| {
            let path = repo_root.join(rel);
            let exists = path.is_file();
            let summary = if exists {
                fs::read_to_string(&path)
                    .ok()
                    .map(|t| summarize_witness(rel, &t))
                    .unwrap_or_else(|| json!({ "parse_ok": false }))
            } else {
                json!({ "parse_ok": false, "missing": true })
            };
            WitnessDigest {
                path: (*rel).to_string(),
                exists,
                summary,
            }
        })
        .collect()
}

fn summarize_witness(rel: &str, text: &str) -> Value {
    let Ok(v) = serde_json::from_str::<Value>(text) else {
        return json!({ "parse_ok": false });
    };
    match rel {
        "debug_runs/stage5_full_app_live.json" => {
            let passes = v.pointer("/readiness/passes").and_then(|p| p.as_bool());
            let violations = v
                .pointer("/readiness/violations")
                .and_then(|a| a.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            let todo_done = v
                .pointer("/readiness/live_todo_board/all_done")
                .and_then(|b| b.as_bool());
            json!({
                "parse_ok": true,
                "profile": v.get("profile"),
                "readiness_passes": passes,
                "violation_count": violations,
                "live_todo_all_done": todo_done,
                "map_mismatch_frames": v.pointer("/map_presentation_stability/mismatch_frames"),
            })
        }
        "debug_runs/viewport_drift.json" => json!({
            "parse_ok": true,
            "status": v.get("status"),
            "canonical_path": v.get("canonical_path"),
        }),
        _ => {
            let mut m = Map::new();
            m.insert("parse_ok".into(), json!(true));
            if let Some(p) = v.get("profile") {
                m.insert("profile".into(), p.clone());
            }
            if let Some(g) = v.get("operational_green").or(v.get("infrastructure_view_isolation_green")) {
                m.insert("green".into(), g.clone());
            }
            Value::Object(m)
        }
    }
}

fn classify_cleanup_markers(
    markers: &[SemanticMarker],
    annotations: &[crate::models::SourceAnnotation],
) -> Vec<CleanupClassification> {
    let do_not_touch: std::collections::HashSet<String> = annotations
        .iter()
        .filter(|a| a.do_not_cleanup)
        .map(|a| format!("{}:{}", a.file, a.line))
        .collect();

    let mut out = Vec::new();
    for m in markers {
        let key = format!("{}:{}", m.file, m.line);
        let (classification, decision, note) = if do_not_touch.contains(&key) {
            (
                "B_transitional",
                "preserve",
                "@orchestrator-do-not-cleanup",
            )
        } else {
            match m.kind.as_str() {
                "REMOVE_AFTER" => (
                    "A_obsolete",
                    "completion_plan",
                    "REMOVE_AFTER requires migration successor",
                ),
                "DEPRECATED" => ("B_transitional", "refactor", "deprecated API"),
                "MIGRATION" => ("B_transitional", "preserve", "active migration"),
                "TEMP" | "HACK" | "WORKAROUND" => ("D_incomplete", "preserve", "staging debt"),
                _ => continue,
            }
        };
        out.push(CleanupClassification {
            file: m.file.clone(),
            line: m.line,
            marker_kind: m.kind.clone(),
            classification: classification.into(),
            decision: decision.into(),
            note: note.into(),
        });
    }
    out.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    out.truncate(80);
    out
}

fn build_debug_routing(
    witnesses: &[WitnessDigest],
    alerts: &[crate::authority_scan::AuthorityAlert],
    cleanup: &[CleanupClassification],
) -> DebugRouting {
    let mut root_cause = Vec::new();
    let mut evidence = Vec::new();
    let mut affected = Vec::new();
    let mut recommendation = Vec::new();
    let mut severity = "LOW".to_string();

    if let Some(stage5) = witnesses.iter().find(|w| w.path.contains("stage5_full_app")) {
        if stage5.exists {
            if let Some(false) = stage5.summary.get("readiness_passes").and_then(|v| v.as_bool()) {
                severity = "HIGH".into();
                root_cause.push("Stage5 FULL_APP readiness/passes false in live witness".into());
            }
            if let Some(n) = stage5.summary.get("violation_count").and_then(|v| v.as_u64()) {
                if n > 0 {
                    severity = "HIGH".into();
                    evidence.push(format!("stage5_full_app_live.json violation_count={n}"));
                }
            }
            if stage5.summary.get("live_todo_all_done") == Some(&json!(false)) {
                severity = if severity == "HIGH" { severity } else { "MED".into() };
                evidence.push("stage5 live_todo_board not all_done".into());
            }
        } else {
            evidence.push("stage5_full_app_live.json missing — run visual FULL_APP probe".into());
        }
    }

    if let Some(vp) = witnesses.iter().find(|w| w.path.contains("viewport_drift")) {
        if let Some(status) = vp.summary.get("status").and_then(|s| s.as_str()) {
            evidence.push(format!("viewport_drift status={status}"));
        }
    }

    for alert in alerts {
        affected.push(format!("{}:{}", alert.resource, alert.severity));
        evidence.push(alert.message.clone());
        if alert.severity == "MED" && severity != "HIGH" {
            severity = "MED".into();
        }
        root_cause.push(format!(
            "Non-canonical ResMut writer(s) for {}",
            alert.resource
        ));
        recommendation.push(format!(
            "Route to @coder: verify schedule order vs viewport_pipeline for {}",
            alert.resource
        ));
    }

    let remove_candidates = cleanup
        .iter()
        .filter(|c| c.decision == "remove")
        .count();
    if remove_candidates > 0 {
        evidence.push(format!("cleanup: {remove_candidates} remove candidate(s) — review Shift B"));
        recommendation.push("Run cleanup-completion-intelligence before any deletion".into());
    }

    if root_cause.is_empty() {
        root_cause.push("No HIGH/MED blockers in static shift scan".into());
    }
    if recommendation.is_empty() {
        recommendation.push("Continue lane work; refresh witnesses after src/ edits".into());
    }

    DebugRouting {
        id: "MAIN-THREAD-SHIFT-001".into(),
        severity,
        root_cause,
        affected,
        evidence,
        recommendation,
        owner: "main-thread-orchestrator".into(),
        confidence: if alerts.is_empty() { 0.85 } else { 0.7 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::find_repo_root;

    #[test]
    fn main_thread_shift_runs_on_repo() {
        let root = find_repo_root(&std::env::current_dir().unwrap());
        let report = run_main_thread_shift(&root);
        assert!(!report.run_id.is_empty());
        assert!(!report.shift_b_debug.id.is_empty());
    }

    #[test]
    fn classify_respects_do_not_cleanup() {
        use crate::models::SourceAnnotation;
        let markers = vec![SemanticMarker {
            file: "src/foo.rs".into(),
            line: 1,
            kind: "REMOVE_AFTER".into(),
            text: "REMOVE_AFTER migration".into(),
        }];
        let ann = vec![SourceAnnotation {
            file: "src/foo.rs".into(),
            line: 1,
            status: "IN_PROGRESS".into(),
            owner: None,
            do_not_cleanup: true,
            note: Some("viewport migration".into()),
        }];
        let c = classify_cleanup_markers(&markers, &ann);
        assert_eq!(c[0].decision, "preserve");
    }
}
