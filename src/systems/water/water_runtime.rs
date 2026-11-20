//! Water look + gameplay wet threshold (DF-2 / PLAN-WATER-DEPTH-TRUTH-v1).
//!
//! Gameplay wet = depth >= [`WET_DEPTH_M`], read through [`super::WaterTileProbe`]. River / lake /
//! ocean are *look labels* derived from the fluid ([`water_look_from_depth_speed`]), never a stored
//! second class map: WM-6 removed the `WaterClassRuntime` hysteresis copy (nothing read it once the
//! probe answered from the field).

use crate::terrain::generation::WaterClass;

pub const WET_DEPTH_M: f32 = 0.3;
/// Channel look when speed exceeds this (m/s approx from qx/qy / depth).
pub const CHANNEL_LOOK_SPEED: f32 = 0.15;

/// Presentation look from depth + speed (+ optional ocean/coast hint). Not a gameplay gate.
#[must_use]
pub fn water_look_from_depth_speed(
    depth_m: f32,
    speed: f32,
    coast_ocean_hint: bool,
) -> WaterClass {
    if depth_m < WET_DEPTH_M {
        WaterClass::Dry
    } else if speed > CHANNEL_LOOK_SPEED {
        WaterClass::River
    } else if coast_ocean_hint {
        WaterClass::Ocean
    } else {
        WaterClass::Lake
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_look_follows_depth_and_speed() {
        assert_eq!(water_look_from_depth_speed(0.1, 3.0, false), WaterClass::Dry);
        assert_eq!(water_look_from_depth_speed(2.0, 1.0, false), WaterClass::River);
        assert_eq!(water_look_from_depth_speed(2.0, 0.0, false), WaterClass::Lake);
        assert_eq!(water_look_from_depth_speed(2.0, 0.0, true), WaterClass::Ocean);
    }
}
