//! Per-view **visible fire chunks** + filtered [`crate::render::sim_visual_extract::FireVisualFrame`] builds.
//!
//! Visibility intersects sim-active chunks with each view's [`ViewInstance::visible_world_rect`]
//! (projection + viewport from view authority). [`VisibleFireChunkSet`] stores [`FxHashSet`]s per
//! [`ViewId`]. When a view disables fire in [`ViewRenderPolicy::overlays`], its set is empty.
//! [`WorldLodBand`] from the view clamps heat-derived [`FireLodBand`] (smoke vs flame at distance).

use std::collections::HashMap;
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

/// Designer policy caps ([`fire_lod_player_read_v1.md`](../dev/fire_lod_player_read_v1.md) **FIRE7-DESIGN-001**).
pub const FIRE_LOD_CAP_STRATEGIC: usize = 32;
pub const FIRE_LOD_CAP_OPERATIONAL: usize = 128;
pub const FIRE_LOD_CAP_TACTICAL: usize = 512;

#[must_use]
pub fn fire_lod_designer_table_wired() -> bool {
    FIRE_LOD_CAP_STRATEGIC < FIRE_LOD_CAP_OPERATIONAL
        && FIRE_LOD_CAP_OPERATIONAL < FIRE_LOD_CAP_TACTICAL
}

/// **FIRE7-F7-C-001** — designer LOD caps enforced in extract path.
#[must_use]
pub fn fire7_f7_c_001_green() -> bool {
    fire_lod_designer_table_wired()
}

#[must_use]
pub fn fire_cap_for_world_band(world: WorldLodBand) -> usize {
    match world {
        WorldLodBand::Macro | WorldLodBand::Strategic => FIRE_LOD_CAP_STRATEGIC,
        WorldLodBand::Operational => FIRE_LOD_CAP_OPERATIONAL,
        WorldLodBand::LocalTactical => FIRE_LOD_CAP_TACTICAL,
    }
}

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
    let _perf = crate::render::PerfScope::new("upd_fire_sync_visible");
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
        // F2-PR-2: tactical visual proof must not residency-cull all seeded fire chunks.
        let skip_residency_cull =
            proof && matches!(id, ViewId::WorldMain | ViewId::SimulationMap);
        if !skip_residency_cull {
            let surface = ViewSurfaceId::from_view_id(id);
            if let Some(windows) = per_view_windows {
                list.retain(|c| per_view_residency_contains(surface, *c, windows));
            } else if let Some(table) = residency {
                list.retain(|c| chunk_in_residency_table(*c, table));
            }
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

/// F7-A-001: true when every instance / heat row lies in `allowed` (strict per-view extract).
#[must_use]
pub fn fire_visual_frame_within_visible_set(
    frame: &FireVisualFrame,
    allowed: &FxHashSet<ChunkCoord>,
) -> bool {
    frame
        .instances
        .iter()
        .all(|row| allowed.contains(&chunk_of_instance(row)))
        && frame
            .chunk_heat
            .iter()
            .all(|h| allowed.contains(&h.chunk))
}

/// Per-view allowed chunks: present key → use set (empty = none); missing key → empty unless
/// visibility writer never ran (`per_view` empty) → tactical views fall back to [`ActiveFireChunkSet`].
fn allowed_chunks_for_view(
    view_id: ViewId,
    vis: &VisibleFireChunkSet,
    active: &ActiveFireChunkSet,
) -> FxHashSet<ChunkCoord> {
    if let Some(set) = vis.per_view.get(&view_id) {
        return set.clone();
    }
    if vis.per_view.is_empty()
        && matches!(view_id, ViewId::WorldMain | ViewId::SimulationMap)
    {
        return active.chunks.iter().copied().collect();
    }
    FxHashSet::default()
}

#[must_use]
pub fn per_view_fire_extract_bounded(
    by_view: &FireVisualFramesByView,
    vis: &VisibleFireChunkSet,
    active: &ActiveFireChunkSet,
) -> bool {
    by_view.by_id.iter().all(|(id, frame)| {
        let allowed = allowed_chunks_for_view(*id, vis, active);
        fire_visual_frame_within_visible_set(frame, &allowed)
    })
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
/// Dirty-gate fingerprint for [`build_fire_visual_frames_by_view`].
///
/// The four-view rebuild only changes when one of these moves: the sim snapshot cadence
/// (`stamp` + instance/heat counts), the active-chunk set, the per-view visible-chunk windows
/// (camera/view rect motion), or the per-view world LOD band (zoom). When unchanged the previous
/// `FireVisualFramesByView` stays valid and downstream consumers read it as before.
#[derive(Clone, PartialEq)]
pub struct FireViewExtractFingerprint {
    stamp: crate::systems::sim_control::SimStepStamp,
    instances_len: usize,
    chunk_heat_len: usize,
    active_len: usize,
    active_digest: u64,
    vis_digest: u64,
    bands: [(crate::gui::WorldLodBand, usize); 4],
    proof: bool,
}

#[inline]
fn xor_chunk_digest<'a>(chunks: impl Iterator<Item = &'a ChunkCoord>) -> u64 {
    // Order-independent XOR of per-chunk hashes — stable for sets regardless of iteration order.
    let mut acc = 0u64;
    for c in chunks {
        let h = (c.x as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
            ^ (c.y as u64).wrapping_mul(0xc2b2_ae3d_27d4_eb4f).rotate_left(17);
        acc ^= h;
    }
    acc
}

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
    mut last_fingerprint: Local<Option<FireViewExtractFingerprint>>,
) {
    let _perf = crate::render::PerfScope::new("upd_fire_build_view");
    let default_band = WorldLodBand::Strategic;
    let proof = launch
        .as_deref()
        .is_some_and(crate::engine::EngineLaunchArgs::visual_tactical_vfx_proof);

    // DIRTY-GATE (perf: fire pipeline per-frame full rebuild). Build the fingerprint of every
    // input that can change the four per-view frames, then skip the rebuild when nothing moved.
    let bands = {
        let mut out_bands = [(default_band, 0usize); 4];
        for (slot, id) in [
            ViewId::WorldMain,
            ViewId::WorldPreview,
            ViewId::Minimap,
            ViewId::SimulationMap,
        ]
        .into_iter()
        .enumerate()
        {
            let world_band = if proof && matches!(id, ViewId::WorldMain | ViewId::SimulationMap) {
                WorldLodBand::LocalTactical
            } else {
                manager
                    .as_deref()
                    .and_then(|m| m.view(id))
                    .map(|v| v.render_policy.lod_band)
                    .unwrap_or(default_band)
            };
            let vis_len = vis.per_view.get(&id).map(|s| s.len()).unwrap_or(0);
            out_bands[slot] = (world_band, vis_len);
        }
        out_bands
    };
    let fingerprint = FireViewExtractFingerprint {
        stamp: sim.stamp,
        instances_len: sim.instances.len(),
        chunk_heat_len: sim.chunk_heat.len(),
        active_len: active.chunks.len(),
        active_digest: xor_chunk_digest(active.chunks.iter()),
        vis_digest: vis
            .per_view
            .iter()
            .fold(0u64, |acc, (id, set)| {
                acc ^ (*id as u64)
                    .wrapping_mul(0x100_0000_01b3)
                    .rotate_left(7)
                    ^ xor_chunk_digest(set.iter())
            }),
        bands,
        proof,
    };
    if last_fingerprint.as_ref() == Some(&fingerprint) {
        return;
    }
    *last_fingerprint = Some(fingerprint);

    out.by_id.clear();
    for id in [
        ViewId::WorldMain,
        ViewId::WorldPreview,
        ViewId::Minimap,
        ViewId::SimulationMap,
    ] {
        let allowed = allowed_chunks_for_view(id, &vis, &active);
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
        {
            let design_cap = fire_cap_for_world_band(world_band);
            let cap = per_view_policy
                .as_deref()
                .map(|p| p.fire_cap_for_view_id(id).min(design_cap))
                .unwrap_or(design_cap);
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
    fire_chunk_witness.f7_a_per_view_extract_bounded =
        per_view_fire_extract_bounded(&out, &vis, &active);
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
    fn allowed_chunks_missing_view_key_is_empty_not_full_sim() {
        let mut sim = FireSimulationSnapshot::default();
        sim.chunk_heat.push(crate::render::sim_visual_extract::ChunkFireHeat {
            chunk: IVec2::new(5, 5),
            ..Default::default()
        });
        let vis = VisibleFireChunkSet::default();
        let active = ActiveFireChunkSet::default();
        let allowed = allowed_chunks_for_view(ViewId::Minimap, &vis, &active);
        assert!(allowed.is_empty());
    }

    #[test]
    fn per_view_fire_frames_respect_visible_sets() {
        let mut sim = FireSimulationSnapshot::default();
        let mut row = FireVisualGpuInstance::default();
        row.chunk_xy_heat_lum = Vec4::new(128.0, 64.0, 0.0, 1.0);
        sim.instances.push(row);
        let chunk = chunk_of_instance(&sim.instances[0]);

        let mut vis = VisibleFireChunkSet::default();
        let mut wm = FxHashSet::default();
        wm.insert(chunk);
        vis.per_view.insert(ViewId::WorldMain, wm);
        vis.per_view.insert(ViewId::Minimap, FxHashSet::default());

        let wm_frame = build_frame_for_allowed(
            &sim,
            &FireChunkLodState::default(),
            vis.per_view.get(&ViewId::WorldMain).unwrap(),
            WorldLodBand::LocalTactical,
        );
        let mm_frame = build_frame_for_allowed(
            &sim,
            &FireChunkLodState::default(),
            vis.per_view.get(&ViewId::Minimap).unwrap(),
            WorldLodBand::Strategic,
        );
        assert_eq!(wm_frame.instances.len(), 1);
        assert!(mm_frame.instances.is_empty());
        assert!(fire_visual_frame_within_visible_set(
            &wm_frame,
            vis.per_view.get(&ViewId::WorldMain).unwrap()
        ));
        assert!(fire_visual_frame_within_visible_set(
            &mm_frame,
            vis.per_view.get(&ViewId::Minimap).unwrap()
        ));
    }

    #[test]
    fn f7_c_lod_caps_differ_strategic_vs_tactical() {
        use crate::render::sim_visual_extract::{ChunkFireHeat, FireVisualGpuInstance};
        let mut sim = FireSimulationSnapshot::default();
        for i in 0..40i32 {
            sim.chunk_heat.push(ChunkFireHeat {
                chunk: IVec2::new(i, 0),
                heat: 0.9,
                smoke: 0.2,
            });
            sim.instances.push(FireVisualGpuInstance::default());
        }
        let lod = FireChunkLodState::default();
        let mut allowed: FxHashSet<ChunkCoord> = (0..40).map(|i| IVec2::new(i, 0)).collect();
        let mut strategic =
            build_frame_for_allowed(&sim, &lod, &allowed, WorldLodBand::Strategic);
        strategic
            .instances
            .truncate(fire_cap_for_world_band(WorldLodBand::Strategic));
        let mut tactical =
            build_frame_for_allowed(&sim, &lod, &allowed, WorldLodBand::LocalTactical);
        tactical
            .instances
            .truncate(fire_cap_for_world_band(WorldLodBand::LocalTactical));
        assert!(strategic.instances.len() <= FIRE_LOD_CAP_STRATEGIC);
        assert!(tactical.instances.len() > strategic.instances.len());
        assert!(fire_lod_designer_table_wired());
    }

    #[test]
    fn per_view_fire_extract_bounded_catches_visible_leak() {
        let mut by_view = FireVisualFramesByView::default();
        let mut frame = FireVisualFrame::default();
        frame.instances.push(FireVisualGpuInstance::default());
        by_view.by_id.insert(ViewId::Minimap, frame);
        let mut vis = VisibleFireChunkSet::default();
        vis.per_view.insert(ViewId::Minimap, FxHashSet::default());
        assert!(!per_view_fire_extract_bounded(
            &by_view,
            &vis,
            &ActiveFireChunkSet::default()
        ));
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
