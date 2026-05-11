//! Unified placement validation (P2-B) — scores + warnings, not only bool.

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
    v
}
