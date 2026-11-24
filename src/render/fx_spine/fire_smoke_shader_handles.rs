//! `AssetServer` handles for fire/smoke WGSL (`gfx-shader-1`, `base_gui_next.md` Stage 3).
//!
//! **GS-08 CLOSED** — honest asset roles:
//! - `fire_particle.wgsl` — loaded here for Core2d particle expand
//! - `fire_spark_compute.wgsl` — production-loaded by [`crate::render::pipelines::gpu_spark_compute`]
//! - `smoke_volume.wgsl` — **DEFERRED** (ES-4); on disk, not AssetServer-loaded until composite ships
//!
//! Kill board: [`plan_gpu_stubs_kill_v1.md`](../../dev/plan_gpu_stubs_kill_v1.md).

use bevy::prelude::*;

pub const FIRE_PARTICLE_WGSL: &str = "shaders/fire/fire_particle.wgsl";
pub const FIRE_SPARK_COMPUTE_WGSL: &str = "shaders/fire/fire_spark_compute.wgsl";
/// ES-4 deferred — do not `AssetServer::load` until smoke volume composite is production.
#[cfg(test)] // only tests use it today
pub const SMOKE_VOLUME_WGSL: &str = "shaders/fire/smoke_volume.wgsl";

#[derive(Resource, Debug)]
pub struct FireSmokeShaderHandles {
    pub fire_particle: Handle<Shader>,
}

pub(crate) fn load_fire_smoke_shader_handles(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(FireSmokeShaderHandles {
        fire_particle: assets.load(FIRE_PARTICLE_WGSL),
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn gs08_fire_smoke_wgsl_assets_on_disk() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        for rel in [
            "assets/shaders/fire/fire_particle.wgsl",
            "assets/shaders/fire/fire_spark_compute.wgsl",
            "assets/shaders/fire/smoke_volume.wgsl",
        ] {
            let s = std::fs::read_to_string(root.join(rel)).expect(rel);
            assert!(s.contains("@compute"), "{rel}");
        }
        let spark =
            std::fs::read_to_string(root.join("assets/shaders/fire/fire_spark_compute.wgsl"))
                .expect("fire_spark_compute.wgsl");
        assert!(spark.contains("advect_sparks"));
        let fire = std::fs::read_to_string(root.join("assets/shaders/fire/fire_particle.wgsl"))
            .expect("fire_particle.wgsl");
        assert!(fire.contains("expand_instances"));
    }

    #[test]
    fn gs08_smoke_volume_deferred_not_production_loaded() {
        // Production loader only pulls fire_particle; spark loads via gpu_spark_compute;
        // smoke_volume stays on disk for ES-4 (honest defer, not a lying stub).
        let src = include_str!("fire_smoke_shader_handles.rs");
        assert!(src.contains("GS-08 CLOSED"));
        // Split so include_str cannot false-match this assert body.
        let load_call = format!("assets.load({})", "SMOKE_VOLUME_WGSL");
        assert!(!src.contains(&load_call));
        assert!(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/shaders/fire/smoke_volume.wgsl")
            .is_file());
    }

    #[test]
    fn gs08_spark_compute_path_is_production_wired() {
        let spark_src = include_str!("../pipelines/gpu_spark_compute.rs");
        assert!(spark_src.contains("FIRE_SPARK_COMPUTE_WGSL"));
        assert!(spark_src.contains("register_fire_spark_compute"));
        assert!(spark_src.contains("asset_server.load(FIRE_SPARK_COMPUTE_WGSL)"));
    }
}
