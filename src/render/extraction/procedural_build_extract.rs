//! PG-2 procedural build assembly extract — grid → StylePack slot → lod0 GLB.

use bevy::prelude::*;
use bevy::world_serialization::WorldAsset;

use crate::construction::procedural::{
    footprint_grid_for_assembly, FootprintCell, FootprintGrid, FootprintToken,
    ProceduralBuildingRequest, ProceduralModuleRegistry, StylePack, StylePackRegistry,
    StylePackSlotKey,
};
use crate::gui::RepresentationResult;
use crate::render::extraction::{
    scene_for_module, ProceduralModuleSceneCatalog, ProceduralModuleVisualPolicy,
};

/// One resolved module placement from PG-2 assembly.
#[derive(Debug, Clone)]
pub struct ProceduralBuildInstance {
    pub module_id: String,
    pub slot_key: String,
    pub grid_x: u32,
    pub grid_y: u32,
    pub floor: u32,
    pub scene: Option<Handle<WorldAsset>>,
    pub hidden: bool,
}

/// Latest PG-2 assembly extract output (read-only for render consumers).
#[derive(Resource, Debug, Default)]
pub struct ProceduralBuildExtract {
    pub instances: Vec<ProceduralBuildInstance>,
    pub module_ids_used: Vec<String>,
    pub smoke_fallback_used: bool,
    pub footprint_cells: u32,
    pub style_pack_id: String,
    pub pg2_wired: bool,
}

#[must_use]
fn slot_key_for_token(token: FootprintToken) -> Option<StylePackSlotKey> {
    match token {
        FootprintToken::Wall => Some(StylePackSlotKey::Wall1u),
        FootprintToken::Door => Some(StylePackSlotKey::DoorDefault),
        FootprintToken::Corner => Some(StylePackSlotKey::CornerOuter),
        FootprintToken::Roof => Some(StylePackSlotKey::RoofDefault),
        FootprintToken::Yard => None,
    }
}

/// Pure assembly path — used by extract system and lib witness/tests.
#[must_use]
pub fn assemble_procedural_build_instances(
    _request: &ProceduralBuildingRequest,
    style_pack: &StylePack,
    grid: &FootprintGrid,
    registry: &ProceduralModuleRegistry,
    catalog: &ProceduralModuleSceneCatalog,
) -> ProceduralBuildExtract {
    let mut extract = ProceduralBuildExtract {
        style_pack_id: style_pack.id.as_str().to_owned(),
        footprint_cells: grid.wdc_cell_count(),
        pg2_wired: true,
        ..Default::default()
    };

    for cell in grid.facade_cells() {
        let Some(slot_key) = slot_key_for_token(cell.token) else {
            continue;
        };
        let slot_name = slot_key.ron_key();
        let Some(raw_module_id) = style_pack.resolve_slot(slot_key) else {
            push_hidden_instance(&mut extract, cell, slot_name, String::new());
            continue;
        };

        let Some(entry) = registry.resolve_module_id(raw_module_id) else {
            push_hidden_instance(&mut extract, cell, slot_name, raw_module_id.to_owned());
            continue;
        };

        if entry.development_tier.is_smoke() || entry.batch_id.starts_with("kit_greybox") {
            extract.smoke_fallback_used = true;
            push_hidden_instance(&mut extract, cell, slot_name, entry.module_id.clone());
            continue;
        }

        let scene = scene_for_module(catalog, registry, &entry.module_id).cloned();
        if !extract.module_ids_used.contains(&entry.module_id) {
            extract.module_ids_used.push(entry.module_id.clone());
        }
        extract.instances.push(ProceduralBuildInstance {
            module_id: entry.module_id.clone(),
            slot_key: slot_name.to_owned(),
            grid_x: cell.x,
            grid_y: cell.y,
            floor: cell.floor,
            scene,
            hidden: false,
        });
    }

    extract
}

fn push_hidden_instance(
    extract: &mut ProceduralBuildExtract,
    cell: &FootprintCell,
    slot_key: &str,
    module_id: String,
) {
    extract.instances.push(ProceduralBuildInstance {
        module_id,
        slot_key: slot_key.to_owned(),
        grid_x: cell.x,
        grid_y: cell.y,
        floor: cell.floor,
        scene: None,
        hidden: true,
    });
}

pub fn extract_procedural_build_assembly(
    request: Res<crate::construction::procedural::ProceduralAssemblyRequest>,
    style_packs: Res<StylePackRegistry>,
    registry: Res<ProceduralModuleRegistry>,
    catalog: Res<ProceduralModuleSceneCatalog>,
    policy: Res<RepresentationResult>,
    visual: Res<ProceduralModuleVisualPolicy>,
    mut extract: ResMut<ProceduralBuildExtract>,
) {
    let _perf = crate::render::PerfScope::new("upd_repr_proc_extract");
    if !policy.procedural_module_meshes || !visual.meshes_active {
        extract.instances.clear();
        extract.module_ids_used.clear();
        extract.pg2_wired = true;
        extract.smoke_fallback_used = false;
        return;
    }

    let req = &request.0;
    let Some(pack) = style_packs.get(req.style.as_str()) else {
        *extract = ProceduralBuildExtract {
            style_pack_id: req.style.as_str().to_owned(),
            pg2_wired: true,
            ..Default::default()
        };
        return;
    };

    let grid = footprint_grid_for_assembly(req);
    *extract = assemble_procedural_build_instances(req, pack, &grid, &registry, &catalog);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::procedural::{
        load_procedural_module_registry, load_style_pack_registry, StylePackId,
    };

    fn victorian_request() -> ProceduralBuildingRequest {
        ProceduralBuildingRequest {
            archetype_id: "rect_perimeter".into(),
            width: 4,
            depth: 2,
            floors: 2,
            style: StylePackId("style_victorian".into()),
            seed: 1,
            arch_dna_preset_id: None,
        }
    }

    #[test]
    fn procedural_build_extract_resolves_lod0_glb() {
        let registry = load_procedural_module_registry();
        assert!(registry.load_errors.is_empty(), "{:?}", registry.load_errors);
        let packs = load_style_pack_registry();
        assert!(packs.load_errors.is_empty(), "{:?}", packs.load_errors);
        let pack = packs.get("style_victorian").expect("style_victorian");
        let grid = FootprintGrid::from_request(&victorian_request());
        let catalog = ProceduralModuleSceneCatalog::default();
        let extract = assemble_procedural_build_instances(
            &victorian_request(),
            pack,
            &grid,
            &registry,
            &catalog,
        );
        assert!(extract.footprint_cells > 0);
        assert!(!extract.smoke_fallback_used);
        let visible: Vec<_> = extract
            .instances
            .iter()
            .filter(|i| !i.hidden)
            .collect();
        assert!(!visible.is_empty(), "expected visible lod0 instances");
        for inst in &visible {
            let entry = registry
                .resolve_module_id(&inst.module_id)
                .expect("lod0 module");
            assert_eq!(entry.development_tier, crate::construction::procedural::DevelopmentTier::Lod0);
            assert!(entry.job_id.contains("_lod0_"));
        }
    }

    #[test]
    fn procedural_build_extract_skips_smoke_row() {
        let registry = load_procedural_module_registry();
        assert!(registry.load_errors.is_empty(), "{:?}", registry.load_errors);
        assert!(registry.resolve_module_id("corner_brick_outer").is_none());
        let mut pack = load_style_pack_registry()
            .get("style_victorian")
            .unwrap()
            .clone();
        pack.slots
            .insert("corner_outer".into(), "corner_brick_outer".into());
        let grid = FootprintGrid::from_request(&victorian_request());
        let extract = assemble_procedural_build_instances(
            &victorian_request(),
            &pack,
            &grid,
            &registry,
            &ProceduralModuleSceneCatalog::default(),
        );
        assert!(!extract.smoke_fallback_used);
        assert!(
            !extract
                .module_ids_used
                .contains(&"corner_brick_outer".to_owned()),
            "smoke-only module must not resolve for assembly"
        );
        assert!(
            extract
                .instances
                .iter()
                .any(|i| i.hidden && i.module_id == "corner_brick_outer"),
            "smoke slot must be hidden"
        );
    }

    #[test]
    fn procedural_build_extract_hide_slot_when_module_missing() {
        let registry = load_procedural_module_registry();
        let mut pack = load_style_pack_registry()
            .get("style_victorian")
            .unwrap()
            .clone();
        pack.slots.insert("wall_1u".into(), "missing_module_xyz".into());
        let grid = FootprintGrid::from_request(&victorian_request());
        let extract = assemble_procedural_build_instances(
            &victorian_request(),
            &pack,
            &grid,
            &registry,
            &ProceduralModuleSceneCatalog::default(),
        );
        assert!(
            extract
                .instances
                .iter()
                .any(|i| i.hidden && i.module_id == "missing_module_xyz"),
            "missing module must hide slot"
        );
        assert!(!extract.smoke_fallback_used);
    }

    #[test]
    fn style_pack_victorian_vs_industrial_different_wall_ids() {
        let packs = load_style_pack_registry();
        let victorian = packs.get("style_victorian").unwrap();
        let industrial = packs.get("style_industrial_west").unwrap();
        assert_ne!(
            victorian.resolve_slot(StylePackSlotKey::Wall1u),
            industrial.resolve_slot(StylePackSlotKey::Wall1u)
        );
        assert_eq!(victorian.resolve_slot(StylePackSlotKey::Wall1u), Some("wall_brick_1u"));
        assert_eq!(
            industrial.resolve_slot(StylePackSlotKey::Wall1u),
            Some("wall_steel_1u")
        );
    }
}
