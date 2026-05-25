//! Per-view **visible fire chunks** + filtered [`crate::render::sim_visual_extract::FireVisualFrame`] builds.
//!
//! Visibility intersects sim-active chunks with each view's [`ViewInstance::visible_world_rect`]
//! (projection + viewport from view authority). [`VisibleFireChunkSet`] stores [`FxHashSet`]s per
//! [`ViewId`]. When a view disables fire in [`ViewRenderPolicy::overlays`], its set is empty.
//! [`WorldLodBand`] from the view clamps heat-derived [`FireLodBand`] (smoke vs flame at distance).

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use bevy::math::Rect;
use bevy::prelude::*;
use rustc_hash::FxHashSet;

use crate::gui::{ViewId, ViewInstance, ViewManager, WorldLodBand};
use crate::render::fire_chunk_runtime::{
    ActiveFireChunkSet, ChunkCoord, FireChunkLodState, FireLodBand, FireSimulationSnapshot, VisibleFireChunkSet,
};
use crate::io::streaming::ChunkResidencyTable;
use crate::render::{
    chunk_in_residency_table, per_view_residency_contains, PerViewResidencyConsumerWindow,
    Stage5FireViewChunkWitness, Stage5ReadinessProfile,
};
use crate::render::view_runtime::ViewSurfaceId;
use crate::render::sim_visual_extract::{FireVisualFrame, FireVisualGpuInstance};

/// World XY extent per chunk index step (aligned with [`crate::gui::camera_focus_debug::DEBUG_CHUNK_SPACING_WORLD`]).

pub const FIRE_VIEW_CHUNK_SPACING_WORLD: f32 = 64.0;

/// Per-view filtered fire frames (isolates minimap / preview from tactical extraction).
#[derive(Resource, Default, Debug, Clone)]
pub struct FireVisualFramesByView {
    pub by_id: HashMap<ViewId, FireVisualFrame>,
}

/// Tactical / primary-window fire extract: [`ViewId::WorldMain`], else [`ViewId::SimulationMap`].
#[must_use]
pub fn tactical_fire_visual(by_view: &FireVisualFramesByView) -> &FireVisualFrame {
    static EMPTY: OnceLock<FireVisualFrame> = OnceLock::new();
    let empty = EMPTY.get_or_init(FireVisualFrame::default);
    if let Some(wm) = by_view.by_id.get(&ViewId::WorldMain) {
        if !wm.instances.is_empty() || !wm.chunk_heat.is_empty() {
            return wm;
        }
    }
    by_view
        .by_id
        .get(&ViewId::SimulationMap)
        .or_else(|| by_view.by_id.get(&ViewId::WorldMain))
        .unwrap_or(empty)
}

fn chunk_bounds_for_world_rect(world: Rect, spacing: f32) -> (ChunkCoord, ChunkCoord) {
    let s = spacing.max(1.0);
    let min_c = ChunkCoord::new(
        (world.min.x / s).floor() as i32,
        (world.min.y / s).floor() as i32,
    );
    let max_c = ChunkCoord::new(
        (world.max.x / s).floor() as i32,
        (world.max.y / s).floor() as i32,
    );
    (min_c, max_c)
}

fn chunk_in_rect(c: ChunkCoord, min_c: ChunkCoord, max_c: ChunkCoord) -> bool {
    c.x >= min_c.x && c.x <= max_c.x && c.y >= min_c.y && c.y <= max_c.y
}

fn visible_chunks_for_view(view: &ViewInstance, active: &ActiveFireChunkSet) -> FxHashSet<ChunkCoord> {
    let world = view.visible_world_rect();
    let (min_c, max_c) = chunk_bounds_for_world_rect(world, FIRE_VIEW_CHUNK_SPACING_WORLD);
    active
        .chunks
        .iter()
        .copied()
        .filter(|c| chunk_in_rect(*c, min_c, max_c))
        .collect()
}

/// Fills [`VisibleFireChunkSet`] from [`ViewManager`] + [`ActiveFireChunkSet`] using each view's
/// [`ViewInstance::visible_world_rect`] and [`ViewRenderPolicy::overlays`].
pub fn sync_visible_fire_chunks_from_views(
    launch: Option<Res<crate::engine::EngineLaunchArgs>>,
    manager: Option<Res<ViewManager>>,
    active: Res<ActiveFireChunkSet>,
    residency: Option<Res<ChunkResidencyTable>>,
    per_view_windows: Option<Res<PerViewResidencyConsumerWindow>>,
    mut visible: ResMut<VisibleFireChunkSet>,
) {
    visible.per_view.clear();
    if active.chunks.is_empty() {
        return;
    }
    let proof = launch
        .as_deref()
        .is_some_and(crate::engine::EngineLaunchArgs::visual_tactical_vfx_proof);
    let tactical_visible: FxHashSet<ChunkCoord> = active.chunks.iter().copied().collect();
    let Some(manager) = manager else {
        if proof {
            for id in [ViewId::WorldMain, ViewId::SimulationMap] {
                visible.per_view.insert(id, tactical_visible.clone());
            }
        }
        return;
    };
    let residency = residency.as_deref();
    let per_view_windows = per_view_windows.as_deref();
    for (&id, view) in &manager.views {
        if !view.render_policy.overlays.bits.fire_heat {
            visible.per_view.insert(id, FxHashSet::default());
            continue;
        }
        let mut list = if proof && matches!(id, ViewId::WorldMain | ViewId::SimulationMap) {
            tactical_visible.clone()
        } else {
            visible_chunks_for_view(view, &active)
        };
        let surface = ViewSurfaceId::from_view_id(id);
        if let Some(windows) = per_view_windows {
            list.retain(|c| per_view_residency_contains(surface, *c, windows));
        } else if let Some(table) = residency {
            list.retain(|c| chunk_in_residency_table(*c, table));
        }
        visible.per_view.insert(id, list);
    }
}

/// Clamp heat-derived [`FireLodBand`] using the view's world LOD band (smoke vs flame at distance).
#[must_use]
pub fn clamp_fire_lod_for_world_band(world: WorldLodBand, sim: FireLodBand) -> FireLodBand {
    if sim == FireLodBand::None {
        return FireLodBand::None;
    }
    match world {
        WorldLodBand::LocalTactical | WorldLodBand::Operational => sim,
        WorldLodBand::Strategic => match sim {
            FireLodBand::FullFlame => FireLodBand::LowFlame,
            other => other,
        },
        WorldLodBand::Macro => match sim {
            FireLodBand::FullFlame | FireLodBand::LowFlame => FireLodBand::SmokeOnly,
            FireLodBand::SmokeOnly | FireLodBand::None => sim,
        },
    }
}

#[inline]
fn chunk_of_instance(row: &FireVisualGpuInstance) -> ChunkCoord {
    let xy = row.chunk_grid_xy();
    ChunkCoord::new(xy.x as i32, xy.y as i32)
}

fn sim_lod_band_for_chunk(lod: &FireChunkLodState, coord: ChunkCoord) -> FireLodBand {
    lod.bands
        .get(&coord)
        .copied()
        .unwrap_or(FireLodBand::FullFlame)
}

/// Downgrade / drop GPU rows per [`FireLodBand`] (CPU policy before particles / GPU specialization).
fn apply_fire_lod_to_row(mut row: FireVisualGpuInstance, band: FireLodBand) -> Option<FireVisualGpuInstance> {
    match band {
        FireLodBand::None => None,
        FireLodBand::SmokeOnly => {
            row.chunk_xy_heat_lum.w *= 0.12;
            row.smoke_ember_vis_priority.y *= 0.08;
            Some(row)
        }
        FireLodBand::LowFlame => {
            row.chunk_xy_heat_lum.w *= 0.5;
            row.smoke_ember_vis_priority.y *= 0.45;
            Some(row)
        }
        FireLodBand::FullFlame => Some(row),
    }
}

fn all_sim_chunk_coords(sim: &FireSimulationSnapshot) -> HashSet<ChunkCoord> {
    let mut s: HashSet<ChunkCoord> = sim.chunk_heat.iter().map(|h| h.chunk).collect();
    for row in &sim.instances {
        s.insert(chunk_of_instance(row));
    }
    s
}

fn allowed_chunks_for_view(view_id: ViewId, vis: &VisibleFireChunkSet, sim: &FireSimulationSnapshot) -> FxHashSet<ChunkCoord> {
    match vis.per_view.get(&view_id) {
        Some(set) => set.clone(),
        None => all_sim_chunk_coords(sim).into_iter().collect(),
    }
}

fn build_frame_for_allowed(
    sim: &FireSimulationSnapshot,
    lod: &FireChunkLodState,
    allowed: &FxHashSet<ChunkCoord>,
    world_band: WorldLodBand,
) -> FireVisualFrame {
    let mut instances = Vec::new();
    for row in &sim.instances {
        let c = chunk_of_instance(row);
        if !allowed.contains(&c) {
            continue;
        }
        let sim_band = sim_lod_band_for_chunk(lod, c);
        let band = clamp_fire_lod_for_world_band(world_band, sim_band);
        if let Some(r) = apply_fire_lod_to_row(*row, band) {
            instances.push(r);
        }
    }
    let mut chunk_heat = Vec::new();
    for h in &sim.chunk_heat {
        if !allowed.contains(&h.chunk) {
            continue;
        }
        let sim_band = sim_lod_band_for_chunk(lod, h.chunk);
        let band = clamp_fire_lod_for_world_band(world_band, sim_band);
        if band == FireLodBand::None {
            continue;
        }
        let mut hh = *h;
        match band {
            FireLodBand::SmokeOnly => {
                hh.smoke = (hh.smoke * 1.15).min(1.0);
            }
            FireLodBand::LowFlame => {
                hh.heat *= 0.85;
            }
            _ => {}
        }
        chunk_heat.push(hh);
    }
    FireVisualFrame {
        stamp: sim.stamp,
        instances,
        chunk_heat,
    }
}

/// Builds [`FireVisualFramesByView`] from the sim snapshot, [`VisibleFireChunkSet`], and [`FireChunkLodState`].
/// When [`ViewManager`] is missing, world LOD defaults to [`WorldLodBand::Strategic`] (headless / tests).
pub fn build_fire_visual_frames_by_view(
    launch: Option<Res<crate::engine::EngineLaunchArgs>>,
    sim: Res<FireSimulationSnapshot>,
    vis: Res<VisibleFireChunkSet>,
    lod: Res<FireChunkLodState>,
    active: Res<ActiveFireChunkSet>,
    manager: Option<Res<ViewManager>>,
    per_view_policy: Option<Res<crate::render::view_runtime::PerViewRepresentationPolicy>>,
    profile: Res<Stage5ReadinessProfile>,
    mut fire_chunk_witness: ResMut<Stage5FireViewChunkWitness>,
    mut out: ResMut<FireVisualFramesByView>,
) {
    out.by_id.clear();
    let default_band = WorldLodBand::Strategic;
    let proof = launch
        .as_deref()
        .is_some_and(crate::engine::EngineLaunchArgs::visual_tactical_vfx_proof);
    for id in [
        ViewId::WorldMain,
        ViewId::WorldPreview,
        ViewId::Minimap,
        ViewId::SimulationMap,
    ] {
        let allowed = allowed_chunks_for_view(id, &vis, &sim);
        let world_band = if proof && matches!(id, ViewId::WorldMain | ViewId::SimulationMap) {
            WorldLodBand::LocalTactical
        } else {
            manager
                .as_deref()
                .and_then(|m| m.view(id))
                .map(|v| v.render_policy.lod_band)
                .unwrap_or(default_band)
        };
        let mut frame = build_frame_for_allowed(&sim, &lod, &allowed, world_band);
        if let Some(policy) = per_view_policy.as_deref() {
            let cap = policy.fire_cap_for_view_id(id);
            if frame.instances.len() > cap {
                frame.instances.truncate(cap);
            }
        }
        out.by_id.insert(id, frame);
    }
    let mut orphans: u32 = 0;
    if let Some(wm_vis) = vis.per_view.get(&ViewId::WorldMain) {
        let outside: usize = wm_vis
            .iter()
            .filter(|c| !active.chunks.contains(*c))
            .count();
        orphans = outside as u32;
        if *profile == Stage5ReadinessProfile::FULL_APP && outside > 0 {
            warn!(
                target: "stage5_fire_view::live",
                "STAGE5_FIRE_VIEW_CROSSCHECK view=WorldMain visible_chunks_not_in_active={outside} active_total={} visible_wm={}",
                active.chunks.len(),
                wm_vis.len(),
            );
        }
    }
    fire_chunk_witness.world_main_visible_orphan_chunks = orphans;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_fire_chunks_respect_residency_table() {
        use crate::io::streaming::{ChunkResidencyEntry, ChunkResidencyRole, ChunkResidencyTable};
        let mut active = ActiveFireChunkSet::default();
        active.chunks.insert(IVec2::ZERO);
        active.chunks.insert(IVec2::new(9, 0));
        let mut table = ChunkResidencyTable::default();
        table.entries.insert(
            IVec2::ZERO,
            ChunkResidencyEntry {
                coord: IVec2::ZERO,
                role: ChunkResidencyRole::Core,
                orb_priority: 0,
            },
        );
        let mut list: FxHashSet<ChunkCoord> = active.chunks.iter().copied().collect();
        list.retain(|c| chunk_in_residency_table(*c, &table));
        assert_eq!(list.len(), 1);
        assert!(list.contains(&IVec2::ZERO));
    }

    #[test]
    fn clamp_macro_forces_smoke_for_flame() {
        assert_eq!(
            clamp_fire_lod_for_world_band(WorldLodBand::Macro, FireLodBand::FullFlame),
            FireLodBand::SmokeOnly
        );
        assert_eq!(
            clamp_fire_lod_for_world_band(WorldLodBand::Macro, FireLodBand::LowFlame),
            FireLodBand::SmokeOnly
        );
    }

    #[test]
    fn clamp_strategic_downgrades_full_to_low() {
        assert_eq!(
            clamp_fire_lod_for_world_band(WorldLodBand::Strategic, FireLodBand::FullFlame),
            FireLodBand::LowFlame
        );
        assert_eq!(
            clamp_fire_lod_for_world_band(WorldLodBand::Strategic, FireLodBand::LowFlame),
            FireLodBand::LowFlame
        );
    }

    #[test]
    fn clamp_tactical_passes_through() {
        assert_eq!(
            clamp_fire_lod_for_world_band(WorldLodBand::LocalTactical, FireLodBand::FullFlame),
            FireLodBand::FullFlame
        );
    }

    #[test]
    fn tactical_fire_visual_prefers_non_empty_simulation_map_frame() {
        let mut by_view = FireVisualFramesByView::default();
        by_view.by_id.insert(ViewId::WorldMain, FireVisualFrame::default());
        let mut sim_frame = FireVisualFrame::default();
        sim_frame
            .instances
            .push(FireVisualGpuInstance::default());
        by_view.by_id.insert(ViewId::SimulationMap, sim_frame);
        assert_eq!(tactical_fire_visual(&by_view).instances.len(), 1);
    }
}
