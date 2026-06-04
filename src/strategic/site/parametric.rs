//! Parametric placement payloads — strategic DTO (raster lives in `crate::construction`).

use bevy::prelude::IVec2;

use crate::strategic::build_order::BuildSiteTile;

/// Committed parametric placement snapshot (carried on [`super::events::CommitConstructionSiteEvent`]).
#[derive(Clone, Debug, PartialEq)]
pub struct CommittedPlacementSnapshot {
    pub origin: BuildSiteTile,
    pub scale_factor: f32,
    pub effective_scale: f32,
    pub rotation_quarter_turns: u8,
    pub mirror_x: bool,
    pub weights: Vec<(IVec2, f32)>,
}

impl CommittedPlacementSnapshot {
    #[must_use]
    pub fn occupied_mass(&self) -> f32 {
        self.weights.iter().map(|(_, w)| *w).sum()
    }
}

/// Live-proof / rollup: commit path carries scale + sparse weights on the site entity.
#[must_use]
pub fn commit_carries_scale_and_weights_witness_green() -> bool {
    commit_roundtrip_self_check().is_ok()
}

fn commit_roundtrip_self_check() -> Result<(), &'static str> {
    use bevy::app::App;
    use bevy::prelude::{MinimalPlugins, Update};
    use crate::strategic::spatial_network::LayerType;

    use super::components::{BuildingScaleParams, SiteWeightedFootprint};
    use super::events::CommitConstructionSiteEvent;
    use super::resources::{FootprintTiles, SiteConstructionBook, SiteId, SiteIdIssuer};
    use super::systems::commit_construction_site_system;
    use super::tile_occupation::TileOccupationBook;
    use super::components::SiteArchetype;

    let origin = BuildSiteTile { x: 3, z: 5 };
    let placement = CommittedPlacementSnapshot {
        origin,
        scale_factor: 1.0,
        effective_scale: 1.0,
        rotation_quarter_turns: 0,
        mirror_x: false,
        weights: vec![(IVec2::new(3, 5), 1.0)],
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<SiteConstructionBook>()
        .init_resource::<SiteIdIssuer>()
        .init_resource::<TileOccupationBook>()
        .add_message::<CommitConstructionSiteEvent>()
        .add_systems(Update, commit_construction_site_system);

    let owner = app.world_mut().spawn_empty().id();
    app.world_mut().write_message(CommitConstructionSiteEvent {
        site_id: SiteId::UNASSIGNED,
        owner,
        archetype: SiteArchetype::Factory,
        origin,
        footprint: FootprintTiles {
            width: 1,
            depth: 1,
        },
        layer: LayerType::Surface,
        catalog_id: Some("witness_parametric".into()),
        placement: Some(placement),
    });
    app.update();

    let world = app.world_mut();
    let mut q = world.query::<(&SiteWeightedFootprint, &BuildingScaleParams)>();
    let Some((wf, scale)) = q.iter_mut(world).next() else {
        return Err("missing_weighted_components");
    };
    if wf.weights.is_empty() {
        return Err("empty_weights");
    }
    if (scale.effective_scale - 1.0).abs() > 0.2 {
        return Err("effective_scale");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_roundtrip_carries_scale() {
        commit_roundtrip_self_check().expect("commit_roundtrip");
    }
}
