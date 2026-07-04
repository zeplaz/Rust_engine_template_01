//! **CITY-P1-001** — block-scale static scene tagging (MIG-A1/A2 rollup).

use super::block_street_visual::{BlockStreetFurniturePiece, BlockStreetFurnitureRoot};
use crate::render::mig_a1_static_transform_optimizations_enabled;

pub const CITY_P1_LIVE_JSON: &str = "debug_runs/city_p1_001_live.json";

/// Street furniture entities participate in the same static-bulk query as PG-2 modules.
pub type BlockStaticBulkMarker = (BlockStreetFurnitureRoot, BlockStreetFurniturePiece);

#[must_use]
pub fn city_p1_static_scene_witness_green() -> bool {
    super::block_street_visual::block_street_visual_fixture_witness_green()
        && mig_a1_static_transform_optimizations_enabled()
}

#[must_use]
pub fn build_city_p1_witness_body() -> serde_json::Value {
    let static_opts = mig_a1_static_transform_optimizations_enabled();
    let street_fixture = super::block_street_visual::block_street_visual_fixture_witness_green();
    let green = static_opts && street_fixture;
    serde_json::json!({
        "gate": "CITY-P1-001",
        "green": green,
        "static_transform_optimizations_default": static_opts,
        "block_street_fixture_green": street_fixture,
        "mig_a_tags": ["NoCpuCulling", "MigAStaticBulk", "BlockStreetFurnitureRoot", "BlockStreetFurniturePiece"],
        "rollup_witness": "debug_runs/mig_bevy_019/mig_a1_a2_a16_enabled.json",
    })
}

#[must_use]
pub fn city_p1_witness_green() -> bool {
    build_city_p1_witness_body()
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[must_use]
pub fn refresh_city_p1_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_city_p1_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "CITY-P1-001",
        "refresh_city_p1_witness",
        CITY_P1_LIVE_JSON,
        body,
    );
    write_debug_run_json(CITY_P1_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_p1_witness_green_lib() {
        assert!(city_p1_witness_green());
    }
}
