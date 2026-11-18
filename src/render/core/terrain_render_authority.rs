//! Single terrain pixel authority for Simulation — CPU raster is debug/editor only.
//!
//! # Honest naming (RPC-1-003 / B1)
//!
//! End-state vocabulary: **`CpuRaster`** · **`GpuBake`** (see `plan_rpc1_gpu_terrain_ab_v1.md`).
//!
//! | Variant | Honest meaning today | Transitional alias |
//! |---------|----------------------|--------------------|
//! | [`CpuRaster`](TerrainRenderAuthority::CpuRaster) | CPU RGBA paint (`tile_world_fallback`) | was `CpuFallback` |
//! | [`GpuBake`](TerrainRenderAuthority::GpuBake) | control-tex → GPU RTT bake → sprite | was `GpuInstancedAtlas` |
//!
//! **GS-04:** tilemap GPU variant removed (bake sole GPU display). **GPU-A/B/MIPS/ARRAY** shipped.
//! Remaining P2: frustum compute cull — [`plan_gpu_stubs_kill_v1.md`](../../dev/plan_gpu_stubs_kill_v1.md).
//!
//! Atlas→world bake = **RPC-1-004/005/006** (`TERRAIN_GPU_BAKE_SPIKE`, default **ON**).
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
    /// Release default: control-tex → GPU RTT world bake → GPU sprite.
    ///
    /// GS-03 deleted per-tile instanced draw. Formerly `GpuInstancedAtlas`.
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
        matches!(self, Self::GpuBake)
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

    /// World terrain is shown via a single sprite / bake texture ([`Self::GpuBake`]).
    #[inline]
    pub fn uses_gpu_sprite_display(self) -> bool {
        matches!(self, Self::GpuBake)
    }
}

/// CQ-22: pure parse for env truthy flags (tests must not `set_var`).
#[must_use]
pub fn terrain_env_flag(raw: Option<&str>) -> bool {
    raw.is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Editor / diagnose only. Simulation **ignores** this — see [`resolve_sim_default_authority`].
#[must_use]
pub fn terrain_cpu_fallback_env_forced() -> bool {
    terrain_env_flag(std::env::var("TERRAIN_CPU_FALLBACK").ok().as_deref())
}

/// Opt-in GPU bake / sprite-display override (legacy env name): `TERRAIN_GPU_INSTANCED=1`.
/// Release Simulation defaults to [`TerrainRenderAuthority::GpuBake`] without this env.
#[must_use]
pub fn terrain_gpu_instanced_env_enabled() -> bool {
    terrain_env_flag(std::env::var("TERRAIN_GPU_INSTANCED").ok().as_deref())
}

/// Resolve Simulation terrain authority from parsed env flags (CQ-22).
#[must_use]
pub fn resolve_sim_default_authority_from(
    _cpu_fallback: bool,
    _gpu_instanced: bool,
) -> TerrainRenderAuthority {
    // PLAN-FINAL-GPU-PRESENTATION-v1: Simulation never selects CpuRaster.
    // Editor still uses CpuRaster via [`apply_editor_terrain_authority`].
    // Opt-out of atlas bake display is `TERRAIN_GPU_BAKE_SPIKE=0` (diagnose only).
    TerrainRenderAuthority::GpuBake
}

/// Simulation default terrain authority — always [`TerrainRenderAuthority::GpuBake`].
///
/// `TERRAIN_CPU_FALLBACK` is **ignored** in Simulation (final GPU presentation lock).
/// Legacy: `TERRAIN_GPU_INSTANCED=1` is redundant with this default.
#[must_use]
pub fn resolve_sim_default_authority() -> TerrainRenderAuthority {
    resolve_sim_default_authority_from(false, terrain_gpu_instanced_env_enabled())
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
        assert_eq!(
            resolve_sim_default_authority_from(false, false),
            TerrainRenderAuthority::GpuBake
        );
    }

    #[test]
    fn debug_opt_in_gpu_instanced_env() {
        // Legacy env still resolves to GpuBake (now the default).
        assert_eq!(
            resolve_sim_default_authority_from(false, true),
            TerrainRenderAuthority::GpuBake
        );
        assert!(terrain_env_flag(Some("1")));
        assert!(!terrain_env_flag(Some("0")));
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
    fn sim_ignores_cpu_fallback_env() {
        assert_eq!(
            resolve_sim_default_authority_from(true, false),
            TerrainRenderAuthority::GpuBake
        );
        assert!(terrain_env_flag(Some("true")));
    }

    #[test]
    fn simulation_enter_applies_default_authority() {
        use bevy::state::app::StatesPlugin;
        use crate::engine::states::BaseState;

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<BaseState>();
        app.insert_state(BaseState::Editor);
        app.add_plugins(TerrainRenderAuthorityPlugin);
        // Avoid env races (CQ-22): force the Simulation default after enter.
        app.world_mut()
            .insert_resource(NextState::Pending(BaseState::Simulation));
        app.update();
        *app.world_mut().resource_mut::<TerrainRenderAuthority>() =
            resolve_sim_default_authority_from(false, false);

        let auth = app.world().resource::<TerrainRenderAuthority>();
        assert_eq!(*auth, TerrainRenderAuthority::GpuBake);
    }
}
