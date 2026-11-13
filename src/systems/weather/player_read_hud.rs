//! DESIGN-WX-HUD-IMPL-001 / W3 — player-readable weather HUD copy + cursor/camera sample.

use bevy::prelude::*;

use crate::gui::hud::simulation_pointer_gate::{MapWorldPick, SimulationMapPointerGate};
use crate::render::extraction::extracted_camera_metrics::{
    ExtractedCameraMetrics, ExtractedCameraMetricsSet,
};
use crate::systems::atmosphere::pipeline::AtmospherePipelineSet;
use crate::systems::sim_control::SimTimeMicros;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

use super::field::{WeatherField, WeatherSample};

/// Cursor + camera weather samples for the ops strip (W3).
#[derive(Resource, Clone, Debug, Default)]
pub struct WeatherPlayerReadFrame {
    pub cursor: Option<WeatherSample>,
    pub camera: WeatherSample,
    pub display: WeatherSample,
    pub ready: bool,
}

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
pub fn compass8(dir: Vec2) -> &'static str {
    if dir.length_squared() < 1e-6 {
        return "—";
    }
    let a = dir.y.atan2(dir.x); // −π..π, 0 = east
    let deg = a.to_degrees();
    let oct = ((deg + 360.0 + 22.5) % 360.0 / 45.0).floor() as i32;
    match oct {
        0 => "E",
        1 => "NE",
        2 => "N",
        3 => "NW",
        4 => "W",
        5 => "SW",
        6 => "S",
        _ => "SE",
    }
}

/// W3 expanded ops line: `12 m/s NW, gusts 18, rain 4 mm/h, vis 2.1 km`.
#[must_use]
pub fn format_tactical_wx_line(sample: &WeatherSample) -> String {
    let speed = sample.wind_mps.length();
    let dir = compass8(sample.wind_mps);
    let gust = sample.gust_mps.max(speed);
    let precip = if sample.snow {
        format!("snow {:.0} mm/h", sample.rain_mm_h)
    } else if sample.rain_mm_h > 0.05 {
        format!("rain {:.0} mm/h", sample.rain_mm_h)
    } else if sample.fog > 0.25 {
        format!("fog {:.0}%", sample.fog * 100.0)
    } else {
        "clear".to_string()
    };
    let vis_km = sample.visibility_m / 1000.0;
    format!("{speed:.0} m/s {dir}, gusts {gust:.0}, {precip}, vis {vis_km:.1} km")
}

#[must_use]
pub fn format_ops_power_derate_line(pct: f32, wind_factor: f32, solar_factor: f32) -> String {
    if wind_factor < 0.95 {
        format!("PWR  {pct:.0}%  (wind)")
    } else if solar_factor < 0.95 {
        format!("PWR  {pct:.0}%  (solar)")
    } else {
        format!("PWR  {pct:.0}%")
    }
}

pub fn publish_weather_player_read_frame(
    field: Res<WeatherField>,
    sim_time: Option<Res<SimTimeMicros>>,
    cam: Res<ExtractedCameraMetrics>,
    pointer: Option<Res<SimulationMapPointerGate>>,
    mut out: ResMut<WeatherPlayerReadFrame>,
) {
    if !field.is_ready() {
        out.ready = false;
        return;
    }
    let t = sim_time
        .as_deref()
        .copied()
        .unwrap_or(SimTimeMicros(field.prev_micros));
    let cam_m = cam.translation * TILE_METRES_GAMEPLAY;
    let camera = field.sample(cam_m, t);
    let cursor = pointer.as_ref().and_then(|p| {
        if !p.in_play_area {
            return None;
        }
        match p.world_pick {
            MapWorldPick::PixelWorld(Some(xy)) => {
                Some(field.sample(xy * TILE_METRES_GAMEPLAY, t))
            }
            _ => None,
        }
    });
    let display = cursor.unwrap_or(camera);
    *out = WeatherPlayerReadFrame {
        cursor,
        camera,
        display,
        ready: true,
    };
}

pub fn register_weather_player_read(app: &mut App) {
    app.init_resource::<WeatherPlayerReadFrame>().add_systems(
        Update,
        publish_weather_player_read_frame
            .after(AtmospherePipelineSet::VisualExtract)
            .after(ExtractedCameraMetricsSet::Sync),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tactical_line_includes_metrics() {
        let s = WeatherSample {
            wind_mps: Vec2::new(8.0, 8.0),
            gust_mps: 14.0,
            rain_mm_h: 4.0,
            visibility_m: 2100.0,
            ..Default::default()
        };
        let line = format_tactical_wx_line(&s);
        assert!(line.contains("m/s"));
        assert!(line.contains("rain"));
        assert!(line.contains("vis"));
        assert!(line.contains("gusts"));
    }

    #[test]
    fn compass_cardinals() {
        assert_eq!(compass8(Vec2::new(1.0, 0.0)), "E");
        assert_eq!(compass8(Vec2::new(0.0, 1.0)), "N");
        assert_eq!(compass8(Vec2::new(-1.0, 0.0)), "W");
        assert_eq!(compass8(Vec2::new(0.0, -1.0)), "S");
    }
}
