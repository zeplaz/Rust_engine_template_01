//! CDR-B-IND-PLAY-WITNESS-001 — industrial activation play witness stable on default scenario.

pub const IND_PLAY_WITNESS_LIVE_JSON: &str = "debug_runs/ind_play_witness_live.json";

#[must_use]
pub fn ind_play_witness_green() -> bool {
    let raw = std::fs::read_to_string("debug_runs/play_scenario_live.json").ok();
    let Some(raw) = raw else {
        return false;
    };
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    doc.pointer("/play_truth_001/ind_e02_in_play_green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
        && doc
            .pointer("/play_truth_001/green")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
}

#[must_use]
pub fn refresh_ind_play_witness_live() -> bool {
    let green = ind_play_witness_green();
    let body = serde_json::json!({
        "gate": "CDR-B-IND-PLAY-WITNESS-001",
        "slice_id": "CDR-B-IND-PLAY-WITNESS-001",
        "green": green,
        "source_witness": "debug_runs/play_scenario_live.json",
        "ind_e02_in_play_green": green,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "CDR-B-IND-PLAY-WITNESS-001",
        "refresh_ind_play_witness_live",
        IND_PLAY_WITNESS_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(IND_PLAY_WITNESS_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ind_play_witness_live_green() {
        let _ = crate::engine::play_scenario::refresh_play_scenario_001_live_witness();
        assert!(refresh_ind_play_witness_live());
    }
}
