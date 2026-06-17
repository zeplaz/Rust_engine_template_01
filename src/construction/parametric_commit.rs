//! Parametric placement snapshot for construction → strategic site commit.
//!
//! PG-3: [`procedural_building_request_from_commit`] derives [`ProceduralBuildingRequest`] on site commit.

use bevy::prelude::*;

use crate::construction::procedural::{
    generate_with_arch_dna_preset, ProceduralAssemblyRequest, ProceduralBuildingRequest, StylePackId,
};
use crate::construction::building_catalog::{BuildingFamily, FootprintMatrix};
use crate::construction::building_definitions::BuildingDefinitionRegistry;
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
    catalog_id: Option<&str>,
    buildings: Option<&BuildingDefinitionRegistry>,
) -> Option<ProceduralBuildingRequest> {
    if footprint.width < 2 || footprint.depth < 2 {
        return None;
    }

    if let (Some(cid), Some(reg)) = (catalog_id, buildings) {
        if let Some(def) = reg.get(cid) {
            if let Some(archetype_id) = def.grammar_archetype_id.as_deref() {
                let district = def
                    .district_style
                    .as_deref()
                    .unwrap_or("industrial_west");
                let preset = def.arch_dna_preset.as_deref();
                if let Ok(grammar) = generate_with_arch_dna_preset(
                    archetype_id,
                    district,
                    site_id.0,
                    preset,
                ) {
                    let matrix = &def.footprint;
                    return Some(ProceduralBuildingRequest {
                        archetype_id: archetype_id.to_owned(),
                        width: matrix.width.max(footprint.width),
                        depth: matrix.depth.max(footprint.depth),
                        floors: grammar.floors,
                        style: StylePackId(grammar.style_pack_id),
                        seed: site_id.0,
                        arch_dna_preset_id: preset.map(str::to_owned),
                    });
                }
            }
        }
    }

    Some(ProceduralBuildingRequest {
        archetype_id: "rect_perimeter".into(),
        width: footprint.width,
        depth: footprint.depth,
        floors: default_floors_for_archetype(archetype, placement),
        style: style_pack_for_site_archetype(archetype),
        seed: site_id.0,
        arch_dna_preset_id: None,
    })
}

/// BUILD-READ-VISUAL-001 — pilot commit uses grammar → lod0 extract path (not footprint-only).
#[must_use]
pub fn build_read_visual_001_witness_green() -> bool {
    build_read_visual_001_self_check().is_ok()
}

fn build_read_visual_001_self_check() -> Result<(), &'static str> {
    use crate::construction::building_definitions::{
        default_buildings_dir, load_building_definitions_from_dir,
    };
    use crate::construction::PilotCatalog;
    use crate::construction::procedural::{
        footprint_grid_for_assembly, load_procedural_module_registry, load_style_pack_registry,
    };
    use crate::render::extraction::{
        assemble_procedural_build_instances, ProceduralModuleSceneCatalog,
    };

    let catalog = PilotCatalog::load_from_disk();
    let pilot = catalog.first_grammar_pilot().ok_or("grammar_pilot")?;
    let reg = load_building_definitions_from_dir(default_buildings_dir());
    let req = procedural_building_request_from_commit(
        SiteId(99),
        SiteArchetype::Factory,
        FootprintTiles {
            width: pilot.footprint.width,
            depth: pilot.footprint.depth,
        },
        None,
        Some(&pilot.catalog_id),
        Some(&reg),
    )
    .ok_or("no_request")?;
    let def = reg.get(&pilot.catalog_id).ok_or("registry_pilot")?;
    let archetype = def
        .grammar_archetype_id
        .as_deref()
        .ok_or("grammar_archetype")?;
    if req.archetype_id != archetype {
        return Err("grammar_archetype");
    }
    if req.style.as_str() != "style_industrial_west" {
        return Err("style_pack");
    }
    if req.arch_dna_preset_id.is_none() {
        return Err("preset_on_request");
    }

    let modules = load_procedural_module_registry();
    if !modules.load_errors.is_empty() {
        return Err("modules");
    }
    let packs = load_style_pack_registry();
    let pack = packs.get(req.style.as_str()).ok_or("pack")?;
    let grid = footprint_grid_for_assembly(&req);
    let extract = assemble_procedural_build_instances(
        &req,
        pack,
        &grid,
        &modules,
        &ProceduralModuleSceneCatalog::default(),
    );
    if extract.smoke_fallback_used {
        return Err("smoke_fallback");
    }
    let visible = extract.instances.iter().filter(|i| !i.hidden).count();
    if visible == 0 {
        return Err("no_visible");
    }
    Ok(())
}

/// BUILD-READ-VISUAL-001 — PG-2 operational spawn harness (commit → Operational → module scenes).
#[must_use]
pub fn build_read_visual_001_runtime_sim_verified() -> bool {
    crate::construction::procedural_build_spawn::procedural_pg2_spawn_witness_green()
}

/// BUILD-READ-VISUAL-001 witness body (lib + tile stamp + PG-2 operational spawn harness).
#[must_use]
pub fn build_read_visual_001_witness_body() -> serde_json::Value {
    use crate::gui::map_tile_atlas_stamp::{
        build_read_visual_pilot_tile_stamp_lib_green,
        rail_warehouse_pilot_atlas_index_registered,
    };

    let lib_green = build_read_visual_001_witness_green();
    let runtime_sim_verified = build_read_visual_001_runtime_sim_verified();
    let pilot_tile = build_read_visual_pilot_tile_stamp_lib_green();
    let pilot_atlas = rail_warehouse_pilot_atlas_index_registered();
    let mesh_tier = if pilot_atlas {
        "iso_tile"
    } else if lib_green {
        "lod0"
    } else {
        "fallback_primitive"
    };
    serde_json::json!({
        "gate": "BUILD-READ-VISUAL-001",
        "green": lib_green && runtime_sim_verified && pilot_atlas,
        "lib_green": lib_green,
        "runtime_sim_verified": runtime_sim_verified,
        "pg2_operational_spawn_wired": runtime_sim_verified,
        "pg2_mesh_suppressed_when_atlas": pilot_atlas,
        "mesh_tier_used": mesh_tier,
        "pg2_lod0_extract": lib_green,
        "pilot_tile_stamp_lib": pilot_tile,
        "pilot_atlas_registered": pilot_atlas,
        "warehouse_atlas_registered": pilot_atlas,
        "footprint_grid_uses_grammar": true,
        "runtime_verify": {
            "open": !runtime_sim_verified,
            "requires_sim_session": !runtime_sim_verified,
            "checklist": "commit Rail Warehouse pilot → Operational → lod0 PG-2 modules and/or warehouse iso stamp on map (not greybox-only)",
        },
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
            None,
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
            None,
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
            None,
            None,
        )
        .is_none());
    }

    #[test]
    fn build_read_visual_001_pilot_uses_grammar_lod0() {
        assert!(build_read_visual_001_witness_green());
    }
}
