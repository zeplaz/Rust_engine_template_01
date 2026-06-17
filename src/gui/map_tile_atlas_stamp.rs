//! Tactical / simulation map tile material swap via [`TileAtlasRegistry::resolve_variant_uv`].
//!
//! Stamps MCP ortho atlas sub-rects onto the CPU fallback raster at committed site footprints.
//! Separate from PG-2 procedural GLB assembly (`procedural_build_extract`).

use std::collections::HashMap;

use bevy::prelude::*;

use crate::construction::procedural::{
    assembly_id_for, TileAtlasRegistry, TileVariantContext, VariantCatalog,
    VisualState,
};
use crate::strategic::{
    BuildSiteTile, ConstructionSite, FootprintTiles, PlannedSite, ProceduralBuildingSpec,
    SiteArchetype, SiteConstructionPhase, SiteFootprint,
};

/// One building iso stamp for the overworld fallback texture.
#[derive(Clone, Debug)]
pub struct TileAtlasStampRequest {
    pub atlas_id: String,
    pub variant_key: String,
    /// TILE-FIX-003 — iso facing used for UV lookup.
    pub facing: u8,
    pub frame: u8,
    pub uv: [f32; 4],
    /// Top-left world tile (column, row).
    pub origin: IVec2,
    pub footprint_w: u32,
    pub footprint_h: u32,
}

#[derive(Resource, Debug, Default)]
pub struct TileAtlasGpuCache {
    pub handles: HashMap<String, Handle<Image>>,
}

/// Build sim context for resolver (PT-4/5). Night/power stub until grid wired.
#[must_use]
pub fn tile_variant_context_for_site(
    site: &ConstructionSite,
    sim_tick: u64,
    fire_heat: f32,
) -> TileVariantContext {
    TileVariantContext {
        phase: site.phase,
        damage: if site.operational_readiness < 0.5 {
            0.5
        } else {
            0.0
        },
        power_on: matches!(
            site.phase,
            SiteConstructionPhase::Operational | SiteConstructionPhase::Provisioning
        ),
        night: false,
        fire_heat,
        sim_tick,
    }
}

/// Legacy phase stub — delegates to resolver when catalog + entry present.
#[must_use]
pub fn variant_key_for_site_phase(phase: SiteConstructionPhase) -> &'static str {
    match phase {
        SiteConstructionPhase::Abandoned => "damaged_night_on",
        SiteConstructionPhase::UnderConstruction | SiteConstructionPhase::Foundation => {
            "under_construction_02"
        }
        _ => "clean_day",
    }
}

/// Production rowhouse tile id in [`TileAtlasRegistry`] (ENG-PT-4-001).
pub const ROWHOUSE_VICTORIAN_TILE_ID: &str = "rowhouse_victorian";

/// Warehouse industrial west v2 tile/atlas ids (TILE-FIX-10 minimum G4).
pub const WAREHOUSE_INDUSTRIAL_TILE_ID: &str = "warehouse_industrial";
pub const WAREHOUSE_INDUSTRIAL_ATLAS_ID: &str = "warehouse_industrial_west_v2";

/// BUILD-READ-VISUAL-001 — rail warehouse pilot iso tile (logistics_rail_warehouse_v0).
pub const RAIL_WAREHOUSE_PILOT_TILE_ID: &str = "tile_rail_warehouse_pilot_v1";
pub const RAIL_WAREHOUSE_PILOT_ATLAS_ID: &str = "rail_warehouse_pilot_v1";
pub const RAIL_WAREHOUSE_PILOT_CATALOG_ID: &str = "pilot:logistics_rail_warehouse_v0";
pub const TILE_FIX_10_WAREHOUSE_WITNESS_JSON: &str =
    "debug_runs/art_pipeline/tile_fix_10_warehouse_industrial_live.json";
pub const ENG_PT4_WAREHOUSE_MAP_STAMP_LIVE_JSON: &str =
    "debug_runs/art_pipeline/eng_pt_4_warehouse_map_stamp_live.json";

fn repo_witness_path(rel: &str) -> std::path::PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| std::path::PathBuf::from(rel))
}

/// TILE-FIX-10 promotion witness must be green before runtime warehouse stamp (Phase D).
#[must_use]
pub fn tile_fix_10_warehouse_witness_green() -> bool {
    let path = repo_witness_path(TILE_FIX_10_WAREHOUSE_WITNESS_JSON);
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(body) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    body.get("green").and_then(|v| v.as_bool()) == Some(true)
        && body
            .get("promotion_validation")
            .and_then(|v| v.get("status"))
            .and_then(|v| v.as_str())
            == Some("passed")
}

/// Index row written only when coder-mcp `--register` ran on green TILE-FIX-10 witness.
#[must_use]
pub fn warehouse_v2_atlas_index_registered() -> bool {
    if !tile_fix_10_warehouse_witness_green() {
        return false;
    }
    let registry = crate::construction::procedural::load_tile_atlas_registry();
    if !registry.load_errors.is_empty() {
        return false;
    }
    registry.get(WAREHOUSE_INDUSTRIAL_ATLAS_ID).is_some_and(|entry| {
        entry.tile_id == WAREHOUSE_INDUSTRIAL_TILE_ID
            && entry.ship_allowed
            && entry.meta_schema_version >= 2
            && entry.runtime_stamp_allowed()
            && !entry.lookups.is_empty()
    })
}

/// BUILD-READ-VISUAL-001 — pilot catalog id resolves to warehouse production atlas (lib).
#[must_use]
pub fn build_read_visual_pilot_tile_stamp_lib_green() -> bool {
    rail_warehouse_pilot_atlas_index_registered()
        && build_read_visual_pilot_stamp_request_green()
}

/// Pilot iso stamp resolves when production atlas row + PNG are on disk.
#[must_use]
pub fn rail_warehouse_pilot_atlas_index_registered() -> bool {
    let registry = crate::construction::procedural::load_tile_atlas_registry();
    if !registry.load_errors.is_empty() {
        return false;
    }
    let Some(entry) = registry.atlas_for_tile_id(RAIL_WAREHOUSE_PILOT_TILE_ID) else {
        return false;
    };
    entry.atlas_id == RAIL_WAREHOUSE_PILOT_ATLAS_ID
        && entry.runtime_stamp_allowed()
        && !entry.lookups.is_empty()
        && std::path::Path::new(&entry.atlas_png).is_file()
}

#[must_use]
pub fn build_read_visual_pilot_stamp_request_green() -> bool {
    use crate::construction::procedural::{
        load_tile_atlas_registry, load_variant_catalog, ProceduralBuildingRequest, StylePackId,
    };
    use crate::construction::PilotCatalog;
    use crate::strategic::{BuildSiteTile, LayerType, SiteId};

    let registry = load_tile_atlas_registry();
    let catalog = match load_variant_catalog() {
        Some(c) => c,
        None => return false,
    };
    let pilots = PilotCatalog::load_from_disk();
    let Some(pilot) = pilots.first_grammar_pilot() else {
        return false;
    };
    if pilot.catalog_id != RAIL_WAREHOUSE_PILOT_CATALOG_ID {
        return false;
    }
    let planned = PlannedSite {
        site_id: SiteId(1),
        origin: BuildSiteTile { x: 0, z: 0 },
        footprint: FootprintTiles {
            width: pilot.footprint.width,
            depth: pilot.footprint.depth,
        },
        archetype: SiteArchetype::Factory,
        layer: LayerType::Surface,
        catalog_id: Some(pilot.catalog_id.clone()),
        placement: None,
    };
    let site = ConstructionSite {
        site_id: 1,
        owner: Entity::PLACEHOLDER,
        archetype: SiteArchetype::Factory,
        phase: SiteConstructionPhase::Operational,
        operational_readiness: 1.0,
    };
    let mut footprint_tiles = Vec::new();
    for dz in 0..planned.footprint.depth {
        for dx in 0..planned.footprint.width {
            footprint_tiles.push(IVec2::new(
                planned.origin.x as i32 + dx as i32,
                planned.origin.z as i32 + dz as i32,
            ));
        }
    }
    let site_fp = SiteFootprint {
        tiles: footprint_tiles,
        layer: planned.layer,
    };
    let spec = ProceduralBuildingSpec(ProceduralBuildingRequest {
        archetype_id: "industrial_warehouse_l".into(),
        width: planned.footprint.width,
        depth: planned.footprint.depth,
        floors: 1,
        style: StylePackId("style_industrial_west".into()),
        seed: 440013,
        arch_dna_preset_id: Some("logistics_rail_warehouse_v0".into()),
    });
    let Some(req) = stamp_request_for_site(
        &registry,
        Some(&catalog),
        0,
        0.0,
        &planned,
        &site,
        &site_fp,
        Some(&spec),
    ) else {
        return false;
    };
    req.atlas_id == RAIL_WAREHOUSE_PILOT_ATLAS_ID
        && req.variant_key == "clean_day"
        && req.footprint_w >= planned.footprint.width
}

#[must_use]
pub fn resolve_atlas_entry_for_planned_site<'a>(
    registry: &'a TileAtlasRegistry,
    planned: &PlannedSite,
    spec: Option<&ProceduralBuildingSpec>,
) -> Option<&'a crate::construction::procedural::TileAtlasEntry> {
    if let Some(catalog) = planned.catalog_id.as_deref() {
        if catalog == RAIL_WAREHOUSE_PILOT_CATALOG_ID
            || (catalog.starts_with("pilot:") && catalog.contains("logistics_rail"))
        {
            if let Some(entry) = registry.atlas_for_tile_id(RAIL_WAREHOUSE_PILOT_TILE_ID) {
                return Some(entry);
            }
        }
        if let Some(entry) = registry.atlas_for_tile_id(catalog) {
            return Some(entry);
        }
        if catalog.contains("rowhouse") || catalog.contains("victorian") {
            if let Some(entry) = registry.atlas_for_tile_id(ROWHOUSE_VICTORIAN_TILE_ID) {
                return Some(entry);
            }
        }
        if catalog.contains("warehouse") || catalog.contains("industrial") {
            if let Some(entry) = registry.atlas_for_tile_id(WAREHOUSE_INDUSTRIAL_TILE_ID) {
                return Some(entry);
            }
        }
    }
    let style = spec
        .map(|s| s.0.style.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("");
    if !style.is_empty() {
        let floors = spec.map(|s| s.0.floors).unwrap_or(2);
        let seed = spec.map(|s| s.0.seed).unwrap_or(42);
        let assembly_id = assembly_id_for(
            style,
            planned.footprint.width,
            planned.footprint.depth,
            floors,
            seed,
        );
        if let Some(entry) = registry.atlas_for_assembly(&assembly_id) {
            return Some(entry);
        }
    }
    None
}

fn footprint_bounds(tiles: &[IVec2]) -> Option<(IVec2, u32, u32)> {
    if tiles.is_empty() {
        return None;
    }
    let mut min = tiles[0];
    let mut max = tiles[0];
    for t in tiles.iter().skip(1) {
        min = min.min(*t);
        max = max.max(*t);
    }
    let w = (max.x - min.x + 1).max(1) as u32;
    let h = (max.y - min.y + 1).max(1) as u32;
    Some((min, w, h))
}

#[must_use]
pub fn stamp_request_for_site(
    registry: &TileAtlasRegistry,
    catalog: Option<&VariantCatalog>,
    sim_tick: u64,
    fire_heat: f32,
    planned: &PlannedSite,
    site: &ConstructionSite,
    footprint: &SiteFootprint,
    spec: Option<&ProceduralBuildingSpec>,
) -> Option<TileAtlasStampRequest> {
    if matches!(site.phase, SiteConstructionPhase::Abandoned) {
        // Resolver may still pick abandoned key; skip only when no atlas.
    }
    let entry = resolve_atlas_entry_for_planned_site(registry, planned, spec)?;
    let ctx = tile_variant_context_for_site(site, sim_tick, fire_heat);
    let rotation = planned
        .placement
        .as_ref()
        .map(|p| p.rotation_quarter_turns % 4)
        .unwrap_or(0);
    let visual = if let Some(cat) = catalog {
        registry.resolve_visual_state_for_site(&entry.atlas_id, cat, ctx, rotation)
    } else {
        let resolved = crate::construction::procedural::ResolvedTileVariant {
            variant_key: variant_key_for_site_phase(site.phase).to_owned(),
            animation_frame: None,
        };
        Some(VisualState::from_resolved(
            &resolved,
            rotation,
            entry.render_facings.max(1),
            entry.quarter_turn_fallback,
        ))
    }?;
    let variant_key = visual.variant_key.clone();
    let uv = registry.resolve_visual_state_uv(&entry.atlas_id, &visual)?;
    let (origin, fw, fh) = footprint_bounds(&footprint.tiles)?;
    Some(TileAtlasStampRequest {
        atlas_id: entry.atlas_id.clone(),
        variant_key,
        facing: visual.facing,
        frame: visual.frame,
        uv,
        origin,
        footprint_w: fw,
        footprint_h: fh,
    })
}

/// **ENG-PT-4-001** — production rowhouse atlas resolves + stamp request for demo footprint.
#[must_use]
pub fn eng_pt_4_001_rowhouse_map_stamp_witness_green() -> bool {
    use crate::construction::procedural::{
        assembly_id_for, load_tile_atlas_registry, load_variant_catalog, ProceduralBuildingRequest,
        StylePackId,
    };
    let registry = load_tile_atlas_registry();
    if !registry.load_errors.is_empty() {
        return false;
    }
    let catalog = match load_variant_catalog() {
        Some(c) => c,
        None => return false,
    };
    let Some(entry) = registry
        .atlas_for_tile_id(ROWHOUSE_VICTORIAN_TILE_ID)
        .or_else(|| registry.atlas_for_assembly("victorian_4x3_s42_a7cb"))
    else {
        return false;
    };
    if !crate::construction::procedural::production_atlas_covers_assembly(entry) {
        return false;
    }
    if !std::path::Path::new(&entry.atlas_png).exists() {
        return false;
    }
    let origin = BuildSiteTile { x: 20, z: 20 };
    let footprint = FootprintTiles {
        width: 4,
        depth: 3,
    };
    let planned = PlannedSite {
        site_id: crate::strategic::SiteId(42),
        origin,
        footprint,
        archetype: SiteArchetype::CivilHousing,
        layer: crate::strategic::LayerType::Surface,
        catalog_id: Some(ROWHOUSE_VICTORIAN_TILE_ID.into()),
        placement: None,
    };
    let site = ConstructionSite {
        site_id: 42,
        owner: Entity::PLACEHOLDER,
        archetype: planned.archetype,
        phase: SiteConstructionPhase::Operational,
        operational_readiness: 1.0,
    };
    let spec = ProceduralBuildingSpec(ProceduralBuildingRequest {
        archetype_id: "rect_perimeter".into(),
        width: 4,
        depth: 3,
        floors: 2,
        style: StylePackId("style_victorian".into()),
        seed: 42,
        arch_dna_preset_id: None,
    });
    let assembly_id = assembly_id_for("style_victorian", 4, 3, 2, 42);
    if assembly_id != "victorian_4x3_s42_a7cb" {
        return false;
    }
    let mut footprint_tiles = Vec::new();
    for dz in 0..footprint.depth {
        for dx in 0..footprint.width {
            footprint_tiles.push(IVec2::new(
                origin.x as i32 + dx as i32,
                origin.z as i32 + dz as i32,
            ));
        }
    }
    let site_fp = SiteFootprint {
        tiles: footprint_tiles,
        layer: planned.layer,
    };
    let Some(req) = stamp_request_for_site(
        &registry,
        Some(&catalog),
        0,
        0.0,
        &planned,
        &site,
        &site_fp,
        Some(&spec),
    ) else {
        return false;
    };
    req.atlas_id == "rowhouse_victorian_production_v1"
        && req.variant_key == "clean_day"
        && req.footprint_w >= 4
        && req.footprint_h >= 3
}

/// **ENG-PT-4-002** — warehouse v2 atlas stamp with `rotation_quarter_turns` → facing UV (Phase D).
#[must_use]
pub fn eng_pt_4_warehouse_map_stamp_witness_green() -> bool {
    use crate::construction::procedural::{
        assembly_id_for, load_tile_atlas_registry, load_variant_catalog, facing_from_rotation_quarter_turns,
        ProceduralBuildingRequest, StylePackId,
    };
    use crate::strategic::CommittedPlacementSnapshot;

    if !tile_fix_10_warehouse_witness_green() || !warehouse_v2_atlas_index_registered() {
        return false;
    }
    let registry = load_tile_atlas_registry();
    if !registry.load_errors.is_empty() {
        return false;
    }
    let catalog = match load_variant_catalog() {
        Some(c) => c,
        None => return false,
    };
    let Some(entry) = registry.atlas_for_tile_id(WAREHOUSE_INDUSTRIAL_TILE_ID) else {
        return false;
    };
    if entry.atlas_id != WAREHOUSE_INDUSTRIAL_ATLAS_ID
        || entry.meta_schema_version < 2
        || !crate::construction::procedural::production_atlas_covers_assembly(entry)
    {
        return false;
    }
    if !std::path::Path::new(&entry.atlas_png).is_file() {
        return false;
    }
    let origin = BuildSiteTile { x: 30, z: 30 };
    let footprint = FootprintTiles {
        width: 4,
        depth: 2,
    };
    let placement_rot1 = CommittedPlacementSnapshot {
        origin,
        scale_factor: 1.0,
        effective_scale: 1.0,
        rotation_quarter_turns: 1,
        mirror_x: false,
        weights: vec![(IVec2::new(origin.x as i32, origin.z as i32), 1.0)],
    };
    let site = ConstructionSite {
        site_id: 879,
        owner: Entity::PLACEHOLDER,
        archetype: SiteArchetype::Factory,
        phase: SiteConstructionPhase::Operational,
        operational_readiness: 1.0,
    };
    let spec = ProceduralBuildingSpec(ProceduralBuildingRequest {
        archetype_id: "rect_perimeter".into(),
        width: 4,
        depth: 2,
        floors: 2,
        style: StylePackId("style_industrial_west".into()),
        seed: 43,
        arch_dna_preset_id: None,
    });
    let assembly_id = assembly_id_for("style_industrial_west", 4, 2, 2, 43);
    if assembly_id != "industrial_west_4x2_s43_a879" {
        return false;
    }
    let mut footprint_tiles = Vec::new();
    for dz in 0..footprint.depth {
        for dx in 0..footprint.width {
            footprint_tiles.push(IVec2::new(
                origin.x as i32 + dx as i32,
                origin.z as i32 + dz as i32,
            ));
        }
    }
    let site_fp = SiteFootprint {
        tiles: footprint_tiles,
        layer: crate::strategic::LayerType::Surface,
    };
    let planned_rot0 = PlannedSite {
        site_id: crate::strategic::SiteId(879),
        origin,
        footprint,
        archetype: SiteArchetype::Factory,
        layer: crate::strategic::LayerType::Surface,
        catalog_id: Some(WAREHOUSE_INDUSTRIAL_TILE_ID.into()),
        placement: Some(CommittedPlacementSnapshot {
            rotation_quarter_turns: 0,
            ..placement_rot1.clone()
        }),
    };
    let planned_rot1 = PlannedSite {
        placement: Some(placement_rot1),
        ..planned_rot0.clone()
    };
    let Some(req0) = stamp_request_for_site(
        &registry,
        Some(&catalog),
        0,
        0.0,
        &planned_rot0,
        &site,
        &site_fp,
        Some(&spec),
    ) else {
        return false;
    };
    let Some(req1) = stamp_request_for_site(
        &registry,
        Some(&catalog),
        0,
        0.0,
        &planned_rot1,
        &site,
        &site_fp,
        Some(&spec),
    ) else {
        return false;
    };
    let expected_facing = facing_from_rotation_quarter_turns(
        1,
        entry.render_facings.max(1),
        entry.quarter_turn_fallback,
    );
    req0.atlas_id == WAREHOUSE_INDUSTRIAL_ATLAS_ID
        && req1.atlas_id == WAREHOUSE_INDUSTRIAL_ATLAS_ID
        && req0.facing == 0
        && req1.facing == expected_facing
        && req1.facing != req0.facing
        && req0.uv != req1.uv
        && req1.variant_key == "clean_day"
}

#[must_use]
pub fn build_eng_pt_4_warehouse_map_stamp_witness_body() -> serde_json::Value {
    serde_json::json!({
        "gate_id": "ENG-PT-4-002",
        "program_id": "PLAN-TILE-FIX-AUTO-BUILD-001",
        "task_id": "TILE-FIX-WAREHOUSE-PHASE-D",
        "green": eng_pt_4_warehouse_map_stamp_witness_green(),
        "tile_fix_10_witness_green": tile_fix_10_warehouse_witness_green(),
        "warehouse_v2_index_registered": warehouse_v2_atlas_index_registered(),
        "atlas_id": WAREHOUSE_INDUSTRIAL_ATLAS_ID,
        "tile_id": WAREHOUSE_INDUSTRIAL_TILE_ID,
        "rotation_quarter_turns_smoke": 1,
        "parent_witness": TILE_FIX_10_WAREHOUSE_WITNESS_JSON,
    })
}

/// Refresh Phase D map-stamp witness (`debug_runs/art_pipeline/eng_pt_4_warehouse_map_stamp_live.json`).
#[must_use]
pub fn refresh_eng_pt_4_warehouse_map_stamp_live_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_eng_pt_4_warehouse_map_stamp_witness_body();
    let wrapped = wrap_debug_run(
        "gui",
        "refresh_eng_pt_4_warehouse_map_stamp_live_witness",
        ENG_PT4_WAREHOUSE_MAP_STAMP_LIVE_JSON,
        body,
    );
    write_debug_run_json(ENG_PT4_WAREHOUSE_MAP_STAMP_LIVE_JSON, wrapped)
        && eng_pt_4_warehouse_map_stamp_witness_green()
}

pub fn preload_tile_atlas_gpu_cache(
    registry: Option<&TileAtlasRegistry>,
    asset_server: &AssetServer,
    cache: &mut TileAtlasGpuCache,
) {
    let Some(registry) = registry else {
        return;
    };
    for entry in &registry.entries {
        if cache.handles.contains_key(&entry.atlas_id) {
            continue;
        }
        let handle: Handle<Image> = asset_server.load(entry.atlas_asset.clone());
        cache.handles.insert(entry.atlas_id.clone(), handle);
    }
}

/// Nearest-neighbor blit of atlas UV sub-rect into overworld RGBA8 (alpha ≥ 128 replaces base).
pub fn stamp_atlas_uv_into_rgba_subregion(
    dest: &mut [u8],
    tex_w: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    atlas: &[u8],
    atlas_w: usize,
    atlas_h: usize,
    dest_origin: IVec2,
    dest_w: u32,
    dest_h: u32,
    uv: [f32; 4],
) {
    let src_x0 = (uv[0] * atlas_w as f32).floor() as usize;
    let src_y0 = (uv[1] * atlas_h as f32).floor() as usize;
    let src_w = (uv[2] * atlas_w as f32).round().max(1.0) as usize;
    let src_h = (uv[3] * atlas_h as f32).round().max(1.0) as usize;
    let dest_w = dest_w.max(1) as usize;
    let dest_h = dest_h.max(1) as usize;
    let base_x = dest_origin.x.max(0) as usize;
    let base_y = dest_origin.y.max(0) as usize;

    for dy in 0..dest_h {
        let ty = base_y + dy;
        if ty < y0 || ty >= y1 {
            continue;
        }
        for dx in 0..dest_w {
            let tx = base_x + dx;
            if tx < x0 || tx >= x1 {
                continue;
            }
            let sx = src_x0 + (dx * src_w / dest_w).min(src_w.saturating_sub(1));
            let sy = src_y0 + (dy * src_h / dest_h).min(src_h.saturating_sub(1));
            let src_i = 4 * (sy * atlas_w + sx);
            if src_i + 3 >= atlas.len() {
                continue;
            }
            let alpha = atlas[src_i + 3];
            if alpha < 128 {
                continue;
            }
            let dst_i = 4 * (ty * tex_w + tx);
            if dst_i + 3 >= dest.len() {
                continue;
            }
            dest[dst_i..dst_i + 4].copy_from_slice(&atlas[src_i..src_i + 4]);
        }
    }
}

pub fn apply_atlas_stamps_to_rgba_subregion(
    dest: &mut [u8],
    tex_w: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    stamps: &[TileAtlasStampRequest],
    atlas_data: &HashMap<String, (&[u8], usize, usize)>,
) {
    for req in stamps {
        let Some((data, atlas_w, atlas_h)) = atlas_data.get(&req.atlas_id) else {
            continue;
        };
        if *atlas_w == 0 || *atlas_h == 0 {
            continue;
        }
        stamp_atlas_uv_into_rgba_subregion(
            dest,
            tex_w,
            x0,
            y0,
            x1,
            y1,
            data,
            *atlas_w,
            *atlas_h,
            req.origin,
            req.footprint_w,
            req.footprint_h,
            req.uv,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_read_pilot_atlas_stamp_when_registered() {
        if !super::rail_warehouse_pilot_atlas_index_registered() {
            eprintln!("skip: rail warehouse pilot atlas not registered for runtime stamp");
            return;
        }
        assert!(super::build_read_visual_pilot_stamp_request_green());
        assert!(super::build_read_visual_pilot_tile_stamp_lib_green());
    }

    #[test]
    fn variant_key_switches_on_abandoned() {
        assert_eq!(
            variant_key_for_site_phase(SiteConstructionPhase::Operational),
            "clean_day"
        );
        assert_eq!(
            variant_key_for_site_phase(SiteConstructionPhase::Abandoned),
            "damaged_night_on"
        );
    }

    #[test]
    fn eng_pt_4_rowhouse_stamp_blocked_until_atlas_v2() {
        assert!(
            !super::eng_pt_4_001_rowhouse_map_stamp_witness_green(),
            "TILE-FIX-001: greybox production de-indexed until v2 promotes"
        );
    }

    #[test]
    fn eng_pt_4_warehouse_map_stamp_rotation_witness() {
        if !super::tile_fix_10_warehouse_witness_green() {
            eprintln!("skip: tile_fix_10 witness not green");
            return;
        }
        assert!(
            super::warehouse_v2_atlas_index_registered(),
            "warehouse v2 index row required (Phase B --register on green witness)"
        );
        assert!(
            super::eng_pt_4_warehouse_map_stamp_witness_green(),
            "warehouse map stamp smoke failed — check atlas meta v2 lookups + PNG"
        );
    }

    #[test]
    fn refresh_eng_pt_4_warehouse_map_stamp_live_witness_writes_json() {
        if !super::tile_fix_10_warehouse_witness_green() {
            return;
        }
        assert!(super::refresh_eng_pt_4_warehouse_map_stamp_live_witness());
    }

    #[test]
    fn stamp_blit_writes_opaque_pixel() {
        let mut dest = vec![0u8; 4 * 4 * 4];
        let atlas = vec![255u8, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        stamp_atlas_uv_into_rgba_subregion(
            &mut dest,
            4,
            0,
            0,
            4,
            4,
            &atlas,
            2,
            2,
            IVec2::new(1, 1),
            1,
            1,
            [0.0, 0.0, 0.5, 1.0],
        );
        let i = 4 * (1 * 4 + 1);
        assert_eq!(&dest[i..i + 4], &[255, 0, 0, 255]);
    }
}
