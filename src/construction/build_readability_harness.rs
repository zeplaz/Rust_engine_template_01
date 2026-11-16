//! BUILD-READABILITY / STUDIO-MASSING-GRAMMAR-BRIDGE — seed catalog pilots for `--test buildread`.
//!
//! Generic loader from `_pilot_catalog.ron` — no warehouse-named Rust branches.
//! Presentation massing may also load the checked-in studio fixture (optional).
//!
//! # `--test buildread` defies VFX (audit confirmations)
//!
//! - **No fire seeds:** `TestScene::seeds_fire_overlay()` is `false` for `BuildReadability`, and
//!   `test_scene_fire_seed_count` / heat+fuel resolve to `0` / `(0,0)` (`engine/test_harness.rs`).
//! - **No fire heat overlay:** `apply_test_scene_defaults` forces `overlays.fire_heat = false`.
//! - **No particle chrome:** the `BuildReadability` arm sets `wx.enabled/overlay/particles = false`
//!   and `gpu.show = false`; `seeds_weather()` / `seeds_atmosphere()` are `false` (no precip/wind seed).
//! - **No volumetric massing:** massing is `voxel_massing::sync_voxel_massing_visual_requests`
//!   (column stacks → `ConstructionVisualRequests::massing_columns` → pixel-world ghost/world-map
//!   frame); no fog/volumetric pass reads it, and `AtmosphereRenderLayers` is pinned all-off (ES-4).
//! - Scope: BuildReadability only — product rain for PixelWorld in other scenes is untouched.

use bevy::prelude::*;

use crate::construction::build_state::{BuildGhostState, BuildPlacementMode};
use crate::construction::build_strip::{BuildStripState, ToolContext};
use crate::construction::build_tool_authority::{ActiveBuildTool, BuildTool};
use crate::construction::building_catalog::{BuildingFamily, BuildingIntentPreview};
use crate::construction::pilot_catalog::PilotCatalog;
use crate::construction::tile_visual::ConstructionTileVisualSettings;
use crate::construction::voxel_massing::{
    StudioMassingOverride, STUDIO_MASSING_CHECKED_IN_FIXTURE_REL,
};
use crate::engine::launch_args::TestScene;
use crate::engine::test_harness::ActiveTestScene;
use crate::gui::{WorldPresentationKind, WorldPresentationMode};
use crate::strategic::BuildSiteTile;

/// Arm the civil tool + ghost from one generic catalog pilot (no archetype-named branches).
///
/// Shared by `--test buildread` and the `--test playquad` NE quadrant seed. Presentation/preview
/// state only — placement still goes through the single construction execute funnel.
pub(crate) fn arm_catalog_pilot_ghost(
    pilot: &crate::construction::pilot_catalog::ResolvedPilotEntry,
    origin: BuildSiteTile,
    rotation_quarter_turns: u8,
    strip: &mut BuildStripState,
    tool: &mut ActiveBuildTool,
    ghost: &mut BuildGhostState,
    settings: &mut ConstructionTileVisualSettings,
) {
    strip.active = ToolContext::Industry;
    tool.tool = BuildTool::Building(
        crate::construction::build_tool_authority::BuildingArchetypeId::Factory,
    );
    tool.building_intent = Some(BuildingIntentPreview {
        label: pilot.label.clone(),
        family: BuildingFamily::Industry,
        footprint: pilot.footprint.clone(),
        construction_cost: 0,
        construction_time_ticks: 0,
        power_consumption: 0.0,
        workers_required: 0,
        unit_kinds: Vec::new(),
        apartment_form: None,
        catalog_id: Some(pilot.catalog_id.clone()),
        arch_dna_preset_id: pilot.arch_dna_preset.clone(),
    });
    ghost.origin = Some(origin);
    ghost.footprint = crate::strategic::FootprintTiles {
        width: pilot.footprint.width.max(1),
        depth: pilot.footprint.depth.max(1),
    };
    ghost.placement_mode = BuildPlacementMode::Adjust;
    ghost.rotation_quarter_turns = rotation_quarter_turns;
    settings.show_occupation_tiles = true;
}

/// One-shot: when [`TestScene::BuildReadability`] is active, arm civil tool + non-rect (or first)
/// catalog pilot footprint + ghost origin, PixelWorld mode, and optional studio massing fixture.
pub fn seed_build_readability_harness_once(
    mut done: Local<bool>,
    scene: Option<Res<ActiveTestScene>>,
    mut strip: ResMut<BuildStripState>,
    mut tool: ResMut<ActiveBuildTool>,
    mut ghost: ResMut<BuildGhostState>,
    mut settings: ResMut<ConstructionTileVisualSettings>,
    mut world_mode: ResMut<WorldPresentationMode>,
    mut studio: ResMut<StudioMassingOverride>,
) {
    if *done {
        return;
    }
    let Some(scene) = scene else {
        return;
    };
    if scene.0 != TestScene::BuildReadability {
        return;
    }
    *done = true;

    let catalog = PilotCatalog::load_from_disk();
    // Prefer non-rect shape QA (L/T/O); else first grammar pilot — never hardcode warehouse id.
    let pilot = catalog
        .non_rect_shape_pilot()
        .or_else(|| catalog.first_grammar_pilot())
        .or_else(|| catalog.pilots.first());
    let Some(pilot) = pilot else {
        bevy::log::warn!("buildread: pilot catalog empty — nothing to seed");
        return;
    };

    arm_catalog_pilot_ghost(
        pilot,
        BuildSiteTile { x: 8, z: 8 },
        1, // prove rotation reads on non-rect
        &mut strip,
        &mut tool,
        &mut ghost,
        &mut settings,
    );
    world_mode.kind = WorldPresentationKind::PixelWorld3D;

    // Optional silhouette: checked-in grammar/pilot massing fixture (presentation only).
    studio.load_from_path(STUDIO_MASSING_CHECKED_IN_FIXTURE_REL);

    bevy::log::info!(
        "buildread: seeded pilot={} catalog_id={} footprint={}x{} non_rect={}",
        pilot.id,
        pilot.catalog_id,
        pilot.footprint.width,
        pilot.footprint.depth,
        pilot.footprint.is_non_rectangular()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_non_rect_or_grammar_pilot() {
        let catalog = PilotCatalog::load_from_disk();
        assert!(
            catalog.non_rect_shape_pilot().is_some() || catalog.first_grammar_pilot().is_some(),
            "need at least one non-rect shape QA or grammar pilot"
        );
        if let Some(p) = catalog.non_rect_shape_pilot() {
            assert!(p.footprint.is_non_rectangular());
            assert!(p.footprint.occupied_count() >= 2);
        }
    }
}
