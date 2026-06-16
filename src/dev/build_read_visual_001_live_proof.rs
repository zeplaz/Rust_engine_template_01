//! **BUILD-READ-VISUAL-001** — refresh `debug_runs/build_read_visual_001_live.json`.

pub const BUILD_READ_VISUAL_001_LIVE_JSON: &str = "debug_runs/build_read_visual_001_live.json";

#[must_use]
pub fn refresh_build_read_visual_001_live_witness() -> bool {
    let body = crate::construction::build_read_visual_001_witness_body();
    if !body.get("green").and_then(|v| v.as_bool()).unwrap_or(false) {
        return false;
    }
    if !body
        .get("runtime_sim_verified")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "BUILD-READ-VISUAL-001",
        "refresh_build_read_visual_001_live_witness",
        BUILD_READ_VISUAL_001_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(BUILD_READ_VISUAL_001_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_read_visual_001_live_witness_refresh_green() {
        assert!(refresh_build_read_visual_001_live_witness());
        let raw = std::fs::read_to_string(BUILD_READ_VISUAL_001_LIVE_JSON).expect("read");
        let doc: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        assert_eq!(doc.get("runtime_sim_verified").and_then(|v| v.as_bool()), Some(true));
    }
}
