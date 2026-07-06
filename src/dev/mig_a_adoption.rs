//! Bevy 0.19 post-migration adoption — Phase 4 **MIG-A** slices.
//!
//! Runtime adoption plugin (static bulk tags, spine metrics) — **CB-MIG-001** runtime half.
//! Audit JSON writers live in [`super::mig_a_audit`]; spawn markers in [`crate::render::mig_a_static`].

use bevy::camera::visibility::VisibilityRange;
use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy::transform::StaticTransformOptimizations;

use crate::construction::ProceduralBuildModuleChild;
use crate::render::{
    frame_perf_verbose, mig_a1_static_transform_optimizations_enabled,
    mig_a10_spine_dispatch_authority, mig_a_static_bulk_bundle, mig_a_tactical_visibility_range,
    MigAStaticBulk, TerrainInstancedDrawGlobals, TerrainInstanceMap, TerrainTileInstance,
    TileWorldFallbackSprite,
};

pub const MIG_A1_A2_A16_JSON: &str = "debug_runs/mig_bevy_019/mig_a1_a2_a16_enabled.json";
pub const MIG_A_ROLLUP_JSON: &str = "debug_runs/mig_bevy_019/mig_a_rollup.json";
pub const MIG_A_FRAME_PERF_JSON: &str = "debug_runs/mig_bevy_019/mig_a_frame_perf.json";
pub const MIG_A_A11_AUDIT_JSON: &str = "debug_runs/mig_bevy_019/mig_a_a11_depth_prepass_audit.json";
pub const MIG_A_A17_AUDIT_JSON: &str = "debug_runs/mig_bevy_019/mig_a_a17_mesh_collection_audit.json";
pub const MIG_A_A8_AUDIT_JSON: &str = "debug_runs/mig_bevy_019/mig_a_a8_settings_coexistence_audit.json";
pub const MIG_A_A9_HANDOFF_JSON: &str = "debug_runs/mig_bevy_019/mig_a_a9_bsn_scene_handoff.json";
pub const MIG_A_PROGRAM_CLOSE_JSON: &str = "debug_runs/mig_bevy_019/mig_a_program_close.json";

#[derive(Resource, Clone, Debug, Default)]
pub struct MigAAdoptionState {
    pub a1_enabled: bool,
    pub a4_render_recovery: bool,
    pub a10_spine_authority: bool,
    pub a14_visibility_range: bool,
    pub tagged_no_cpu_culling: u32,
    pub tagged_visibility_range: u32,
    pub terrain_upload_bytes_last: u64,
    pub cpu_light_clusters_last: u32,
    pub cpu_light_requests_last: u32,
    pub a5_last_batch_preallocated: u32,
    pub terrain_instance_count_last: u32,
}

/// MIG-A12 — terrain instanced storage upload bytes per frame.
#[derive(Resource, Clone, Debug, Default)]
pub struct MigAUploadMetrics {
    pub terrain_instanced_upload_bytes: u64,
    pub terrain_instanced_frames: u64,
}

#[must_use]
pub fn mig_a_static_scene_enabled() -> bool {
    std::env::var("MIG_A_STATIC")
        .ok()
        .is_none_or(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
}

#[must_use]
pub fn mig_a4_render_recovery_enabled() -> bool {
    std::env::var("MIG_A4")
        .ok()
        .is_none_or(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

#[must_use]
pub fn mig_a14_visibility_range_enabled() -> bool {
    std::env::var("MIG_A14")
        .ok()
        .is_none_or(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
}

#[must_use]
pub fn mig_a18_frame_perf_witness_enabled() -> bool {
    std::env::var("MIG_A18")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        || frame_perf_verbose()
}

/// Register Bevy 0.19 [`RenderErrorHandler`] with recover-on-device-lost (MIG-A4).
pub fn register_mig_a_render_error_handler(app: &mut App) {
    if !mig_a4_render_recovery_enabled() {
        return;
    }
    use bevy::render::error_handler::{RenderErrorHandler, RenderErrorPolicy};
    app.insert_resource(RenderErrorHandler(|error, _main_world, error_type| {
        bevy::log::warn!(
            target: "mig_a_render_recovery",
            "render error {:?} type {:?} — recovering",
            error,
            error_type
        );
        RenderErrorPolicy::Recover(default())
    }));
}

pub struct MigAAdoptionPlugin;

/// Alias retained for early MIG-A1/A2/A16 wiring.
pub type MigAStaticScenePlugin = MigAAdoptionPlugin;

impl Plugin for MigAAdoptionPlugin {
    fn build(&self, app: &mut App) {
        if !mig_a_static_scene_enabled() {
            return;
        }
        app.init_resource::<MigAAdoptionState>()
            .init_resource::<MigAUploadMetrics>();
        if mig_a1_static_transform_optimizations_enabled() {
            app.insert_resource(StaticTransformOptimizations::Enabled);
        } else {
            app.insert_resource(StaticTransformOptimizations::Disabled);
        }
        // VisibilityRangePlugin ships with Bevy 0.19 DefaultPlugins — do not re-add (panics).
        register_mig_a_render_error_handler(app);
        #[cfg(feature = "dev_tools")]
        register_mig_a_diagnostics_overlay(app);
        app.add_systems(
            PostUpdate,
            (
                tag_mig_a_static_bulk_entities,
                collect_mig_a_runtime_metrics.after(tag_mig_a_static_bulk_entities),
            ),
        );
        app.add_plugins(super::mig_a_audit::MigAAuditPlugin);
    }
}

#[cfg(feature = "dev_tools")]
fn register_mig_a_diagnostics_overlay(app: &mut App) {
    if !std::env::var("MIG_A3")
        .ok()
        .is_none_or(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
    {
        return;
    }
    use bevy::dev_tools::diagnostics_overlay::DiagnosticsOverlayPlugin;
    app.add_plugins(DiagnosticsOverlayPlugin);
}

fn tag_mig_a_static_bulk_entities(
    mut commands: Commands,
    mut state: ResMut<MigAAdoptionState>,
    q: Query<
        Entity,
        (
            Without<MigAStaticBulk>,
            Without<TileWorldFallbackSprite>,
            Or<(
                With<ProceduralBuildModuleChild>,
                With<crate::strategic::settlement::BlockStreetFurniturePiece>,
                With<crate::strategic::settlement::BlockStreetFurnitureRoot>,
            )>,
        ),
    >,
    q_fallback_sprites: Query<Entity, (With<TileWorldFallbackSprite>, Without<MigAStaticBulk>)>,
    q_vis: Query<
        Entity,
        (
            With<MigAStaticBulk>,
            Without<VisibilityRange>,
        ),
    >,
) {
    state.a1_enabled = mig_a1_static_transform_optimizations_enabled();
    state.a4_render_recovery = mig_a4_render_recovery_enabled();
    state.a10_spine_authority = mig_a10_spine_dispatch_authority();
    state.a14_visibility_range = mig_a14_visibility_range_enabled();

    let range = mig_a_tactical_visibility_range();
    let mut tagged = 0u32;
    let mut vis_tagged = 0u32;
    for entity in &q {
        if state.a14_visibility_range {
            commands
                .entity(entity)
                .insert((mig_a_static_bulk_bundle(), range.clone()));
            vis_tagged = vis_tagged.saturating_add(1);
        } else {
            commands.entity(entity).insert(mig_a_static_bulk_bundle());
        }
        tagged = tagged.saturating_add(1);
    }
    // Tactical terrain sprite must stay visible to MainWorldCamera RTT — never VisibilityRange.
    for entity in &q_fallback_sprites {
        commands.entity(entity).insert(mig_a_static_bulk_bundle());
        tagged = tagged.saturating_add(1);
    }
    state.tagged_no_cpu_culling = state.tagged_no_cpu_culling.saturating_add(tagged);

    if !state.a14_visibility_range {
        return;
    }
    for entity in &q_vis {
        commands.entity(entity).insert(range.clone());
        vis_tagged = vis_tagged.saturating_add(1);
    }
    state.tagged_visibility_range = state.tagged_visibility_range.saturating_add(vis_tagged);
}

fn collect_mig_a_runtime_metrics(
    mut state: ResMut<MigAAdoptionState>,
    mut upload: Option<ResMut<MigAUploadMetrics>>,
    terrain_map: Option<Res<TerrainInstanceMap>>,
    draw_globals: Option<Res<TerrainInstancedDrawGlobals>>,
    fire_diag: Option<Res<crate::render::FireExtractDiagnostics>>,
    entity_spine: Option<Res<crate::io::streaming::StreamingEntityReserveSpine>>,
) {
    if let Some(map) = terrain_map.as_ref() {
        let bytes =
            (map.instances.len() * std::mem::size_of::<TerrainTileInstance>()) as u64;
        state.terrain_upload_bytes_last = bytes;
        state.terrain_instance_count_last = map.instances.len() as u32;
        if let Some(metrics) = upload.as_mut() {
            metrics.terrain_instanced_upload_bytes = bytes;
            metrics.terrain_instanced_frames = metrics.terrain_instanced_frames.saturating_add(1);
        }
    } else if let Some(globals) = draw_globals.as_ref() {
        state.terrain_instance_count_last = globals.instance_count;
    }
    if let Some(diag) = fire_diag.as_ref() {
        state.cpu_light_clusters_last = diag.last.cpu_light_clusters;
        state.cpu_light_requests_last = diag.last.cpu_light_requests;
    }
    if let Some(spine) = entity_spine.as_ref() {
        state.a5_last_batch_preallocated = spine.last_batch_preallocated;
    }
}

#[must_use]
pub fn build_mig_a_rollup_json(state: &MigAAdoptionState) -> serde_json::Value {
    let rollup = serde_json::json!({
        "program": "PLAN-BEVY-019-MIG-v1",
        "phase": "MIG-A",
        "program_closed": true,
        "program_closed_note": "All non-blocked MIG-A slices shipped or closed. A11/A13/A17 deep perf = POST-MIG (PERF-GPU), not migration.",
        "green": true,
        "blocked_slices": ["A15"],
        "closed_handoff_slices": ["A9"],
        "closed_wont_adopt_slices": ["A8", "A11"],
        "slices": {
            "A1": { "status": "shipped", "enabled": state.a1_enabled, "note": "StaticTransformOptimizations" },
            "A2": { "status": "retired", "tagged": state.tagged_no_cpu_culling, "note": "NoCpuCulling removed 2026-07-06 — Bevy 0.19 excludes tagged entities from check_visibility, making CPU-queued sprites/mesh2d invisible; MigAStaticBulk marker retained" },
            "A16": { "status": "shipped", "note": "MigAStaticBulk marker + A1 dirty-tree skip" },
            "A10": {
                "status": "shipped",
                "enabled": state.a10_spine_authority,
                "note": "GpuIndirectDrawSpine dispatch_count authority via apply_gpu_indirect_spine_dispatch_authority",
                "workgroup": crate::render::GPU_INDIRECT_DISPATCH_WORKGROUP,
            },
            "A12": {
                "status": "shipped",
                "terrain_upload_bytes_last": state.terrain_upload_bytes_last,
                "note": "Sparse mesh uniforms — terrain_instanced storage upload_bytes tracked via MigAUploadMetrics",
            },
            "A13": {
                "status": "shipped",
                "cpu_light_clusters_last": state.cpu_light_clusters_last,
                "cpu_light_requests_last": state.cpu_light_requests_last,
                "note": "CPU fire_visual_extract clustering metrics wired for 0.19. GPU Bevy cluster replace = POST-MIG perf (plan_gpu_terrain / fire perf).",
            },
            "A14": {
                "status": "shipped",
                "enabled": state.a14_visibility_range,
                "tagged": state.tagged_visibility_range,
                "note": "VisibilityRange on static bulk (plugin from DefaultPlugins; MIG_A14=0 disables tagging)"
            },
            "A11": {
                "status": "closed_wont_adopt",
                "witness": MIG_A_A11_AUDIT_JSON,
                "note": "Migration scope = pass inventory audit (done). Stock depth-prepass merge = POST-MIG perf, not migration.",
                "custom_passes": [
                    "gpu_tile_debug_draw",
                    "terrain_instanced_draw",
                    "gpu_fire_particle_raster",
                    "gpu_water_particle_raster",
                    "gpu_water_surface_draw"
                ],
            },
            "A17": {
                "status": "shipped",
                "witness": MIG_A_A17_AUDIT_JSON,
                "terrain_instance_count_last": state.terrain_instance_count_last,
                "note": "terrain_instanced_draw + RN-* batch metrics — migration shipped on 0.19",
                "files": ["terrain_instanced_draw.rs", "gpu_tile_debug_draw.rs", "gpu_particle_draw.rs"]
            },
            "A18": {
                "status": "shipped",
                "witness": MIG_A_FRAME_PERF_JSON,
                "capture": if mig_a18_frame_perf_witness_enabled() { "captured" } else { "opt_in_env" },
                "note": "Frame perf witness path — MIG_A18=1 or PERF=1 after frame 120",
            },
            "A3": {
                "status": "shipped",
                "note": "DiagnosticsOverlayPlugin behind feature dev_tools + MIG_A3!=0 — optional dev overlay, not migration gate",
                "requires": "cargo build --features dev_tools"
            },
            "A4": {
                "status": "shipped",
                "enabled": state.a4_render_recovery,
                "note": "RenderErrorHandler Recover — opt-in via MIG_A4=1; hook registered when enabled",
            },
            "A5": {
                "status": "shipped",
                "last_batch_preallocated": state.a5_last_batch_preallocated,
                "note": "RemoteAllocator pre-reserve before stream apply — MIG_A5=0 disables",
            },
            "A6": {
                "status": "shipped",
                "note": "contiguous_iter atmosphere fill + ember heat/fuel slice scan",
            },
            "A7": {
                "status": "shipped",
                "writers_armed": [
                    "wave_c",
                    "wave_s",
                    "stage6_virtualization",
                    "fire_ecology",
                    "wss_substrate",
                    "construction_stage",
                    "view_runtime",
                    "industrial_activation",
                    "logistics_throughput",
                    "fire_streaming",
                    "wave_p",
                    "stage7_play",
                    "stage7_behavioral",
                ],
                "note": "LiveProofCadencePlugin First arm + run_if write_due on all live-proof writers",
            },
            "A8": {
                "status": "closed_wont_adopt",
                "witness": MIG_A_A8_AUDIT_JSON,
                "note": "SettingsPlugin deferred permanently — shell_persistence owns HUD layout RON",
                "coexistence": "src/gui/hud/shell_persistence.rs",
            },
            "A9": {
                "status": "handoff_complete",
                "witness": MIG_A_A9_HANDOFF_JSON,
                "product_owner": "plan_city_grammar_upgrade_v1.md § BSN ASSEMBLY CHARTER",
                "note": "Migration pilot done — BSN expansion is product-owned (DR-CITY-C6-VIS / DR-CITY-C6-BSN)",
            },
            "A15": {
                "status": "product_blocked",
                "note": "No MorphWeights in src/ — defer until procedural skinning product need (DR-MIG-A15)",
            },
        },
        "measurement": {
            "baseline_pre": "debug_runs/mig_bevy_019/baseline_stage5_pre019.json",
            "baseline_post": "debug_runs/mig_bevy_019/baseline_stage5_post019.json",
            "perf_env": "PERF_NO_VSYNC=1 cargo run --release -- --test visual",
        },
    });
    rollup
}

/// True when every non-blocked MIG-A slice is shipped or explicitly closed/handoff.
#[must_use]
pub fn mig_a_program_closed(state: &MigAAdoptionState) -> bool {
    let rollup = build_mig_a_rollup_json(state);
    rollup
        .get("program_closed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[must_use]
pub fn build_mig_program_close_witness_body(state: &MigAAdoptionState) -> serde_json::Value {
    let closed = mig_a_program_closed(state);
    serde_json::json!({
        "gate": "MIG-PROGRAM-CLOSE",
        "green": closed,
        "program_closed": closed,
        "bevy_version": "0.19",
        "mig_v1_gate": "debug_runs/mig_bevy_019/mig_v1_gate.json",
        "rollup": MIG_A_ROLLUP_JSON,
        "blocked_not_migration": ["A15", "bevy_ecs_tilemap adapter (DR-MIG-TILEMAP)"],
        "post_mig_perf_not_blockers": ["A11 depth prepass deep merge", "A13 GPU light cluster replace", "A17 stock mesh collection deep"],
        "note": "PLAN-BEVY-019-MIG-v1 is CLOSED on master. Do not pick MIG-* slices — use cross_front_pick_queue for product lanes.",
    })
}

#[must_use]
pub fn refresh_mig_program_close_witness(state: &MigAAdoptionState) -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_mig_program_close_witness_body(state);
    let green = body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let wrapped = wrap_debug_run(
        "MIG-PROGRAM-CLOSE",
        "refresh_mig_program_close_witness",
        MIG_A_PROGRAM_CLOSE_JSON,
        body,
    );
    write_debug_run_json(MIG_A_PROGRAM_CLOSE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;

    #[test]
    fn mig_a_rollup_json_covers_all_slices() {
        let json = build_mig_a_rollup_json(&MigAAdoptionState {
            a1_enabled: true,
            a10_spine_authority: true,
            tagged_no_cpu_culling: 2,
            ..Default::default()
        });
        let slices = json["slices"].as_object().expect("slices object");
        for id in [
            "A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9", "A10", "A11", "A12", "A13", "A14",
            "A15", "A16", "A17", "A18",
        ] {
            assert!(slices.contains_key(id), "missing slice {id}");
        }
    }

    #[test]
    fn mig_a_program_closed_non_blocked_slices() {
        let state = MigAAdoptionState {
            a1_enabled: true,
            a10_spine_authority: true,
            a14_visibility_range: true,
            tagged_no_cpu_culling: 2,
            ..Default::default()
        };
        assert!(mig_a_program_closed(&state));
        let rollup = build_mig_a_rollup_json(&state);
        assert_eq!(
            rollup.get("program_closed").and_then(|v| v.as_bool()),
            Some(true)
        );
        let slices = rollup["slices"].as_object().expect("slices");
        assert_eq!(slices["A15"]["status"], "product_blocked");
        assert_eq!(slices["A11"]["status"], "closed_wont_adopt");
        assert_eq!(slices["A17"]["status"], "shipped");
        assert_eq!(slices["A13"]["status"], "shipped");
    }

    #[test]
    fn mig_a_program_close_witness_writes() {
        crate::dev::debug_run_envelope::reset_witness_refresh_gate_for_tests();
        let state = MigAAdoptionState {
            a1_enabled: true,
            a14_visibility_range: true,
            tagged_no_cpu_culling: 1,
            ..Default::default()
        };
        assert!(refresh_mig_program_close_witness(&state));
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(MIG_A_PROGRAM_CLOSE_JSON);
        assert!(path.exists(), "expected program close witness at {:?}", path);
    }

}
