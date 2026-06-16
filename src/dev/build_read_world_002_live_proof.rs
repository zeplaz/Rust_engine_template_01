//! **BUILD-READ-WORLD-002** — refresh `debug_runs/build_read_world_002_live.json`.

pub const BUILD_READ_WORLD_002_LIVE_JSON: &str = "debug_runs/build_read_world_002_live.json";

#[must_use]
pub fn refresh_build_read_world_002_live_witness() -> bool {
    let body = crate::construction::build_read_world_002_witness_body();
    if !body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "BUILD-READ-WORLD-002",
        "refresh_build_read_world_002_live_witness",
        BUILD_READ_WORLD_002_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(BUILD_READ_WORLD_002_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_read_world_002_live_witness_refresh_green() {
        assert!(refresh_build_read_world_002_live_witness());
    }
}
