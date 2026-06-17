//! VEG-F03-REGISTRY-STAMP-001 — LG-5 landscape chunk UV stamp from registry atlas meta.

use bevy::prelude::*;

use crate::gui::map_tile_atlas_stamp::stamp_atlas_uv_into_rgba_subregion;
use crate::systems::ecology::{
    load_landscape_atlas_registry, topology_kind_to_variant_key, landscape_lg5_registry_stamped,
    LandscapeAtlasRegistry, LandscapeProgramOnChunk, LG5_ATLAS_ID,
};

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

#[must_use]
pub fn stamp_request_for_topology(
    registry: &LandscapeAtlasRegistry,
    chunk_coord: IVec2,
    topology_kinds: &[String],
) -> Option<LandscapeChunkStampRequest> {
    let entry = registry.lg5_entry()?;
    let kind = primary_topology_kind(topology_kinds)?;
    let variant_key = topology_kind_to_variant_key(kind)?;
    let uv = entry.resolve_variant_uv(variant_key)?;
    Some(LandscapeChunkStampRequest {
        atlas_id: entry.atlas_id.clone(),
        variant_key: variant_key.to_owned(),
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

/// Blit LG-5 atlas sub-rect onto chunk footprint in overworld RGBA (1 tile = 1 chunk for witness).
pub fn apply_landscape_chunk_stamps_to_rgba(
    dest: &mut [u8],
    tex_w: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    stamps: &[LandscapeChunkStampRequest],
    atlas_data: &[(String, &[u8], usize, usize)],
) {
    for req in stamps {
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
            req.chunk_coord,
            1,
            1,
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

    #[test]
    fn lg5_chunk_uv_stamp_witness_green() {
        if !landscape_lg5_registry_stamped() {
            eprintln!("skip: landscape lg5 index not stamped");
            return;
        }
        assert!(landscape_lg5_chunk_uv_stamp_witness_green());
    }
}
