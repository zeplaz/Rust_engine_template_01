//! RGR-H3-001 split — tactical VFX zoom/witness gate flags and evaluation.
//! Carved verbatim from `stage5_full_app_harness.rs` (pre-split monolith).

use bevy::prelude::*;

use crate::render::extraction::RenderProjectionGraph;
use crate::render::gpu_particles::WorldFireParticleFrame;
use crate::render::gpu_water_particles::WorldWaterParticleFrame;
use crate::render::WaterSurfaceVisualCatalog;

/// Tactical zoom band for Phase 2 VFX witness gates (matches §7 / D-F09).
pub(crate) const TACTICAL_VFX_ZOOM_ALPHA_MIN: f32 = 0.65;

/// When set, `--test visual` blocks proof commit unless [`TacticalVfxWitnessGates::all_green`].
#[inline]
pub(crate) fn tactical_vfx_proof_enabled() -> bool {
    matches!(
        std::env::var("TACTICAL_VFX_PROOF").as_deref(),
        Ok("1") | Ok("true") | Ok("on")
    )
}

/// P2-VFX-VISUAL-001: `--test visual` / VfxSandbox always require tactical particle witness before proof commit.
#[inline]
pub(crate) fn visual_tactical_vfx_witness_required(launch: &crate::engine::EngineLaunchArgs) -> bool {
    tactical_vfx_proof_enabled()
        || matches!(
            launch.test_scene,
            crate::engine::TestScene::Visual | crate::engine::TestScene::VfxSandbox
        )
}

/// Only hard-lock camera pose when explicit tactical VFX proof mode is enabled.
/// Disabled — operator scroll/wheel owns WorldMain pose; witness uses one-shot harness zoom.
#[inline]
pub(crate) fn visual_tactical_vfx_camera_lock_required() -> bool {
    false
}

/// P0-VFX-ZOOM-LOCK-001 witness — interactive scenes never hard-lock camera zoom.
#[inline]
#[must_use]
pub fn visual_tactical_vfx_camera_lock_enabled() -> bool {
    false
}

/// P0-VFX-ZOOM-LOCK-001 witness — VfxSandbox wheel is never capped by proof lock.
#[inline]
#[must_use]
pub fn vfx_sandbox_scroll_zoom_free(_launch: Option<&crate::engine::EngineLaunchArgs>) -> bool {
    true
}

/// P0-VFX-ZOOM-LOCK-001 witness — `TACTICAL_VFX_PROOF` no longer forces camera lock.
#[inline]
#[must_use]
pub fn tactical_vfx_hard_lock_enabled() -> bool {
    false
}

/// P2-VFX-WITNESS-001 / P2-WATER-WITNESS-002 JSON gate evaluation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TacticalVfxWitnessGates {
    pub fire_tactical_zoom: bool,
    pub fire_spark_rows_gt_0: bool,
    pub fire_spark_011_green: bool,
    pub water_tactical_zoom: bool,
    pub water_has_river_segments: bool,
    pub water_particle_rows_gt_0: bool,
    pub water_particle_river_streaks_when_rivers: bool,
    pub water_shader_motion_always_on: bool,
    pub water_strategic_001_shader_motion_green: bool,
    pub water_particle_strategic_not_culled: bool,
    pub water_w1_river_read_green: bool,
    pub water_strategic_001_green: bool,
    pub water_witness_001_green: bool,
    pub water_witness_foam_or_ocean_green: bool,
    pub water_w2_foam_001_green: bool,
    /// F2-PR-2 — `RenderProjectionGraph.fire.instance_buffer` non-empty at witness time.
    pub fire_instance_buffer_rows_gt_0: bool,
    /// F2-PR-2 — sparks routed from graph buffer (not overlay/chunk_heat bootstrap).
    pub fire_projection_graph_native: bool,
    /// F2-PR-3 — primary path fell back to overlay heat seeding.
    pub fire_degraded_overlay_bootstrap: bool,
    /// F2-PR-1 — particle snapshot stamp matches graph projection stamp.
    pub fire_projection_stamp_aligned: bool,
}

impl TacticalVfxWitnessGates {
    pub(crate) fn evaluate(
        particles: Option<&WorldFireParticleFrame>,
        water_catalog: Option<&WaterSurfaceVisualCatalog>,
        water_particles: Option<&WorldWaterParticleFrame>,
        projection: Option<&RenderProjectionGraph>,
    ) -> Self {
        let fire_zoom = particles
            .map(|p| p.spark_witness.zoom_alpha)
            .unwrap_or(0.0);
        let fire_tactical = fire_zoom >= TACTICAL_VFX_ZOOM_ALPHA_MIN;
        let fire_rows = particles.map(|p| p.spark_witness.rows).unwrap_or(0);
        let fire_spark_011 = particles
            .map(|p| crate::render::gpu_particles::fire_spark_011_green(&p.spark_witness))
            .unwrap_or(false);

        let water_zoom = water_particles
            .map(|p| p.witness.zoom_alpha)
            .unwrap_or(0.0);
        let water_tactical = water_zoom >= TACTICAL_VFX_ZOOM_ALPHA_MIN;
        let water_has_rivers = water_catalog
            .map(|c| !c.river_segments.is_empty())
            .unwrap_or(false);
        let water_rows = water_particles.map(|p| p.witness.rows).unwrap_or(0);
        let water_streaks = water_particles
            .map(|p| p.witness.river_streaks)
            .unwrap_or(0);
        let water_bands = water_catalog.map(|c| {
            crate::render::gpu_water_particles::evaluate_water_vfx_witness_bands(
                c,
                water_zoom.max(crate::render::gpu_water_particles::WATER_TACTICAL_WITNESS_ZOOM_ALPHA),
                0.0,
            )
        });
        let water_shader_particle = water_particles
            .map(|p| p.witness.shader_motion_always_on)
            .or_else(|| {
                water_bands
                    .as_ref()
                    .map(|b| b.tactical.shader_motion_always_on)
            })
            .unwrap_or(false);
        let water_shader_w1 = water_catalog
            .map(crate::render::water_strategic_001_shader_motion_green)
            .unwrap_or(true);
        let water_shader_on = water_shader_particle && water_shader_w1;
        let water_not_culled = water_particles
            .map(|p| !p.witness.strategic_culled)
            .unwrap_or(false);
        let water_river_read = water_catalog
            .map(|c| {
                c.w1_river_read_green_at_zoom(
                    crate::render::water_surface_visual::WATER_STRATEGIC_ZOOM_ALPHA * 0.5,
                )
            })
            .unwrap_or(false);
        let water_strategic_001 = water_bands
            .as_ref()
            .map(crate::render::gpu_water_particles::water_strategic_001_green)
            .unwrap_or(false);
        let water_witness_001 = water_catalog
            .zip(water_bands)
            .map(|(c, b)| crate::render::gpu_water_particles::water_witness_001_green(c, &b))
            .unwrap_or(false);
        let water_foam_or_ocean = water_catalog
            .zip(water_bands)
            .map(|(c, b)| {
                crate::render::gpu_water_particles::water_witness_foam_or_ocean_green(
                    c,
                    &b.tactical,
                )
            })
            .unwrap_or(false);
        let water_w2_foam = water_catalog
            .zip(water_bands)
            .map(|(c, b)| crate::render::gpu_water_particles::water_w2_foam_001_green(c, &b))
            .unwrap_or(false);

        let buffer_rows = projection
            .map(|g| g.fire.instance_buffer.len())
            .unwrap_or(0);
        let proj_view = particles
            .map(|p| p.spark_witness.projection_view)
            .unwrap_or("");
        let fire_degraded_overlay_bootstrap = proj_view == "overlay_bootstrap"
            && std::env::var("RUST_ENGINE_FIRE_DEGRADED_OVERLAY")
                .ok()
                .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
        let fire_projection_graph_native = buffer_rows > 0
            && !fire_degraded_overlay_bootstrap
            && proj_view != "chunk_heat_fallback";
        let fire_projection_stamp_aligned = match (projection, particles) {
            (Some(graph), Some(p)) => graph.fire.snapshot_stamp == p.snapshot_stamp,
            _ => true,
        };

        Self {
            fire_tactical_zoom: fire_tactical,
            fire_spark_rows_gt_0: fire_rows > 0,
            fire_spark_011_green: fire_spark_011,
            fire_instance_buffer_rows_gt_0: buffer_rows > 0,
            fire_projection_graph_native,
            fire_degraded_overlay_bootstrap,
            fire_projection_stamp_aligned,
            water_tactical_zoom: water_tactical,
            water_has_river_segments: water_has_rivers,
            water_particle_rows_gt_0: water_rows > 0,
            water_particle_river_streaks_when_rivers: !water_has_rivers || water_streaks > 0,
            water_shader_motion_always_on: water_shader_on,
            water_strategic_001_shader_motion_green: water_shader_w1,
            water_particle_strategic_not_culled: water_not_culled,
            water_w1_river_read_green: water_river_read,
            water_strategic_001_green: water_strategic_001,
            water_witness_001_green: water_witness_001,
            water_witness_foam_or_ocean_green: water_foam_or_ocean,
            water_w2_foam_001_green: water_w2_foam,
        }
    }

    /// WATER-STRATEGIC-001 — dual-band strategic cull + shader motion (D-W09).
    #[must_use]
    pub(crate) fn water_strategic_gates_green(&self) -> bool {
        self.water_strategic_001_green && self.water_shader_motion_always_on
    }

    /// WATER-WITNESS-001 rollup (tactical foam/ocean + strategic band); separate from live zoom rows.
    #[must_use]
    pub(crate) fn water_witness_rollup_green(&self) -> bool {
        self.water_strategic_gates_green()
            && self.water_witness_001_green
            && self.water_w2_foam_001_green
    }

    #[must_use]
    pub(crate) fn all_green(&self) -> bool {
        self.all_green_for_visual_proof(false)
    }

    #[must_use]
    pub(crate) fn all_green_for_visual_proof(&self, require_fire_rows: bool) -> bool {
        let fire_tactical_ok = if require_fire_rows {
            self.fire_tactical_zoom && self.fire_spark_rows_gt_0
        } else {
            !self.fire_tactical_zoom || self.fire_spark_rows_gt_0
        };
        let fire_tune_ok = !self.fire_tactical_zoom || self.fire_spark_011_green;
        let fire_ok = fire_tactical_ok && fire_tune_ok;
        let water_live_ok = !self.water_tactical_zoom
            || (self.water_particle_rows_gt_0
                && self.water_particle_river_streaks_when_rivers
                && self.water_particle_strategic_not_culled);
        let water_ok = self.water_strategic_gates_green() && water_live_ok;
        fire_ok && water_ok
    }
}

/// Re-stamp water particles after tactical camera fix (same zoom resource as fire).
pub(crate) fn refresh_visual_proof_water_particles(
    launch: Option<Res<crate::engine::EngineLaunchArgs>>,
    time: Res<Time>,
    catalog: Option<Res<WaterSurfaceVisualCatalog>>,
    cam: Res<crate::render::gpu_particles::FireParticleCameraScale>,
    mut frame: ResMut<WorldWaterParticleFrame>,
) {
    let Some(_launch) = launch.as_ref() else {
        return;
    };
    if !visual_tactical_vfx_camera_lock_required() {
        return;
    }
    let Some(catalog) = catalog.as_ref() else {
        return;
    };
    let mut cam_snap = *cam;
    cam_snap.zoom_alpha = cam_snap
        .zoom_alpha
        .max(crate::render::gpu_water_particles::WATER_TACTICAL_WITNESS_ZOOM_ALPHA);
    crate::render::gpu_water_particles::update_world_water_particles_from_catalog(
        catalog,
        frame.as_mut(),
        cam_snap,
        time.elapsed_secs(),
    );
}

/// Re-stamp fire particles after tactical camera fix (Update emit may have used strategic zoom).
pub(crate) fn refresh_visual_proof_fire_particles(
    launch: Option<Res<crate::engine::EngineLaunchArgs>>,
    overlay: Res<crate::render::SharedOverlayFieldBuffers>,
    graph: Res<crate::render::extraction::RenderProjectionGraph>,
    chunk_lod: Res<crate::render::FireChunkLodState>,
    cam: Res<crate::render::gpu_particles::FireParticleCameraScale>,
    view_manager: Option<Res<crate::gui::ViewManager>>,
    mut particles: ResMut<WorldFireParticleFrame>,
) {
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !visual_tactical_vfx_witness_required(launch) {
        return;
    }
    crate::render::gpu_particles::update_world_fire_particles_from_projection(
        graph.as_ref(),
        particles.as_mut(),
        Some(chunk_lod.as_ref()),
        *cam,
        view_manager.as_deref(),
    );
    if particles.spark_witness.rows > 0 {
        return;
    }
    // F2-PR-3: do not overlay-bootstrap when graph already projected instance rows.
    if !graph.fire.instance_buffer.is_empty() {
        return;
    }
    // Projection graph empty (view cull / stamp) but overlay has seeded fire — witness bootstrap.
    if overlay.chunk_fire_heat.is_empty() {
        return;
    }
    crate::render::gpu_particles::seed_world_fire_particles_from_overlay_heat(
        &overlay.chunk_fire_heat,
        particles.as_mut(),
        *cam,
    );
}
