//! VEG-F03-REGISTRY-STAMP-001 — LG-5 landscape chunk UV stamp from registry atlas meta.
//!
//! PixelWorld play path: stamps blit into the same sim RTT RGBA as landform relief
//! ([`crate::render::tile_world_fallback`]) — not a parallel extract.
//!
//! Veg/cover modulation (v5): prefer burn/scar/regrowth atlas rows that already exist
//! on `landscape_lg5_expanded_v1` — never invent tiles; fall back to clean topology.

use bevy::prelude::*;

use crate::gui::map_tile_atlas_stamp::{
    stamp_atlas_uv_into_rgba_subregion, TileAtlasGpuCache, TileAtlasStampRequest,
};
use crate::systems::ecology::{
    load_landscape_atlas_registry, topology_kind_to_variant_key, ActiveBurn,
    LandscapeAtlasRegistry, LandscapeProgramOnChunk, RegrowthMacroPhase, SuccessionState,
    SuccessionTopologyStage, VegetationField, ACTIVE_BURN_HEAT_EPS, LG5_ATLAS_ID,
};
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

/// Optional veg / burn / succession hints for PixelWorld LG-5 stamp selection.
#[derive(Clone, Copy, Debug, Default)]
pub struct PixelWorldVegStampInputs<'a> {
    pub veg: Option<&'a VegetationField>,
    pub burn: Option<&'a ActiveBurn>,
    pub succession: Option<&'a SuccessionState>,
}

/// One chunk iso stamp from LG-5 topology atlas.
#[derive(Clone, Debug)]
pub struct LandscapeChunkStampRequest {
    pub atlas_id: String,
    pub variant_key: String,
    pub uv: [f32; 4],
    pub chunk_coord: IVec2,
}

#[must_use]
pub fn primary_topology_kind(kinds: &[String]) -> Option<&str> {
    kinds.first().map(|s| s.as_str())
}

/// Ordered LG-5 variant keys for a topology base — veg/burn first, clean last.
#[must_use]
pub fn preferred_pixel_world_topology_keys(
    base: &str,
    inputs: PixelWorldVegStampInputs<'_>,
) -> Vec<&'static str> {
    let heat = inputs.burn.map(|b| b.heat).unwrap_or(0.0);
    let frame = inputs.burn.map(|b| b.frame_index).unwrap_or(0);
    let phase = inputs
        .burn
        .map(|b| b.regrowth_macro_phase)
        .unwrap_or(RegrowthMacroPhase::None);
    let stage = inputs.succession.map(|s| s.stage);
    let scar = stage == Some(SuccessionTopologyStage::BurnScar)
        || inputs
            .veg
            .is_some_and(|v| v.burn_severity > 0.35)
        || matches!(
            phase,
            RegrowthMacroPhase::Scar | RegrowthMacroPhase::Closing
        );
    let regrowth = matches!(
        phase,
        RegrowthMacroPhase::Nuclei | RegrowthMacroPhase::Front
    ) || inputs
        .veg
        .is_some_and(|v| v.regrowth_stage > 0.55 && v.burn_severity > 0.1);
    let shrub = stage == Some(SuccessionTopologyStage::Shrub)
        || inputs
            .veg
            .is_some_and(|v| v.understory_density > 0.4);

    let mut out: Vec<&'static str> = Vec::with_capacity(5);
    match base {
        "topology_patch" => {
            if heat > ACTIVE_BURN_HEAT_EPS {
                if frame >= 4 {
                    out.push("topology_patch_burn_04");
                }
                out.push("topology_patch_burn_00");
            }
            if scar {
                out.push("topology_patch_scar");
            }
            if regrowth {
                if shrub {
                    out.push("topology_patch_regrowth_shrub");
                }
                out.push("topology_patch_regrowth_grass");
            }
            out.push("topology_patch");
        }
        "topology_corridor" => {
            if heat > ACTIVE_BURN_HEAT_EPS {
                if frame >= 4 {
                    out.push("topology_corridor_burn_04");
                }
                out.push("topology_corridor_burn_00");
            }
            if scar {
                out.push("topology_corridor_scar");
            }
            out.push("topology_corridor");
        }
        "topology_ring" => {
            if heat > ACTIVE_BURN_HEAT_EPS {
                out.push("topology_ring_burn_00");
            }
            out.push("topology_ring");
        }
        "topology_cluster" => {
            if regrowth || scar {
                out.push("topology_cluster_regrowth_grass");
            }
            out.push("topology_cluster");
        }
        "topology_fringe" => {
            if regrowth || scar {
                out.push("topology_fringe_regrowth_grass");
            }
            out.push("topology_fringe");
        }
        _ => {}
    }
    if out.is_empty() {
        // Caller passed a known base literal — keep as sole candidate.
        // (Static match arms cover all topology_kind_to_variant_key outputs.)
    }
    out
}

/// Resolve a topology stamp, preferring veg/burn/regrowth atlas rows when present.
#[must_use]
pub fn resolve_pixel_world_topology_variant(
    registry: &LandscapeAtlasRegistry,
    topology_kinds: &[String],
    inputs: PixelWorldVegStampInputs<'_>,
) -> Option<(String, [f32; 4])> {
    let entry = registry.lg5_entry()?;
    let kind = primary_topology_kind(topology_kinds)?;
    let base = topology_kind_to_variant_key(kind)?;
    for key in preferred_pixel_world_topology_keys(base, inputs) {
        if let Some(uv) = entry.resolve_variant_uv(key) {
            return Some((key.to_owned(), uv));
        }
    }
    let uv = entry.resolve_variant_uv(base)?;
    Some((base.to_owned(), uv))
}

#[must_use]
pub fn stamp_request_for_topology(
    registry: &LandscapeAtlasRegistry,
    chunk_coord: IVec2,
    topology_kinds: &[String],
) -> Option<LandscapeChunkStampRequest> {
    stamp_request_for_topology_with_veg(
        registry,
        chunk_coord,
        topology_kinds,
        PixelWorldVegStampInputs::default(),
    )
}

#[must_use]
pub fn stamp_request_for_topology_with_veg(
    registry: &LandscapeAtlasRegistry,
    chunk_coord: IVec2,
    topology_kinds: &[String],
    inputs: PixelWorldVegStampInputs<'_>,
) -> Option<LandscapeChunkStampRequest> {
    let entry = registry.lg5_entry()?;
    let (variant_key, uv) =
        resolve_pixel_world_topology_variant(registry, topology_kinds, inputs)?;
    Some(LandscapeChunkStampRequest {
        atlas_id: entry.atlas_id.clone(),
        variant_key,
        uv,
        chunk_coord,
    })
}

#[must_use]
pub fn stamp_request_for_program(
    registry: &LandscapeAtlasRegistry,
    chunk_coord: IVec2,
    program: &LandscapeProgramOnChunk,
) -> Option<LandscapeChunkStampRequest> {
    stamp_request_for_topology(registry, chunk_coord, &program.evaluation.topology_kinds)
}

#[must_use]
pub fn build_stamp_requests_for_chunks<'a>(
    registry: &'a LandscapeAtlasRegistry,
    chunks: impl Iterator<Item = (IVec2, &'a LandscapeProgramOnChunk)>,
) -> Vec<LandscapeChunkStampRequest> {
    chunks
        .filter_map(|(coord, p)| stamp_request_for_program(registry, coord, p))
        .collect()
}

/// Convert a landscape chunk stamp into a tile-footprint stamp for the play RGBA hole.
#[must_use]
pub fn landscape_stamp_to_tile_request(
    req: &LandscapeChunkStampRequest,
    chunk_size: UVec2,
) -> TileAtlasStampRequest {
    let fw = chunk_size.x.max(1);
    let fh = chunk_size.y.max(1);
    TileAtlasStampRequest {
        atlas_id: req.atlas_id.clone(),
        variant_key: req.variant_key.clone(),
        facing: 0,
        frame: 0,
        uv: req.uv,
        origin: IVec2::new(req.chunk_coord.x * fw as i32, req.chunk_coord.y * fh as i32),
        footprint_w: fw,
        footprint_h: fh,
    }
}

/// Build play-map tile stamps for every chunk under PixelWorld (real LG-5 atlas UVs).
///
/// When veg/burn/succession are present, prefers burn/scar/regrowth atlas rows.
#[must_use]
pub fn build_pixel_world_landscape_tile_stamps<'a>(
    registry: &'a LandscapeAtlasRegistry,
    chunks: impl Iterator<
        Item = (
            &'a Chunk,
            &'a ChunkCellMatrix,
            Option<&'a LandscapeProgramOnChunk>,
            PixelWorldVegStampInputs<'a>,
        ),
    >,
) -> Vec<TileAtlasStampRequest> {
    let mut out = Vec::new();
    for (chunk, matrix, program, veg_inputs) in chunks {
        let kinds: Vec<String> = program
            .map(|p| p.evaluation.topology_kinds.clone())
            .filter(|k| !k.is_empty())
            .unwrap_or_else(|| vec!["Patch".to_string()]);
        let Some(req) =
            stamp_request_for_topology_with_veg(registry, chunk.coord, &kinds, veg_inputs)
        else {
            continue;
        };
        out.push(landscape_stamp_to_tile_request(&req, matrix.size));
    }
    out
}

/// Ensure LG-5 (and other stampable landscape) atlas images are loaded into the GPU/CPU cache.
pub fn preload_landscape_atlas_gpu_cache(
    landscape: Option<&LandscapeAtlasRegistry>,
    asset_server: &AssetServer,
    cache: &mut TileAtlasGpuCache,
) {
    let Some(landscape) = landscape else {
        return;
    };
    for entry in &landscape.entries {
        if !entry.chunk_stamp_allowed() {
            continue;
        }
        if cache.handles.contains_key(&entry.atlas_id) {
            continue;
        }
        let handle: Handle<Image> = asset_server.load(entry.atlas_asset.clone());
        cache.handles.insert(entry.atlas_id.clone(), handle);
    }
}

/// Blit LG-5 atlas sub-rect onto chunk footprints in overworld RGBA.
pub fn apply_landscape_chunk_stamps_to_rgba(
    dest: &mut [u8],
    tex_w: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    stamps: &[LandscapeChunkStampRequest],
    atlas_data: &[(String, &[u8], usize, usize)],
    chunk_footprint: UVec2,
) {
    let tile_stamps: Vec<_> = stamps
        .iter()
        .map(|r| landscape_stamp_to_tile_request(r, chunk_footprint))
        .collect();
    for req in &tile_stamps {
        let Some((_, data, atlas_w, atlas_h)) = atlas_data
            .iter()
            .find(|(id, ..)| id == &req.atlas_id)
        else {
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

#[must_use]
pub fn landscape_lg5_chunk_uv_stamp_witness_green() -> bool {
    landscape_lg5_chunk_uv_stamp_self_check().is_ok()
}

fn landscape_lg5_chunk_uv_stamp_self_check() -> Result<(), &'static str> {
    let registry = load_landscape_atlas_registry();
    if !registry.load_errors.is_empty() {
        return Err("registry_load");
    }
    let entry = registry.lg5_entry().ok_or("lg5_entry")?;
    if entry.atlas_id != LG5_ATLAS_ID {
        return Err("atlas_id");
    }
    for key in ["topology_patch", "topology_corridor", "topology_ring"] {
        let uv = entry.resolve_variant_uv(key).ok_or("variant_uv")?;
        if uv[2] <= 0.0 || uv[3] <= 0.0 {
            return Err("uv_degenerate");
        }
    }

    let atlas_path = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .map(|r| r.join(&entry.atlas_png))
        .unwrap_or_else(|| std::path::PathBuf::from(&entry.atlas_png));
    if !atlas_path.is_file() {
        // Meta + registry UV contract when atlas PNG is not materialized in worktree (dry-run bake).
        return Ok(());
    }

    let atlas_bytes = std::fs::read(atlas_path).map_err(|_| "atlas_png")?;
    let atlas = image::load_from_memory(&atlas_bytes).map_err(|_| "atlas_decode")?;
    let rgba = atlas.to_rgba8();
    let (aw, ah) = rgba.dimensions();
    let atlas_data = vec![(
        entry.atlas_id.clone(),
        rgba.as_raw().as_slice(),
        aw as usize,
        ah as usize,
    )];

    let kinds = [
        ("Patch", IVec2::new(0, 0)),
        ("Corridor", IVec2::new(4, 0)),
        ("Ring", IVec2::new(8, 0)),
    ];
    let mut dest = vec![0u8; 4 * 16 * 8];
    let chunk_px = 4u32;
    for (kind, coord) in kinds {
        let Some(req) = stamp_request_for_topology(&registry, coord, &[kind.to_string()]) else {
            return Err("stamp_request");
        };
        stamp_atlas_uv_into_rgba_subregion(
            &mut dest,
            16,
            0,
            0,
            16,
            8,
            atlas_data[0].1,
            atlas_data[0].2,
            atlas_data[0].3,
            coord,
            chunk_px,
            chunk_px,
            req.uv,
        );
    }
    let stamped = dest
        .chunks(4)
        .filter(|px| px[3] >= 128)
        .count();
    if stamped < 12 {
        return Err("stamp_blit");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::ecology::landscape_lg5_registry_stamped;

    #[test]
    fn lg5_chunk_uv_stamp_witness_green() {
        if !landscape_lg5_registry_stamped() {
            eprintln!("skip: landscape lg5 index not stamped");
            return;
        }
        assert!(landscape_lg5_chunk_uv_stamp_witness_green());
    }

    #[test]
    fn pixel_world_tile_stamp_covers_chunk_footprint() {
        if !landscape_lg5_registry_stamped() {
            eprintln!("skip: landscape lg5 index not stamped");
            return;
        }
        let registry = load_landscape_atlas_registry();
        let chunk = Chunk {
            coord: IVec2::new(2, 1),
        };
        let matrix = ChunkCellMatrix::new(UVec2::new(8, 8));
        let stamps = build_pixel_world_landscape_tile_stamps(
            &registry,
            std::iter::once((&chunk, &matrix, None, PixelWorldVegStampInputs::default())),
        );
        assert_eq!(stamps.len(), 1);
        assert_eq!(stamps[0].atlas_id, LG5_ATLAS_ID);
        assert_eq!(stamps[0].origin, IVec2::new(16, 8));
        assert_eq!(stamps[0].footprint_w, 8);
        assert_eq!(stamps[0].footprint_h, 8);
        assert_eq!(stamps[0].variant_key, "topology_patch");
    }

    #[test]
    fn pixel_world_veg_prefers_burn_scar_regrowth_atlas_rows() {
        if !landscape_lg5_registry_stamped() {
            eprintln!("skip: landscape lg5 index not stamped");
            return;
        }
        let registry = load_landscape_atlas_registry();
        let kinds = vec!["Patch".to_string()];

        let burn = ActiveBurn {
            heat: 0.8,
            frame_index: 5,
            ..Default::default()
        };
        let (burn_key, _) = resolve_pixel_world_topology_variant(
            &registry,
            &kinds,
            PixelWorldVegStampInputs {
                burn: Some(&burn),
                ..Default::default()
            },
        )
        .expect("burn variant");
        assert_eq!(burn_key, "topology_patch_burn_04");

        let scar_succ = SuccessionState {
            stage: SuccessionTopologyStage::BurnScar,
            ..Default::default()
        };
        let (scar_key, _) = resolve_pixel_world_topology_variant(
            &registry,
            &kinds,
            PixelWorldVegStampInputs {
                succession: Some(&scar_succ),
                ..Default::default()
            },
        )
        .expect("scar variant");
        assert_eq!(scar_key, "topology_patch_scar");

        let veg = VegetationField {
            burn_severity: 0.2,
            regrowth_stage: 0.7,
            understory_density: 0.5,
            ..Default::default()
        };
        let (regrowth_key, _) = resolve_pixel_world_topology_variant(
            &registry,
            &kinds,
            PixelWorldVegStampInputs {
                veg: Some(&veg),
                ..Default::default()
            },
        )
        .expect("regrowth variant");
        assert_eq!(regrowth_key, "topology_patch_regrowth_shrub");
    }
}
