//! MAP-SIM-RELIEF-001 — play-map ridge lighter than hollow (lib proof + witness).

use crate::gui::editor::world_preview::PreviewLayers;
use crate::gui::map_tile_raster::raster_sim_minimap_layered_to_rgba;
use crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID;

pub const MAP_SIM_RELIEF_001_LIVE_JSON: &str = "debug_runs/map_sim_relief_001_live.json";

/// Ridge (peak corner) green channel vs hollow (center) after play-map relief shading.
#[must_use]
pub fn play_map_ridge_hollow_green() -> (u8, u8) {
    let mut data = vec![0u8; 4 * 3 * 3];
    let fam = DEFAULT_TERRAIN_FAMILY_ID;
    let tiles = [
        (0usize, 1usize, fam, 0.8f32, 0.0, 0.0),
        (1, 0, fam, 0.8, 0.0, 0.0),
        (1, 1, fam, 0.2, 0.0, 0.0),
        (1, 2, fam, 0.8, 0.0, 0.0),
        (2, 1, fam, 0.8, 0.0, 0.0),
        (0, 0, fam, 0.2, 0.0, 0.0),
        (2, 0, fam, 0.2, 0.0, 0.0),
        (0, 2, fam, 0.2, 0.0, 0.0),
        (2, 2, fam, 0.95, 0.0, 0.0),
    ];
    raster_sim_minimap_layered_to_rgba(
        &mut data,
        3,
        3,
        tiles.into_iter(),
        std::iter::empty(),
        PreviewLayers::BIOME,
        &[],
        None,
        None,
    );
    let hollow = data[4 * (1 * 3 + 1) + 1];
    let ridge = data[4 * (2 * 3 + 2) + 1];
    (ridge, hollow)
}

#[must_use]
pub fn map_sim_relief_001_witness_green() -> bool {
    let (ridge, hollow) = play_map_ridge_hollow_green();
    ridge > hollow
}

#[must_use]
pub fn refresh_map_sim_relief_001_live_witness() -> bool {
    let (ridge_g, hollow_g) = play_map_ridge_hollow_green();
    let green = ridge_g > hollow_g;
    let body = serde_json::json!({
        "gate": "MAP-SIM-RELIEF-001",
        "slice_id": "MAP-SIM-RELIEF-001",
        "program_id": "PLAN-LANDFORM-VEG-DRAIN-001",
        "green": green,
        "exit": "play_map_ridge_is_lighter_than_hollow",
        "exit_test": "gui::map_tile_raster::tests::play_map_ridge_is_lighter_than_hollow",
        "authority": "landform_relief_rgba via raster_sim_minimap_layered_to_rgba",
        "relief_shared_with_preview": true,
        "second_chunk_derived_metrics_writer": false,
        "env_flag_added": false,
        "ridge_green": ridge_g,
        "hollow_green": hollow_g,
        "ridge_lighter_than_hollow": green,
        "queue": "tools/orchestrator/queues/landform_veg_drain_v1.json",
        "board": "src/dev/landform_veg_drain_todos_v1.md",
        "docs": [
            "src/gui/map_tile_raster.rs",
            "src/gui/editor/world_preview/overlays.rs",
            "src/terrain/generation/derived.rs"
        ],
        "blocker": null,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "MAP-SIM-RELIEF-001",
        "refresh_map_sim_relief_001_live_witness",
        MAP_SIM_RELIEF_001_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(MAP_SIM_RELIEF_001_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_sim_relief_001_live_witness_green() {
        assert!(
            refresh_map_sim_relief_001_live_witness(),
            "MAP-SIM-RELIEF-001: ridge must be lighter than hollow on play-map raster"
        );
    }
}
