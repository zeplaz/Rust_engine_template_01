//! Stage 7 behavioral witness — `debug_runs/stage7_behavioral_live.json` (DEV-CONTAIN-005).

use bevy::prelude::*;

use crate::dev::stage7_behavioral_witness::{
    build_stage7_behavioral_witness_payload, s7p_play_witness_ok_from_disk,
};
use crate::engine::states::BaseState;
use crate::render::{EcologyVisualSnapshot, LogisticsVisualSnapshot};
use crate::strategic::{
    ensure_stage7_behavioral_m3_witness_fields, ensure_stage7_m4_play_witness_fields,
    Stage7BehavioralHud, Stage7BehavioralWitnessState, Stage7BeliefState, StrategicCommandQueue,
};

use super::io::{write_enveloped_witness, write_enveloped_witness_unchecked};

pub const STAGE7_BEHAVIORAL_LIVE_JSON: &str = "debug_runs/stage7_behavioral_live.json";

#[derive(Resource, Debug)]
pub struct Stage7BehavioralLiveProofState {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
}

impl Default for Stage7BehavioralLiveProofState {
    fn default() -> Self {
        Self {
            frames_since_write: 0,
            write_interval: 120,
            written: false,
        }
    }
}

static STAGE7_BEHAVIORAL_PROOF_FILE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn stage7_behavioral_proof_file_lock() -> std::sync::MutexGuard<'static, ()> {
    STAGE7_BEHAVIORAL_PROOF_FILE_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

#[must_use]
pub fn commit_stage7_behavioral_witness(
    queue: &StrategicCommandQueue,
    behavioral: &Stage7BehavioralWitnessState,
    hud: &Stage7BehavioralHud,
) -> bool {
    let _lock = stage7_behavioral_proof_file_lock();
    let mut behavioral = behavioral.clone();
    let beliefs = Stage7BeliefState::default();
    ensure_stage7_behavioral_m3_witness_fields(&mut behavioral, &beliefs, None, None, 0);
    ensure_stage7_m4_play_witness_fields(queue, &mut behavioral);
    let s7p_ok = s7p_play_witness_ok_from_disk();
    let body = build_stage7_behavioral_witness_payload(queue, &behavioral, hud, s7p_ok);
    write_enveloped_witness_unchecked(
        "STAGE7_BEHAVIORAL",
        "stage7_behavioral_witness",
        STAGE7_BEHAVIORAL_LIVE_JSON,
        body,
    )
}

pub fn write_stage7_behavioral_witness_system(
    base: Res<State<BaseState>>,
    mut state: ResMut<Stage7BehavioralLiveProofState>,
    queue: Res<StrategicCommandQueue>,
    mut behavioral: ResMut<Stage7BehavioralWitnessState>,
    beliefs: Res<Stage7BeliefState>,
    logistics: Option<Res<LogisticsVisualSnapshot>>,
    ecology: Option<Res<EcologyVisualSnapshot>>,
    hud: Res<Stage7BehavioralHud>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    if state.frames_since_write < state.write_interval {
        return;
    }
    state.frames_since_write = 0;

    ensure_stage7_behavioral_m3_witness_fields(
        behavioral.as_mut(),
        beliefs.as_ref(),
        logistics.as_deref(),
        ecology.as_deref(),
        0,
    );
    ensure_stage7_m4_play_witness_fields(queue.as_ref(), behavioral.as_mut());

    let s7p_ok = s7p_play_witness_ok_from_disk();
    let body = build_stage7_behavioral_witness_payload(
        queue.as_ref(),
        behavioral.as_ref(),
        hud.as_ref(),
        s7p_ok,
    );
    if write_enveloped_witness(
        "STAGE7_BEHAVIORAL",
        "stage7_behavioral_witness",
        STAGE7_BEHAVIORAL_LIVE_JSON,
        body,
    ) {
        state.written = true;
    }
}
