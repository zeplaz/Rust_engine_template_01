//! P0-TERRAIN-BLOB-001 — `debug_runs/terrain_blob_diag_live.json` lib refresh.

pub const TERRAIN_BLOB_DIAG_LIVE_JSON: &str = "debug_runs/terrain_blob_diag_live.json";

#[must_use]
pub fn refresh_terrain_blob_diag_live_witness() -> bool {
    use crate::render::{
        terrain_blob_diag_witness_body, TileWorldFallbackRasterCtrl, TileWorldFallbackRasterDirty,
        TileWorldFallbackState,
    };

    let state = TileWorldFallbackState {
        last_w: 512,
        last_h: 512,
        ..Default::default()
    };
    let dirty = TileWorldFallbackRasterDirty::default();
    let mut ctrl = TileWorldFallbackRasterCtrl::default();
    ctrl.chunk_grid.resize_for_world(512, 512);
    ctrl.chunk_grid.mark_chunk(2, 2);

    let body = terrain_blob_diag_witness_body(&state, &dirty, &ctrl);
    if body["seam_aligned_at_center"].as_bool() != Some(true) {
        return false;
    }

    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "P0-TERRAIN-BLOB-001",
        "refresh_terrain_blob_diag_live_witness",
        TERRAIN_BLOB_DIAG_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(TERRAIN_BLOB_DIAG_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_blob_diag_live_witness_refresh_green() {
        assert!(refresh_terrain_blob_diag_live_witness());
    }
}
