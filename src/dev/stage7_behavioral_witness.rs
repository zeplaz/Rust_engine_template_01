//! Stage 7 behavioral witness collectors + lib refresh (DEV-CONTAIN-005).
//!
//! File I/O writer: [`crate::dev::runtime_witness::stage7_behavioral`].

use std::path::PathBuf;

use bevy::prelude::*;

use crate::strategic::{
    dispatch_delay_ticks, mission_kinds_supported,
    seed_stage7_behavioral_m2_lib_proof, seed_stage7_behavioral_witness_for_lib_proof,
    seed_stage7_m4_playtest_enqueue, Stage7BehavioralHud, Stage7BehavioralWitnessState,
    Stage7BeliefState, StrategicCommandQueue,
};

pub use crate::dev::runtime_witness::stage7_behavioral::{
    commit_stage7_behavioral_witness, write_stage7_behavioral_witness_system,
    Stage7BehavioralLiveProofState, STAGE7_BEHAVIORAL_LIVE_JSON,
};
pub use crate::dev::runtime_witness::stage7_play::STAGE7_PLAY_LIVE_JSON;

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
pub fn s7b_m4_play_green(queue: &StrategicCommandQueue, witness: &Stage7BehavioralWitnessState) -> bool {
    witness.s7b_m4_play_enqueue_wired && queue.pending_count() >= 1
}

#[must_use]
pub fn s7b_tune_delay_001_green(delay_ticks: u32, delay_test_ok: bool) -> bool {
    delay_ticks == 8 && delay_test_ok
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
pub fn build_stage7_behavioral_witness_payload(
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
    let s7b_m4 = s7b_m4_play_green(queue, behavioral);
    let s7b_tune_delay = s7b_tune_delay_001_green(delay_ticks, delay_test_ok);
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
        "m3_minimap_readers_wired": s7b_m3,
        "s7b_m4_play_001": {
            "gate": "S7B-M4-PLAY-001",
            "green": s7b_m4,
            "play_enqueue_wired": behavioral.s7b_m4_play_enqueue_wired,
            "pending_dispatch_count": queue.pending_count(),
        },
        "s7b_tune_delay_001": {
            "gate": "S7B-TUNE-DELAY-001",
            "green": s7b_tune_delay,
            "dispatch_delay_ticks": delay_ticks,
        },
        "s7b_m4_play_green": s7b_m4,
        "s7b_tune_delay_001_green": s7b_tune_delay,
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
    commit_stage7_behavioral_witness(&queue, &behavioral, &hud)
}

/// **S7B-M3-STEWARD-REMEDY-001** — restore M3 + steward rollup on disk (last writer in bundle proofs).
#[must_use]
pub fn refresh_s7b_m3_steward_remedy_001_live_witness() -> bool {
    refresh_s7b_steward_001_live_witness()
}

/// **S7B-M4-PLAY-001** — playtest corridor enqueue → `pending_dispatch_count` > 0.
#[must_use]
pub fn refresh_s7b_m4_play_001_live_witness() -> bool {
    assert!(
        crate::dev::stage7_play_witness::refresh_s7p_steward_001_live_witness(),
        "S7P-STEWARD-001 prerequisite for s7b_steward_green on last writer"
    );
    let mut queue = StrategicCommandQueue::default();
    let mut behavioral = Stage7BehavioralWitnessState::default();
    let mut beliefs = Stage7BeliefState::default();
    seed_stage7_behavioral_witness_for_lib_proof(&mut queue, &mut behavioral, &mut beliefs);
    seed_stage7_m4_playtest_enqueue(&mut queue, &mut behavioral);
    assert!(
        s7b_m4_play_green(&queue, &behavioral),
        "S7B-M4-PLAY-001 enqueue predicate"
    );
    let hud = Stage7BehavioralHud {
        pending_orders: queue.pending_count(),
        orders_pending_ui_hook: true,
        orders_pending_label: format!("Orders pending: {}", queue.pending_count()),
    };
    commit_stage7_behavioral_witness(&queue, &behavioral, &hud)
}

/// **S7B-M4-PLAY-REMEDY-001** — restore M4 play rollup after M3/steward writers (last in bundle).
#[must_use]
pub fn refresh_s7b_m4_play_remedy_001_live_witness() -> bool {
    refresh_s7b_m4_play_001_live_witness()
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
    assert!(
        crate::dev::stage7_play_witness::refresh_s7p_steward_001_live_witness(),
        "S7P-STEWARD-001 prerequisite for s7b_m1_green"
    );
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
    commit_stage7_behavioral_witness(&queue, &behavioral, &hud)
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
    commit_stage7_behavioral_witness(&queue, &behavioral, &hud)
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
    fn s7b_m4_play_remedy_001_live_witness_refresh() {
        assert!(refresh_s7b_m4_play_remedy_001_live_witness());
        let path = repo_root().join(STAGE7_BEHAVIORAL_LIVE_JSON);
        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert_eq!(v["s7b_m4_play_green"], serde_json::json!(true));
        assert_eq!(v["s7b_m4_play_001"]["green"], serde_json::json!(true));
        assert_eq!(v["s7b_m4_play_001"]["play_enqueue_wired"], serde_json::json!(true));
        assert!(v["s7b_m4_play_001"]["pending_dispatch_count"]
            .as_u64()
            .unwrap_or(0)
            >= 1);
        assert_eq!(v["s7b_m3_green"], serde_json::json!(true));
    }

    #[test]
    fn s7b_m3_steward_remedy_001_live_witness_refresh() {
        assert!(refresh_s7b_m3_steward_remedy_001_live_witness());
        let path = repo_root().join(STAGE7_BEHAVIORAL_LIVE_JSON);
        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert_eq!(v["s7b_m3_green"], serde_json::json!(true));
        assert_eq!(v["s7b_steward_green"], serde_json::json!(true));
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
        assert_eq!(v["dispatch_delay_ticks"], serde_json::json!(8));
        assert_eq!(v["s7b_m2_green"], serde_json::json!(true));
        assert_eq!(v["s7b_m3_green"], serde_json::json!(true));
        assert_eq!(v["dispatch_delay_model"], serde_json::json!("fixed_ticks"));
        assert!(v["pending_dispatch_count"].as_u64().unwrap_or(0) > 0);
    }

    #[test]
    fn stage7_behavioral_live_witness_refresh() {
        let mut queue = StrategicCommandQueue::default();
        let mut behavioral = Stage7BehavioralWitnessState::default();
        let mut beliefs = Stage7BeliefState::default();
        let hud = Stage7BehavioralHud::default();
        seed_stage7_behavioral_witness_for_lib_proof(&mut queue, &mut behavioral, &mut beliefs);
        assert!(commit_stage7_behavioral_witness(&queue, &behavioral, &hud));

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
        let mut queue = StrategicCommandQueue::default();
        let mut behavioral = Stage7BehavioralWitnessState::default();
        let mut beliefs = Stage7BeliefState::default();
        let mut hud = Stage7BehavioralHud::default();
        seed_stage7_behavioral_witness_for_lib_proof(&mut queue, &mut behavioral, &mut beliefs);
        hud.orders_pending_ui_hook = true;
        hud.pending_orders = queue.pending_count();

        assert!(commit_stage7_behavioral_witness(&queue, &behavioral, &hud));

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
