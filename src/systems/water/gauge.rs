//! One GaugeBook kernel — typed sites, no per-gauge clone systems (anti-clone).
//! Used by any hydro scene / player river gauges — not DamFlood-private.

use bevy::math::IVec2;
use bevy::prelude::*;

use super::SurfaceWaterField;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GaugeId {
    G1,
    G2,
    G3,
    Custom(u8),
}

impl GaugeId {
    #[must_use]
    pub fn from_str_id(s: &str) -> Self {
        match s {
            "G1" | "g1" => Self::G1,
            "G2" | "g2" => Self::G2,
            "G3" | "g3" => Self::G3,
            _ => Self::Custom(0),
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::G1 => "G1",
            Self::G2 => "G2",
            Self::G3 => "G3",
            Self::Custom(_) => "GX",
        }
    }
}

#[derive(Clone, Debug)]
pub struct GaugeSite {
    pub id: GaugeId,
    pub label: String,
    pub tile: IVec2,
    pub cross_section_half_tiles: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GaugeReading {
    /// Conveyance estimate of the cross-section (flux, or depth x width x 0.5 m/s when quiet).
    pub discharge_m3s: f32,
    /// Water the solver's face fluxes really carry across the cross-section.
    pub flux_m3s: f32,
    pub stage_m: f32,
    pub velocity_mps: f32,
}

/// Sole gauge authority for river / reservoir monitoring.
#[derive(Resource, Debug, Clone, Default)]
pub struct GaugeBook {
    pub sites: Vec<GaugeSite>,
    pub readings: Vec<GaugeReading>,
}

impl GaugeBook {
    pub fn clear(&mut self) {
        self.sites.clear();
        self.readings.clear();
    }

    pub fn seed_sites(&mut self, sites: impl IntoIterator<Item = GaugeSite>) {
        self.clear();
        for site in sites {
            self.sites.push(site);
            self.readings.push(GaugeReading::default());
        }
    }

    /// Placeholder until the surface-water field is wired (tests / pre-solve).
    pub fn publish_placeholder(&mut self, baseflow_m3s: f32) {
        for (i, site) in self.sites.iter().enumerate() {
            let Some(r) = self.readings.get_mut(i) else {
                continue;
            };
            let scale = match site.id {
                GaugeId::G1 | GaugeId::G2 => 1.0,
                GaugeId::G3 => 0.95,
                GaugeId::Custom(_) => 0.0,
            };
            r.discharge_m3s = baseflow_m3s * scale;
            r.stage_m = if scale > 0.0 { 2.0 * scale } else { 0.0 };
            r.velocity_mps = scale;
        }
    }

    /// Sample live discharge from [`SurfaceWaterField`] (DF-2 gauge Accept).
    pub fn publish_from_field(&mut self, field: &SurfaceWaterField) {
        for (i, site) in self.sites.iter().enumerate() {
            let Some(r) = self.readings.get_mut(i) else {
                continue;
            };
            let (q, stage, vel) =
                field.gauge_discharge(site.tile.x, site.tile.y, site.cross_section_half_tiles);
            r.discharge_m3s = q;
            r.flux_m3s = field.gauge_flux_m3s(site.tile.x, site.tile.y, site.cross_section_half_tiles);
            r.stage_m = stage;
            r.velocity_mps = vel;
        }
    }

    #[must_use]
    pub fn reading(&self, id: GaugeId) -> Option<GaugeReading> {
        self.sites
            .iter()
            .position(|s| s.id == id)
            .and_then(|i| self.readings.get(i).copied())
    }
}

pub fn publish_gauge_readings_system(
    book: Option<ResMut<GaugeBook>>,
    field: Option<Res<SurfaceWaterField>>,
) {
    let Some(mut book) = book else {
        return;
    };
    if book.sites.is_empty() {
        return;
    }
    if let Some(field) = field.as_ref().filter(|f| f.wired) {
        book.publish_from_field(field);
        return;
    }
    let base = field
        .as_ref()
        .map(|f| f.inflow_m3s)
        .filter(|q| *q > 0.0)
        .unwrap_or(0.0);
    if base > 0.0 {
        book.publish_placeholder(base);
    }
}

pub struct GaugeBookPlugin;

impl Plugin for GaugeBookPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GaugeBook>().add_systems(
            Update,
            publish_gauge_readings_system.run_if(|book: Option<Res<GaugeBook>>| {
                book.is_some_and(|b| !b.sites.is_empty())
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauge_kernel_seeds_three_sites() {
        let mut book = GaugeBook::default();
        book.seed_sites([
            GaugeSite {
                id: GaugeId::G1,
                label: "up".into(),
                tile: IVec2::new(10, 20),
                cross_section_half_tiles: 4,
            },
            GaugeSite {
                id: GaugeId::G2,
                label: "mid".into(),
                tile: IVec2::new(10, 10),
                cross_section_half_tiles: 4,
            },
            GaugeSite {
                id: GaugeId::G3,
                label: "down".into(),
                tile: IVec2::new(10, 0),
                cross_section_half_tiles: 4,
            },
        ]);
        assert_eq!(book.sites.len(), 3);
        book.publish_placeholder(150.0);
        let g1 = book.reading(GaugeId::G1).expect("G1");
        assert!((g1.discharge_m3s - 150.0).abs() < 1e-3);
    }
}
