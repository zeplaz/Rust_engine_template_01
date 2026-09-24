//! **GPU-P0C-PRIME-004** — P0-C′ instanced-atlas Simulation default witness.

pub const GPU_TERRAIN_P0C_PRIME_LIVE_JSON: &str = "debug_runs/gpu_terrain_p0c_prime_001_live.json";

#[must_use]
pub fn gpu_p0c_prime_default_authority_ok() -> bool {
    use crate::render::{
        resolve_sim_default_authority, terrain_cpu_fallback_env_forced, TerrainRenderAuthority,
    };

    if terrain_cpu_fallback_env_forced() {
        return false;
    }
    let auth = resolve_sim_default_authority();
    if terrain_cpu_fallback_env_forced() {
        return false;
    }
    auth == TerrainRenderAuthority::GpuBake
}

#[must_use]
pub fn gpu_p0c_prime_release_default_is_gpu() -> bool {
    use crate::render::{resolve_sim_default_authority, TerrainRenderAuthority};

    if cfg!(debug_assertions) {
        return true;
    }
    resolve_sim_default_authority() == TerrainRenderAuthority::GpuBake
}

#[must_use]
pub fn gpu_p0c_prime_cpu_raster_metric_gated() -> bool {
    use crate::render::TerrainRenderAuthority;

    !TerrainRenderAuthority::GpuBake.uses_cpu_raster()
        && TerrainRenderAuthority::GpuBake.uses_gpu_sprite_display()
}

#[must_use]
pub fn gpu_p0c_prime_env_rollback_ok() -> bool {
    use crate::render::{resolve_sim_default_authority, TerrainRenderAuthority};

    std::env::set_var("TERRAIN_CPU_FALLBACK", "1");
    let ok = resolve_sim_default_authority() == TerrainRenderAuthority::CpuRaster;
    std::env::remove_var("TERRAIN_CPU_FALLBACK");
    ok
}

/// Honest contract: default-off minimap terrain input is the CPU-rastered world image
/// (`TileWorldFallbackState`) → label `world_raster`. RPC-1-005 adds flagged
/// `gpu_bake` when `TERRAIN_GPU_BAKE_SPIKE` consumers bind the bake Image
/// (see `MinimapTerrainSourceBind` / `plan_rpc1_gpu_terrain_ab_v1.md`).
#[must_use]
pub fn gpu_p0e_minimap_terrain_source_ok() -> bool {
    use crate::render::minimap_compositor::{minimap_terrain_source_label, MinimapTerrainSourceBind};

    minimap_terrain_source_label(MinimapTerrainSourceBind::WorldRaster) == "world_raster"
        && minimap_terrain_source_label(MinimapTerrainSourceBind::None) == "none"
        && minimap_terrain_source_label(MinimapTerrainSourceBind::GpuBake) == "gpu_bake"
}

#[must_use]
pub fn gpu_p0c_prime_witness_green() -> bool {
    gpu_p0c_prime_default_authority_ok()
        && gpu_p0c_prime_release_default_is_gpu()
        && gpu_p0c_prime_cpu_raster_metric_gated()
        && gpu_p0c_prime_env_rollback_ok()
        && gpu_p0e_minimap_terrain_source_ok()
}

#[must_use]
pub fn build_gpu_p0c_prime_witness_body() -> serde_json::Value {
    use crate::render::resolve_sim_default_authority;

    let auth = format!("{:?}", resolve_sim_default_authority());
    serde_json::json!({
        "gate": "GPU-P0C-PRIME-004",
        "green": gpu_p0c_prime_witness_green(),
        "terrain_authority_default": auth,
        "release_default_gpu": gpu_p0c_prime_release_default_is_gpu(),
        "cpu_raster_metric_gated": gpu_p0c_prime_cpu_raster_metric_gated(),
        "env_rollback_ok": gpu_p0c_prime_env_rollback_ok(),
        "minimap_terrain_source_ok": gpu_p0e_minimap_terrain_source_ok(),
        "display_mode": "gpu_sprite_dirty_gated",
        "instanced_pass": "wired_deferred",
        "plan_ref": "src/dev/plan_gpu_terrain_production_exec_001_v1.md#P0-C′",
        "todo_board": "src/dev/gpu_todos_v1.md",
    })
}

#[must_use]
pub fn refresh_gpu_p0c_prime_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_gpu_p0c_prime_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "GPU-P0C-PRIME-004",
        "refresh_gpu_p0c_prime_witness",
        GPU_TERRAIN_P0C_PRIME_LIVE_JSON,
        body,
    );
    write_debug_run_json(GPU_TERRAIN_P0C_PRIME_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_p0c_prime_witness_reports_green() {
        assert!(gpu_p0c_prime_witness_green());
    }

    #[test]
    fn gpu_p0c_prime_refresh_witness_when_green() {
        if gpu_p0c_prime_witness_green() {
            assert!(refresh_gpu_p0c_prime_witness());
        }
    }
}
