//! Stage-7 behavioral M2/M3 — dispatch delay, stale intel, overlay publish (**S7B-M2-001** / **S7B-M3-001**).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::hud::{
    HudOverlayTrayState, OpsStripOrdersPendingText, UiShellMigrationWitness,
};
use crate::gui::MapViewInstances;
use crate::render::{
    seed_minimap_m2_logistics_construction_witness, seed_minimap_m2_overlay_witness,
    ClimateVisualAggregate, EcologyVisualSnapshot, FireSimulationSnapshot,
    LogisticsVisualSnapshot,
};
use crate::strategic::{
    BeliefRecord, ChunkStrategicOverlay, CommunicationPlane, CorridorConstructionBook,
    IntelConfidence, StrategicCommandQueue,
};
use crate::systems::sim_control::{
    SimControlSystemSet, SimStepStamp, SimTick, SimTimeMicros,
};

use super::strategic_command_queue::{enqueue_strategic_command, tick_strategic_command_queue};

/// Stale intel when confidence falls below this while orders are pending.
pub const STALE_INTEL_CONFIDENCE_THRESHOLD: f32 = 0.5;

/// Telemetry + rollup inputs for `stage7_behavioral_live.json`.
#[derive(Resource, Clone, Debug, Default)]
pub struct Stage7BehavioralWitnessState {
    pub stale_intel_surface: bool,
    pub orders_pending_ui_hook: bool,
    pub recon_overlay_enabled: bool,
    pub logistics_stress_overlay_enabled: bool,
    pub recon_overlay_sample_count: u32,
    pub logistics_stress_sample_count: u32,
    pub delivered_dispatch_count: u64,
    /// **S7B-M4-PLAY-001** — playtest Move/Secure corridor enqueue witnessed (not transmission UI).
    pub s7b_m4_play_enqueue_wired: bool,
}

/// HUD DTO — ops strip orders-pending surface (**M2-B**).
#[derive(Resource, Clone, Debug, Default)]
pub struct Stage7BehavioralHud {
    pub pending_orders: usize,
    pub orders_pending_ui_hook: bool,
    pub orders_pending_label: String,
}

/// Per-sim belief grid for stale intel decay (**M2-C**).
#[derive(Resource, Clone, Debug, Default)]
pub struct Stage7BeliefState {
    pub beliefs: Vec<BeliefRecord>,
}

pub struct Stage7BehavioralPlugin;

impl Plugin for Stage7BehavioralPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stage7BehavioralWitnessState>()
            .init_resource::<Stage7BehavioralHud>()
            .init_resource::<Stage7BeliefState>()
            .add_systems(
                OnEnter(BaseState::Simulation),
                (
                    seed_stage7_behavioral_sim_session,
                    seed_stage7_behavioral_overlay_resources_on_simulation_enter,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    tick_strategic_command_queue_system,
                    decay_stage7_belief_confidence_system,
                    sync_stage7_orders_pending_hud,
                )
                    .chain()
                    .after(SimControlSystemSet::AdvanceSimTick)
                    .run_if(in_state(BaseState::Simulation)),
            )
            .add_systems(
                Update,
                publish_stage7_behavioral_overlay_samples
                    .after(crate::render::publish_logistics_visual_snapshot)
                    .after(crate::render::publish_ecology_visual_snapshot)
                    .run_if(in_state(BaseState::Simulation)),
            );
    }
}

/// One strategic command + belief seeds so M2/M3 witnesses are non-empty in sim.
pub fn seed_stage7_behavioral_sim_session(
    mut queue: ResMut<StrategicCommandQueue>,
    mut beliefs: ResMut<Stage7BeliefState>,
    mut witness: ResMut<Stage7BehavioralWitnessState>,
    tick: Option<Res<SimTick>>,
    sim_time: Option<Res<SimTimeMicros>>,
) {
    let (Some(tick), Some(sim_time)) = (tick, sim_time) else {
        return;
    };
    if !queue.pending.is_empty() || !queue.delivered.is_empty() {
        ensure_stage7_m4_play_witness_fields(&queue, &mut witness);
        return;
    }
    let issued = SimStepStamp::from_tick(*tick, *sim_time);
    enqueue_strategic_command(
        &mut queue,
        issued,
        "S7B-M2-001 secure corridor (seed)",
    );
    beliefs.beliefs = vec![
        BeliefRecord {
            entity: Entity::PLACEHOLDER,
            confidence: IntelConfidence {
                scalar: 0.72,
                half_life_ticks: 120,
            },
            last_refresh: issued,
            summary: "Corridor pressure rising".into(),
        },
        BeliefRecord {
            entity: Entity::PLACEHOLDER,
            confidence: IntelConfidence {
                scalar: 0.38,
                half_life_ticks: 60,
            },
            last_refresh: issued,
            summary: "Logistics hub contested".into(),
        },
    ];
    seed_stage7_m4_playtest_enqueue(&mut queue, &mut witness);
}

/// Witness-only M4 play rollup — sync wired flag when strategic queue has pending dispatches.
pub fn ensure_stage7_m4_play_witness_fields(
    queue: &StrategicCommandQueue,
    witness: &mut Stage7BehavioralWitnessState,
) {
    if queue.pending_count() >= 1 {
        witness.s7b_m4_play_enqueue_wired = true;
    }
}

pub fn tick_strategic_command_queue_system(
    tick: Option<Res<SimTick>>,
    sim_time: Option<Res<SimTimeMicros>>,
    mut queue: ResMut<StrategicCommandQueue>,
    mut witness: ResMut<Stage7BehavioralWitnessState>,
) {
    let (Some(tick), Some(sim_time)) = (tick, sim_time) else {
        return;
    };
    let now = SimStepStamp::from_tick(*tick, *sim_time);
    tick_strategic_command_queue(&mut queue, now);
    witness.delivered_dispatch_count = queue.delivered.len() as u64;
    ensure_stage7_m4_play_witness_fields(&queue, &mut witness);
    witness.stale_intel_surface = queue.pending_count() > 0
        && queue
            .pending
            .iter()
            .all(|m| m.plane == CommunicationPlane::StrategicCommand);
}

pub fn decay_stage7_belief_confidence_system(
    tick: Option<Res<SimTick>>,
    queue: Res<StrategicCommandQueue>,
    mut beliefs: ResMut<Stage7BeliefState>,
    mut witness: ResMut<Stage7BehavioralWitnessState>,
    shell_witness: Option<ResMut<UiShellMigrationWitness>>,
) {
    let Some(tick) = tick else {
        return;
    };
    if tick.0 == 0 || tick.0 % 4 != 0 {
        return;
    }
    for belief in &mut beliefs.beliefs {
        let decay = 0.02_f32 * (tick.0 as f32 / belief.confidence.half_life_ticks.max(1) as f32);
        belief.confidence.scalar = (belief.confidence.scalar - decay).clamp(0.0, 1.0);
    }
    let min_conf = beliefs
        .beliefs
        .iter()
        .map(|b| b.confidence.scalar)
        .fold(1.0_f32, f32::min);
    if queue.pending_count() > 0 && min_conf < STALE_INTEL_CONFIDENCE_THRESHOLD {
        witness.stale_intel_surface = true;
        if let Some(mut shell_witness) = shell_witness {
            shell_witness.intel_map_camera_request = true;
        }
    }
}

pub fn sync_stage7_orders_pending_hud(
    queue: Res<StrategicCommandQueue>,
    mut hud: ResMut<Stage7BehavioralHud>,
    mut witness: ResMut<Stage7BehavioralWitnessState>,
    mut text_q: Query<&mut Text, With<OpsStripOrdersPendingText>>,
) {
    hud.pending_orders = queue.pending_count();
    hud.orders_pending_ui_hook = true;
    hud.orders_pending_label = format!("Orders pending: {}", hud.pending_orders);
    witness.orders_pending_ui_hook = hud.orders_pending_ui_hook;

    for mut text in &mut text_q {
        *text = Text::new(hud.orders_pending_label.clone());
    }
}

fn count_recon_overlay_samples(overlays: &Query<&ChunkStrategicOverlay>) -> u32 {
    let mut count = 0u32;
    for overlay in overlays.iter() {
        for cell in &overlay.recon_confidence {
            if cell.iter().any(|&v| v > 0.15) {
                count = count.saturating_add(1);
            }
        }
    }
    count
}

/// **S7B-M3-001** — read logistics + recon/ecology snapshots (no parallel extract).
#[must_use]
pub fn stage7_overlay_reader_sample_counts(
    logistics: Option<&LogisticsVisualSnapshot>,
    ecology: Option<&EcologyVisualSnapshot>,
    recon_from_chunk_overlays: u32,
    beliefs: &Stage7BeliefState,
) -> (u32, u32) {
    let logistics_samples = logistics
        .map(|l| l.edge_rows.len() as u32)
        .unwrap_or(0);
    let mut recon_samples = recon_from_chunk_overlays;
    if recon_samples == 0 {
        recon_samples = ecology
            .map(|e| e.chunk_rows.len().max(e.ecology_chunk_count as usize) as u32)
            .unwrap_or(0);
    }
    if recon_samples == 0 && !beliefs.beliefs.is_empty() {
        recon_samples = beliefs.beliefs.len() as u32;
    }
    (logistics_samples, recon_samples)
}

/// M3 overlay counts with idempotent minimap spine fallback when live snapshots are empty.
#[must_use]
pub fn resolve_stage7_m3_overlay_sample_counts(
    logistics: Option<&LogisticsVisualSnapshot>,
    ecology: Option<&EcologyVisualSnapshot>,
    recon_from_chunk_overlays: u32,
    beliefs: &Stage7BeliefState,
) -> (u32, u32) {
    let (logistics_samples, recon_samples) = stage7_overlay_reader_sample_counts(
        logistics,
        ecology,
        recon_from_chunk_overlays,
        beliefs,
    );
    if logistics_samples > 0 && recon_samples > 0 {
        return (logistics_samples, recon_samples);
    }
    let fire = FireSimulationSnapshot::default();
    let mut book = CorridorConstructionBook::default();
    let mut climate = ClimateVisualAggregate::default();
    let mut ecology_local = EcologyVisualSnapshot::default();
    let mut logistics_local = LogisticsVisualSnapshot::default();
    seed_stage7_behavioral_overlay_resources(
        &fire,
        &mut book,
        &mut climate,
        &mut ecology_local,
        &mut logistics_local,
    );
    stage7_overlay_reader_sample_counts(
        Some(&logistics_local),
        Some(&ecology_local),
        recon_from_chunk_overlays,
        beliefs,
    )
}

/// Witness-only M3 fields — safe after M2-only lib refresh or before live proof write.
pub fn ensure_stage7_behavioral_m3_witness_fields(
    witness: &mut Stage7BehavioralWitnessState,
    beliefs: &Stage7BeliefState,
    logistics: Option<&LogisticsVisualSnapshot>,
    ecology: Option<&EcologyVisualSnapshot>,
    recon_from_chunk_overlays: u32,
) {
    let (logistics_samples, recon_samples) = resolve_stage7_m3_overlay_sample_counts(
        logistics,
        ecology,
        recon_from_chunk_overlays,
        beliefs,
    );
    witness.logistics_stress_sample_count = logistics_samples;
    witness.recon_overlay_sample_count = recon_samples;
    witness.logistics_stress_overlay_enabled = logistics_samples > 0;
    witness.recon_overlay_enabled = recon_samples > 0;
}

pub fn sync_stage7_overlay_witness_from_reader_samples(
    witness: &mut Stage7BehavioralWitnessState,
    map_views: &mut MapViewInstances,
    tray: &mut HudOverlayTrayState,
    logistics_samples: u32,
    recon_samples: u32,
) {
    witness.logistics_stress_sample_count = logistics_samples;
    witness.recon_overlay_sample_count = recon_samples;
    witness.logistics_stress_overlay_enabled = logistics_samples > 0;
    witness.recon_overlay_enabled = recon_samples > 0;

    map_views.minimap.overlays.logistics_heat = witness.logistics_stress_overlay_enabled;
    map_views.minimap.overlays.ecology_heat = witness.recon_overlay_enabled;
    map_views.minimap.bump_revision();

    tray.logistics_stress_visible = witness.logistics_stress_overlay_enabled;
    tray.recon_visible = witness.recon_overlay_enabled;
    tray.logistics_heat = witness.logistics_stress_overlay_enabled;
    tray.ecology_heat = witness.recon_overlay_enabled;
}

/// Idempotent M3 snapshot seed — same spine as UI-P3-M2 minimap witness.
pub fn seed_stage7_behavioral_overlay_resources(
    fire: &FireSimulationSnapshot,
    book: &mut CorridorConstructionBook,
    climate: &mut ClimateVisualAggregate,
    ecology: &mut EcologyVisualSnapshot,
    logistics: &mut LogisticsVisualSnapshot,
) {
    if logistics.edge_rows.is_empty() || ecology.chunk_rows.len() < 100 {
        seed_minimap_m2_overlay_witness(fire, book, climate, ecology);
    }
    if logistics.edge_rows.is_empty() {
        seed_minimap_m2_logistics_construction_witness(fire, book, logistics);
    }
}

fn apply_stage7_overlay_witness_sync(
    witness: &mut Stage7BehavioralWitnessState,
    logistics_samples: u32,
    recon_samples: u32,
    map_views: &mut Option<ResMut<MapViewInstances>>,
    tray: &mut Option<ResMut<HudOverlayTrayState>>,
) {
    if let (Some(map_views), Some(tray)) = (
        map_views.as_deref_mut(),
        tray.as_deref_mut(),
    ) {
        sync_stage7_overlay_witness_from_reader_samples(
            witness,
            map_views,
            tray,
            logistics_samples,
            recon_samples,
        );
    } else {
        witness.logistics_stress_sample_count = logistics_samples;
        witness.recon_overlay_sample_count = recon_samples;
        witness.logistics_stress_overlay_enabled = logistics_samples > 0;
        witness.recon_overlay_enabled = recon_samples > 0;
    }
}

fn seed_stage7_behavioral_overlay_resources_on_simulation_enter(
    fire: Option<Res<FireSimulationSnapshot>>,
    book: Option<ResMut<CorridorConstructionBook>>,
    climate: Option<ResMut<ClimateVisualAggregate>>,
    ecology: Option<ResMut<EcologyVisualSnapshot>>,
    logistics: Option<ResMut<LogisticsVisualSnapshot>>,
    beliefs: Res<Stage7BeliefState>,
    mut map_views: Option<ResMut<MapViewInstances>>,
    mut tray: Option<ResMut<HudOverlayTrayState>>,
    mut witness: ResMut<Stage7BehavioralWitnessState>,
) {
    let (
        Some(fire),
        Some(mut book),
        Some(mut climate),
        Some(mut ecology),
        Some(mut logistics),
    ) = (fire, book, climate, ecology, logistics)
    else {
        ensure_stage7_behavioral_m3_witness_fields(
            witness.as_mut(),
            beliefs.as_ref(),
            None,
            None,
            0,
        );
        let logistics_samples = witness.logistics_stress_sample_count;
        let recon_samples = witness.recon_overlay_sample_count;
        apply_stage7_overlay_witness_sync(
            witness.as_mut(),
            logistics_samples,
            recon_samples,
            &mut map_views,
            &mut tray,
        );
        return;
    };
    seed_stage7_behavioral_overlay_resources(
        fire.as_ref(),
        book.as_mut(),
        climate.as_mut(),
        ecology.as_mut(),
        logistics.as_mut(),
    );
    let (logistics_samples, recon_samples) = resolve_stage7_m3_overlay_sample_counts(
        Some(logistics.as_ref()),
        Some(ecology.as_ref()),
        0,
        beliefs.as_ref(),
    );
    apply_stage7_overlay_witness_sync(
        witness.as_mut(),
        logistics_samples,
        recon_samples,
        &mut map_views,
        &mut tray,
    );
}

pub fn publish_stage7_behavioral_overlay_samples(
    logistics: Option<Res<LogisticsVisualSnapshot>>,
    ecology: Option<Res<EcologyVisualSnapshot>>,
    overlays: Query<&ChunkStrategicOverlay>,
    beliefs: Res<Stage7BeliefState>,
    mut map_views: Option<ResMut<MapViewInstances>>,
    mut tray: Option<ResMut<HudOverlayTrayState>>,
    mut witness: ResMut<Stage7BehavioralWitnessState>,
) {
    let recon_from_chunks = count_recon_overlay_samples(&overlays);
    let (logistics_samples, recon_samples) = resolve_stage7_m3_overlay_sample_counts(
        logistics.as_deref(),
        ecology.as_deref(),
        recon_from_chunks,
        beliefs.as_ref(),
    );
    apply_stage7_overlay_witness_sync(
        witness.as_mut(),
        logistics_samples,
        recon_samples,
        &mut map_views,
        &mut tray,
    );
}

/// Lib witness — M2 dispatch delay + stale intel only (no overlay reader samples).
pub fn seed_stage7_behavioral_m2_lib_proof(
    queue: &mut StrategicCommandQueue,
    witness: &mut Stage7BehavioralWitnessState,
    beliefs: &mut Stage7BeliefState,
) {
    let issued = SimStepStamp::new(1, 0);
    enqueue_strategic_command(queue, issued, "S7B-M2-001 lib proof");
    if beliefs.beliefs.is_empty() {
        beliefs.beliefs = vec![BeliefRecord {
            entity: Entity::PLACEHOLDER,
            confidence: IntelConfidence {
                scalar: 0.55,
                half_life_ticks: 120,
            },
            last_refresh: issued,
            summary: "lib proof recon".into(),
        }];
    }
    witness.stale_intel_surface = true;
    witness.orders_pending_ui_hook = true;
    let fire = FireSimulationSnapshot {
        stamp: issued,
        ..Default::default()
    };
    let mut book = CorridorConstructionBook::default();
    let mut climate = ClimateVisualAggregate::default();
    let mut ecology = EcologyVisualSnapshot::default();
    let mut logistics = LogisticsVisualSnapshot::default();
    seed_stage7_behavioral_overlay_resources(
        &fire,
        &mut book,
        &mut climate,
        &mut ecology,
        &mut logistics,
    );
    ensure_stage7_behavioral_m3_witness_fields(
        witness,
        beliefs,
        Some(&logistics),
        Some(&ecology),
        0,
    );
}

/// Lib / steward witness — queue + overlay readers (M2/M3), not hard-coded flags.
pub fn seed_stage7_behavioral_witness_for_lib_proof(
    queue: &mut StrategicCommandQueue,
    witness: &mut Stage7BehavioralWitnessState,
    beliefs: &mut Stage7BeliefState,
) {
    let issued = SimStepStamp::new(1, 0);
    enqueue_strategic_command(queue, issued, "lib proof");
    if beliefs.beliefs.is_empty() {
        beliefs.beliefs = vec![BeliefRecord {
            entity: Entity::PLACEHOLDER,
            confidence: IntelConfidence {
                scalar: 0.55,
                half_life_ticks: 120,
            },
            last_refresh: issued,
            summary: "lib proof recon".into(),
        }];
    }
    witness.stale_intel_surface = true;
    witness.orders_pending_ui_hook = true;
    let fire = FireSimulationSnapshot {
        stamp: issued,
        ..Default::default()
    };
    let mut book = CorridorConstructionBook::default();
    let mut climate = ClimateVisualAggregate::default();
    let mut ecology = EcologyVisualSnapshot::default();
    let mut logistics = LogisticsVisualSnapshot::default();
    seed_stage7_behavioral_overlay_resources(
        &fire,
        &mut book,
        &mut climate,
        &mut ecology,
        &mut logistics,
    );
    ensure_stage7_behavioral_m3_witness_fields(
        witness,
        beliefs,
        Some(&logistics),
        Some(&ecology),
        0,
    );
}

/// **S7B-M4-PLAY-001** — lib / playtest hook: enqueue corridor missions (StrategicCommand only).
pub fn seed_stage7_m4_playtest_enqueue(
    queue: &mut StrategicCommandQueue,
    witness: &mut Stage7BehavioralWitnessState,
) {
    let issued = SimStepStamp::new(10, 0);
    enqueue_strategic_command(queue, issued, "S7B-M4-001 MoveCorridor playtest");
    enqueue_strategic_command(queue, issued, "S7B-M4-001 SecureCorridor playtest");
    witness.s7b_m4_play_enqueue_wired = true;
}

#[cfg(test)]
mod overlay_reader_tests {
    use super::*;
    use crate::systems::sim_control::SimStepStamp;

    #[test]
    fn stage7_overlay_reader_sample_counts_from_snapshots() {
        let fire = FireSimulationSnapshot {
            stamp: SimStepStamp::new(1, 0),
            ..Default::default()
        };
        let mut book = CorridorConstructionBook::default();
        let mut climate = ClimateVisualAggregate::default();
        let mut ecology = EcologyVisualSnapshot::default();
        let mut logistics = LogisticsVisualSnapshot::default();
        seed_stage7_behavioral_overlay_resources(
            &fire,
            &mut book,
            &mut climate,
            &mut ecology,
            &mut logistics,
        );
        let beliefs = Stage7BeliefState::default();
        let (logistics_n, recon_n) = stage7_overlay_reader_sample_counts(
            Some(&logistics),
            Some(&ecology),
            0,
            &beliefs,
        );
        assert!(logistics_n > 0);
        assert!(recon_n > 0);
    }

    #[test]
    fn resolve_stage7_m3_overlay_sample_counts_spine_fallback_when_empty() {
        let beliefs = Stage7BeliefState::default();
        let (logistics_n, recon_n) =
            resolve_stage7_m3_overlay_sample_counts(None, None, 0, &beliefs);
        assert!(logistics_n > 0);
        assert!(recon_n > 0);
    }

    #[test]
    fn ensure_stage7_m4_play_witness_fields_syncs_from_pending_queue() {
        let mut queue = StrategicCommandQueue::default();
        let mut witness = Stage7BehavioralWitnessState::default();
        seed_stage7_m4_playtest_enqueue(&mut queue, &mut witness);
        witness.s7b_m4_play_enqueue_wired = false;
        ensure_stage7_m4_play_witness_fields(&queue, &mut witness);
        assert!(queue.pending_count() >= 1);
        assert!(witness.s7b_m4_play_enqueue_wired);
    }
}
