//! Single terrain pixel authority for Simulation — CPU fallback is debug/editor only.

use bevy::prelude::*;

use crate::engine::states::BaseState;

/// Which path owns main-map terrain pixels this frame.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Default, Reflect)]
pub enum TerrainRenderAuthority {
    /// CPU `tile_world_fallback` RGBA paint + sprite (editor / explicit rollback).
    #[default]
    CpuFallback,
    /// `bevy_ecs_tilemap` GPU layer (ECS sync; render when tilemap draw enabled).
    GpuTilemap,
    /// Instanced atlas quads sampling [`crate::render::terrain_material_atlas::TerrainMaterialAtlasGpu`].
    GpuInstancedAtlas,
}

impl TerrainRenderAuthority {
    #[inline]
    pub fn is_gpu(self) -> bool {
        matches!(self, Self::GpuTilemap | Self::GpuInstancedAtlas)
    }

    #[inline]
    pub fn uses_cpu_fallback_raster(self) -> bool {
        matches!(self, Self::CpuFallback)
    }

    /// GPU terrain uses a single world sprite texture (dirty-gated bake) on the tactical RTT.
    /// Applies to both [`GpuTilemap`] and release-default [`GpuInstancedAtlas`]; per-tile
    /// instancing remains wired but inactive until this returns false for instanced authority.
    #[inline]
    pub fn uses_gpu_sprite_display(self) -> bool {
        matches!(self, Self::GpuTilemap | Self::GpuInstancedAtlas)
    }
}

/// Debug rollback: `TERRAIN_CPU_FALLBACK=1` forces CPU paint in any build.
#[must_use]
pub fn terrain_cpu_fallback_env_forced() -> bool {
    std::env::var("TERRAIN_CPU_FALLBACK")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Opt-in GPU instanced atlas override (legacy): `TERRAIN_GPU_INSTANCED=1`.
/// Release Simulation defaults to [`TerrainRenderAuthority::GpuInstancedAtlas`] without this env.
#[must_use]
pub fn terrain_gpu_instanced_env_enabled() -> bool {
    std::env::var("TERRAIN_GPU_INSTANCED")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Simulation default terrain authority (P0-C′-PRIME).
///
/// Release: [`TerrainRenderAuthority::GpuInstancedAtlas`] (dirty-gated GPU sprite display).
/// Debug: [`TerrainRenderAuthority::CpuFallback`] unless `TERRAIN_GPU_INSTANCED=1`.
/// Rollback: `TERRAIN_CPU_FALLBACK=1` forces CPU paint in any build.
#[must_use]
pub fn resolve_sim_default_authority() -> TerrainRenderAuthority {
    if terrain_cpu_fallback_env_forced() {
        return TerrainRenderAuthority::CpuFallback;
    }
    if terrain_gpu_instanced_env_enabled() {
        return TerrainRenderAuthority::GpuInstancedAtlas;
    }
    #[cfg(not(debug_assertions))]
    {
        return TerrainRenderAuthority::GpuInstancedAtlas;
    }
    #[cfg(debug_assertions)]
    {
        TerrainRenderAuthority::CpuFallback
    }
}

pub fn apply_simulation_terrain_authority(mut authority: ResMut<TerrainRenderAuthority>) {
    *authority = resolve_sim_default_authority();
}

/// Editor always uses CPU-rastered terrain — its live-paint tools are only wired to
/// `tile_world_fallback`, and world preview authority owns editor GPU display separately. There
/// is no GPU terrain path in Editor, so `TERRAIN_CPU_FALLBACK` has nothing to roll back here.
pub fn apply_editor_terrain_authority(mut authority: ResMut<TerrainRenderAuthority>) {
    *authority = TerrainRenderAuthority::CpuFallback;
}

pub struct TerrainRenderAuthorityPlugin;

impl Plugin for TerrainRenderAuthorityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainRenderAuthority>()
            .add_systems(OnEnter(BaseState::Simulation), apply_simulation_terrain_authority)
            .add_systems(OnEnter(BaseState::Editor), apply_editor_terrain_authority);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_default_gpu_in_release_cpu_in_debug() {
        std::env::remove_var("TERRAIN_CPU_FALLBACK");
        std::env::remove_var("TERRAIN_GPU_INSTANCED");
        let auth = resolve_sim_default_authority();
        if cfg!(debug_assertions) {
            assert_eq!(auth, TerrainRenderAuthority::CpuFallback);
        } else {
            assert_eq!(auth, TerrainRenderAuthority::GpuInstancedAtlas);
        }
    }

    #[test]
    fn debug_opt_in_gpu_instanced_env() {
        if !cfg!(debug_assertions) {
            return;
        }
        std::env::remove_var("TERRAIN_CPU_FALLBACK");
        std::env::set_var("TERRAIN_GPU_INSTANCED", "1");
        assert_eq!(
            resolve_sim_default_authority(),
            TerrainRenderAuthority::GpuInstancedAtlas
        );
        std::env::remove_var("TERRAIN_GPU_INSTANCED");
    }

    #[test]
    fn gpu_authority_skips_cpu_fallback_raster_metric() {
        assert!(!TerrainRenderAuthority::GpuInstancedAtlas.uses_cpu_fallback_raster());
        assert!(TerrainRenderAuthority::GpuInstancedAtlas.uses_gpu_sprite_display());
    }

    #[test]
    fn env_rollback_forces_cpu() {
        std::env::set_var("TERRAIN_CPU_FALLBACK", "1");
        let auth = resolve_sim_default_authority();
        assert_eq!(auth, TerrainRenderAuthority::CpuFallback);
        std::env::remove_var("TERRAIN_CPU_FALLBACK");
    }

    #[test]
    fn simulation_enter_applies_default_authority() {
        use bevy::state::app::StatesPlugin;
        use crate::engine::states::BaseState;

        std::env::remove_var("TERRAIN_CPU_FALLBACK");
        std::env::remove_var("TERRAIN_GPU_INSTANCED");

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<BaseState>();
        app.insert_state(BaseState::Editor);
        app.add_plugins(TerrainRenderAuthorityPlugin);

        app.world_mut()
            .insert_resource(NextState::Pending(BaseState::Simulation));
        app.update();

        let auth = app.world().resource::<TerrainRenderAuthority>();
        if cfg!(debug_assertions) {
            assert_eq!(*auth, TerrainRenderAuthority::CpuFallback);
        } else {
            assert_eq!(*auth, TerrainRenderAuthority::GpuInstancedAtlas);
        }
    }
}
