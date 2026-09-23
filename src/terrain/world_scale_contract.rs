//! Published world-scale contract — single authority for sim tile units, chunk slabs,
//! building footprint norms, camera operational anchors, and VFX px-per-tile gates.
//!
//! **Sim:** 1 tile = 1 world unit on XZ ([`TILE_SIM_UNIT`]). **Symbolic:** [`TILE_METERS_SYMBOLIC`]
//! for lore / world-gen rhythm only. **Display grid split** (8-tile tactical) is VSS-T1-004 — not here.
//!
//! Plan: `src/dev/plan_world_scale_contract_v1.md`

/// One logical sim tile = one world unit on XZ (unchanged).
pub const TILE_SIM_UNIT: f32 = 1.0;

/// Symbolic metres per tile (lore / WG rhythm — not survey-grade).
pub const TILE_METERS_SYMBOLIC: f32 = 100.0;

/// Sim substrate chunk slab — WSS, fire tick, weather, dense cache.
pub const CHUNK_TILES_SIM: u32 = 32;

/// Tactical display grid target (overlay / minimap district) — contract only until VSS-T1-004.
pub const CHUNK_TILES_DISPLAY: u32 = 8;

/// Typical primary building footprint (median pilot catalog mass).
pub const BUILDING_TYPICAL_FOOTPRINT_TILES: (u32, u32) = (3, 3);

/// Largest pilot primary shape (`logistics_rail_warehouse_l_6x5`).
pub const BUILDING_LARGE_FOOTPRINT_TILES: (u32, u32) = (6, 5);

/// Typical site envelope (yard + rail void inside site box).
pub const SITE_TYPICAL_ENVELOPE_TILES: (u32, u32) = (10, 8);

/// Editor default world extent per axis.
pub const WORLD_DEFAULT_TILES_AXIS: u32 = 512;

/// Visual harness / medium-small play world per axis.
pub const WORLD_HARNESS_TILES_AXIS: u32 = 320;

/// Designer operational play anchor (`design_zoom_fire_read_v1.md`).
pub const OPERATIONAL_ZOOM_ALPHA: f32 = 0.42;

/// Sparse sparks read at operational play (FIRE-VIS-001).
pub const OPERATIONAL_PX_PER_TILE: f32 = 2.5;

/// Full scatter density ceiling (px-per-tile axis).
pub const TACTICAL_PX_PER_TILE: f32 = 4.0;

/// `--test visual` / harness proof band — not G-PLAY default.
pub const TACTICAL_PROOF_ZOOM_ALPHA: f32 = 0.85;

/// Designer target: typical 3×3 primaries visible per operational frame.
pub const BUILDINGS_PER_FRAME_TARGET: (u32, u32) = (4, 9);

/// ~8 tiles across the shorter viewport edge at max zoom-in (`map_zoom_limits_for_world`).
pub const TILE_MIN_SCREEN_PX: f32 = 8.0;

/// Whole-map fit margin used by legacy sim-entry and Fire/Atmosphere harness paths.
pub const WHOLE_MAP_FIT_MARGIN: f32 = 0.9;

/// Green band for measured sim-entry zoom alpha vs [`OPERATIONAL_ZOOM_ALPHA`].
pub const SIM_ENTRY_ZOOM_ALPHA_TOLERANCE: f32 = 0.05;

/// One-release compat: restore whole-map fit on interactive `OnEnter(Simulation)`.
#[inline]
#[must_use]
pub fn map_zoom_legacy_fit_enabled() -> bool {
    matches!(
        std::env::var("RUST_ENGINE_MAP_ZOOM_LEGACY_FIT").as_deref(),
        Ok("1") | Ok("true") | Ok("on")
    )
}

/// Contract mirror for witness JSON (`contract` object).
#[must_use]
pub fn contract_mirror_json() -> serde_json::Value {
    serde_json::json!({
        "TILE_SIM_UNIT": TILE_SIM_UNIT,
        "TILE_METERS_SYMBOLIC": TILE_METERS_SYMBOLIC,
        "CHUNK_TILES_SIM": CHUNK_TILES_SIM,
        "CHUNK_TILES_DISPLAY": CHUNK_TILES_DISPLAY,
        "BUILDING_TYPICAL_FOOTPRINT_TILES": [BUILDING_TYPICAL_FOOTPRINT_TILES.0, BUILDING_TYPICAL_FOOTPRINT_TILES.1],
        "BUILDING_LARGE_FOOTPRINT_TILES": [BUILDING_LARGE_FOOTPRINT_TILES.0, BUILDING_LARGE_FOOTPRINT_TILES.1],
        "SITE_TYPICAL_ENVELOPE_TILES": [SITE_TYPICAL_ENVELOPE_TILES.0, SITE_TYPICAL_ENVELOPE_TILES.1],
        "WORLD_DEFAULT_TILES_AXIS": WORLD_DEFAULT_TILES_AXIS,
        "WORLD_HARNESS_TILES_AXIS": WORLD_HARNESS_TILES_AXIS,
        "OPERATIONAL_ZOOM_ALPHA": OPERATIONAL_ZOOM_ALPHA,
        "OPERATIONAL_PX_PER_TILE": OPERATIONAL_PX_PER_TILE,
        "TACTICAL_PX_PER_TILE": TACTICAL_PX_PER_TILE,
        "TACTICAL_PROOF_ZOOM_ALPHA": TACTICAL_PROOF_ZOOM_ALPHA,
        "BUILDINGS_PER_FRAME_TARGET": [BUILDINGS_PER_FRAME_TARGET.0, BUILDINGS_PER_FRAME_TARGET.1],
        "TILE_MIN_SCREEN_PX": TILE_MIN_SCREEN_PX,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_scale_contract_chunk_slab_is_32() {
        assert_eq!(CHUNK_TILES_SIM, 32);
        assert_eq!(CHUNK_TILES_DISPLAY, 8);
    }

    #[test]
    fn world_scale_contract_operational_zoom_alpha() {
        assert!((OPERATIONAL_ZOOM_ALPHA - 0.42).abs() < 1e-6);
    }

    #[test]
    fn world_scale_contract_fire_px_per_tile_anchors() {
        assert_eq!(OPERATIONAL_PX_PER_TILE, 2.5);
        assert_eq!(TACTICAL_PX_PER_TILE, 4.0);
        assert_eq!(TACTICAL_PROOF_ZOOM_ALPHA, 0.85);
    }
}
