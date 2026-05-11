//! Construction / site planning — uses [`evaluate_site_placement_stubs`](crate::strategic::site::evaluate_site_placement_stubs); no bypass.

use bevy::prelude::*;

use crate::strategic::{
    BuildSiteTile, CommitConstructionSiteEvent, FootprintTiles, LayerType, SiteArchetype, SiteId,
    evaluate_site_placement_stubs,
};

#[derive(Resource, Clone, Copy, Debug)]
pub struct ConstructionAiOwner(pub Entity);

#[derive(Resource, Clone, Debug)]
pub struct ConstructionAiConfig {
    /// When true, periodically commits a probe [`CommitConstructionSiteEvent`] if validation passes.
    pub enabled: bool,
    pub frames_between_probes: u32,
    pub origin_tile: BuildSiteTile,
    pub archetype: SiteArchetype,
}

impl Default for ConstructionAiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            frames_between_probes: 480,
            origin_tile: BuildSiteTile { x: 4, z: 4 },
            archetype: SiteArchetype::FuelDepot,
        }
    }
}

#[derive(Resource, Default)]
struct ConstructionAiProbeState {
    frame: u32,
}

fn construction_ai_shared_validation_probe_system(
    cfg: Res<ConstructionAiConfig>,
    owner: Res<ConstructionAiOwner>,
    mut state: ResMut<ConstructionAiProbeState>,
    mut writer: MessageWriter<CommitConstructionSiteEvent>,
) {
    if !cfg.enabled {
        return;
    }
    state.frame = state.frame.wrapping_add(1);
    if state.frame % cfg.frames_between_probes.max(1) != 0 {
        return;
    }
    let report = evaluate_site_placement_stubs();
    if !report.valid {
        return;
    }
    writer.write(CommitConstructionSiteEvent {
        site_id: SiteId::UNASSIGNED,
        owner: owner.0,
        archetype: cfg.archetype,
        origin: cfg.origin_tile,
        footprint: FootprintTiles { width: 1, depth: 1 },
        layer: LayerType::Surface,
    });
}

pub struct ConstructionAiPlugin;

impl Plugin for ConstructionAiPlugin {
    fn build(&self, app: &mut App) {
        let owner = app.world_mut().spawn_empty().id();
        app.insert_resource(ConstructionAiOwner(owner))
            .init_resource::<ConstructionAiConfig>()
            .init_resource::<ConstructionAiProbeState>()
            .add_systems(Update, construction_ai_shared_validation_probe_system);
    }
}
