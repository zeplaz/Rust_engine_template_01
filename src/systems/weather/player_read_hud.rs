//! DESIGN-WX-HUD-IMPL-001 — player-readable weather HUD copy + witness helpers.



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

