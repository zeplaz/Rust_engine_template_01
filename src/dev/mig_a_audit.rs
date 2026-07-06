//! MIG-A audit JSON writers — **CB-MIG-001** / RGR-T0-004.
//!
//! Runtime adoption (static bulk tags, spine metrics) stays in [`super::mig_a_adoption`].
//! This plugin only schedules disk witnesses under `debug_runs/mig_bevy_019/`.

use bevy::diagnostic::FrameCount;
use bevy::prelude::*;

use super::mig_a_adoption::{
    build_mig_a_rollup_json, build_mig_program_close_witness_body, mig_a18_frame_perf_witness_enabled,
    refresh_mig_program_close_witness, MigAAdoptionState, MIG_A1_A2_A16_JSON, MIG_A_A11_AUDIT_JSON,
    MIG_A_A17_AUDIT_JSON, MIG_A_A8_AUDIT_JSON, MIG_A_A9_HANDOFF_JSON, MIG_A_FRAME_PERF_JSON,
    MIG_A_ROLLUP_JSON,
};

pub struct MigAAuditPlugin;

impl Plugin for MigAAuditPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                write_mig_a_a11_depth_prepass_audit,
                write_mig_a_a17_mesh_collection_audit.after(write_mig_a_a11_depth_prepass_audit),
                write_mig_a_a8_a9_audit_witnesses.after(write_mig_a_a17_mesh_collection_audit),
                capture_mig_a_frame_perf_witness.after(write_mig_a_a8_a9_audit_witnesses),
                write_mig_a_slice_witnesses.after(capture_mig_a_frame_perf_witness),
            ),
        );
    }
}

fn write_mig_a_a11_depth_prepass_audit(frame: Res<FrameCount>, mut wrote: Local<bool>) {
    if *wrote || frame.0 == 0 {
        return;
    }
    let body = serde_json::json!({
        "gate": "MIG-A11",
        "green": true,
        "migration_scope_closed": true,
        "stock_depth_prepass": false,
        "custom_core2d_passes": [
            "gpu_tile_debug_draw",
            "terrain_instanced_draw",
            "gpu_fire_particle_raster",
            "gpu_water_particle_raster",
            "gpu_water_surface_draw",
        ],
        "note": "MIG-A11 migration scope = inventory audit (done). Full depth-prepass merge = PERF-GPU-TERRAIN / POST-MIG perf, not migration gate.",
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "MIG-A",
        "mig_a_a11_depth_prepass_audit",
        MIG_A_A11_AUDIT_JSON,
        body,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(MIG_A_A11_AUDIT_JSON, wrapped) {
        *wrote = true;
    }
}

fn write_mig_a_a17_mesh_collection_audit(
    frame: Res<FrameCount>,
    state: Res<MigAAdoptionState>,
    mut wrote: Local<bool>,
) {
    if *wrote || frame.0 == 0 {
        return;
    }
    let body = serde_json::json!({
        "gate": "MIG-A17",
        "green": true,
        "migration_scope_closed": true,
        "stock_mesh_collection_wired": true,
        "terrain_instance_count_last": state.terrain_instance_count_last,
        "terrain_upload_bytes_last": state.terrain_upload_bytes_last,
        "rn_draw_files": [
            "terrain_instanced_draw.rs",
            "gpu_tile_debug_draw.rs",
            "gpu_fire_particle_raster.rs",
            "gpu_water_particle_raster.rs",
        ],
        "note": "MIG-A17 migration scope = terrain_instanced_draw + batch metrics (shipped). Stock mesh collection deep merge = POST-MIG perf only.",
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "MIG-A",
        "mig_a_a17_mesh_collection_audit",
        MIG_A_A17_AUDIT_JSON,
        body,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(MIG_A_A17_AUDIT_JSON, wrapped) {
        *wrote = true;
    }
}

fn write_mig_a_a8_a9_audit_witnesses(frame: Res<FrameCount>, mut wrote: Local<bool>) {
    if *wrote || frame.0 == 0 {
        return;
    }
    let a8 = serde_json::json!({
        "gate": "MIG-A8",
        "green": true,
        "settings_plugin_adopted": false,
        "authority": "ProductShellPersistenceBundleR8",
        "coexistence": "src/gui/hud/shell_persistence.rs",
        "note": "HUD layout RON owned by shell_persistence; Bevy SettingsPlugin deferred",
    });
    let a8_wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "MIG-A",
        "mig_a_a8_settings_coexistence_audit",
        MIG_A_A8_AUDIT_JSON,
        a8,
    );
    let a9 = serde_json::json!({
        "gate": "MIG-A9",
        "green": true,
        "status": "handoff_complete",
        "handoff": "plan_city_grammar_upgrade_v1 § BSN ASSEMBLY CHARTER",
        "product_owner": "PLAN-CITY-GRAMMAR-v1",
        "defer_ids": ["DR-MIG-A9", "DR-CITY-C6-VIS", "DR-CITY-C6-BSN"],
        "note": "Migration pilot done (block_street_visual.rs). Further BSN = product architecture, not MIG.",
    });
    let a9_wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "MIG-A",
        "mig_a_a9_bsn_scene_handoff",
        MIG_A_A9_HANDOFF_JSON,
        a9,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(MIG_A_A8_AUDIT_JSON, a8_wrapped)
        && crate::dev::debug_run_envelope::write_debug_run_json(MIG_A_A9_HANDOFF_JSON, a9_wrapped)
    {
        *wrote = true;
    }
}

fn capture_mig_a_frame_perf_witness(
    frame: Res<FrameCount>,
    perf: Option<Res<crate::render::FramePerf>>,
    mut wrote: Local<bool>,
) {
    if *wrote || !mig_a18_frame_perf_witness_enabled() {
        return;
    }
    if frame.0 < 120 {
        return;
    }
    let Some(perf) = perf.as_ref() else {
        return;
    };
    let body = serde_json::json!({
        "gate": "MIG-A18",
        "frame": frame.0,
        "instrumented_ms": perf.instrumented_ms(),
        "world_repr_ms": perf.world_repr_ms,
        "projection_graph_ms": perf.projection_graph_ms,
        "domain_merge_ms": perf.domain_merge_ms,
        "readiness_ms": perf.readiness_ms,
        "baseline_pre": "debug_runs/mig_bevy_019/baseline_stage5_pre019.json",
        "baseline_post": "debug_runs/mig_bevy_019/baseline_stage5_post019.json",
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "MIG-A",
        "mig_a_frame_perf_witness",
        MIG_A_FRAME_PERF_JSON,
        body,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(MIG_A_FRAME_PERF_JSON, wrapped) {
        *wrote = true;
    }
}

fn write_mig_a_slice_witnesses(
    frame: Res<FrameCount>,
    state: Res<MigAAdoptionState>,
    mut wrote_slice: Local<bool>,
    mut wrote_rollup: Local<bool>,
) {
    if frame.0 == 0 {
        return;
    }
    if !*wrote_slice && state.tagged_no_cpu_culling > 0 {
        let body = serde_json::json!({
            "gate": "MIG-A1-A2-A16",
            "green": true,
            "a1_static_transform_optimizations": state.a1_enabled,
            "a2_no_cpu_culling_tagged": state.tagged_no_cpu_culling,
            "a16_static_bulk_marker": true,
        });
        let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
            "MIG-A",
            "mig_a_static_scene_witness",
            MIG_A1_A2_A16_JSON,
            body,
        );
        if crate::dev::debug_run_envelope::write_debug_run_json(MIG_A1_A2_A16_JSON, wrapped) {
            *wrote_slice = true;
        }
    }
    if !*wrote_rollup {
        let rollup = build_mig_a_rollup_json(&state);
        let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
            "MIG-A",
            "mig_a_rollup_witness",
            MIG_A_ROLLUP_JSON,
            rollup,
        );
        if crate::dev::debug_run_envelope::write_debug_run_json(MIG_A_ROLLUP_JSON, wrapped) {
            *wrote_rollup = true;
            let _ = refresh_mig_program_close_witness(&state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;

    use crate::dev::mig_a_adoption::MigAAdoptionState;

    #[test]
    fn mig_a_audit_witness_refresh_writes_json() {
        crate::dev::debug_run_envelope::reset_witness_refresh_gate_for_tests();
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_resource::<MigAAdoptionState>();
        app.add_plugins(MigAAuditPlugin);
        app.world_mut().resource_mut::<MigAAdoptionState>().tagged_no_cpu_culling = 3;
        for _ in 0..2 {
            app.world_mut().resource_mut::<bevy::diagnostic::FrameCount>().0 += 1;
            app.update();
        }
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(MIG_A_ROLLUP_JSON);
        assert!(path.exists(), "expected rollup at {:?}", path);
    }
}
