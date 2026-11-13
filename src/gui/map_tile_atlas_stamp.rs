//! Simulation map building iso stamps via [`TileAtlasRegistry::resolve_variant_uv`].
//!
//! **Dual-view:** CPU/GPU stamp blit into the play RTT is queued only under
//! [`crate::gui::WorldPresentationKind::PixelWorld3D`] (see `tile_world_fallback`).
//! TacticalFlat2D keeps occupation / icon tokens — no APS module-kit dump.
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
    ConstructionSite, PlannedSite, ProceduralBuildingSpec, SiteConstructionPhase, SiteFootprint,
};

/// GPU terrain stamp indices (world tile → material index), not CPU RGBA blit.
#[derive(Resource, Debug, Default, Clone)]
pub struct TerrainGpuStampIndices {
    pub tiles: HashMap<IVec2, u32>,
}

/// Record building stamp tiles for GPU terrain instance patch (P0-D).
pub fn queue_gpu_terrain_stamp_indices(
    stamps: &[TileAtlasStampRequest],
    out: &mut TerrainGpuStampIndices,
) {
    out.tiles.clear();
    for req in stamps {
        let material_index = req.frame as u32;
        for dy in 0..req.footprint_h {
            for dx in 0..req.footprint_w {
                let tile = req.origin + IVec2::new(dx as i32, dy as i32);
                out.tiles.insert(tile, material_index);
            }
        }
    }
}

/// Patch instanced terrain rows at stamped world tiles (no CPU blit).
pub fn apply_gpu_stamps_to_terrain_instances(
    stamps: &TerrainGpuStampIndices,
    instances: &mut [crate::render::TerrainTileInstance],
) {
    if stamps.tiles.is_empty() {
        return;
    }
    for inst in instances.iter_mut() {
        let tile = IVec2::new(
            (inst.world_pos[0] - 0.5).round() as i32,
            (inst.world_pos[1] - 0.5).round() as i32,
        );
        if let Some(idx) = stamps.tiles.get(&tile) {
            inst.material_index = *idx;
        }
    }
}

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

/// BUILD-READ-VISUAL-001 — first grammar pilot iso tile/atlas ids (tile atlas registry).
pub const RAIL_WAREHOUSE_PILOT_TILE_ID: &str = "tile_rail_warehouse_pilot_v1";
pub const RAIL_WAREHOUSE_PILOT_ATLAS_ID: &str = "rail_warehouse_pilot_v1";
pub const TILE_FIX_10_WAREHOUSE_WITNESS_JSON: &str =
    "debug_runs/art_pipeline/tile_fix_10_warehouse_industrial_live.json";
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
pub fn resolve_atlas_entry_for_planned_site<'a>(
    registry: &'a TileAtlasRegistry,
    planned: &PlannedSite,
    spec: Option<&ProceduralBuildingSpec>,
) -> Option<&'a crate::construction::procedural::TileAtlasEntry> {
    if let Some(catalog_id) = planned.catalog_id.as_deref() {
        let pilots = crate::construction::PilotCatalog::load_from_disk();
        if catalog_id.starts_with("pilot:") && pilots.by_catalog_id(catalog_id).is_some() {
            if let Some(entry) = registry.atlas_for_tile_id(RAIL_WAREHOUSE_PILOT_TILE_ID) {
                return Some(entry);
            }
        }
        if let Some(entry) = registry.atlas_for_tile_id(catalog_id) {
            return Some(entry);
        }
        if catalog_id.contains("rowhouse") || catalog_id.contains("victorian") {
            if let Some(entry) = registry.atlas_for_tile_id(ROWHOUSE_VICTORIAN_TILE_ID) {
                return Some(entry);
            }
        }
        if catalog_id.contains("warehouse") || catalog_id.contains("industrial") {
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

/// CPU RGBA stamp blit is allowed only under [`TerrainRenderAuthority::CpuRaster`].
#[must_use]
pub fn stamp_cpu_rgba_blit_enabled(authority: crate::render::TerrainRenderAuthority) -> bool {
    authority.uses_cpu_raster()
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
    fn stamp_cpu_blit_gated_on_fallback_authority() {
        use crate::render::TerrainRenderAuthority;
        assert!(stamp_cpu_rgba_blit_enabled(TerrainRenderAuthority::CpuRaster));
        assert!(!stamp_cpu_rgba_blit_enabled(TerrainRenderAuthority::GpuBake));
        assert!(!stamp_cpu_rgba_blit_enabled(TerrainRenderAuthority::GpuTilemap));
    }

    #[test]
    fn gpu_stamp_path_avoids_cpu_blit_contract() {
        use crate::render::TerrainRenderAuthority;
        let gpu = TerrainRenderAuthority::GpuBake;
        assert!(gpu.is_gpu());
        assert!(!stamp_cpu_rgba_blit_enabled(gpu));
    }

    #[test]
    fn gpu_stamp_updates_instance_index_not_cpu_buffer() {
        let mut stamps = TerrainGpuStampIndices::default();
        let req = TileAtlasStampRequest {
            atlas_id: "test".into(),
            variant_key: "clean_day".into(),
            facing: 0,
            frame: 7,
            uv: [0.0, 0.0, 0.25, 0.25],
            origin: IVec2::new(4, 4),
            footprint_w: 2,
            footprint_h: 2,
        };
        queue_gpu_terrain_stamp_indices(&[req], &mut stamps);
        let mut instances = vec![crate::render::TerrainTileInstance {
            world_pos: [4.5, 4.5],
            material_index: 0,
            _pad: 0,
        }];
        apply_gpu_stamps_to_terrain_instances(&stamps, &mut instances);
        assert_eq!(instances[0].material_index, 7);
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
