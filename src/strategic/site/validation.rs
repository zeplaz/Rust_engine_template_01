//! Unified placement validation (P2-B) — scores + warnings, not only bool.

use bevy::prelude::{IVec2, Query};

use crate::strategic::build_order::BuildSiteTile;
use crate::strategic::network_flow::sample_network_flow_at_world_tile;
use crate::strategic::transport_bridge::StrategicRasterConfig;
use crate::strategic::ChunkStrategicOverlay;

use super::resources::FootprintTiles;

/// Terrain gate for site placement — replace with slope / hydrology / zoning queries.
#[inline]
pub fn validate_terrain_for_site() -> bool {
    true
}

/// Network reachability gate — replace with graph / distance-to-road queries.
#[inline]
pub fn validate_network_access_for_site() -> bool {
    true
}

/// Legacy combined bool; prefer [`evaluate_site_placement_stubs`] for scores + warnings (AI / UX).
#[inline]
pub fn validate_site_placement_stubs() -> bool {
    validate_terrain_for_site() && validate_network_access_for_site()
}

/// Result of validation for ghost UX + AI scoring.
#[derive(Clone, Debug, Default)]
pub struct SitePlacementValidation {
    pub valid: bool,
    /// Final gate for commit (may add policy beyond `valid`).
    pub allows_commit: bool,
    pub terrain_score: f32,
    pub logistics_score: f32,
    pub strategic_score: f32,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Stub evaluator — replace with slope / hydrology / graph / overlay queries.
pub fn evaluate_site_placement_stubs() -> SitePlacementValidation {
    let t_ok = validate_terrain_for_site();
    let n_ok = validate_network_access_for_site();
    let valid = t_ok && n_ok;
    let mut v = SitePlacementValidation {
        valid,
        allows_commit: false,
        terrain_score: if t_ok { 1.0 } else { 0.0 },
        logistics_score: if n_ok { 1.0 } else { 0.0 },
        strategic_score: 1.0,
        errors: Vec::new(),
        warnings: Vec::new(),
    };
    if !t_ok {
        v.errors.push("terrain".to_string());
    }
    if !n_ok {
        v.errors.push("network_access".to_string());
    }
    v.allows_commit = v.valid && v.errors.is_empty();
    v
}

/// World-tile placement using shared stubs plus optional overlay flow sampling.
///
/// When overlay samples are trivial (zeros), logistics scoring stays on the stub path so
/// games without flow solvers do not false-fail commits.
pub fn evaluate_site_placement_at_world_tile(
    origin: BuildSiteTile,
    _footprint: FootprintTiles,
    _config: Option<&StrategicRasterConfig>,
    overlays: &Query<&ChunkStrategicOverlay>,
) -> SitePlacementValidation {
    let mut v = evaluate_site_placement_stubs();
    let sample = sample_network_flow_at_world_tile(
        overlays,
        IVec2::new(origin.x as i32, origin.z as i32),
    );
    let activity = sample.power_flow.abs()
        + sample.logistics_flow.abs()
        + sample.control_pressure.abs()
        + sample.visibility.abs();
    if activity > 1e-4 {
        v.logistics_score = sample.logistics_flow.clamp(0.0, 1.0);
        if sample.logistics_flow < 0.05 {
            v.warnings.push("sparse_logistics_reach".to_string());
        }
    }
    v.allows_commit = v.valid && v.errors.is_empty();
    v
}

#[cfg(test)]
mod placement_tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{App, MinimalPlugins};

    #[test]
    fn world_tile_eval_allows_commit_when_stubs_pass() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let _ = app.world_mut()
            .run_system_once(|overlay: Query<&ChunkStrategicOverlay>| {
                let v = evaluate_site_placement_at_world_tile(
                    BuildSiteTile { x: 1, z: 1 },
                    FootprintTiles { width: 1, depth: 1 },
                    None,
                    &overlay,
                );
                assert!(v.allows_commit, "{v:?}");
            });
    }
}
