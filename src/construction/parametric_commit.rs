//! Parametric placement snapshot for construction → strategic site commit.
//!
//! PG-3: [`procedural_building_request_from_commit`] derives [`ProceduralBuildingRequest`] on site commit.

use bevy::prelude::*;

use crate::construction::procedural::{ProceduralAssemblyRequest, ProceduralBuildingRequest, StylePackId};
use crate::construction::building_catalog::{BuildingFamily, FootprintMatrix};
use crate::construction::placement_scaling::{clamp_scale_factor, default_scale_factor_for_family};
use crate::strategic::{
    BuildSiteTile, CommittedPlacementSnapshot, CommitConstructionSiteEvent, FootprintTiles,
    LayerType, ProceduralBuildingSpec, SiteArchetype, SiteConstructionBook, SiteId, SiteIdIssuer,
};
use crate::strategic::commit_construction_site_system;

#[must_use]
pub fn parametric_placement_snapshot(
    footprint: &FootprintMatrix,
    family: BuildingFamily,
    origin: BuildSiteTile,
    rotation_quarter_turns: u8,
    mirror_x: bool,
    scale_factor: Option<f32>,
) -> CommittedPlacementSnapshot {
    let scale_factor =
        clamp_scale_factor(scale_factor.unwrap_or_else(|| default_scale_factor_for_family(family)));
    let mut weights = Vec::new();
    for z in 0..footprint.depth {
        for x in 0..footprint.width {
            let idx = (z * footprint.width + x) as usize;
            if footprint.cells.get(idx).copied().unwrap_or(0) == 0 {
                continue;
            }
            let (tx, tz) = transform_footprint_cell(
                x,
                z,
                footprint.width,
                footprint.depth,
                rotation_quarter_turns,
                mirror_x,
            );
            weights.push((
                IVec2::new(
                    origin.x as i32 + i32::try_from(tx).unwrap_or(0),
                    origin.z as i32 + i32::try_from(tz).unwrap_or(0),
                ),
                1.0,
            ));
        }
    }
    CommittedPlacementSnapshot {
        origin,
        scale_factor,
        effective_scale: scale_factor,
        rotation_quarter_turns,
        mirror_x,
        weights,
    }
}

fn transform_footprint_cell(
    x: u32,
    z: u32,
    width: u32,
    depth: u32,
    quarter_turns: u8,
    mirror_x: bool,
) -> (u32, u32) {
    let mut tx = x;
    let tz = z;
    if mirror_x {
        tx = width.saturating_sub(1).saturating_sub(tx);
    }
    match quarter_turns % 4 {
        0 => (tx, tz),
        1 => (
            depth.saturating_sub(1).saturating_sub(tz),
            tx,
        ),
        2 => (
            width.saturating_sub(1).saturating_sub(tx),
            depth.saturating_sub(1).saturating_sub(tz),
        ),
        _ => (
            tz,
            width.saturating_sub(1).saturating_sub(tx),
        ),
    }
}

/// Default StylePack for a committed site archetype (PG-3 pilot mapping).
#[must_use]
pub fn style_pack_for_site_archetype(archetype: SiteArchetype) -> StylePackId {
    StylePackId(match archetype {
        SiteArchetype::CivilHousing => "style_victorian",
        SiteArchetype::Factory
        | SiteArchetype::PowerPlant
        | SiteArchetype::FuelDepot
        | SiteArchetype::WaterPlant => "style_industrial_west",
        SiteArchetype::RailDepot => "style_industrial_soviet",
        SiteArchetype::MilitaryBase
        | SiteArchetype::BunkerComplex
        | SiteArchetype::RadarSite
        | SiteArchetype::SensorPost
        | SiteArchetype::TrenchLine => "style_military",
    }
    .into())
}

fn default_floors_for_archetype(
    archetype: SiteArchetype,
    placement: Option<&CommittedPlacementSnapshot>,
) -> u32 {
    let base = match archetype {
        SiteArchetype::CivilHousing => 2,
        SiteArchetype::Factory
        | SiteArchetype::PowerPlant
        | SiteArchetype::FuelDepot
        | SiteArchetype::WaterPlant
        | SiteArchetype::RailDepot => 1,
        _ => 2,
    };
    let scale_bonus = placement
        .map(|p| ((p.effective_scale - 1.0).max(0.0) * 2.0).round() as u32)
        .unwrap_or(0);
    (base + scale_bonus).clamp(1, 6)
}

/// Derive PG-2 assembly input from a committed site footprint (min 2×2 perimeter grammar).
#[must_use]
pub fn procedural_building_request_from_commit(
    site_id: SiteId,
    archetype: SiteArchetype,
    footprint: FootprintTiles,
    placement: Option<&CommittedPlacementSnapshot>,
) -> Option<ProceduralBuildingRequest> {
    if footprint.width < 2 || footprint.depth < 2 {
        return None;
    }
    Some(ProceduralBuildingRequest {
        archetype_id: "rect_perimeter".into(),
        width: footprint.width,
        depth: footprint.depth,
        floors: default_floors_for_archetype(archetype, placement),
        style: style_pack_for_site_archetype(archetype),
        seed: site_id.0,
    })
}

/// Mirror the highest-seed committed site into the PG-2 demo resource until multi-site extract lands.
pub fn sync_procedural_assembly_request_from_sites(
    sites: Query<&ProceduralBuildingSpec>,
    mut request: ResMut<ProceduralAssemblyRequest>,
) {
    let Some(spec) = sites.iter().max_by_key(|s| s.0.seed) else {
        return;
    };
    request.0 = spec.0.clone();
}

/// Live-proof / rollup: Portland-style commit attaches [`ProceduralBuildingSpec`].
#[must_use]
pub fn construction_procedural_build_001_witness_green() -> bool {
    commit_procedural_spec_self_check().is_ok()
}

fn commit_procedural_spec_self_check() -> Result<(), &'static str> {
    use bevy::app::App;
    use bevy::prelude::{MinimalPlugins, Update};

    let origin = BuildSiteTile { x: 40, z: 40 };
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<SiteConstructionBook>()
        .init_resource::<SiteIdIssuer>()
        .add_message::<CommitConstructionSiteEvent>()
        .add_systems(Update, commit_construction_site_system);

    let owner = app.world_mut().spawn_empty().id();
    app.world_mut().write_message(CommitConstructionSiteEvent {
        site_id: SiteId::UNASSIGNED,
        owner,
        archetype: SiteArchetype::Factory,
        origin,
        footprint: FootprintTiles {
            width: 3,
            depth: 2,
        },
        layer: LayerType::Surface,
        catalog_id: Some("concrete_aggregate_mine".into()),
        placement: None,
    });
    app.update();

    let world = app.world_mut();
    let mut q = world.query::<&ProceduralBuildingSpec>();
    let spec = q.iter_mut(world).next().ok_or("missing_procedural_spec")?;
    if spec.0.width != 3 || spec.0.depth != 2 {
        return Err("footprint_dims");
    }
    if spec.0.style.as_str() != "style_industrial_west" {
        return Err("style_pack");
    }
    if spec.0.archetype_id != "rect_perimeter" {
        return Err("archetype_id");
    }
    if spec.0.seed == 0 {
        return Err("seed");
    }
    Ok(())
}

#[cfg(test)]
mod pg3_tests {
    use super::*;

    #[test]
    fn procedural_request_from_portland_footprint() {
        let req = procedural_building_request_from_commit(
            SiteId(42),
            SiteArchetype::Factory,
            FootprintTiles {
                width: 3,
                depth: 2,
            },
            None,
        )
        .expect("request");
        assert_eq!(req.width, 3);
        assert_eq!(req.depth, 2);
        assert_eq!(req.style.as_str(), "style_industrial_west");
        assert_eq!(req.seed, 42);
    }

    #[test]
    fn commit_attaches_procedural_building_spec() {
        commit_procedural_spec_self_check().expect("commit spec");
    }

    #[test]
    fn housing_maps_to_victorian_style() {
        let req = procedural_building_request_from_commit(
            SiteId(1),
            SiteArchetype::CivilHousing,
            FootprintTiles {
                width: 4,
                depth: 3,
            },
            None,
        )
        .expect("request");
        assert_eq!(req.style.as_str(), "style_victorian");
        assert_eq!(req.floors, 2);
    }

    #[test]
    fn sub_2x2_footprint_skips_spec() {
        assert!(procedural_building_request_from_commit(
            SiteId(1),
            SiteArchetype::Factory,
            FootprintTiles {
                width: 1,
                depth: 1,
            },
            None,
        )
        .is_none());
    }
}
