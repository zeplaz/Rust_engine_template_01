//! Mission / dev overlay mapping (`base_fire2_smoke.md` §10–11).

use super::field::AtmosphereCell;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OverlayMode {
    Terrain,
    FireHeat,
    SmokeDensity,
    FogDensity,
    Toxicity,
    Visibility,
    EmberPressure,
    LogisticsCost,
    AiVisibility,
}

/// RGBA for tile debug / minimap overlays.
pub fn atmosphere_overlay_rgba(cell: &AtmosphereCell, mode: OverlayMode) -> [u8; 4] {
    match mode {
        OverlayMode::Terrain => [40, 60, 40, 255],
        OverlayMode::FireHeat => heat_color(cell.heat_distortion),
        OverlayMode::SmokeDensity => gray(cell.smoke_density),
        OverlayMode::FogDensity => gray(cell.fog_density),
        OverlayMode::Toxicity => {
            let g = (cell.toxicity * 255.0) as u8;
            [0, g, 0, 255]
        }
        OverlayMode::Visibility => {
            let r = ((1.0 - cell.visibility) * 255.0) as u8;
            [r, 0, 0, 255]
        }
        OverlayMode::EmberPressure => heat_color(cell.ember_density),
        OverlayMode::LogisticsCost => [80, 80, 200, 255],
        OverlayMode::AiVisibility => gray(cell.visibility),
    }
}

fn gray(v: f32) -> [u8; 4] {
    let u = (v.clamp(0.0, 1.0) * 255.0) as u8;
    [u, u, u, 255]
}

fn heat_color(h: f32) -> [u8; 4] {
    let h = h.clamp(0.0, 1.0);
    let r = (h * 255.0) as u8;
    let g = ((1.0 - h) * 120.0) as u8;
    [r, g, 20, 255]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toxicity_overlay_green_channel() {
        let c = AtmosphereCell {
            toxicity: 0.5,
            ..Default::default()
        };
        let px = atmosphere_overlay_rgba(&c, OverlayMode::Toxicity);
        assert!(px[1] > 120);
    }
}
