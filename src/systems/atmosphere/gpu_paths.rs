//! Asset paths + `include_str!` smoke for atmosphere WGSL stubs (`atm-ren-1a`, `atm-part-1a`).
//! Layout: `assets/shaders/atmosphere/*.wgsl` (`gfx-shader-2`); ping-pong field under `post/`.

/// Relative to `assets/` (Bevy `AssetServer::load`).
pub const ATMOSPHERE_GROUND_HAZE_WGSL: &str = "shaders/atmosphere/ground_haze.wgsl";
pub const ATMOSPHERE_SMOKE_COLUMN_WGSL: &str = "shaders/atmosphere/smoke_column.wgsl";
pub const ATMOSPHERE_HEAT_DISTORTION_WGSL: &str = "shaders/atmosphere/heat_distortion.wgsl";
pub const ATMOSPHERE_ASHFALL_WGSL: &str = "shaders/atmosphere/ashfall.wgsl";
pub const ATMOSPHERE_PARTICLE_INSTANCING_WGSL: &str = "shaders/atmosphere/particle_instancing.wgsl";

/// Ping-pong weather + fire **field** compute (`GpuWeatherFireFieldPlugin`).
pub const WEATHER_FIRE_FIELD_WGSL: &str = "shaders/post/weather_fire_field.wgsl";

#[cfg(test)]
mod tests {
    #[test]
    fn stub_wgsl_files_exist_and_non_empty() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        for rel in [
            "assets/shaders/atmosphere/ground_haze.wgsl",
            "assets/shaders/atmosphere/smoke_column.wgsl",
            "assets/shaders/atmosphere/heat_distortion.wgsl",
            "assets/shaders/atmosphere/ashfall.wgsl",
            "assets/shaders/atmosphere/particle_instancing.wgsl",
            "assets/shaders/post/weather_fire_field.wgsl",
        ] {
            let p = root.join(rel);
            let s = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()));
            assert!(s.contains("@compute"), "{rel} should contain a compute entry");
        }
    }
}
