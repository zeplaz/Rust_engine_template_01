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
    /// Instanced atlas quads sampling [`super::terrain_material_atlas::TerrainMaterialAtlasGpu`].
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

    /// GPU terrain uses a single world sprite texture (dirty-gated bake), not per-frame CPU paint.
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

/// Opt-in GPU instanced atlas (P0-C′ experimental): `TERRAIN_GPU_INSTANCED=1`.
#[must_use]
pub fn terrain_gpu_instanced_env_enabled() -> bool {
    std::env::var("TERRAIN_GPU_INSTANCED")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Release Simulation default: CPU fallback raster (minimap + full-world paint).
/// GPU instanced atlas is opt-in via [`terrain_gpu_instanced_env_enabled`].
#[must_use]
pub fn resolve_sim_default_authority() -> TerrainRenderAuthority {
    if terrain_cpu_fallback_env_forced() {
        return TerrainRenderAuthority::CpuFallback;
    }
    if terrain_gpu_instanced_env_enabled() {
        return TerrainRenderAuthority::GpuInstancedAtlas;
    }
    TerrainRenderAuthority::CpuFallback
}

pub fn apply_simulation_terrain_authority(mut authority: ResMut<TerrainRenderAuthority>) {
    *authority = resolve_sim_default_authority();
}

pub fn apply_editor_terrain_authority(mut authority: ResMut<TerrainRenderAuthority>) {
    if terrain_cpu_fallback_env_forced() {
        *authority = TerrainRenderAuthority::CpuFallback;
    } else {
        *authority = TerrainRenderAuthority::CpuFallback;
    }
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
    fn sim_default_cpu_until_gpu_opt_in() {
        std::env::remove_var("TERRAIN_CPU_FALLBACK");
        std::env::remove_var("TERRAIN_GPU_INSTANCED");
        let auth = resolve_sim_default_authority();
        assert_eq!(auth, TerrainRenderAuthority::CpuFallback);
        std::env::set_var("TERRAIN_GPU_INSTANCED", "1");
        let auth = resolve_sim_default_authority();
        assert_eq!(auth, TerrainRenderAuthority::GpuInstancedAtlas);
        std::env::remove_var("TERRAIN_GPU_INSTANCED");
    }

    #[test]
    fn env_rollback_forces_cpu() {
        std::env::set_var("TERRAIN_CPU_FALLBACK", "1");
        let auth = resolve_sim_default_authority();
        assert_eq!(auth, TerrainRenderAuthority::CpuFallback);
        std::env::remove_var("TERRAIN_CPU_FALLBACK");
    }
}
