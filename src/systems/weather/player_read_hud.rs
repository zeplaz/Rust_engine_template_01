//! DESIGN-WX-HUD-IMPL-001 — player-readable weather HUD copy + witness helpers.

use serde_json::{json, Value};

use crate::systems::weather::{
    GlobalRenewableWeatherFactors, WeatherPrecipVisualSample,
};

#[must_use]
pub fn format_ops_wx_line(rain: f32, snow: f32, fog: f32) -> String {
    format!("WX  r {rain:.2}  s {snow:.2}  f {fog:.2}")
}

#[must_use]
pub fn format_ops_wx_line_with_vis(rain: f32, snow: f32, fog: f32, visibility: f32) -> String {
    let base = format_ops_wx_line(rain, snow, fog);
    if visibility < 0.85 {
        format!("{base}  VIS low")
    } else {
        base
    }
}

#[must_use]
pub fn format_ops_power_derate_line(
    pct: f32,
    wind_factor: f32,
    solar_factor: f32,
) -> String {
    if wind_factor < 0.95 {
        format!("PWR  {pct:.0}%  (wind)")
    } else if solar_factor < 0.95 {
        format!("PWR  {pct:.0}%  (solar)")
    } else {
        format!("PWR  {pct:.0}%")
    }
}

#[must_use]
pub fn weather_hud_player_read_witness_payload(
    sample: &WeatherPrecipVisualSample,
    visibility_sample: f32,
    renewables: &GlobalRenewableWeatherFactors,
    power_proxy_pct: f32,
) -> Value {
    let ops_wx_line = format_ops_wx_line_with_vis(
        sample.rain,
        sample.snow,
        sample.fog,
        visibility_sample,
    );
    let ops_power_line = format_ops_power_derate_line(
        power_proxy_pct,
        renewables.wind_capacity_factor,
        renewables.solar_capacity_factor,
    );
    let ops_zones_wired = sample.rain > 0.0 || sample.snow > 0.0 || sample.fog > 0.0;
    let visibility_suffix_wired = visibility_sample < 0.85;
    let precip_overlay_sample_wired = sample.rain > 0.0 || sample.chunk_count > 0;
    let wx_accessible = ops_zones_wired && !ops_wx_line.is_empty();
    let green = ops_zones_wired
        && visibility_suffix_wired
        && precip_overlay_sample_wired
        && wx_accessible
        && renewables.wind_capacity_factor < 0.95;

    json!({
        "gate": "DESIGN-WX-HUD-IMPL-001",
        "green": green,
        "ops_wx_line": ops_wx_line,
        "ops_power_line": ops_power_line,
        "ops_zones_wired": ops_zones_wired,
        "precip_overlay_sample_wired": precip_overlay_sample_wired,
        "visibility_sample": visibility_sample,
        "visibility_suffix_wired": visibility_suffix_wired,
        "wind_capacity_factor": renewables.wind_capacity_factor,
        "solar_capacity_factor": renewables.solar_capacity_factor,
        "wx_accessible": wx_accessible,
    })
}

#[must_use]
pub fn weather_hud_player_read_lib_fixture() -> (
    WeatherPrecipVisualSample,
    f32,
    GlobalRenewableWeatherFactors,
    f32,
) {
    (
        WeatherPrecipVisualSample {
            rain: 0.2,
            snow: 0.1,
            fog: 0.05,
            chunk_count: 1,
            ..Default::default()
        },
        0.72,
        GlobalRenewableWeatherFactors {
            wind_capacity_factor: 0.82,
            solar_capacity_factor: 1.0,
            ..Default::default()
        },
        78.0,
    )
}

#[must_use]
pub fn weather_hud_player_read_witness_green() -> bool {
    let (sample, visibility, renewables, pct) = weather_hud_player_read_lib_fixture();
    weather_hud_player_read_witness_payload(&sample, visibility, &renewables, pct)
        .get("green")
        .and_then(|v| v.as_bool())
        == Some(true)
}
