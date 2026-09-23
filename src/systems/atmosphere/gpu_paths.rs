//! Atmosphere WGSL path constants.
//!
//! **ES-4 ALTERNATIVE (quarantine):** composite stubs live under
//! `assets/shaders/experiments/atmosphere/` — **not** production AssetServer loads.
//! Live weather/fire field compute remains under `shaders/post/`.

/// ES-4: composites not wired into the render graph this session.
pub const ATMOSPHERE_COMPOSITE_WIRED: bool = false;
/// ES-4: unwired WGSL moved out of production `shaders/atmosphere/`.
pub const ATMOSPHERE_WGSL_QUARANTINED: bool = true;

/// Relative to `assets/` — quarantined experiment paths (do not `AssetServer::load` in prod).
pub const ATMOSPHERE_GROUND_HAZE_WGSL: &str = "shaders/experiments/atmosphere/ground_haze.wgsl";
pub const ATMOSPHERE_SMOKE_COLUMN_WGSL: &str = "shaders/experiments/atmosphere/smoke_column.wgsl";
pub const ATMOSPHERE_HEAT_DISTORTION_WGSL: &str =
    "shaders/experiments/atmosphere/heat_distortion.wgsl";
pub const ATMOSPHERE_ASHFALL_WGSL: &str = "shaders/experiments/atmosphere/ashfall.wgsl";
pub const ATMOSPHERE_PARTICLE_INSTANCING_WGSL: &str =
    "shaders/experiments/atmosphere/particle_instancing.wgsl";
pub const ATMOSPHERE_FIELD_PAGE_TABLE_WGSL: &str =
    "shaders/experiments/atmosphere/field_page_table.wgsl";

/// Ping-pong weather + fire **field** compute (`GpuWeatherFireFieldPlugin`) — production path.
pub const WEATHER_FIRE_FIELD_WGSL: &str = "shaders/post/weather_fire_field.wgsl";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atmosphere_composite_quarantine_flags() {
        assert!(!ATMOSPHERE_COMPOSITE_WIRED);
        assert!(ATMOSPHERE_WGSL_QUARANTINED);
    }

    #[test]
    fn quarantined_wgsl_under_experiments_and_non_empty() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        for rel in [
            "assets/shaders/experiments/atmosphere/ground_haze.wgsl",
            "assets/shaders/experiments/atmosphere/smoke_column.wgsl",
            "assets/shaders/experiments/atmosphere/heat_distortion.wgsl",
            "assets/shaders/experiments/atmosphere/ashfall.wgsl",
            "assets/shaders/experiments/atmosphere/particle_instancing.wgsl",
            "assets/shaders/experiments/atmosphere/field_page_table.wgsl",
            "assets/shaders/post/weather_fire_field.wgsl",
        ] {
            let p = root.join(rel);
            let s = std::fs::read_to_string(&p)
                .unwrap_or_else(|e| panic!("read {}: {e}", p.display()));
            assert!(s.contains("@compute"), "{rel} should contain a compute entry");
        }
        // Production tree must not claim live composite shaders.
        let prod_atm = root.join("assets/shaders/atmosphere");
        assert!(
            !prod_atm.is_dir()
                || std::fs::read_dir(&prod_atm)
                    .map(|d| d.filter_map(|e| e.ok()).count() == 0)
                    .unwrap_or(true),
            "assets/shaders/atmosphere/ must be empty or absent after ES-4 quarantine"
        );
        for c in [
            ATMOSPHERE_GROUND_HAZE_WGSL,
            ATMOSPHERE_SMOKE_COLUMN_WGSL,
            ATMOSPHERE_HEAT_DISTORTION_WGSL,
            ATMOSPHERE_ASHFALL_WGSL,
            ATMOSPHERE_PARTICLE_INSTANCING_WGSL,
            ATMOSPHERE_FIELD_PAGE_TABLE_WGSL,
        ] {
            assert!(
                c.starts_with("shaders/experiments/atmosphere/"),
                "prod path const must point at experiments: {c}"
            );
        }
    }
}
