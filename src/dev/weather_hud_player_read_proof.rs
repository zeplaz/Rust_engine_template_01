//! DESIGN-WX-HUD-IMPL-001 — `debug_runs/weather_hud_player_read_live.json` lib refresh.

use crate::systems::weather::player_read_hud::{
    weather_hud_player_read_lib_fixture, weather_hud_player_read_witness_payload,
};

pub const WEATHER_HUD_PLAYER_READ_LIVE_JSON: &str = "debug_runs/weather_hud_player_read_live.json";

#[must_use]
pub fn refresh_weather_hud_player_read_live_witness() -> bool {
    let (sample, visibility, renewables, pct) = weather_hud_player_read_lib_fixture();
    let body = weather_hud_player_read_witness_payload(&sample, visibility, &renewables, pct);
    if body.get("green").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "WEATHER_HUD_PLAYER_READ",
        "refresh_weather_hud_player_read_live_witness",
        WEATHER_HUD_PLAYER_READ_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(WEATHER_HUD_PLAYER_READ_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_hud_player_read_live_witness_refresh_green() {
        assert!(refresh_weather_hud_player_read_live_witness());
    }
}
