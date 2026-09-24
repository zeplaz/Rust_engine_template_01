//! `AssetServer` handles for fire/smoke WGSL (`gfx-shader-1`, `base_gui_next.md` Stage 3).

use bevy::prelude::*;

pub const FIRE_PARTICLE_WGSL: &str = "shaders/fire/fire_particle.wgsl";
pub const FIRE_SPARK_COMPUTE_WGSL: &str = "shaders/fire/fire_spark_compute.wgsl";
// DEBT-009: `assets/shaders/fire/smoke_volume.wgsl` stays on disk for ES-4 — do not AssetServer::load.

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
    fn fire_smoke_wgsl_stubs_exist() {
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
    fn smoke_volume_not_loaded_until_composite() {
        // Production loader only pulls fire_particle — smoke_volume stays on disk for ES-4.
        assert!(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/shaders/fire/smoke_volume.wgsl")
            .is_file());
    }
}
