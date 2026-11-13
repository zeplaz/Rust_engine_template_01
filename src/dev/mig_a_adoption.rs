//! Bevy 0.19 post-migration adoption — Phase 4 **MIG-A** slices.
//!
//! Runtime adoption plugin (static bulk tags, spine metrics) — **CB-MIG-001** runtime half.
//! Spawn markers in [`crate::render::mig_a_static`].

use bevy::camera::visibility::VisibilityRange;
use bevy::prelude::*;
use bevy::transform::StaticTransformOptimizations;

use crate::construction::ProceduralBuildModuleChild;
use crate::render::{
    mig_a1_static_transform_optimizations_enabled,
    mig_a10_spine_dispatch_authority, mig_a_static_bulk_bundle, mig_a_tactical_visibility_range,
    MigAStaticBulk, TerrainInstancedDrawGlobals, TerrainInstanceMap, TerrainTileInstance,
    TileWorldFallbackSprite,
};


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








