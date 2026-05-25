//! Demolish tool: pick target → pending → confirm despawns overlapping sites.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

use crate::gui::{MapCameraDesired, SimulationMapViewport};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::economy::activation::BuildingDefinitionRef;
use crate::strategic::{
    BuildSiteTile, CommitConstructionSiteEvent, ConstructionSite, FootprintTiles, LayerType,
    PlannedSite, SiteArchetype, SiteFootprint, SiteId,
};

use super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::pending_construction::{
    PendingBuildBlueprint, PendingConstructionQueue, PendingEntryKind,
};
use super::roads::cursor_world_on_map;

pub fn demolish_pick_queue_system(
    buttons: Res<ButtonInput<MouseButton>>,
    tool: Res<ActiveBuildTool>,
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<crate::render::view_runtime::ViewProjectionAuthority>>,
    desired: Res<MapCameraDesired>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
    mut pending: ResMut<PendingConstructionQueue>,
    mut egui_ctx: EguiContexts,
) {
    if tool.tool != BuildTool::Demolish {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(ctx) = egui_ctx.ctx_mut() else {
        return;
    };
    if ctx.wants_pointer_input() {
        return;
    }
    let Ok(window) = win.single() else {
        return;
    };
    let Some(world) = cursor_world_on_map(
        &window,
        authority.as_deref(),
        desired.as_ref(),
        map_vp.as_ref(),
        params.as_ref(),
    ) else {
        return;
    };
    let origin = BuildSiteTile {
        x: world.x.floor().max(0.0) as u32,
        z: world.z.floor().max(0.0) as u32,
    };
    pending.push(PendingBuildBlueprint {
        kind: PendingEntryKind::Demolish,
        label: format!("demolish:{},{}", origin.x, origin.z),
        archetype: SiteArchetype::MilitaryBase,
        origin,
        footprint: FootprintTiles {
            width: 1,
            depth: 1,
        },
        layer: LayerType::Surface,
        rotation_quarter_turns: 0,
        mirror_x: false,
        approved: false,
        catalog_id: None,
    });
}

/// Despawn construction sites whose footprint covers `tile`. Returns commit events for undo restore.
pub fn execute_demolish_at_tile(
    commands: &mut Commands,
    tile: BuildSiteTile,
    sites: &Query<(
        Entity,
        &ConstructionSite,
        &PlannedSite,
        &SiteFootprint,
        Option<&BuildingDefinitionRef>,
    )>,
) -> (u32, Vec<CommitConstructionSiteEvent>) {
    let target = IVec2::new(tile.x as i32, tile.z as i32);
    let mut n = 0u32;
    let mut restored = Vec::new();
    for (entity, site, planned, footprint, catalog) in sites.iter() {
        if !footprint.tiles.iter().any(|t| *t == target) {
            continue;
        }
        restored.push(CommitConstructionSiteEvent {
            site_id: SiteId(site.site_id),
            owner: site.owner,
            archetype: planned.archetype,
            origin: planned.origin,
            footprint: planned.footprint,
            layer: planned.layer,
            catalog_id: catalog.map(|c| c.catalog_id.clone()),
        });
        commands.entity(entity).despawn();
        n += 1;
    }
    (n, restored)
}
