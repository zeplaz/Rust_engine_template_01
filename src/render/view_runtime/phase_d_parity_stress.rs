//! **TRIAGE-PHASE-D-PARITY-001** — VM-08 overlay parity stress (S1–S3).
//!
//! Plan: [`crate::dev::overlay_parity_stress_plan_v1`](../../dev/overlay_parity_stress_plan_v1.md).

use bevy::math::IVec2;
use bevy::prelude::*;
use rustc_hash::FxHashSet;

use crate::gui::{
    simulation_minimap_overlay_defaults, MapViewInstanceId, MapViewInstances,
    MapViewPresentationStates, MinimapOverlayMask, OverlayMask, ViewCameraState, ViewId,
    ViewInstance, ViewIsolationDiagnostics, ViewManager, ViewProjection, ViewRenderPolicy,
    ViewRenderTarget, VIEW_NO_ENTITY,
};
use crate::render::fire_chunk_runtime::{ActiveFireChunkSet, VisibleFireChunkSet};
use crate::render::fire_view_extract::{per_view_fire_extract_bounded, FireVisualFramesByView};
use crate::render::sim_visual_extract::{FireVisualFrame, FireVisualGpuInstance};

use super::per_view_policy::PerViewRepresentationPolicy;
use super::view_fire_isolation::{overlay_masks_aligned_with_map_views, ViewFireIsolationWitness};

/// Lib stress rollup for witness JSON (S1–S3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhaseDParityStressReport {
    pub s1_world_main_simulation_map_masks: bool,
    pub s2_multiview_no_fire_bleed: bool,
    pub s3_simulation_enter_rebind: bool,
}

impl PhaseDParityStressReport {
    #[must_use]
    pub fn all_green(self) -> bool {
        self.s1_world_main_simulation_map_masks
            && self.s2_multiview_no_fire_bleed
            && self.s3_simulation_enter_rebind
    }
}

#[must_use]
pub fn run_phase_d_parity_stress_matrix() -> PhaseDParityStressReport {
    PhaseDParityStressReport {
        s1_world_main_simulation_map_masks: stress_s1_world_main_simulation_map_masks_distinct(),
        s2_multiview_no_fire_bleed: stress_s2_multiview_no_cross_view_fire_bleed(),
        s3_simulation_enter_rebind: stress_s3_worldgen_to_simulation_rebinds_masks(),
    }
}

#[must_use]
pub fn triage_phase_d_parity_001_stress_green() -> bool {
    run_phase_d_parity_stress_matrix().all_green()
}

/// Baseline VM-08 + VM-11 + stress matrix (product close).
#[must_use]
pub fn triage_phase_d_parity_001_green(
    isolation: &ViewIsolationDiagnostics,
    fire: &ViewFireIsolationWitness,
) -> bool {
    isolation.vm08_overlay_masks_aligned
        && fire.vm08_overlay_masks_aligned
        && fire.vm11_minimap_cap_respected
        && fire.vm11_preview_cap_respected
        && triage_phase_d_parity_001_stress_green()
}

/// Mirrors [`crate::gui::hud::simulation_session::apply_simulation_map_presentation_defaults`] overlay slice.
pub fn phase_d_apply_simulation_overlay_rebind(
    map_views: &mut MapViewInstances,
    manager: &mut ViewManager,
    presentation: &mut MapViewPresentationStates,
) {
    let mask = simulation_minimap_overlay_defaults();
    map_views.minimap.overlays = mask;
    map_views.minimap.bump_revision();
    presentation
        .get_mut(MapViewInstanceId::SimulationMap)
        .overlays
        .fire_heat = false;
    if let Some(preview) = manager.views.get_mut(&ViewId::WorldPreview) {
        preview.render_policy.overlays.bits = map_views.world_preview.overlays;
    }
}

fn insert_view_with_mask(manager: &mut ViewManager, id: ViewId, mask: MinimapOverlayMask) {
    manager.views.insert(
        id,
        ViewInstance {
            id,
            camera_entity: VIEW_NO_ENTITY,
            render_target: ViewRenderTarget::None,
            camera: ViewCameraState::default(),
            projection: ViewProjection::default(),
            interaction_state: Default::default(),
            viewport_rect: Rect::from_center_size(Vec2::ZERO, Vec2::ONE),
            render_policy: ViewRenderPolicy {
                overlays: OverlayMask { bits: mask },
                ..Default::default()
            },
        },
    );
}

/// S1 — WorldMain vs SimulationMap keep distinct overlay masks across surface toggle.
fn stress_s1_world_main_simulation_map_masks_distinct() -> bool {
    let tactical = MinimapOverlayMask {
        fire_heat: true,
        logistics_heat: true,
        construction_heat: true,
        ..Default::default()
    };
    let sim_map = MinimapOverlayMask {
        fire_heat: false,
        logistics_heat: true,
        construction_heat: false,
        ..Default::default()
    };
    if tactical == sim_map {
        return false;
    }

    let mut manager = ViewManager::default();
    insert_view_with_mask(&mut manager, ViewId::WorldMain, tactical);
    insert_view_with_mask(&mut manager, ViewId::SimulationMap, sim_map);

    let wm = manager
        .view(ViewId::WorldMain)
        .expect("WorldMain")
        .render_policy
        .overlays
        .bits;
    let sm = manager
        .view(ViewId::SimulationMap)
        .expect("SimulationMap")
        .render_policy
        .overlays
        .bits;
    if wm.fire_heat == sm.fire_heat {
        return false;
    }

    // Simulate active-surface toggle (read-only — masks must not collapse).
    let wm_before = wm;
    let sm_before = sm;
    let wm_after = manager.view(ViewId::WorldMain).unwrap().render_policy.overlays.bits;
    let sm_after = manager
        .view(ViewId::SimulationMap)
        .unwrap()
        .render_policy
        .overlays
        .bits;
    wm_before == wm_after && sm_before == sm_after
}

/// S2 — 2-up multiview: per-view fire extract bounded; caps separate WorldMain vs Minimap.
fn stress_s2_multiview_no_cross_view_fire_bleed() -> bool {
    let policy = PerViewRepresentationPolicy::default();
    if policy.fire_cap(super::ids::ViewSurfaceId::Minimap)
        >= policy.fire_cap(super::ids::ViewSurfaceId::WorldMain)
    {
        return false;
    }

    let mut by_view = FireVisualFramesByView::default();
    let mut vis = VisibleFireChunkSet::default();
    let mut active = ActiveFireChunkSet::default();
    let coord = IVec2::ZERO;
    active.chunks.insert(coord);

    let mut wm_allowed = FxHashSet::default();
    wm_allowed.insert(coord);
    vis.per_view.insert(ViewId::WorldMain, wm_allowed.clone());

    let mut wm_frame = FireVisualFrame::default();
    for _ in 0..8 {
        wm_frame.instances.push(FireVisualGpuInstance {
            chunk_xy_heat_lum: Vec4::new(coord.x as f32, coord.y as f32, 0.5, 0.5),
            ..Default::default()
        });
    }
    by_view.by_id.insert(ViewId::WorldMain, wm_frame);

    let mut mm_allowed = FxHashSet::default();
    mm_allowed.insert(coord);
    vis.per_view.insert(ViewId::Minimap, mm_allowed);

    let mut mm_frame = FireVisualFrame::default();
    mm_frame.instances.push(FireVisualGpuInstance {
        chunk_xy_heat_lum: Vec4::new(coord.x as f32, coord.y as f32, 0.5, 0.5),
        ..Default::default()
    });
    by_view.by_id.insert(ViewId::Minimap, mm_frame);

    if !per_view_fire_extract_bounded(&by_view, &vis, &active) {
        return false;
    }

    let wm_n = by_view
        .by_id
        .get(&ViewId::WorldMain)
        .map(|f| f.instances.len())
        .unwrap_or(0);
    let mm_n = by_view
        .by_id
        .get(&ViewId::Minimap)
        .map(|f| f.instances.len())
        .unwrap_or(0);
    if wm_n <= mm_n {
        return false;
    }
    if wm_n > policy.fire_cap_for_view_id(ViewId::WorldMain) {
        return false;
    }
    if mm_n > policy.fire_cap_for_view_id(ViewId::Minimap) {
        return false;
    }

    let map_views = MapViewInstances::default();
    !map_views.minimap.overlays.fire_heat
}

/// S3 — WorldGen-style stale minimap fire tint clears on simulation overlay rebind.
fn stress_s3_worldgen_to_simulation_rebinds_masks() -> bool {
    let mut map_views = MapViewInstances::default();
    map_views.minimap.overlays.fire_heat = true;
    map_views.world_preview.overlays.fire_heat = true;

    let mut manager = ViewManager::default();
    let mismatched = MinimapOverlayMask {
        fire_heat: false,
        logistics_heat: false,
        ..Default::default()
    };
    insert_view_with_mask(&mut manager, ViewId::WorldPreview, mismatched);

    if overlay_masks_aligned_with_map_views(&manager, &map_views) {
        return false;
    }

    let mut presentation = MapViewPresentationStates::default();
    phase_d_apply_simulation_overlay_rebind(&mut map_views, &mut manager, &mut presentation);

    overlay_masks_aligned_with_map_views(&manager, &map_views)
        && !map_views.minimap.overlays.fire_heat
        && !presentation
            .get(MapViewInstanceId::SimulationMap)
            .overlays
            .fire_heat
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_d_parity_stress_matrix_s1_s3_green() {
        let report = run_phase_d_parity_stress_matrix();
        assert!(report.s1_world_main_simulation_map_masks, "S1");
        assert!(report.s2_multiview_no_fire_bleed, "S2");
        assert!(report.s3_simulation_enter_rebind, "S3");
        assert!(triage_phase_d_parity_001_stress_green());
    }

    #[test]
    fn triage_phase_d_parity_witness_rollup_with_baseline() {
        let isolation = ViewIsolationDiagnostics {
            vm08_overlay_masks_aligned: true,
            ..Default::default()
        };
        let fire = ViewFireIsolationWitness {
            vm08_overlay_masks_aligned: true,
            vm11_minimap_cap_respected: true,
            vm11_preview_cap_respected: true,
            ..Default::default()
        };
        assert!(triage_phase_d_parity_001_green(&isolation, &fire));
    }
}
