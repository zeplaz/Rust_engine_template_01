//! TRIAGE-MAP-ZOOM-SMOOTH-001 — `debug_runs/map_zoom_coherence_live.json`.

pub const MAP_ZOOM_COHERENCE_LIVE_JSON: &str = "debug_runs/map_zoom_coherence_live.json";

#[must_use]
pub fn refresh_map_zoom_coherence_live_witness() -> bool {
    if !crate::gui::map_zoom_coherence_001_witness_green() {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "MAP-ZOOM-COHERENCE-001",
        "refresh_map_zoom_coherence_live_witness",
        MAP_ZOOM_COHERENCE_LIVE_JSON,
        crate::gui::map_zoom_coherence_001_witness_json(),
    );
    crate::dev::debug_run_envelope::write_debug_run_json(MAP_ZOOM_COHERENCE_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_zoom_coherence_live_witness_refresh_green() {
        assert!(refresh_map_zoom_coherence_live_witness());
    }
}
