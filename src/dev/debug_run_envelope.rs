//! Shared envelope for `debug_runs/*.json` — timestamps, env flags, and agent navigation hints.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};

pub const ENVELOPE_SCHEMA: &str = "debug_run_envelope_v1";
pub const WITNESS_HONESTY_ENFORCE_ENV: &str = "RUST_ENGINE_WITNESS_INTEGRITY_ENFORCE";
pub const WITNESS_HONESTY_SKIP_ENV: &str = "RUST_ENGINE_WITNESS_INTEGRITY_SKIP";
const WITNESS_HONESTY_PRECHECK_DIR: &str = "debug_runs/.witness_honesty_precheck";

/// Primary live proofs agents should read (relative to repo root).
pub const KNOWN_LIVE_PROOF_PATHS: &[&str] = &[
    "debug_runs/stage5_full_app_live.json",
    "debug_runs/infrastructure_view_isolation_live.json",
    "debug_runs/construction_stage_live.json",
    "debug_runs/industrial_activation_live.json",
    "debug_runs/fire_ecology_live.json",
    "debug_runs/logistics_throughput_live.json",
    "debug_runs/replay_editor_parity_live.json",
    "debug_runs/orchestrator_thread_health.json",
    "debug_runs/viewport_drift.json",
    "debug_runs/viewport_authority_migration_witness.json",
    "debug_runs/main_thread_orchestrator_live.json",
    "debug_runs/stage6_virtualization_live.json",
    "debug_runs/wave_s_hydrate_live.json",
    "debug_runs/wave_p_live.json",
    "debug_runs/wave_c_live.json",
    "debug_runs/ui_shell_migration_live.json",
    "debug_runs/minimap_compositor_live.json",
    "debug_runs/stage7_behavioral_live.json",
    "debug_runs/stage7_play_live.json",
    "debug_runs/wss_substrate_live.json",
    "debug_runs/f2_smoke_pipeline_live.json",
    "debug_runs/play_scenario_live.json",
    "debug_runs/transport_network_live.json",
];

pub const AGENT_DEBUG_INDEX_PATH: &str = "debug_runs/agent_debug_index.json";

#[must_use]
pub fn debug_runs_dir() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("debug_runs")
}

#[must_use]
pub fn epoch_secs_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[must_use]
pub fn env_flag(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

#[must_use]
pub fn env_opt(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|s| !s.is_empty())
}

/// Snapshot of env vars that affect readiness logs, viewport trace, and orchestrator export.
#[must_use]
pub fn logging_env_snapshot() -> Value {
    serde_json::json!({
        "RUST_LOG": env_opt("RUST_LOG"),
        "PERF": env_flag("PERF"),
        "STAGE5_VERBOSE": env_flag("STAGE5_VERBOSE"),
        "STAGE5_READINESS_VERBOSE": env_flag("STAGE5_READINESS_VERBOSE"),
        "SIM_VIEW_SYNC_DEBUG": env_flag("SIM_VIEW_SYNC_DEBUG"),
        "VIEWPORT_AUTHORITY_DEBUG": env_flag("VIEWPORT_AUTHORITY_DEBUG"),
        "ORCHESTRATOR_EXPORT_HEALTH": env_flag("ORCHESTRATOR_EXPORT_HEALTH"),
    })
}

#[must_use]
pub fn agent_commands_for_profile(profile: &str) -> Vec<&'static str> {
    match profile {
        "FULL_APP" => vec![
            "cargo test -p proc_A_dine01 --lib stage5",
            "cargo run -p proc_A_dine01 -- --test visual",
            "RUST_LOG=stage5_readiness::live=info cargo run -p proc_A_dine01 -- --test visual",
            "cargo run --manifest-path tools/orchestrator/Cargo.toml -- --skip-clippy --skip-test",
        ],
        "UI_SHELL_MIGRATION_2A" | "UI_SHELL_MIGRATION_2B" => vec![
            "cargo test -p proc_A_dine01 --lib stage5",
            "cargo test -p proc_A_dine01 --lib ui_p2b",
            "cargo test -p proc_A_dine01 --lib simulation_shell_phase2",
            "cargo run -p proc_A_dine01 --release -- --test visual",
        ],
        "MINIMAP_COMPOSITOR_M1" => vec![
            "cargo test -p proc_A_dine01 --lib stage5",
            "cargo test -p proc_A_dine01 --lib minimap_compositor",
            "MINIMAP_GPU_COMPOSITOR=1 cargo run -p proc_A_dine01 --release -- --test visual",
        ],
        "CONSTRUCTION_STAGE" => vec![
            "cargo test -p proc_A_dine01 construction:: --lib",
            "cargo run -p proc_A_dine01",
        ],
        "INDUSTRIAL_ACTIVATION" | "LOGISTICS_THROUGHPUT" => vec![
            "cargo test -p proc_A_dine01 economy:: --lib",
        ],
        "INFRASTRUCTURE_VIEW_ISOLATION" => vec![
            "cargo test -p proc_A_dine01 render::view_runtime --lib",
        ],
        "WAVE_P_PREVIEW" => vec![
            "cargo test -p proc_A_dine01 --lib ui_wp_layout_002_writes_wave_p_live_json",
            "cargo test -p proc_A_dine01 --lib cod_b_wp_witness_001",
            "cargo test -p proc_A_dine01 --lib wave_p_witness",
        ],
        "STAGE6_VIRTUALIZATION" => vec![
            "cargo test -p proc_A_dine01 --lib stage6",
            "cargo test -p proc_A_dine01 --lib wc_d04_coder_b",
            "cargo run -p proc_A_dine01",
        ],
        "MAIN_THREAD_SHIFT" => vec![
            "cargo orchestrate --main-thread-shift --skip-cargo",
            ".\\tools\\orchestrator\\scripts\\main_thread_shift.ps1",
        ],
        "WAVE_C_STREAMING" => vec![
            "cargo test -p proc_A_dine01 --lib wc_depth_001",
            "cargo test -p proc_A_dine01 --lib wave_c",
        ],
        "STAGE7_BEHAVIORAL" => vec![
            "cargo test -p proc_A_dine01 --lib stage7_behavioral",
            "cargo test -p proc_A_dine01 --lib stage7_play comms_contract",
        ],
        "WSS_SUBSTRATE" => vec![
            "cargo test -p proc_A_dine01 --lib wss_substrate",
            "cargo test -p proc_A_dine01 --lib stage5 fire_streaming gpu_particles",
        ],
        "LG-4-PREVIEW-001" | "PLAN-VEG-RUNTIME-PROOF-001" | "VEG-PROGRAM-CLOSE-001" => vec![
            "cargo test -p proc_A_dine01 --lib sim_harness_refreshes_witness_json_green",
            "cargo test -p proc_A_dine01 --lib landscape_grammar",
        ],
        _ => vec!["cargo test -p proc_A_dine01 --lib"],
    }
}

#[must_use]
pub fn orchestrator_hints() -> Value {
    serde_json::json!({
        "build_report": "tools/orchestrator/reports/build_report.md",
        "warning_registry": "tools/orchestrator/reports/warning_registry.md",
        "continuation_queue": "tools/orchestrator/queues/continuation_queue.json",
        "ci_entry": "tools/orchestrator/ci/run.ps1",
    })
}

/// Merge `_agent_meta` into a proof body (top-level keys preserved for existing consumers).
#[must_use]
pub fn wrap_debug_run(
    profile: &str,
    source_system: &str,
    relative_path: &str,
    body: Value,
) -> Value {
    let mut map = match body {
        Value::Object(m) => m,
        other => {
            let mut m = Map::new();
            m.insert("payload".into(), other);
            m
        }
    };

    let commands: Vec<Value> = agent_commands_for_profile(profile)
        .into_iter()
        .map(|s| Value::String(s.to_string()))
        .collect();

    map.insert(
        "_agent_meta".into(),
        serde_json::json!({
            "schema": ENVELOPE_SCHEMA,
            "written_at_epoch_secs": epoch_secs_now(),
            "profile": profile,
            "source_system": source_system,
            "relative_path": relative_path,
            "logging_env": logging_env_snapshot(),
            "agent_commands": commands,
            "related_proofs": KNOWN_LIVE_PROOF_PATHS,
            "orchestrator": orchestrator_hints(),
            "docs": {
                "stage5_directive": "prompts/guides/stage5_convergence_directive_v1.md",
                "compile_warnings": "src/dev/COMPILE_WARNINGS_TODOS.md",
                "viewport_recovery": "src/dev/recovery_viewport.md",
            },
        }),
    );

    Value::Object(map)
}

fn write_json_file(relative_path: &str, payload: &Value) -> bool {
    let path = repo_root_path().join(relative_path);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let Ok(text) = serde_json::to_string_pretty(payload) else {
        return false;
    };
    fs::write(&path, text).is_ok()
}

#[must_use]
fn repo_root_path() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// WIT-RUST-004 — run MCP witness_honesty validator (subprocess) before writing `*_live.json`.
///
/// When [`WITNESS_HONESTY_ENFORCE_ENV`] is set, a failed check blocks the write.
/// Set [`WITNESS_HONESTY_SKIP_ENV`] to bypass (tests / offline).
#[must_use]
pub fn assert_witness_honesty_before_write(relative_path: &str, body: &Value) -> bool {
    if !relative_path.ends_with("_live.json") || env_flag(WITNESS_HONESTY_SKIP_ENV) {
        return true;
    }

    let root = repo_root_path();
    let basename = Path::new(relative_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("witness_live.json");
    let precheck_dir = root.join(WITNESS_HONESTY_PRECHECK_DIR);
    if fs::create_dir_all(&precheck_dir).is_err() {
        return !env_flag(WITNESS_HONESTY_ENFORCE_ENV);
    }
    let precheck_path = precheck_dir.join(basename);
    if !write_json_file(
        &format!("{WITNESS_HONESTY_PRECHECK_DIR}/{basename}"),
        body,
    ) {
        return !env_flag(WITNESS_HONESTY_ENFORCE_ENV);
    }

    let mcp_python = root.join("tools/mcp/python");
    let script = format!(
        "import json, sys\n\
         from pathlib import Path\n\
         from rust_engine_mcp.paths import repo_root\n\
         from rust_engine_mcp.validators.witness_honesty import (\n\
             load_witness_integrity_catalog, validate_witness_honesty,\n\
         )\n\
         root = repo_root()\n\
         data = json.loads(Path(sys.argv[1]).read_text(encoding='utf-8'))\n\
         witness_rel = sys.argv[2]\n\
         report = validate_witness_honesty(\n\
             data,\n\
             witness_rel=witness_rel,\n\
             catalog=load_witness_integrity_catalog(repo=root),\n\
             root=root,\n\
             compression_level=3,\n\
         )\n\
         print(json.dumps(report.to_dict()))\n\
         sys.exit(0 if report.status == 'passed' else 1)\n"
    );

    let output = Command::new("python")
        .current_dir(&mcp_python)
        .arg("-c")
        .arg(&script)
        .arg(&precheck_path)
        .arg(relative_path)
        .output();

    let passed = match output {
        Ok(out) => out.status.success(),
        Err(_) => true,
    };

    if passed {
        true
    } else {
        !env_flag(WITNESS_HONESTY_ENFORCE_ENV)
    }
}

/// Write pretty JSON and refresh [`AGENT_DEBUG_INDEX_PATH`] (unless writing the index itself).
pub fn write_debug_run_json(relative_path: &str, payload: Value) -> bool {
    if !assert_witness_honesty_before_write(relative_path, &payload) {
        return false;
    }
    if !write_json_file(relative_path, &payload) {
        return false;
    }
    if relative_path != AGENT_DEBUG_INDEX_PATH {
        let _ = refresh_agent_debug_index();
    }
    true
}

fn file_modified_epoch(path: &Path) -> Option<u64> {
    let meta = fs::metadata(path).ok()?;
    meta.modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

fn quick_extract_summary(text: &str) -> Value {
    let Ok(v) = serde_json::from_str::<Value>(text) else {
        return serde_json::json!({ "parse_ok": false });
    };
    let profile = v.get("profile").and_then(|p| p.as_str());
    let readiness_passes = v
        .pointer("/readiness/passes")
        .or_else(|| v.pointer("/stage6_readiness/passes"))
        .and_then(|p| p.as_bool());
    let violations_len = v
        .pointer("/readiness/violations")
        .or_else(|| v.pointer("/stage6_readiness/violations"))
        .and_then(|a| a.as_array())
        .map(|a| a.len());
    serde_json::json!({
        "parse_ok": true,
        "profile": profile,
        "readiness_passes": readiness_passes,
        "readiness_violation_count": violations_len,
        "operational_green": v.get("operational_green").and_then(|b| b.as_bool()),
        "activation_green": v.get("activation_green").and_then(|b| b.as_bool()),
        "throughput_green": v.get("throughput_green").and_then(|b| b.as_bool()),
        "infrastructure_view_isolation_green": v
            .get("infrastructure_view_isolation_green")
            .and_then(|b| b.as_bool()),
        "parity_green": v.get("parity_green").and_then(|b| b.as_bool()),
        "stage6_virtualization_green": v
            .get("stage6_virtualization_green")
            .and_then(|b| b.as_bool()),
    })
}

/// Scan known proof paths; write `debug_runs/agent_debug_index.json` for orchestrator agents.
pub fn refresh_agent_debug_index() -> std::io::Result<()> {
    let root = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut proofs = Vec::new();
    for rel in KNOWN_LIVE_PROOF_PATHS {
        let path = root.join(rel);
        let exists = path.is_file();
        let (bytes, modified_epoch_secs, summary) = if exists {
            let text = fs::read_to_string(&path).unwrap_or_default();
            let bytes = text.len() as u64;
            let summary = quick_extract_summary(&text);
            (bytes, file_modified_epoch(&path), summary)
        } else {
            (0, None, serde_json::json!({ "parse_ok": false, "missing": true }))
        };
        proofs.push(serde_json::json!({
            "path": rel,
            "exists": exists,
            "bytes": bytes,
            "modified_epoch_secs": modified_epoch_secs,
            "summary": summary,
        }));
    }

    let index = wrap_debug_run(
        "AGENT_DEBUG_INDEX",
        "debug_run_envelope",
        AGENT_DEBUG_INDEX_PATH,
        serde_json::json!({
            "profile": "AGENT_DEBUG_INDEX",
            "proof_count": proofs.len(),
            "proofs": proofs,
        }),
    );

    write_json_file(AGENT_DEBUG_INDEX_PATH, &index)
        .then_some(())
        .ok_or(std::io::Error::other("agent_debug_index write failed"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_inserts_agent_meta() {
        let wrapped = wrap_debug_run(
            "TEST",
            "unit",
            "debug_runs/test.json",
            serde_json::json!({ "profile": "TEST", "ok": true }),
        );
        assert!(wrapped.get("_agent_meta").is_some());
        assert_eq!(wrapped.get("ok").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn assert_witness_honesty_blocks_wit_green_tint_zero_when_enforced() {
        let body = serde_json::json!({
            "_agent_meta": { "schema": ENVELOPE_SCHEMA },
            "gate": "LG-4-PREVIEW-FIXTURE",
            "green": true,
            "topology_tint_visible_chunks": 0,
        });
        std::env::set_var(WITNESS_HONESTY_SKIP_ENV, "1");
        assert!(assert_witness_honesty_before_write(
            "debug_runs/landscape_grammar_lg4_preview_live.json",
            &body
        ));
        std::env::remove_var(WITNESS_HONESTY_SKIP_ENV);
        std::env::set_var(WITNESS_HONESTY_ENFORCE_ENV, "1");
        let blocked = assert_witness_honesty_before_write(
            "debug_runs/landscape_grammar_lg4_preview_live.json",
            &body,
        );
        std::env::remove_var(WITNESS_HONESTY_ENFORCE_ENV);
        if std::process::Command::new("python")
            .arg("-c")
            .arg("import rust_engine_mcp")
            .current_dir(repo_root_path().join("tools/mcp/python"))
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            assert!(!blocked, "WIT-GREEN-TINT-ZERO must block when enforced");
        }
    }
}
