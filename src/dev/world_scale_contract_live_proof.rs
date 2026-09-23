//! **VSS-T1-002** — world scale contract witness (`debug_runs/world_scale_contract_live.json`).

use bevy::prelude::*;

use crate::terrain::world_map_scale::{TileExtentPreset, WorldMapScale};
use crate::terrain::world_scale_contract::{
    BUILDING_TYPICAL_FOOTPRINT_TILES, CHUNK_TILES_SIM, OPERATIONAL_ZOOM_ALPHA,
    OPERATIONAL_PX_PER_TILE, SITE_TYPICAL_ENVELOPE_TILES, SIM_ENTRY_ZOOM_ALPHA_TOLERANCE,
    TACTICAL_PX_PER_TILE, TACTICAL_PROOF_ZOOM_ALPHA, TILE_METERS_SYMBOLIC,
    WORLD_DEFAULT_TILES_AXIS, contract_mirror_json, map_zoom_legacy_fit_enabled,
};

pub const WORLD_SCALE_CONTRACT_LIVE_JSON: &str = "debug_runs/world_scale_contract_live.json";

/// Lib-computed measurements for the default interactive sim-entry path (512² world).
#[derive(Clone, Copy, Debug)]
pub struct WorldScaleContractMeasured {
    pub world_tiles_axis: u32,
    pub world_height_tiles: u32,
    pub meters_per_tile: f32,
    pub chunk_tiles_sim: u32,
    pub chunk_tiles_display: Option<u32>,
    pub sim_entry_zoom_alpha: f32,
    pub sim_entry_px_per_tile: f32,
    pub operational_px_per_tile: f32,
    pub chunk_slab_screen_px: f32,
    pub building_typical_screen_px: [f32; 2],
    pub buildings_per_frame: f32,
    pub site_envelope_screen_px: [f32; 2],
    pub viewport_logical_px: [f32; 2],
}

/// Deterministic viewport for witness math (matches map_zoom_coherence lib harness).
fn witness_map_viewport() -> crate::gui::SimulationMapViewport {
    let mut vp = crate::gui::SimulationMapViewport::default();
    vp.valid = true;
    vp.min = Vec2::new(100.0, 50.0);
    vp.max = Vec2::new(900.0, 550.0);
    vp
}

#[must_use]
pub fn compute_interactive_sim_entry_zoom(
    world_tiles_axis: u32,
    window_px: Vec2,
    map_vp: &crate::gui::SimulationMapViewport,
) -> (f32, f32, f32) {
    let world_w = world_tiles_axis as f32;
    let world_h = world_tiles_axis as f32;
    let viewport = crate::gui::map_camera_viewport_pixels(window_px, Some(map_vp));
    let (zoom_lo, zoom_hi) =
        crate::gui::map_zoom_limits_for_world(world_w, world_h, viewport);
    let zoom = if map_zoom_legacy_fit_enabled() {
        let margin = crate::terrain::world_scale_contract::WHOLE_MAP_FIT_MARGIN;
        let fit = margin * (viewport.x / world_w).min(viewport.y / world_h);
        fit.clamp(zoom_lo, zoom_hi)
    } else {
        crate::gui::map_scale_for_zoom_alpha(OPERATIONAL_ZOOM_ALPHA, zoom_lo, zoom_hi)
    };
    let alpha = crate::gui::map_zoom_alpha_with_limits(zoom, zoom_lo, zoom_hi);
    (zoom, alpha, viewport.x)
}

#[must_use]
pub fn compute_world_scale_contract_measured() -> WorldScaleContractMeasured {
    let world_tiles_axis = WORLD_DEFAULT_TILES_AXIS;
    let window_px = Vec2::new(1280.0, 720.0);
    let vp = witness_map_viewport();
    let (sim_entry_px, sim_entry_alpha, viewport_w) =
        compute_interactive_sim_entry_zoom(world_tiles_axis, window_px, &vp);
    let viewport = crate::gui::map_camera_viewport_pixels(window_px, Some(&vp));

    let world_w = world_tiles_axis as f32;
    let world_h = world_tiles_axis as f32;
    let (zoom_lo, zoom_hi) =
        crate::gui::map_zoom_limits_for_world(world_w, world_h, viewport);
    let operational_px = crate::gui::map_scale_for_zoom_alpha(
        OPERATIONAL_ZOOM_ALPHA,
        zoom_lo,
        zoom_hi,
    );

    let chunk_slab_screen_px = CHUNK_TILES_SIM as f32 * sim_entry_px;
    let building_typical_screen_px = [
        BUILDING_TYPICAL_FOOTPRINT_TILES.0 as f32 * sim_entry_px,
        BUILDING_TYPICAL_FOOTPRINT_TILES.1 as f32 * sim_entry_px,
    ];
    let site_envelope_screen_px = [
        SITE_TYPICAL_ENVELOPE_TILES.0 as f32 * sim_entry_px,
        SITE_TYPICAL_ENVELOPE_TILES.1 as f32 * sim_entry_px,
    ];
    let buildings_per_frame = if building_typical_screen_px[0] > 0.0 {
        viewport_w / building_typical_screen_px[0]
    } else {
        0.0
    };

    WorldScaleContractMeasured {
        world_tiles_axis,
        world_height_tiles: world_tiles_axis,
        meters_per_tile: WorldMapScale::default().meters_per_tile,
        chunk_tiles_sim: CHUNK_TILES_SIM,
        chunk_tiles_display: None,
        sim_entry_zoom_alpha: sim_entry_alpha,
        sim_entry_px_per_tile: sim_entry_px,
        operational_px_per_tile: operational_px,
        chunk_slab_screen_px,
        building_typical_screen_px,
        buildings_per_frame,
        site_envelope_screen_px,
        viewport_logical_px: [viewport.x, viewport.y],
    }
}

#[must_use]
pub fn world_scale_contract_p2_green(measured: &WorldScaleContractMeasured) -> bool {
    if map_zoom_legacy_fit_enabled() {
        return false;
    }
    (measured.sim_entry_zoom_alpha - OPERATIONAL_ZOOM_ALPHA).abs()
        <= SIM_ENTRY_ZOOM_ALPHA_TOLERANCE
        && measured.chunk_tiles_sim == CHUNK_TILES_SIM
        && (measured.meters_per_tile - TILE_METERS_SYMBOLIC).abs() < 1e-3
        && measured.world_tiles_axis == WORLD_DEFAULT_TILES_AXIS
        && (measured.operational_px_per_tile - measured.sim_entry_px_per_tile).abs() < 1e-3
}

#[must_use]
pub fn world_scale_contract_witness_json(measured: &WorldScaleContractMeasured) -> serde_json::Value {
    let green = world_scale_contract_p2_green(measured);
    serde_json::json!({
        "schema": "world_scale_contract_witness_v1",
        "status": if green { "green" } else { "measured" },
        "program_id": "VSS-001",
        "slice_id": "VSS-T1-002",
        "green": green,
        "contract": contract_mirror_json(),
        "anchors": {
            "fire_operational_px_per_tile": OPERATIONAL_PX_PER_TILE,
            "fire_full_scatter_px_per_tile": TACTICAL_PX_PER_TILE,
            "tactical_proof_zoom_alpha": TACTICAL_PROOF_ZOOM_ALPHA,
            "design_operational_zoom_alpha": OPERATIONAL_ZOOM_ALPHA,
        },
        "measured": {
            "world_tiles_axis": measured.world_tiles_axis,
            "world_height_tiles": measured.world_height_tiles,
            "meters_per_tile": measured.meters_per_tile,
            "chunk_tiles_sim": measured.chunk_tiles_sim,
            "chunk_tiles_display": measured.chunk_tiles_display,
            "sim_entry_zoom_alpha": measured.sim_entry_zoom_alpha,
            "sim_entry_px_per_tile": measured.sim_entry_px_per_tile,
            "operational_px_per_tile": measured.operational_px_per_tile,
            "chunk_slab_screen_px": measured.chunk_slab_screen_px,
            "building_typical_screen_px": measured.building_typical_screen_px,
            "buildings_per_frame": measured.buildings_per_frame,
            "site_envelope_screen_px": measured.site_envelope_screen_px,
            "viewport_logical_px": measured.viewport_logical_px,
        },
        "footprint_pilot_audit": {
            "median_primary_tiles": [BUILDING_TYPICAL_FOOTPRINT_TILES.0, BUILDING_TYPICAL_FOOTPRINT_TILES.1],
            "largest_primary_tiles": [6, 5],
            "typical_site_envelope_tiles": [SITE_TYPICAL_ENVELOPE_TILES.0, SITE_TYPICAL_ENVELOPE_TILES.1],
            "source": "assets/configs/buildings/_pilot_catalog.ron"
        },
        "presets": {
            "harness_tiles_axis": TileExtentPreset::MediumSmall.tiles_per_axis(),
            "editor_default_tiles_axis": WORLD_DEFAULT_TILES_AXIS,
        },
        "tribunal": {
            "designer_signed_off": false,
            "dissent": []
        },
        "legacy_fit_env": map_zoom_legacy_fit_enabled(),
        "notes": "VSS-T1-002 — interactive sim-entry commits OPERATIONAL_ZOOM_ALPHA; test scenes/scenario override."
    })
}

#[must_use]
pub fn refresh_world_scale_contract_live_witness() -> bool {
    let measured = compute_world_scale_contract_measured();
    if !world_scale_contract_p2_green(&measured) {
        return false;
    }
    let body = world_scale_contract_witness_json(&measured);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "VSS-T1-002",
        "refresh_world_scale_contract_live_witness",
        WORLD_SCALE_CONTRACT_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(WORLD_SCALE_CONTRACT_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_scale_sim_entry_zoom_alpha_on_512_default_world() {
        let measured = compute_world_scale_contract_measured();
        assert!(
            (measured.sim_entry_zoom_alpha - OPERATIONAL_ZOOM_ALPHA).abs()
                <= SIM_ENTRY_ZOOM_ALPHA_TOLERANCE,
            "alpha={} expected≈{}",
            measured.sim_entry_zoom_alpha,
            OPERATIONAL_ZOOM_ALPHA
        );
        assert_eq!(measured.chunk_tiles_sim, CHUNK_TILES_SIM);
        assert_eq!(measured.world_tiles_axis, WORLD_DEFAULT_TILES_AXIS);
    }

    #[test]
    fn world_scale_contract_live_witness_refresh_green() {
        assert!(refresh_world_scale_contract_live_witness());
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(WORLD_SCALE_CONTRACT_LIVE_JSON);
        assert!(path.is_file(), "witness missing at {}", path.display());
        let raw = std::fs::read_to_string(&path).expect("read witness");
        let v: serde_json::Value = serde_json::from_str(&raw).expect("parse witness");
        let alpha = v
            .pointer("/measured/sim_entry_zoom_alpha")
            .and_then(|x| x.as_f64())
            .expect("sim_entry_zoom_alpha");
        assert!(
            (alpha - OPERATIONAL_ZOOM_ALPHA as f64).abs() <= SIM_ENTRY_ZOOM_ALPHA_TOLERANCE as f64,
            "alpha={alpha}"
        );
        assert_eq!(v.get("green").and_then(|x| x.as_bool()), Some(true));
    }
}
