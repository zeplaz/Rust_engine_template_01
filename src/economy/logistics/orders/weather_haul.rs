//! W4 — truck haul speed multiplier from local weather sample.

use bevy::prelude::*;

use crate::systems::weather::{WeatherPollingStations, WeatherSample};

use super::plugin::LogisticsOrderSimRes;

/// Dump / matrix evidence for haul weather derate.
#[derive(Resource, Debug, Clone, Default)]
pub struct HaulWeatherDump {
    pub samples: u64,
    pub last_mul: f32,
    pub min_mul: f32,
}

impl HaulWeatherDump {
    pub fn record(&mut self, mul: f32) {
        self.samples = self.samples.wrapping_add(1);
        self.last_mul = mul;
        if self.min_mul <= 0.0 {
            self.min_mul = mul;
        } else {
            self.min_mul = self.min_mul.min(mul);
        }
    }
}

/// Rain / snow / mud slow haul speed into `(0.25..=1.0)`.
#[must_use]
pub fn weather_haul_speed_mul(sample: &WeatherSample) -> f32 {
    let rain = (sample.rain_mm_h / 10.0).clamp(0.0, 1.0);
    let snow = if sample.snow {
        (sample.rain_mm_h / 8.0).clamp(0.0, 1.0)
    } else {
        0.0
    };
    // Humidity as a cheap mud proxy when soil isn't in the sample.
    let mud = ((sample.humidity - 0.55).max(0.0) / 0.45).clamp(0.0, 1.0);
    let slow = 0.35 * rain + 0.45 * snow + 0.30 * mud;
    (1.0 - slow).clamp(0.25, 1.0)
}

/// Sole writer of [`LogisticsOrderSim::weather_haul_mul`] from the field centre sample.
pub fn sync_haul_weather_mul_system(
    stations: Res<WeatherPollingStations>,
    mut res: ResMut<LogisticsOrderSimRes>,
    mut dump: ResMut<HaulWeatherDump>,
) {
    let mul = if stations.ready {
        weather_haul_speed_mul(&stations.centre)
    } else {
        1.0
    };
    res.sim.weather_haul_mul = mul;
    dump.record(mul);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec2;

    #[test]
    fn dry_is_full_speed() {
        let s = WeatherSample::default();
        assert!((weather_haul_speed_mul(&s) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn heavy_rain_slows() {
        let s = WeatherSample {
            rain_mm_h: 10.0,
            humidity: 0.95,
            ..Default::default()
        };
        assert!(weather_haul_speed_mul(&s) < 0.7);
    }

    #[test]
    fn snow_slows_more_than_light_rain() {
        let rain = WeatherSample {
            rain_mm_h: 4.0,
            snow: false,
            ..Default::default()
        };
        let snow = WeatherSample {
            rain_mm_h: 4.0,
            snow: true,
            wind_mps: Vec2::ZERO,
            ..Default::default()
        };
        assert!(weather_haul_speed_mul(&snow) < weather_haul_speed_mul(&rain));
    }
}
