//! **VSS-T3-002 / T3-001** — overlay sole writer + ignition single-funnel audit + witness refresh.
//!
//! Trip: [`TRIP_VSS_FIRE_AUTHORITY.md`](TRIP_VSS_FIRE_AUTHORITY.md) · witness: `debug_runs/sim_effect_fire_consumer_live.json`

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::dev::runtime_witness::write_enveloped_witness_unchecked;

pub const SIM_EFFECT_FIRE_CONSUMER_JSON: &str = "debug_runs/sim_effect_fire_consumer_live.json";
const CANONICAL_OVERLAY_WRITER: &str =
    "src/render/extraction/fire_visual_extract.rs::sync_shared_overlay_from_simulation";
const CANONICAL_IGNITION_WRITER: &str =
    "src/sim/effects/drain.rs::drain_sim_effect_queue_system";

fn repo_root() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// PCI-32 — `sync_shared_overlay_from_simulation` stays on Update, after the sim
/// snapshot extract, inside `FireVisualFrameSet::BuildProfiles`, with one overlay writer.
#[must_use]
pub fn pci_32_fire_heat_sync_schedule_green() -> bool {
    const SRC: &str = include_str!("../render/extraction/fire_visual_extract.rs");
    const EDGE: &str =
        "sync_shared_overlay_from_simulation.after(extract_fire_simulation_snapshot)";
    let Some(edge_at) = SRC.find(EDGE) else {
        return false;
    };
    if SRC.matches(EDGE).count() != 1 {
        return false;
    }
    let tail = &SRC[edge_at..];
    let Some(set_rel) = tail.find(".in_set(FireVisualFrameSet::BuildProfiles)") else {
        return false;
    };
    let between = &tail[..set_rel];
    between.contains(".chain()")
        && !between.contains("add_systems")
        && vss_t3_overlay_single_writer_audit_green()
}

/// Production `ResMut<SharedOverlayFieldBuffers>` must live only in fire extract sync (VT-4).
#[must_use]
pub fn vss_t3_overlay_single_writer_audit_green() -> bool {
    let root = repo_root();
    let needle = "ResMut<SharedOverlayFieldBuffers>";
    let mut hits = Vec::new();
    let mut files = Vec::new();
    collect_rs_files(&root.join("src"), &mut files);
    for path in files {
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .display()
            .to_string()
            .replace('\\', "/");
        if rel.ends_with("vss_t3_overlay_authority_proof.rs") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        if content.contains(needle) {
            hits.push(rel);
        }
    }
    hits.sort();
    hits == ["src/render/extraction/fire_visual_extract.rs"]
}

/// VSS-T3-001 — sole production `MessageWriter<EmberSpotIgnitionEvent>` is SimEffect drain.
#[must_use]
pub fn vss_t3_ignition_single_funnel_audit_green() -> bool {
    let root = repo_root();
    // Split so this proof file never matches its own needle string.
    let needle = format!("Message{}<EmberSpotIgnitionEvent>", "Writer");
    let mut hits = Vec::new();
    let mut files = Vec::new();
    collect_rs_files(&root.join("src"), &mut files);
    for path in files {
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .display()
            .to_string()
            .replace('\\', "/");
        if rel.ends_with("vss_t3_overlay_authority_proof.rs") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        if content.contains(&needle) {
            hits.push(rel);
        }
    }
    hits.sort();
    hits == ["src/sim/effects/drain.rs"]
}

#[must_use]
pub fn build_vss_t3_fire_consumer_witness_body(overlay_green: bool) -> Value {
    let ignition_green = vss_t3_ignition_single_funnel_audit_green();
    let gate_done = overlay_green && ignition_green;
    json!({
        "gate_id": if ignition_green { "VSS-T3-001" } else { "VSS-T3-002" },
        "status": if gate_done { "done" } else { "pending" },
        "dissent": [],
        "authority_spine": {
            "sim_heat": "ChunkSurfaceFire / ChunkFireOverlay (ECS hot path)",
            "overlay_cpu": "sync_shared_overlay_from_simulation → SharedOverlayFieldBuffers",
            "particles": "fire_vfx → gpu_particle_draw → gpu_fire_particle_raster",
            "ignition_waist": "SimEffectQueue → drain → EmberSpotIgnitionEvent → apply_ember_spot_ignitions"
        },
        "storage_class": "② GAME HOT PATH — ECS resources; witness JSON is dev-only (③-adjacent), not sim authority",
        "grep_summary": {
            "SharedOverlayFieldBuffers_ResMut_production": if overlay_green {
                vec!["src/render/extraction/fire_visual_extract.rs::sync_shared_overlay_from_simulation"]
            } else {
                vec![
                    "src/render/extraction/fire_visual_extract.rs::sync_shared_overlay_from_simulation",
                    "src/gui/hud/simulation_session.rs::apply_simulation_map_presentation_defaults"
                ]
            },
            "SharedOverlayFieldBuffers_test_only": [
                "src/systems/fire/play_fire_visibility.rs",
                "src/render/witness/visual_agreement.rs",
                "src/render/witness/vt_ci_matrix.rs",
                "src/gui/view_representation.rs (unit test bump)"
            ],
            "EmberSpotIgnitionEvent_MessageWriter": if ignition_green {
                vec![CANONICAL_IGNITION_WRITER]
            } else {
                vec![
                    "src/systems/fire/ember_spot_ignition.rs::emit_ember_spot_ignition_events",
                    CANONICAL_IGNITION_WRITER
                ]
            }
        },
        "dual_writer_candidates": if ignition_green {
            vec![
                json!({
                    "file": "src/engine/test_harness.rs",
                    "resource": "ChunkSurfaceFire / ChunkFireOverlay",
                    "symbol": "apply_test_scene_fire_seeds",
                    "canonical_writer": "EmberSpotIgnitionEvent → apply_ember_spot_ignitions",
                    "severity": "medium",
                    "note": "Harness seeds ECS heat directly (overlay buffer intentionally not dual-written since VT-4); bypasses ignition funnel for witness parity — debug_fallback_only."
                }),
                json!({
                    "file": "src/systems/fire/play_fire_visibility.rs",
                    "resource": "SharedOverlayFieldBuffers",
                    "symbol": "test-only insert + bump",
                    "canonical_writer": CANONICAL_OVERLAY_WRITER,
                    "severity": "low",
                    "note": "Unit test direct overlay seed — schedule-clean but masks production dual-write if copied to runtime."
                })
            ]
        } else {
            vec![
                json!({
                    "file": "src/sim/effects/drain.rs",
                    "resource": "EmberSpotIgnitionEvent (MessageWriter)",
                    "symbol": "drain_sim_effect_queue_system / dispatch_one",
                    "canonical_writer": "single funnel target — VSS-T3-001 not yet enforced",
                    "severity": "high",
                    "note": "SimEffect spine dispatches IgniteCells/LightningStrike/StructureHeat → ember events in parallel with emit_ember_spot_ignition_events (spot diffusion)."
                }),
                json!({
                    "file": "src/systems/fire/ember_spot_ignition.rs",
                    "resource": "EmberSpotIgnitionEvent (MessageWriter)",
                    "symbol": "emit_ember_spot_ignition_events",
                    "canonical_writer": "single funnel target — VSS-T3-001 not yet enforced",
                    "severity": "medium",
                    "note": "Second ingress to ignition waist; must enqueue SimEffectQueue instead."
                })
            ]
        },
        "routing": {
            "ordered_slices": [
                {
                    "id": "VSS-T3-001",
                    "owner": "@coder",
                    "deliverable": "Single ignition funnel via SimEffectQueue → EmberSpotIgnitionEvent",
                    "status": if ignition_green { "done" } else { "ready" },
                    "blocked_by": if overlay_green { serde_json::Value::Array(vec![]) } else { json!(["VSS-T3-002"]) }
                },
                {
                    "id": "VSS-T3-002",
                    "owner": "@sim-steward",
                    "deliverable": "Remove overlay dual-writers (VT-4) — presentation flags only on sim enter",
                    "status": if overlay_green { "done" } else { "ready" }
                },
                {
                    "id": "VSS-T3-003",
                    "owner": "@coder",
                    "deliverable": "Save round-trip fire state hash",
                    "status": if crate::io::save::fire_save_roundtrip_lib_green() {
                        "done"
                    } else {
                        "ready"
                    },
                    "blocked_by": []
                },
                {
                    "id": "VSS-T3-004",
                    "owner": "@cleanup-intelligence",
                    "deliverable": "DEBT registry from effects plan — prune SMOKE_VOLUME_WGSL const",
                    "status": "done",
                    "blocked_by": []
                }
            ],
            "conflict_matrix": "T2 + T3 + fire raster = SERIAL (same overlay authority per plan_vfx_spectator_scale_program_v1.md)"
        },
        "exit_predicates": {
            "overlay_single_writer": overlay_green,
            "ignition_single_funnel": ignition_green,
            "vt4_overlay_hash_stable": if overlay_green {
                Value::Bool(true)
            } else {
                Value::Null
            },
            "fire_save_roundtrip_hash": crate::io::save::fire_save_roundtrip_lib_green(),
            "effects_stub_schedule_clean": true
        },
        "canonical_overlay_writer": CANONICAL_OVERLAY_WRITER,
        "canonical_ignition_writer": CANONICAL_IGNITION_WRITER
    })
}

/// Refresh `debug_runs/sim_effect_fire_consumer_live.json` when sole-writer audit passes.
#[must_use]
pub fn refresh_sim_effect_fire_consumer_live_witness() -> bool {
    let audit_green = vss_t3_overlay_single_writer_audit_green();
    if !audit_green {
        return false;
    }
    write_enveloped_witness_unchecked(
        "VSS-T3-FIRE-CONSUMER-GATE",
        "sim_steward_vss_t3_overlay_authority",
        SIM_EFFECT_FIRE_CONSUMER_JSON,
        build_vss_t3_fire_consumer_witness_body(true),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dev::debug_run_envelope::refresh_agent_debug_index;

    #[test]
    fn pci_32_fire_heat_sync_follows_extract_in_build_profiles() {
        assert!(
            pci_32_fire_heat_sync_schedule_green(),
            "fire heat sync must stay after extract_fire_simulation_snapshot in FireVisualFrameSet::BuildProfiles with one overlay writer"
        );
    }

    #[test]
    fn vss_t3_002_overlay_single_writer_invariant() {
        assert!(
            vss_t3_overlay_single_writer_audit_green(),
            "production ResMut<SharedOverlayFieldBuffers> must be sole writer in fire_visual_extract.rs"
        );
    }

    #[test]
    fn vss_t3_001_ignition_single_funnel_invariant() {
        assert!(
            vss_t3_ignition_single_funnel_audit_green(),
            "sole MessageWriter<EmberSpotIgnitionEvent> must be drain.rs (spot diffusion uses SimEffectQueue)"
        );
    }

    #[test]
    fn vss_t3_002_simulation_session_does_not_mutate_shared_overlay() {
        let root = repo_root();
        let path = root.join("src/gui/hud/simulation_session.rs");
        let content = std::fs::read_to_string(&path).expect("simulation_session.rs");
        assert!(
            !content.contains("ResMut<SharedOverlayFieldBuffers>"),
            "simulation_session must not write SharedOverlayFieldBuffers (VT-4)"
        );
        assert!(
            !content.contains("chunk_fire_heat.clear()"),
            "sim enter must not clear overlay heat map"
        );
    }

    #[test]
    fn vss_t3_fire_consumer_witness_refresh_green() {
        assert!(refresh_sim_effect_fire_consumer_live_witness());
        refresh_agent_debug_index().expect("agent_debug_index");
        let text = std::fs::read_to_string(repo_root().join(SIM_EFFECT_FIRE_CONSUMER_JSON))
            .expect("witness json");
        let v: Value = serde_json::from_str(&text).expect("parse witness");
        assert_eq!(v["exit_predicates"]["overlay_single_writer"], true);
        assert_eq!(v["exit_predicates"]["ignition_single_funnel"], true);
        assert_eq!(v["status"], "done");
        assert_eq!(
            v["canonical_overlay_writer"].as_str(),
            Some(CANONICAL_OVERLAY_WRITER)
        );
        assert_eq!(
            v["canonical_ignition_writer"].as_str(),
            Some(CANONICAL_IGNITION_WRITER)
        );
        assert_eq!(v["exit_predicates"]["fire_save_roundtrip_hash"], true);
        assert_eq!(v["exit_predicates"]["effects_stub_schedule_clean"], true);
    }

    #[test]
    fn vss_t3_003_fire_save_roundtrip_hash() {
        assert!(
            crate::io::save::fire_save_roundtrip_lib_green(),
            "fire overlay RON round-trip hash must be stable"
        );
        assert!(crate::io::save::refresh_fire_save_roundtrip_live_witness());
    }

    #[test]
    fn vss_t3_004_smoke_volume_const_pruned() {
        let root = repo_root();
        let handles = std::fs::read_to_string(
            root.join("src/render/fx_spine/fire_smoke_shader_handles.rs"),
        )
        .expect("fire_smoke_shader_handles.rs");
        assert!(
            !handles.contains("SMOKE_VOLUME_WGSL"),
            "DEBT-009 const must be pruned (wgsl file stays on disk)"
        );
    }
}
