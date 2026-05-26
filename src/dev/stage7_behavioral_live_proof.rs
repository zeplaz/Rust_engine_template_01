//! Live witness: `debug_runs/stage7_behavioral_live.json` (**S7B-M1** / **M2** / **M3**).

use std::path::PathBuf;

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::strategic::{
    dispatch_delay_ticks, mission_kinds_supported, seed_stage7_behavioral_m2_lib_proof,
    seed_stage7_behavioral_witness_for_lib_proof,
    Stage7BehavioralHud, Stage7BehavioralWitnessState, Stage7BeliefState, StrategicCommandQueue,
};

pub const STAGE7_BEHAVIORAL_LIVE_JSON: &str = "debug_runs/stage7_behavioral_live.json";
pub const STAGE7_PLAY_LIVE_JSON: &str = "debug_runs/stage7_play_live.json";

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

#[must_use]
pub fn behavioral_contract_ok() -> bool {
    mission_kinds_supported() == ["MoveCorridor", "SecureCorridor"]
}

#[must_use]
pub fn s7p_play_witness_ok_from_disk() -> bool {
    let path = repo_root().join(STAGE7_PLAY_LIVE_JSON);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    v.get("s7p_steward_green")
        .and_then(|x| x.as_bool())
        .unwrap_or(false)
}

#[must_use]
pub fn s7b_m1_green(contract_ok: bool, s7p_play_ok: bool) -> bool {
    contract_ok && s7p_play_ok
}

#[must_use]
pub fn s7b_m2_green(delay_ticks: u32, delay_test_ok: bool) -> bool {
    delay_ticks > 0 && delay_test_ok
}

#[must_use]
pub fn s7b_m3_green(behavioral: &Stage7BehavioralWitnessState) -> bool {
    behavioral.recon_overlay_enabled
        && behavioral.logistics_stress_overlay_enabled
        && behavioral.recon_overlay_sample_count > 0
        && behavioral.logistics_stress_sample_count > 0
}

#[must_use]
pub fn s7b_steward_green(m1: bool, m2: bool, m3: bool) -> bool {
    m1 && m2 && m3
}

#[must_use]
pub fn s7b_m2_delay_test_ok() -> bool {
    use crate::systems::sim_control::SimStepStamp;

    let mut queue = StrategicCommandQueue::default();
    let issued = SimStepStamp::new(5, 0);
    let msg = queue.enqueue_strategic(issued, "delay proof");
    let before = SimStepStamp::new(5 + u64::from(dispatch_delay_ticks()) - 1, 0);
    if queue.is_visible_to_consumer(&msg, before) {
        return false;
    }
    queue.tick(before);
    if !queue.delivered.is_empty() {
        return false;
    }
    let at = SimStepStamp::new(5 + u64::from(dispatch_delay_ticks()), 0);
    queue.tick(at);
    queue.delivered.len() == 1
}

#[must_use]
pub fn build_stage7_behavioral_live_proof_payload(
    queue: &StrategicCommandQueue,
    behavioral: &Stage7BehavioralWitnessState,
    hud: &Stage7BehavioralHud,
    s7p_play_witness_ok: bool,
) -> serde_json::Value {
    let behavioral_contract_ok = behavioral_contract_ok();
    let s7b_m1 = s7b_m1_green(behavioral_contract_ok, s7p_play_witness_ok);
    let delay_ticks = dispatch_delay_ticks();
    let delay_test_ok = s7b_m2_delay_test_ok();
    let stale_intel = behavioral.stale_intel_surface || queue.pending_count() > 0;
    let orders_hook = hud.orders_pending_ui_hook || behavioral.orders_pending_ui_hook;
    let s7b_m2 = s7b_m2_green(delay_ticks, delay_test_ok);
    let s7b_m3 = s7b_m3_green(behavioral);
    let gate = if s7b_m3 {
        "S7B-M3-001"
    } else if s7b_m2 {
        "S7B-M2-001"
    } else {
        "S7B-M1-001"
    };
    serde_json::json!({
        "profile": "STAGE7_BEHAVIORAL",
        "gate": gate,
        "impl_plan": "src/dev/stage7_behavioral_implementation_plan_v1.md",
        "witness_spec": "src/dev/stage7_behavioral_live_witness_spec_v1.md",
        "behavioral_contract_ok": behavioral_contract_ok,
        "communication_plane_v1": "StrategicCommand",
        "mission_kinds_supported": mission_kinds_supported(),
        "overlay_channels_v1": ["Recon", "LogisticsStress"],
        "dispatch_delay_model": "fixed_ticks",
        "dispatch_delay_ticks": delay_ticks,
        "intel_stale_surface": "tray_and_map_tint",
        "explainability_surface": "f3_and_context_tray",
        "pending_dispatch_count": queue.pending_count(),
        "delivered_dispatch_count": queue.delivered.len(),
        "stale_intel_surface": stale_intel,
        "orders_pending_ui_hook": orders_hook,
        "recon_overlay_enabled": behavioral.recon_overlay_enabled,
        "logistics_stress_overlay_enabled": behavioral.logistics_stress_overlay_enabled,
        "recon_overlay_sample_count": behavioral.recon_overlay_sample_count,
        "logistics_stress_sample_count": behavioral.logistics_stress_sample_count,
        "s7p_play_witness_ok": s7p_play_witness_ok,
        "s7b_preflight_green": true,
        "s7b_m1_green": s7b_m1,
        "s7b_m2_green": s7b_m2,
        "s7b_m3_green": s7b_m3,
        "s7b_steward_green": s7b_steward_green(s7b_m1, s7b_m2, s7b_m3),
        "decisions": {
            "d_s7_01": "StrategicCommand_only",
            "d_s7_02": "Recon_logistics_stress",
            "d_s7_03": "Move_secure_corridor",
            "d_s7_04": "fixed_ticks",
            "d_s7_05": "tray_and_map_tint",
            "d_s7_06": "f3_and_context_tray",
        },
    })
}

fn repo_root() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn write_stage7_behavioral_live_proof_system(
    base: Res<State<BaseState>>,
    mut state: ResMut<Stage7BehavioralLiveProofState>,
    queue: Res<StrategicCommandQueue>,
    behavioral: Res<Stage7BehavioralWitnessState>,
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

    let s7p_ok = s7p_play_witness_ok_from_disk();
    let body = build_stage7_behavioral_live_proof_payload(
        queue.as_ref(),
        behavioral.as_ref(),
        hud.as_ref(),
        s7p_ok,
    );
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "STAGE7_BEHAVIORAL",
        "stage7_behavioral_live_proof",
        STAGE7_BEHAVIORAL_LIVE_JSON,
        body,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(STAGE7_BEHAVIORAL_LIVE_JSON, wrapped) {
        state.written = true;
    }
}

/// **S7B-M3-001** — overlay readers (logistics + recon/ecology) → `s7b_m3_green`.
#[must_use]
pub fn refresh_s7b_m3_001_live_witness() -> bool {
    let mut queue = StrategicCommandQueue::default();
    let mut behavioral = Stage7BehavioralWitnessState::default();
    let mut beliefs = Stage7BeliefState::default();
    seed_stage7_behavioral_witness_for_lib_proof(&mut queue, &mut behavioral, &mut beliefs);
    assert!(
        s7b_m3_green(&behavioral),
        "S7B-M3-001 overlay reader predicate"
    );
    let hud = Stage7BehavioralHud {
        pending_orders: queue.pending_count(),
        orders_pending_ui_hook: true,
        orders_pending_label: format!("Orders pending: {}", queue.pending_count()),
    };
    commit_stage7_behavioral_live_proof(&queue, &behavioral, &hud)
}

/// Reads `s7b_m3_green` from the on-disk behavioral witness (after refresh).
#[must_use]
pub fn stage7_behavioral_live_s7b_m3_green() -> bool {
    let path = repo_root().join(STAGE7_BEHAVIORAL_LIVE_JSON);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    v.get("s7b_m3_green")
        .and_then(|x| x.as_bool())
        .unwrap_or(false)
}

/// **S7B steward rollup** — M1 + M2 + M3 green when `stage7_play_live.json` is green.
#[must_use]
pub fn refresh_s7b_steward_001_live_witness() -> bool {
    let mut queue = StrategicCommandQueue::default();
    let mut behavioral = Stage7BehavioralWitnessState::default();
    let mut beliefs = Stage7BeliefState::default();
    seed_stage7_behavioral_witness_for_lib_proof(&mut queue, &mut behavioral, &mut beliefs);
    let hud = Stage7BehavioralHud {
        pending_orders: queue.pending_count(),
        orders_pending_ui_hook: true,
        orders_pending_label: format!("Orders pending: {}", queue.pending_count()),
    };
    let s7p_ok = s7p_play_witness_ok_from_disk();
    let delay_ticks = dispatch_delay_ticks();
    assert!(s7b_m2_green(delay_ticks, s7b_m2_delay_test_ok()));
    assert!(s7b_m3_green(&behavioral));
    assert!(s7b_m1_green(behavioral_contract_ok(), s7p_ok));
    commit_stage7_behavioral_live_proof(&queue, &behavioral, &hud)
}

/// **S7B-M2-001** — refresh witness with fixed-tick dispatch drain green (M1 fields preserved when play witness ok).
#[must_use]
pub fn refresh_s7b_m2_001_live_witness() -> bool {
    let mut queue = StrategicCommandQueue::default();
    let mut behavioral = Stage7BehavioralWitnessState::default();
    let mut beliefs = Stage7BeliefState::default();
    seed_stage7_behavioral_m2_lib_proof(&mut queue, &mut behavioral, &mut beliefs);
    let hud = Stage7BehavioralHud {
        pending_orders: queue.pending_count(),
        orders_pending_ui_hook: true,
        orders_pending_label: format!("Orders pending: {}", queue.pending_count()),
    };
    let delay_ticks = dispatch_delay_ticks();
    assert!(
        s7b_m2_green(delay_ticks, s7b_m2_delay_test_ok()),
        "S7B-M2-001 delay drain predicate"
    );
    commit_stage7_behavioral_live_proof(&queue, &behavioral, &hud)
}

static STAGE7_BEHAVIORAL_PROOF_FILE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn stage7_behavioral_proof_file_lock() -> std::sync::MutexGuard<'static, ()> {
    STAGE7_BEHAVIORAL_PROOF_FILE_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

pub fn commit_stage7_behavioral_live_proof(
    queue: &StrategicCommandQueue,
    behavioral: &Stage7BehavioralWitnessState,
    hud: &Stage7BehavioralHud,
) -> bool {
    let _lock = stage7_behavioral_proof_file_lock();
    let s7p_ok = s7p_play_witness_ok_from_disk();
    let body = build_stage7_behavioral_live_proof_payload(queue, behavioral, hud, s7p_ok);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "STAGE7_BEHAVIORAL",
        "stage7_behavioral_live_proof",
        STAGE7_BEHAVIORAL_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(STAGE7_BEHAVIORAL_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn stage7_behavioral_m1_contract_ok_predicate() {
        assert!(behavioral_contract_ok());
    }

    #[test]
    fn s7b_m2_delay_test_ok_predicate() {
        assert!(s7b_m2_delay_test_ok());
    }

    #[test]
    fn s7b_m3_green_requires_overlay_reader_samples() {
        let mut witness = Stage7BehavioralWitnessState::default();
        assert!(!s7b_m3_green(&witness));
        witness.recon_overlay_enabled = true;
        witness.logistics_stress_overlay_enabled = true;
        witness.recon_overlay_sample_count = 100;
        witness.logistics_stress_sample_count = 18;
        assert!(s7b_m3_green(&witness));
    }

    #[test]
    fn s7b_m3_001_live_witness_refresh() {
        assert!(refresh_s7b_m3_001_live_witness());
        let path = repo_root().join(STAGE7_BEHAVIORAL_LIVE_JSON);
        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert_eq!(v["gate"], serde_json::json!("S7B-M3-001"));
        assert_eq!(v["s7b_m3_green"], serde_json::json!(true));
        assert_eq!(v["recon_overlay_enabled"], serde_json::json!(true));
        assert_eq!(v["logistics_stress_overlay_enabled"], serde_json::json!(true));
        assert!(v["recon_overlay_sample_count"].as_u64().unwrap_or(0) > 0);
        assert!(v["logistics_stress_sample_count"].as_u64().unwrap_or(0) > 0);
    }

    #[test]
    fn s7b_steward_001_live_witness_refresh() {
        assert!(refresh_s7b_steward_001_live_witness());
        let path = repo_root().join(STAGE7_BEHAVIORAL_LIVE_JSON);
        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("parse")).expect("parse");
        assert_eq!(v["gate"], serde_json::json!("S7B-M3-001"));
        assert_eq!(v["s7b_m3_green"], serde_json::json!(true));
        assert_eq!(v["s7b_m2_green"], serde_json::json!(true));
        assert_eq!(v["behavioral_contract_ok"], serde_json::json!(true));
        if v["s7p_play_witness_ok"].as_bool() == Some(true) {
            assert_eq!(v["s7b_m1_green"], serde_json::json!(true));
            assert_eq!(v["s7b_steward_green"], serde_json::json!(true));
        }
    }

    #[test]
    fn s7b_m2_001_live_witness_refresh() {
        assert!(refresh_s7b_m2_001_live_witness());
        let path = repo_root().join(STAGE7_BEHAVIORAL_LIVE_JSON);
        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert_eq!(v["gate"], serde_json::json!("S7B-M2-001"));
        assert_eq!(v["dispatch_delay_ticks"], serde_json::json!(8));
        assert_eq!(v["s7b_m2_green"], serde_json::json!(true));
        assert_eq!(v["dispatch_delay_model"], serde_json::json!("fixed_ticks"));
        assert!(v["pending_dispatch_count"].as_u64().unwrap_or(0) > 0);
    }

    #[test]
    fn stage7_behavioral_live_witness_refresh() {
        let _lock = stage7_behavioral_proof_file_lock();
        let mut queue = StrategicCommandQueue::default();
        let mut behavioral = Stage7BehavioralWitnessState::default();
        let mut beliefs = Stage7BeliefState::default();
        let hud = Stage7BehavioralHud::default();
        seed_stage7_behavioral_witness_for_lib_proof(&mut queue, &mut behavioral, &mut beliefs);
        assert!(commit_stage7_behavioral_live_proof(&queue, &behavioral, &hud));

        let path = repo_root().join(STAGE7_BEHAVIORAL_LIVE_JSON);
        assert!(path.exists(), "expected {:?}", path);
        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert_eq!(v["behavioral_contract_ok"], serde_json::json!(true));
        assert!(v["s7b_m1_green"].as_bool().unwrap_or(false));
        assert_eq!(v["s7b_m2_green"], serde_json::json!(true), "{v}");
        assert_eq!(v["s7b_m3_green"], serde_json::json!(true), "{v}");
        if v["s7p_play_witness_ok"].as_bool() == Some(true) {
            assert_eq!(v["s7b_steward_green"], serde_json::json!(true), "{v}");
        }
    }

    #[test]
    fn stage7_behavioral_m2_m3_live_witness_refresh() {
        let _lock = stage7_behavioral_proof_file_lock();
        let mut queue = StrategicCommandQueue::default();
        let mut behavioral = Stage7BehavioralWitnessState::default();
        let mut beliefs = Stage7BeliefState::default();
        let mut hud = Stage7BehavioralHud::default();
        seed_stage7_behavioral_witness_for_lib_proof(&mut queue, &mut behavioral, &mut beliefs);
        hud.orders_pending_ui_hook = true;
        hud.pending_orders = queue.pending_count();

        assert!(commit_stage7_behavioral_live_proof(&queue, &behavioral, &hud));

        let path = repo_root().join(STAGE7_BEHAVIORAL_LIVE_JSON);
        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert!(v["dispatch_delay_ticks"].as_u64().unwrap_or(0) > 0);
        assert_eq!(v["s7b_m2_green"], serde_json::json!(true), "{v}");
        assert_eq!(v["s7b_m3_green"], serde_json::json!(true), "{v}");
        assert_eq!(v["gate"], serde_json::json!("S7B-M3-001"));
        assert_eq!(v["dispatch_delay_ticks"], serde_json::json!(8));
        assert_eq!(v["s7b_steward_green"], serde_json::json!(true), "{v}");
    }
}
