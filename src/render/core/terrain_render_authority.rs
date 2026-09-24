//! Single terrain pixel authority for Simulation — CPU raster is debug/editor only.
//!
//! # Honest naming (RPC-1-003 / B1)
//!
//! End-state vocabulary: **`CpuRaster`** · **`GpuBake`** (see `plan_rpc1_gpu_terrain_ab_v1.md`).
//!
//! | Variant | Honest meaning today | Transitional alias |
//! |---------|----------------------|--------------------|
//! | [`CpuRaster`](TerrainRenderAuthority::CpuRaster) | CPU RGBA paint (`tile_world_fallback`) | was `CpuFallback` |
//! | [`GpuBake`](TerrainRenderAuthority::GpuBake) | CPU dirty-bake → world texture → GPU sprite | was `GpuInstancedAtlas` (did **not** instanced-draw) |
//! | [`GpuTilemap`](TerrainRenderAuthority::GpuTilemap) | Deferred (`DR-MIG-TILEMAP`) — **never constructed** | keep gated until milestone |
//!
//! Real GPU atlas→world bake spike = **RPC-1-004/005** (`TERRAIN_GPU_BAKE_SPIKE`, default OFF).
//! Minimap may label `gpu_bake` only when spike consumers bind the bake Image (RPC-1-005).

use bevy::prelude::*;

use crate::engine::states::BaseState;

/// Which path owns main-map terrain pixels this frame.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Default, Reflect)]
pub enum TerrainRenderAuthority {
    /// CPU `tile_world_fallback` RGBA paint + sprite (editor / explicit rollback).
    /// Honest end-state name for this path; rustdoc alias: `CpuFallback`.
    #[doc(alias = "CpuFallback")]
    #[default]
    CpuRaster,
    /// `bevy_ecs_tilemap` GPU layer — **deferred** (`DR-MIG-TILEMAP`).
    /// Never constructed today; keep the variant for the milestone, do not select it in
    /// `resolve_sim_default_authority` until tilemap draw is unblocked.
    GpuTilemap,
    /// Release default: CPU dirty-gated bake → world texture displayed as a GPU sprite.
    ///
    /// **Not** per-tile GPU atlas instancing yet (`terrain_instanced_draw` remains dormant
    /// while [`uses_gpu_sprite_display`](Self::uses_gpu_sprite_display) is true). Opt-in A0
    /// bake index feed: `TERRAIN_GPU_BAKE_SPIKE=1` ([`crate::render::pipelines::terrain_gpu_bake_spike`]).
    /// Formerly `GpuInstancedAtlas`.
    #[doc(alias = "GpuInstancedAtlas")]
    GpuBake,
}

impl TerrainRenderAuthority {
    /// Transitional const alias — prefer [`Self::CpuRaster`].
    #[allow(non_upper_case_globals)]
    pub const CpuFallback: Self = Self::CpuRaster;

    /// Transitional const alias — prefer [`Self::GpuBake`].
    /// Historical name implied instanced atlas draw; path is CPU dirty-bake → texture.
    #[allow(non_upper_case_globals)]
    pub const GpuInstancedAtlas: Self = Self::GpuBake;

    #[inline]
    pub fn is_gpu(self) -> bool {
        matches!(self, Self::GpuTilemap | Self::GpuBake)
    }

    /// True when this frame uses the CPU RGBA paint path ([`Self::CpuRaster`]).
    #[inline]
    pub fn uses_cpu_raster(self) -> bool {
        matches!(self, Self::CpuRaster)
    }

    /// Transitional name for [`Self::uses_cpu_raster`] (RPC-1-003).
    #[inline]
    pub fn uses_cpu_fallback_raster(self) -> bool {
        self.uses_cpu_raster()
    }

    /// World terrain is shown via a single sprite texture (dirty-gated CPU bake uploaded to GPU).
    /// Applies to deferred [`Self::GpuTilemap`] and release-default [`Self::GpuBake`].
    /// Per-tile instancing stays inactive while this returns true.
    #[inline]
    pub fn uses_gpu_sprite_display(self) -> bool {
        matches!(self, Self::GpuTilemap | Self::GpuBake)
    }
}

/// Debug rollback: `TERRAIN_CPU_FALLBACK=1` forces CPU paint in any build.
#[must_use]
pub fn terrain_cpu_fallback_env_forced() -> bool {
    std::env::var("TERRAIN_CPU_FALLBACK")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Opt-in GPU bake / sprite-display override (legacy env name): `TERRAIN_GPU_INSTANCED=1`.
/// Release Simulation defaults to [`TerrainRenderAuthority::GpuBake`] without this env.
#[must_use]
pub fn terrain_gpu_instanced_env_enabled() -> bool {
    std::env::var("TERRAIN_GPU_INSTANCED")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Simulation default terrain authority (P0-C′-PRIME / RPC-1-003 naming).
///
/// Default: [`TerrainRenderAuthority::GpuBake`] (dirty-gated CPU bake → GPU sprite)
/// in **both** debug and release Simulation.
/// Rollback: `TERRAIN_CPU_FALLBACK=1` forces CPU paint in any build.
/// Legacy: `TERRAIN_GPU_INSTANCED=1` still selects GpuBake (redundant with default).
#[must_use]
pub fn resolve_sim_default_authority() -> TerrainRenderAuthority {
    if terrain_cpu_fallback_env_forced() {
        return TerrainRenderAuthority::CpuRaster;
    }
    // GpuBake is the Simulation default (debug + release). Tilemap remains deferred.
    let _ = terrain_gpu_instanced_env_enabled();
    TerrainRenderAuthority::GpuBake
}

pub fn apply_simulation_terrain_authority(mut authority: ResMut<TerrainRenderAuthority>) {
    *authority = resolve_sim_default_authority();
}

/// Editor always uses CPU-rastered terrain — its live-paint tools are only wired to
/// `tile_world_fallback`, and world preview authority owns editor GPU display separately. There
/// is no GPU terrain path in Editor, so `TERRAIN_CPU_FALLBACK` has nothing to roll back here.
pub fn apply_editor_terrain_authority(mut authority: ResMut<TerrainRenderAuthority>) {
    *authority = TerrainRenderAuthority::CpuRaster;
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
    fn sim_default_gpu_bake_unless_cpu_rollback() {
        std::env::remove_var("TERRAIN_CPU_FALLBACK");
        std::env::remove_var("TERRAIN_GPU_INSTANCED");
        assert_eq!(
            resolve_sim_default_authority(),
            TerrainRenderAuthority::GpuBake
        );
    }

    #[test]
    fn debug_opt_in_gpu_instanced_env() {
        // Legacy env still resolves to GpuBake (now the default).
        std::env::remove_var("TERRAIN_CPU_FALLBACK");
        std::env::set_var("TERRAIN_GPU_INSTANCED", "1");
        assert_eq!(
            resolve_sim_default_authority(),
            TerrainRenderAuthority::GpuBake
        );
        std::env::remove_var("TERRAIN_GPU_INSTANCED");
    }

    #[test]
    fn gpu_authority_skips_cpu_fallback_raster_metric() {
        assert!(!TerrainRenderAuthority::GpuBake.uses_cpu_raster());
        assert!(!TerrainRenderAuthority::GpuBake.uses_cpu_fallback_raster());
        assert!(TerrainRenderAuthority::GpuBake.uses_gpu_sprite_display());
    }

    #[test]
    fn transitional_aliases_match_honest_variants() {
        assert_eq!(
            TerrainRenderAuthority::CpuFallback,
            TerrainRenderAuthority::CpuRaster
        );
        assert_eq!(
            TerrainRenderAuthority::GpuInstancedAtlas,
            TerrainRenderAuthority::GpuBake
        );
    }

    #[test]
    fn env_rollback_forces_cpu() {
        std::env::set_var("TERRAIN_CPU_FALLBACK", "1");
        let auth = resolve_sim_default_authority();
        assert_eq!(auth, TerrainRenderAuthority::CpuRaster);
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
        assert_eq!(*auth, TerrainRenderAuthority::GpuBake);
    }
}
