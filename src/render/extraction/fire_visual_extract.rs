//! **Fire sim → chunk runtime → view visibility → LOD → per-view frames** (CPU). ECS reads happen only in
//! [`extract_fire_simulation_snapshot`]; [`crate::render::fire_view_extract::build_fire_visual_frames_by_view`] fills
//! [`crate::render::fire_view_extract::FireVisualFramesByView`] using per-view [`VisibleFireChunkSet`] and
//! [`WorldLodBand`](crate::gui::WorldLodBand)-clamped fire LOD. GPU / projection use [`crate::render::fire_view_extract::tactical_fire_visual`]
//! ([`ViewId::WorldMain`]); other [`ViewId`]s read their entry directly.
//! [`crate::render::extraction::RenderProjectionGraph`] (fire node evaluation) → render upload (`base_fire2_smoke.md`).
//! [`crate::compute::ComputeDispatchGraph`] runs on the same snapshots **before** render projection (compute LOD policy).
//!
//! ## Contract
//! 1. **`FireSimulationSnapshot`** — full per-chunk **sim** snapshot from one ECS pass (same rows as legacy `FireVisualFrame` source).
//!    [`extract_fire_simulation_snapshot`] is the **only** place that reads [`ChunkSurfaceFire`] for this path.
//! 2. **`FireVisualFramesByView`** — **render-facing** per view; must not read ECS fire components.
//! 3. **[`RenderProjectionGraph`]** — CPU projection orchestrator; fire node output is **extracted** for GPU instance upload.
//! 4. **`SharedOverlayFieldBuffers`** — derived from **full** [`FireSimulationSnapshot::chunk_heat`] (global overlay truth).

use bevy::math::IVec2;
use bevy::prelude::*;

use std::collections::HashMap;

use crate::gui::{MapCameraSystemSet, ViewAuthoritySystemSet};
use crate::render::{
    attrib_fire_build_view_after, attrib_fire_build_view_before, attrib_fire_particles_after,
    attrib_fire_particles_before, attrib_fire_pipeline_after, attrib_fire_pipeline_before,
    attrib_fire_project_after, attrib_fire_project_before,
    build_fire_visual_frames_by_view, sync_active_fire_chunk_set, sync_fire_chunk_lod_from_snapshot,
    sync_visible_fire_chunks_from_views, tactical_fire_visual, ActiveFireChunkSet, ChunkCoord, FireChunk,
    FireChunkLodState, FireChunkRuntime, FireSimulationSnapshot, FireVisualFramesByView,
    VisibleFireChunkSet, FIRE_SIM_CHUNK_ACTIVE_EPS,
};
use crate::render::overlay_field_buffers::{
    chunk_fire_heat_maps_differ, CHUNK_FIRE_OVERLAY_DISPLAY_MIN,
};
use crate::render::sim_visual_extract::{
    ChunkFireHeat, FireVisualGpuInstance, SimFireEmitterVisualExtract,
    FIRE_VISUAL_ACTIVE_HEAT_EPS,
};
use crate::render::SharedOverlayFieldBuffers;
use crate::render::light::{LightCategory, RequestLocalLight};
use crate::render::lighting::{
    build_fire_light_clusters, FireLightCluster, FireLightEmission as VisFireLightSample, FireLightType,
};
use crate::systems::atmosphere::AtmosphereDiagnostics;
use crate::systems::ecology::ChunkEcology;
use crate::systems::fire::{
    ChunkFireOverlay, ChunkFuelProfile, ChunkSmokeField, ChunkSurfaceFire, FireLightEmission as SimFireLightEmission,
};
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::material::MaterializedChunk;

use super::fire_emission_profile::infer_fire_emission_profile;
use super::render_projection_graph::{run_render_projection_graph, RenderProjectionGraph};
use super::smoke_visual_extract::{build_smoke_visual_extract, SmokeVisualBridgeWitness};
use crate::render::visual_snapshot_commit::{commit_fire_visual_snapshot, CommittedVisualSnapshotFence};
use crate::render::gpu_particles::{emit_world_fire_particles_from_projection, WorldFireParticleFrame};

/// Chunk-scale **regional** hints for fog / atmosphere (from `FireVisualFrame::instances`; no ECS scan).
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct FireAtmosphereAggregate {
    pub smoke_density: f32,
    pub smoke_color: Vec3,
    pub heat_energy: f32,
    pub ember_density: f32,
    pub visibility_loss: f32,
}

/// Frame-local cluster rows between cluster build and light emit (not entities).
#[derive(Resource, Default, Debug)]
struct FireClusterScratch {
    clusters: Vec<FireLightCluster>,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum FireVisualFrameSet {
    /// Sim ECS scan → [`FireSimulationSnapshot`] + [`FireChunkRuntime`]; stub view visibility + LOD; then [`FireVisualFrame`].
    BuildProfiles,
    /// Greedy merge → [`FireClusterScratch`].
    BuildClusters,
    /// Regional averages → [`FireAtmosphereAggregate`].
    BuildAtmosphere,
    /// `RequestLocalLight` messages for pooled lights.
    EmitLights,
    /// Future: GPU smoke / volume extract (buffer only).
    EmitSmoke,
    /// Post-LOD [`WorldFireParticleFrame`] from [`RenderProjectionGraph`].
    EmitParticles,
    /// Logistics / ecology overlay rows from projection + committed snapshots.
    EmitDomainOverlays,
    /// [`RenderProjectionGraph`] — fire node only in v1.
    ProjectGpu,
}

pub struct FireVisualFramePlugin;

impl Plugin for FireVisualFramePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::render::fire_streaming::FireStreamingPlugin);
        app.add_message::<RequestLocalLight>()
            .configure_sets(Update, ViewAuthoritySystemSet::SyncViewManager)
            .init_resource::<FireSimulationSnapshot>()
            .init_resource::<FireChunkRuntime>()
            .init_resource::<VisibleFireChunkSet>()
            .init_resource::<FireChunkLodState>()
            .init_resource::<FireVisualFramesByView>()
            .init_resource::<RenderProjectionGraph>()
            .init_resource::<CommittedVisualSnapshotFence>()
            .init_resource::<crate::render::LogisticsVisualSnapshot>()
            .init_resource::<crate::render::EcologyVisualSnapshot>()
            .init_resource::<SimFireEmitterVisualExtract>()
            .init_resource::<FireAtmosphereAggregate>()
            .init_resource::<FireClusterScratch>()
            .init_resource::<WorldFireParticleFrame>()
            .init_resource::<crate::render::gpu_particles::FireParticleCameraScale>()
            .init_resource::<crate::render::DomainOverlayGpuFrame>()
            .init_resource::<ActiveFireChunkSet>()
            .init_resource::<crate::render::Stage5FireViewChunkWitness>()
            .init_resource::<crate::render::FirePlaybackStabilityWitness>()
            .init_resource::<crate::render::Stage5ReadinessProfile>()
            .init_resource::<crate::render::view_runtime::PerViewRepresentationPolicy>()
            .init_resource::<SmokeVisualBridgeWitness>()
            .init_resource::<crate::render::FireExtractCadence>()
            .init_resource::<crate::render::FireExtractClock>()
            .init_resource::<crate::render::FireExtractDiagnostics>()
            .configure_sets(Update, crate::render::fire_streaming::FireStreamingSleepWakeSet)
            .configure_sets(
                Update,
                (
                    FireVisualFrameSet::BuildProfiles
                        .after(ViewAuthoritySystemSet::SyncViewManager)
                        .after(MapCameraSystemSet::Smooth),
                    FireVisualFrameSet::BuildClusters.after(FireVisualFrameSet::BuildProfiles),
                    FireVisualFrameSet::BuildAtmosphere.after(FireVisualFrameSet::BuildClusters),
                    FireVisualFrameSet::EmitLights.after(FireVisualFrameSet::BuildAtmosphere),
                    FireVisualFrameSet::EmitSmoke.after(FireVisualFrameSet::EmitLights),
                    FireVisualFrameSet::ProjectGpu.after(FireVisualFrameSet::EmitSmoke),
                    FireVisualFrameSet::EmitParticles.after(FireVisualFrameSet::ProjectGpu),
                    FireVisualFrameSet::EmitDomainOverlays.after(FireVisualFrameSet::ProjectGpu),
                ),
            )
            .add_systems(
                Update,
                (
                    crate::render::fire_streaming::apply_fire_streaming_sleep_wake_system
                        .after(extract_fire_simulation_snapshot)
                        .in_set(crate::render::fire_streaming::FireStreamingSleepWakeSet),
                    sync_active_fire_chunk_set
                        .after(crate::render::fire_streaming::FireStreamingSleepWakeSet)
                        .before(build_fire_visual_frames_by_view),
                    crate::render::stall_substage_fire_sync_active,
                ),
            )
            .add_systems(
                Update,
                (
                    attrib_fire_pipeline_before,
                    extract_fire_simulation_snapshot,
                    crate::render::stall_substage_fire_sim_snapshot,
                    sync_shared_overlay_from_simulation.after(extract_fire_simulation_snapshot),
                    crate::render::stall_substage_fire_sync_overlay,
                    sync_visible_fire_chunks_from_views.after(extract_fire_simulation_snapshot),
                    crate::render::stall_substage_fire_sync_visible,
                    sync_fire_chunk_lod_from_snapshot.after(extract_fire_simulation_snapshot),
                    crate::render::stall_substage_fire_sync_lod,
                    // PERF-INSTR-VFX-001: bracket the per-view extract/LOD rebuild → `fire_build_view`.
                    attrib_fire_build_view_before
                        .after(sync_active_fire_chunk_set)
                        .after(sync_visible_fire_chunks_from_views)
                        .after(sync_fire_chunk_lod_from_snapshot),
                    build_fire_visual_frames_by_view
                        .after(attrib_fire_build_view_before)
                        .after(sync_active_fire_chunk_set)
                        .after(sync_visible_fire_chunks_from_views)
                        .after(sync_fire_chunk_lod_from_snapshot),
                    crate::render::stall_substage_fire_build_view,
                    attrib_fire_build_view_after.after(build_fire_visual_frames_by_view),
                    sync_sim_fire_emitter_visual_from_frame.after(build_fire_visual_frames_by_view),
                    crate::render::stall_substage_fire_emitter_sync,
                    sync_atmosphere_diag_fire_instance_count.after(sync_sim_fire_emitter_visual_from_frame),
                    commit_fire_visual_snapshot.after(sync_atmosphere_diag_fire_instance_count),
                    crate::render::stall_substage_fire_commit,
                    // Close the BuildProfiles timer here — do NOT wait for Clusters/ProjectGpu or
                    // unrelated Update systems (world repr, minimap compositor) inflate `fire_pipeline_ms`.
                    attrib_fire_pipeline_after,
                )
                    .chain()
                    .in_set(FireVisualFrameSet::BuildProfiles),
            )
            .add_systems(
                Update,
                crate::render::fire_streaming::write_fire_streaming_live_proof_system
                    .after(FireVisualFrameSet::BuildProfiles)
                    .run_if(in_state(crate::engine::states::BaseState::Simulation)),
            )
            .add_systems(
                Update,
                build_fire_clusters_into_scratch.in_set(FireVisualFrameSet::BuildClusters),
            )
            .add_systems(
                Update,
                aggregate_fire_atmosphere_from_frame.in_set(FireVisualFrameSet::BuildAtmosphere),
            )
            .add_systems(
                Update,
                emit_fire_light_requests_from_cluster_scratch.in_set(FireVisualFrameSet::EmitLights),
            )
            .add_systems(
                Update,
                build_smoke_visual_extract.in_set(FireVisualFrameSet::EmitSmoke),
            )
            .add_systems(
                Update,
                (
                    // PERF-INSTR-VFX-001: bracket the fire-node CPU projection → `fire_project`.
                    attrib_fire_project_before,
                    run_render_projection_graph,
                    attrib_fire_project_after,
                )
                    .chain()
                    .in_set(FireVisualFrameSet::ProjectGpu),
            )
            .add_systems(
                Update,
                (
                    // PERF-INSTR-VFX-001: bracket the WorldFireParticleFrame build → `fire_particles`.
                    attrib_fire_particles_before,
                    crate::render::gpu_particles::sync_fire_particle_camera_scale,
                    emit_world_fire_particles_from_projection,
                    attrib_fire_particles_after,
                )
                    .chain()
                    .in_set(FireVisualFrameSet::EmitParticles),
            )
            .add_systems(
                Update,
                crate::render::emit_domain_overlay_frame_from_projection
                    .in_set(FireVisualFrameSet::EmitDomainOverlays),
            );
    }
}

fn sim_has_display_chunk_heat(
    sim: &FireSimulationSnapshot,
    residency: Option<&crate::io::streaming::ChunkResidencyTable>,
) -> bool {
    sim.chunk_heat.iter().any(|h| {
        if h.heat < CHUNK_FIRE_OVERLAY_DISPLAY_MIN {
            return false;
        }
        residency.is_none_or(|table| crate::render::chunk_in_residency_table(h.chunk, table))
    })
}

fn build_chunk_fire_heat_overlay_map(
    sim: &FireSimulationSnapshot,
    residency: Option<&crate::io::streaming::ChunkResidencyTable>,
) -> HashMap<IVec2, f32> {
    let mut next = HashMap::new();
    for h in &sim.chunk_heat {
        if h.heat < CHUNK_FIRE_OVERLAY_DISPLAY_MIN {
            continue;
        }
        if let Some(table) = residency {
            if !crate::render::chunk_in_residency_table(h.chunk, table) {
                continue;
            }
        }
        let e = next.entry(h.chunk).or_insert(0.0);
        *e = f32::max(*e, h.heat);
    }
    next
}

pub fn sync_shared_overlay_from_simulation(
    sim: Res<FireSimulationSnapshot>,
    residency: Option<Res<crate::io::streaming::ChunkResidencyTable>>,
    mut shared: ResMut<SharedOverlayFieldBuffers>,
    profile: Res<crate::render::Stage5ReadinessProfile>,
    mut fire_playback: ResMut<crate::render::FirePlaybackStabilityWitness>,
) {
    let _perf = crate::render::PerfScope::new("upd_fire_sync_overlay");
    shared.stamp = sim.stamp;
    let residency = residency.as_deref();
    let mut next = build_chunk_fire_heat_overlay_map(&sim, residency);
    let sim_has_heat = sim_has_display_chunk_heat(&sim, residency);
    // PLAY-06c: one empty sim snapshot must not wipe overlay (minimap/world tint blink).
    if next.is_empty() && !shared.chunk_fire_heat.is_empty() {
        fire_playback.note_held_overlay_frame();
        return;
    }
    // MAP-BLINK-001 / PLAY-06d: sim still has burning chunks but overlay filter emptied (residency).
    if next.is_empty() && sim_has_heat && !shared.chunk_fire_heat.is_empty() {
        fire_playback.note_held_overlay_frame();
        return;
    }
    fire_playback.held_empty_snapshot_frames = 0;
    // MAP-BLINK-001: cold-start ramp — soften first overlay revision bumps (operator pop-in).
    if shared.chunk_fire_heat.is_empty() && !next.is_empty() {
        let frames = fire_playback.overlay_warmup_frames;
        if frames < crate::render::overlay_field_buffers::OVERLAY_WARMUP_BLEND_FRAMES {
            let alpha = (frames as f32 + 1.0)
                / crate::render::overlay_field_buffers::OVERLAY_WARMUP_BLEND_FRAMES as f32;
            for heat in next.values_mut() {
                *heat *= alpha;
            }
            fire_playback.note_overlay_warmup_frame();
        }
    } else if !next.is_empty() {
        fire_playback.overlay_warmup_frames =
            crate::render::FirePlaybackStabilityWitness::OVERLAY_WARMUP_BLEND_FRAMES;
    }
    fire_playback.note_overlay_frame(next.len());
    if chunk_fire_heat_maps_differ(&shared.chunk_fire_heat, &next) {
            shared.chunk_fire_heat = next;
            shared.bump();
            if *profile == crate::render::Stage5ReadinessProfile::FULL_APP
                && crate::render::frame_perf_verbose()
            {
                info!(
                target: "stage5_overlay::live",
                "STAGE5_OVERLAY_SHARED_BUFFERS revision={} chunk_cells={} sim_tick={}",
                shared.revision,
                shared.chunk_fire_heat.len(),
                sim.stamp.tick,
            );
        }
    }
}

fn sync_sim_fire_emitter_visual_from_frame(
    by_view: Res<FireVisualFramesByView>,
    mut sim_fire: ResMut<SimFireEmitterVisualExtract>,
) {
    let frame = tactical_fire_visual(by_view.as_ref());
    sim_fire.instances.clear();
    sim_fire.instances.reserve(frame.instances.len());
    for row in &frame.instances {
        sim_fire.instances.push(row.to_fire_emitter_gpu());
    }
}

fn sync_atmosphere_diag_fire_instance_count(
    by_view: Res<FireVisualFramesByView>,
    mut diag: ResMut<AtmosphereDiagnostics>,
) {
    let frame = tactical_fire_visual(by_view.as_ref());
    diag.last_emitter_extract_count = frame.instances.len();
}

fn fire_extract_input_fingerprint(
    runtime: &FireChunkRuntime,
    residency_cells: u32,
) -> crate::render::FireExtractInputFingerprint {
    let mut active_digest = 0u64;
    for c in runtime.chunks.values() {
        if c.active || c.visual_active {
            active_digest ^= (c.coord.x as u64)
                .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                ^ (c.coord.y as u64)
                    .wrapping_mul(0xc2b2_ae3d_27d4_eb4f)
                    .rotate_left(11)
                ^ (c.max_heat.to_bits() as u64).rotate_left(23);
        }
    }
    crate::render::FireExtractInputFingerprint {
        runtime_len: runtime.chunks.len() as u32,
        active_digest,
        residency_cells,
    }
}

pub fn extract_fire_simulation_snapshot(
    tick: Res<crate::systems::sim_control::SimTick>,
    sim_time: Res<crate::systems::sim_control::SimTimeMicros>,
    time: Res<Time>,
    cadence: Res<crate::render::FireExtractCadence>,
    mut clock: ResMut<crate::render::FireExtractClock>,
    mut extract_diag: ResMut<crate::render::FireExtractDiagnostics>,
    mut sim: ResMut<FireSimulationSnapshot>,
    mut runtime: ResMut<FireChunkRuntime>,
    ecs_retire: Option<Res<crate::substrate::EcsRetireState>>,
    substrate: Option<Res<crate::substrate::WorldSubstrateRegistry>>,
    residency: Option<Res<crate::io::streaming::ChunkResidencyTable>>,
    spike_guard: Option<Res<crate::engine::UxFrameSpikeGuard>>,
    q: Query<(
        &Chunk,
        &ChunkCellMatrix,
        Option<&ChunkFireOverlay>,
        &ChunkSurfaceFire,
        &SimFireLightEmission,
        Option<&ChunkSmokeField>,
        Option<&ChunkFuelProfile>,
        Option<&ChunkEcology>,
        Option<&ChunkWeather>,
        Option<&MaterializedChunk>,
    )>,
) {
    let extract_started = std::time::Instant::now();
    let mut report = crate::render::FireExtractFrameReport {
        min_interval_secs: cadence.min_interval_secs,
        ..Default::default()
    };
    // PERF-INSTR-VFX-001: name the fire-sim ECS snapshot scan separately from the per-view rebuild
    // (`fire_build_view`) so an idle frame where this scan is the cost lands on a named line.
    let _perf = crate::render::PerfScope::new("upd_fire_sim_snapshot");
    let stamp = crate::systems::sim_control::SimStepStamp::from_tick(*tick, *sim_time);
    sim.stamp = stamp;

    let now = time.elapsed_secs();
    let tick_changed = clock.last_tick != tick.0;
    report.tick_changed = tick_changed;
    let interval_elapsed =
        (now - clock.last_full_extract_secs).max(0.0) >= cadence.min_interval_secs;
    report.interval_elapsed = interval_elapsed;
    let spike_active = spike_guard.as_deref().is_some_and(|g| g.spike_active);
    report.spike_active = spike_active;
    // Under frame spike, never run full-world ECS scan every sim tick — that locks ~200ms+ and
    // prevents raster/minimap from catching up (death spiral at ~1.5 FPS).
    let cadence_due = if spike_active {
        interval_elapsed
    } else if cadence.full_scan_on_sim_tick {
        tick_changed || interval_elapsed
    } else {
        interval_elapsed
    };
    report.cadence_due = cadence_due;

    let residency_cells = residency
        .as_ref()
        .map(|t| t.entries.len() as u32)
        .unwrap_or(0);
    let input_fingerprint = fire_extract_input_fingerprint(&runtime, residency_cells);

    // Fingerprint skip is safe even during UX spike — spike throttling above already caps cadence;
    // blocking skip here caused a death spiral (~250ms frames → spike latch → full ECS scan every
    // cadence tick → never recover).
    if cadence_due && input_fingerprint == clock.last_input_fingerprint {
        report.cadence_skipped = true;
        report.fingerprint_skipped = true;
        report.extract_ms = extract_started.elapsed().as_secs_f32() * 1000.0;
        extract_diag.last = report;
        return;
    }

    if !cadence_due {
        report.cadence_skipped = true;
        report.extract_ms = extract_started.elapsed().as_secs_f32() * 1000.0;
        extract_diag.last = report;
        return;
    }
    clock.last_tick = tick.0;
    clock.last_full_extract_secs = now;
    clock.last_input_fingerprint = input_fingerprint;
    report.ran_full_scan = true;

    sim.instances.clear();
    sim.chunk_heat.clear();

    let prev = std::mem::take(&mut runtime.chunks);
    let tick_u32 = tick.0.min(u64::from(u32::MAX)) as u32;

    let residency_table = residency.as_ref();
    let scope_residency = cadence.residency_scoped
        && residency_table.is_some_and(|t| !t.entries.is_empty());
    report.residency_scoped = scope_residency;

    let slab_fire_extract = ecs_retire
        .as_ref()
        .is_some_and(|r| r.cutover_complete && !r.hybrid_fire_authoritative)
        && substrate.is_some();

    for (chunk, matrix, overlay, fire, em, smoke, prof, eco, wx, mat_chunk) in &q {
        report.chunks_iterated = report.chunks_iterated.saturating_add(1);
        let coord = chunk.coord;
        if scope_residency {
            let in_residency = residency_table
                .is_some_and(|t| crate::render::chunk_in_residency_table(coord, t));
            let was_active = prev
                .get(&coord)
                .is_some_and(|p| p.visual_active || p.active);
            if !in_residency && !was_active {
                continue;
            }
        }
        let slab_heat = if slab_fire_extract {
            substrate
                .as_ref()
                .map(|reg| crate::substrate::slab_surface_heat(reg, chunk.coord))
        } else {
            None
        };
        let mut quick_heat = fire.heat.max(slab_heat.unwrap_or(0.0));
        if let Some(ovl) = overlay {
            if !ovl.heat.is_empty() {
                quick_heat = quick_heat.max(ovl.heat.iter().copied().fold(0.0_f32, f32::max));
            }
        }
        let was_warm = prev
            .get(&coord)
            .is_some_and(|p| p.visual_active || p.active);
        if quick_heat <= FIRE_SIM_CHUNK_ACTIVE_EPS && !was_warm {
            report.chunks_fast_path = report.chunks_fast_path.saturating_add(1);
            runtime.chunks.insert(
                coord,
                FireChunk {
                    coord,
                    active: false,
                    visual_active: false,
                    heat_sum: quick_heat,
                    max_heat: quick_heat,
                    last_active_tick: prev
                        .get(&coord)
                        .map(|p| p.last_active_tick)
                        .unwrap_or(0),
                    dirty: false,
                },
            );
            continue;
        }
        report.chunks_profiled = report.chunks_profiled.saturating_add(1);
        let profile = infer_fire_emission_profile(
            chunk,
            fire,
            overlay,
            em,
            smoke,
            eco,
            prof,
            wx,
            matrix,
            mat_chunk,
            slab_heat,
        );
        sim.instances.push(FireVisualGpuInstance::from(&profile));
        report.instances_written = report.instances_written.saturating_add(1);
        let coord = profile.chunk_coord;
        sim.chunk_heat.push(ChunkFireHeat {
            chunk: coord,
            heat: profile.heat,
            smoke: profile.smoke_density,
        });
        report.chunk_heat_written = report.chunk_heat_written.saturating_add(1);

        let visual_active = profile.heat > FIRE_VISUAL_ACTIVE_HEAT_EPS;
        let active = profile.heat > FIRE_SIM_CHUNK_ACTIVE_EPS;
        let last_active_tick = if active {
            tick_u32
        } else {
            prev.get(&coord).map(|p| p.last_active_tick).unwrap_or(0)
        };
        let dirty = prev
            .get(&coord)
            .map(|p| {
                (p.max_heat - profile.heat).abs() > 1e-4
                    || p.visual_active != visual_active
                    || (p.heat_sum - profile.heat).abs() > 1e-4
            })
            .unwrap_or(true);

        runtime.chunks.insert(
            coord,
            FireChunk {
                coord,
                active,
                visual_active,
                heat_sum: profile.heat,
                max_heat: profile.heat,
                last_active_tick,
                dirty,
            },
        );
    }

    // vm-08 / fire-active-chunk-runtime: rim chunks adjacent to burning sim cells stay visually active
    // (CPU-only policy; GPU path still consumes the same snapshot).
    const NEIGHBOR: [IVec2; 4] = [
        IVec2::new(1, 0),
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
        IVec2::new(0, -1),
    ];
    let mut neighbor_glow: Vec<ChunkCoord> = Vec::new();
    for (&c, ch) in &runtime.chunks {
        if ch.visual_active {
            continue;
        }
        for d in &NEIGHBOR {
            let n = c + *d;
            if runtime
                .chunks
                .get(&n)
                .is_some_and(|x| x.max_heat > FIRE_VISUAL_ACTIVE_HEAT_EPS)
            {
                neighbor_glow.push(c);
                break;
            }
        }
    }
    for c in neighbor_glow {
        if let Some(ch) = runtime.chunks.get_mut(&c) {
            if ch.max_heat <= FIRE_VISUAL_ACTIVE_HEAT_EPS {
                ch.visual_active = true;
                ch.last_active_tick = tick_u32;
                ch.dirty = true;
            }
        }
    }

    // Cool-down: rim-only `visual_active` clears shortly after no adjacent burning chunk.
    const RIM_VISUAL_HOLD_TICKS: u32 = 12;
    #[derive(Clone, Copy)]
    enum RimDecayOp {
        RefreshTick,
        ClearVisual,
    }
    let mut decay_ops: Vec<(ChunkCoord, RimDecayOp)> = Vec::new();
    for (&coord, ch) in &runtime.chunks {
        if ch.max_heat > FIRE_VISUAL_ACTIVE_HEAT_EPS || !ch.visual_active {
            continue;
        }
        let neighbor_hot = NEIGHBOR.iter().any(|d| {
            runtime
                .chunks
                .get(&(coord + *d))
                .map(|n| n.max_heat > FIRE_VISUAL_ACTIVE_HEAT_EPS)
                .unwrap_or(false)
        });
        if neighbor_hot {
            decay_ops.push((coord, RimDecayOp::RefreshTick));
        } else if tick_u32.saturating_sub(ch.last_active_tick) > RIM_VISUAL_HOLD_TICKS {
            decay_ops.push((coord, RimDecayOp::ClearVisual));
        }
    }
    for (coord, op) in decay_ops {
        if let Some(ch) = runtime.chunks.get_mut(&coord) {
            match op {
                RimDecayOp::RefreshTick => {
                    ch.last_active_tick = tick_u32;
                    ch.dirty = true;
                }
                RimDecayOp::ClearVisual => {
                    ch.visual_active = false;
                    ch.dirty = true;
                }
            }
        }
    }

    report.runtime_chunks = runtime.chunks.len().min(u32::MAX as usize) as u32;
    report.extract_ms = extract_started.elapsed().as_secs_f32() * 1000.0;
    extract_diag.last = report;
}

fn build_fire_clusters_into_scratch(
    by_view: Res<FireVisualFramesByView>,
    mut scratch: ResMut<FireClusterScratch>,
) {
    let frame = tactical_fire_visual(by_view.as_ref());
    scratch.clusters.clear();
    let samples: Vec<VisFireLightSample> = frame
        .instances
        .iter()
        .map(FireVisualGpuInstance::cluster_emission)
        .collect();
    scratch.clusters = build_fire_light_clusters(&samples);
}

fn aggregate_fire_atmosphere_from_frame(
    mut agg: ResMut<FireAtmosphereAggregate>,
    by_view: Res<FireVisualFramesByView>,
) {
    let frame = tactical_fire_visual(by_view.as_ref());
    if frame.instances.is_empty() {
        *agg = FireAtmosphereAggregate::default();
        return;
    }
    let n = frame.instances.len() as f32;
    let mut total_smoke = 0f32;
    let mut mean_color = Vec3::ZERO;
    let mut heat_energy = 0f32;
    let mut ember = 0f32;
    let mut vis = 0f32;
    for row in &frame.instances {
        total_smoke += row.smoke_ember_vis_priority.x;
        mean_color += row.smoke_color_toxic.xyz();
        heat_energy += row.heat() * row.luminosity().max(0.01);
        ember += row.smoke_ember_vis_priority.y;
        vis += row.smoke_ember_vis_priority.z;
    }
    agg.smoke_density = (total_smoke / n).clamp(0.0, 1.0);
    agg.smoke_color = (mean_color / n.max(1.0)).clamp(Vec3::ZERO, Vec3::ONE);
    agg.heat_energy = heat_energy;
    agg.ember_density = (ember / n).clamp(0.0, 1.0);
    agg.visibility_loss = (vis / n).clamp(0.0, 1.0);
}

const LUMINOSITY_TO_POINTLIGHT: f32 = 24_000.0;

fn emit_fire_light_requests_from_cluster_scratch(
    scratch: Res<FireClusterScratch>,
    mut writer: MessageWriter<RequestLocalLight>,
) {
    for cluster in &scratch.clusters {
        writer.write(cluster_to_request(cluster));
    }
}

fn cluster_to_request(cluster: &FireLightCluster) -> RequestLocalLight {
    let color = match cluster.dominant_type {
        FireLightType::Forest => Color::srgb(1.0, 0.42, 0.18),
        FireLightType::Fuel => Color::srgb(1.0, 0.55, 0.25),
        FireLightType::Chemical => Color::srgb(0.4, 1.0, 0.3),
        FireLightType::Electrical => Color::srgb(0.6, 0.8, 1.0),
        FireLightType::Structure => Color::srgb(1.0, 0.35, 0.12),
    };

    let intensity = cluster.total_luminosity.max(0.05)
        * LUMINOSITY_TO_POINTLIGHT
        * (cluster.member_count as f32).sqrt().max(1.0);
    let range = 80.0 + cluster.radius * 1.8;
    let priority = cluster.total_heat + cluster.member_count as f32 * 0.25;
    let flicker_phase = cluster.centroid.x * 0.013;
    let flicker_strength = 0.03 + cluster.total_heat * 0.05;

    RequestLocalLight {
        position: cluster.centroid,
        color,
        intensity,
        range,
        priority,
        category: LightCategory::Fire,
        flicker_phase,
        flicker_strength,
    }
}

#[cfg(test)]
mod vt1_full_world_fire_extract_tests {
    //! VT-1 (`visual-test-matrix-upgrade`): fire visual rows must span **multiple chunks** in world space,
    //! catching chunk-index collapse or local-space bugs that show up as “everything in one square”.

    use super::FireVisualFramePlugin;
    use crate::render::extraction::{
        RenderProjectionGraph, CLUSTERED_FIRE_INSTANCE_CAP,
    };
    use crate::render::{tactical_fire_visual, FireVisualFramesByView};
    use crate::gui::{
        build_representation_inputs, build_representation_result, LodInputs, LodZoneRegistry,
        RepresentationResult, VisualBudgetSettings, VisualCadence, WorldLodBand, WorldLodMap,
        WorldLodPolicyEngine, WorldRepresentationFrame,
    };
    use crate::render::light::RequestLocalLight;
    use crate::render::SharedOverlayFieldBuffersPlugin;
    use crate::systems::atmosphere::AtmosphereDiagnostics;
    use crate::systems::fire::{ChunkSurfaceFire, FireLightEmission};
    use crate::terrain::generation::{Chunk, ChunkCellMatrix};
    use bevy::math::{IVec2, UVec2};
    use bevy::prelude::*;

    fn sample_emitter() -> FireLightEmission {
        FireLightEmission {
            radius: 120.0,
            base_intensity: 1.0,
            current_intensity: 1.0,
            flicker_strength: 0.1,
            flicker_phase: 0.0,
            extract_priority: 1.0,
        }
    }

    fn sync_representation_policy(world: &mut World) {
        let mut frame = world.resource::<WorldRepresentationFrame>().clone();
        let band = frame.global_band();
        frame.visibility = crate::gui::visibility_for_band(band);
        frame.resolution = crate::gui::resolution_for_band(band);
        let sim = *world.resource::<crate::systems::sim_control::SimTick>();
        let micros = *world.resource::<crate::systems::sim_control::SimTimeMicros>();
        let stamp = crate::systems::sim_control::SimStepStamp::from_tick(sim, micros);
        let budgets = VisualBudgetSettings::default();
        let inputs = build_representation_inputs(
            &Default::default(),
            &LodZoneRegistry::default(),
            &budgets,
            &VisualCadence::from(&budgets),
            stamp,
        );
        *world.resource_mut::<RepresentationResult>() =
            build_representation_result(&frame, &inputs);
    }

    #[test]
    fn fire_visual_instances_span_multiple_chunks_in_world_space() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<RequestLocalLight>();
        app.add_plugins(SharedOverlayFieldBuffersPlugin);
        app.init_resource::<AtmosphereDiagnostics>();
        app.init_resource::<crate::systems::sim_control::SimTick>();
        app.init_resource::<crate::systems::sim_control::SimTimeMicros>();
        app.init_resource::<WorldLodMap>();
        app.init_resource::<WorldRepresentationFrame>();
        app.init_resource::<RepresentationResult>();
        app.add_plugins(FireVisualFramePlugin);

        let cell = UVec2::new(16, 16);
        for coord in [
            IVec2::new(0, 0),
            IVec2::new(4, 0),
            IVec2::new(-3, 2),
        ] {
            app.world_mut().spawn((
                Chunk { coord },
                ChunkCellMatrix::new(cell),
                ChunkSurfaceFire {
                    heat: 0.55,
                    fuel: 1.0,
                },
                sample_emitter(),
            ));
        }

        sync_representation_policy(app.world_mut());
        app.update();

        let by_view = app.world().resource::<FireVisualFramesByView>();
        let frame = tactical_fire_visual(by_view);
        assert_eq!(
            frame.instances.len(),
            3,
            "expected one GPU row per burning chunk"
        );

        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut chunk_keys = std::collections::HashSet::new();
        for row in &frame.instances {
            let gx = row.chunk_xy_heat_lum.x;
            let gy = row.chunk_xy_heat_lum.y;
            chunk_keys.insert((gx as i32, gy as i32));
            let w = row.world_xyz_radius;
            min_x = min_x.min(w.x);
            max_x = max_x.max(w.x);
            min_y = min_y.min(w.y);
            max_y = max_y.max(w.y);
        }
        assert_eq!(chunk_keys.len(), 3, "chunk indices must not collapse");

        let span_x = max_x - min_x;
        let span_y = max_y - min_y;
        let chunk_w = cell.x as f32;
        assert!(
            span_x > chunk_w * 1.5 || span_y > chunk_w * 1.5,
            "world sample positions should span more than one chunk width (span_x={span_x}, span_y={span_y}, chunk_w={chunk_w})"
        );
    }

    #[test]
    fn strategic_band_keeps_full_frame_but_projection_drops_gpu_instances() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<RequestLocalLight>();
        app.add_plugins(SharedOverlayFieldBuffersPlugin);
        app.init_resource::<AtmosphereDiagnostics>();
        app.init_resource::<crate::systems::sim_control::SimTick>();
        app.init_resource::<crate::systems::sim_control::SimTimeMicros>();
        app.init_resource::<WorldLodMap>();
        app.init_resource::<WorldRepresentationFrame>();
        app.init_resource::<RepresentationResult>();
        app.add_plugins(FireVisualFramePlugin);

        {
            let mut w = app.world_mut().resource_mut::<WorldRepresentationFrame>();
            w.bands.global = WorldLodBand::Strategic;
        }
        sync_representation_policy(app.world_mut());

        let cell = UVec2::new(8, 8);
        app.world_mut().spawn((
            Chunk { coord: IVec2::new(1, 0) },
            ChunkCellMatrix::new(cell),
            ChunkSurfaceFire {
                heat: 0.6,
                fuel: 1.0,
            },
            sample_emitter(),
        ));

        app.update();

        let by_view = app.world().resource::<FireVisualFramesByView>();
        let frame = tactical_fire_visual(by_view);
        assert_eq!(frame.instances.len(), 1);
        assert_eq!(frame.chunk_heat.len(), 1);
        assert_eq!(frame.chunk_heat[0].chunk, IVec2::new(1, 0));

        let graph = app.world().resource::<RenderProjectionGraph>();
        let gpu = &graph.fire;
        assert_eq!(gpu.instance_buffer.len(), 1);
        assert_eq!(gpu.chunk_heat.len(), 1);
    }

    #[test]
    fn operational_band_caps_projected_instances_full_frame_untouched() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<RequestLocalLight>();
        app.add_plugins(SharedOverlayFieldBuffersPlugin);
        app.init_resource::<AtmosphereDiagnostics>();
        app.init_resource::<crate::systems::sim_control::SimTick>();
        app.init_resource::<crate::systems::sim_control::SimTimeMicros>();
        app.init_resource::<WorldLodMap>();
        app.init_resource::<WorldRepresentationFrame>();
        app.init_resource::<RepresentationResult>();
        app.add_plugins(FireVisualFramePlugin);

        {
            let engine = WorldLodPolicyEngine::default();
            let mut w = app.world_mut().resource_mut::<WorldRepresentationFrame>();
            *w = engine.evaluate(
                LodInputs {
                    zoom_level: 0.5,
                    ..Default::default()
                },
                &[],
            );
        }
        sync_representation_policy(app.world_mut());

        let cell = UVec2::new(4, 4);
        let n = CLUSTERED_FIRE_INSTANCE_CAP + 7;
        for i in 0..n {
            app.world_mut().spawn((
                Chunk {
                    coord: IVec2::new(i as i32, 0),
                },
                ChunkCellMatrix::new(cell),
                ChunkSurfaceFire {
                    heat: 0.95,
                    fuel: 1.0,
                },
                sample_emitter(),
            ));
        }

        sync_representation_policy(app.world_mut());
        app.update();

        let by_view = app.world().resource::<FireVisualFramesByView>();
        let frame = tactical_fire_visual(by_view);
        assert_eq!(frame.chunk_heat.len(), n);
        assert_eq!(frame.instances.len(), n);

        let graph = app.world().resource::<RenderProjectionGraph>();
        let gpu = &graph.fire;
        assert_eq!(gpu.instance_buffer.len(), CLUSTERED_FIRE_INSTANCE_CAP);
        assert!(gpu.chunk_heat.len() < n, "chunk heat should be binned for GPU projection");
    }

    #[test]
    fn committed_fence_matches_fire_stamp_before_projection() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<RequestLocalLight>();
        app.add_plugins(SharedOverlayFieldBuffersPlugin);
        app.init_resource::<AtmosphereDiagnostics>();
        app.init_resource::<crate::systems::sim_control::SimTick>();
        app.init_resource::<crate::systems::sim_control::SimTimeMicros>();
        app.init_resource::<WorldLodMap>();
        app.init_resource::<WorldRepresentationFrame>();
        app.init_resource::<RepresentationResult>();
        app.add_plugins(FireVisualFramePlugin);

        let cell = UVec2::new(8, 8);
        app.world_mut().spawn((
            Chunk { coord: IVec2::ZERO },
            ChunkCellMatrix::new(cell),
            ChunkSurfaceFire {
                heat: 0.6,
                fuel: 1.0,
            },
            sample_emitter(),
        ));

        sync_representation_policy(app.world_mut());
        app.update();

        let by_view = app.world().resource::<FireVisualFramesByView>();
        let frame = tactical_fire_visual(by_view);
        let fence = app
            .world()
            .resource::<crate::render::CommittedVisualSnapshotFence>();
        assert_eq!(fence.fire, frame.stamp);
        let graph = app.world().resource::<RenderProjectionGraph>();
        assert!(!graph.fire.instance_buffer.is_empty());
    }
}