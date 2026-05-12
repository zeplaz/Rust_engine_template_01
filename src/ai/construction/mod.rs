//! Construction / site planning — uses [`evaluate_site_placement_at_world_tile`](crate::strategic::evaluate_site_placement_at_world_tile); no bypass.

use bevy::prelude::*;

use crate::strategic::{
    evaluate_site_placement_at_world_tile, BuildSiteTile, CommitConstructionSiteEvent, FootprintTiles,
    LayerType, SiteArchetype, SiteId, StrategicRasterConfig,
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
    /// Inclusive tile radius around [`Self::origin_tile`] evaluated each probe.
    pub search_radius: i32,
}

impl Default for ConstructionAiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            frames_between_probes: 480,
            origin_tile: BuildSiteTile { x: 4, z: 4 },
            archetype: SiteArchetype::FuelDepot,
            search_radius: 2,
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
    overlay: Query<&crate::strategic::ChunkStrategicOverlay>,
    raster: Option<Res<StrategicRasterConfig>>,
) {
    if !cfg.enabled {
        return;
    }
    state.frame = state.frame.wrapping_add(1);
    if state.frame % cfg.frames_between_probes.max(1) != 0 {
        return;
    }

    let fp = FootprintTiles {
        width: 1,
        depth: 1,
    };
    let r = cfg.search_radius.max(0);
    let mut best: Option<(BuildSiteTile, f32)> = None;

    for dz in -r..=r {
        for dx in -r..=r {
            let xi = cfg.origin_tile.x as i32 + dx;
            let zi = cfg.origin_tile.z as i32 + dz;
            if xi < 0 || zi < 0 {
                continue;
            }
            let origin = BuildSiteTile {
                x: xi as u32,
                z: zi as u32,
            };
            let report =
                evaluate_site_placement_at_world_tile(origin, fp, raster.as_deref(), &overlay);
            if !report.allows_commit {
                continue;
            }
            let score = report.terrain_score + report.logistics_score + report.strategic_score;
            if best.as_ref().map_or(true, |(_, s)| score > *s) {
                best = Some((origin, score));
            }
        }
    }

    let Some((origin, _)) = best else {
        return;
    };

    writer.write(CommitConstructionSiteEvent {
        site_id: SiteId::UNASSIGNED,
        owner: owner.0,
        archetype: cfg.archetype,
        origin,
        footprint: fp,
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
