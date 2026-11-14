//! D8 — presentation frame from MissionBook hints + [`AirPresentationPolicy`].
//!
//! Read-only vs sim: builds a capped sprite/hero request list for future overlay extract.
//! Does not spawn meshes; D8 chrome / world-map overlay consumes this resource.

use bevy::prelude::*;

use crate::combat::MissionBook;

use super::air_presentation_policy::{
    AirGeometryTier, AirPresentationBand, AirPresentationPolicy, AirPresentationSortieHint,
    FlightPathFidelity,
};

/// One drawable mark requested under current LOD policy.
#[derive(Clone, Debug, PartialEq)]
pub struct AirPresentationMark {
    pub mission_id: u64,
    pub band: AirPresentationBand,
    pub geometry: AirGeometryTier,
    pub flight: FlightPathFidelity,
    pub wind_mps: Vec2,
    pub heading: Vec2,
    pub role_mass: bool,
}

/// Sole writer: [`sync_air_presentation_frame`]. Consumed by D8/D7 overlays (read-only).
#[derive(Resource, Clone, Debug, Default)]
pub struct AirPresentationFrame {
    pub marks: Vec<AirPresentationMark>,
    pub density_only: u32,
    pub revision: u64,
}

/// Map zoom proxy 0..1 (1 = tactical near). Callers may override via resource later.
#[derive(Resource, Clone, Debug)]
pub struct AirPresentationZoom {
    /// 0 = strategic far, 1 = tactical near.
    pub nearness: f32,
}

impl Default for AirPresentationZoom {
    fn default() -> Self {
        Self { nearness: 0.35 }
    }
}

fn band_for(nearness: f32, mass: bool, density_high: bool) -> AirPresentationBand {
    if nearness < 0.2 {
        AirPresentationBand::Far
    } else if nearness < 0.55 {
        AirPresentationBand::Mid
    } else if mass && density_high {
        AirPresentationBand::Mid
    } else if nearness > 0.85 && !mass {
        AirPresentationBand::Hero
    } else {
        AirPresentationBand::Near
    }
}

/// Build presentation marks from mission hints under policy caps (never writes MissionBook).
pub fn sync_air_presentation_frame(
    book: Option<Res<MissionBook>>,
    policy: Res<AirPresentationPolicy>,
    zoom: Res<AirPresentationZoom>,
    mut frame: ResMut<AirPresentationFrame>,
) {
    let Some(book) = book else {
        if !frame.marks.is_empty() {
            frame.marks.clear();
            frame.density_only = 0;
            frame.revision = frame.revision.wrapping_add(1);
        }
        return;
    };
    let hints = &book.presentation_hints;
    let mass_n = hints.iter().filter(|h| h.role_mass).count();
    let density_high = mass_n > policy.sprite_cap_per_view as usize / 2;

    let mut marks = Vec::new();
    let mut density_only = 0u32;
    let mut sprites = 0u32;
    let mut heroes = 0u32;

    for h in hints {
        let band = band_for(zoom.nearness, h.role_mass, density_high);
        if matches!(band, AirPresentationBand::Far) {
            density_only = density_only.saturating_add(1);
            continue;
        }
        let (geometry, flight) = if h.role_mass {
            (policy.mass_geometry(), policy.mass_flight())
        } else {
            (policy.scarce_geometry_tier, policy.scarce_flight())
        };
        if matches!(geometry, AirGeometryTier::VoxelOrMesh) {
            if heroes >= policy.voxel_hero_budget {
                density_only = density_only.saturating_add(1);
                continue;
            }
            heroes = heroes.saturating_add(1);
        } else if matches!(band, AirPresentationBand::Mid | AirPresentationBand::Near) {
            if sprites >= policy.sprite_cap_per_view {
                density_only = density_only.saturating_add(1);
                continue;
            }
            if matches!(band, AirPresentationBand::Near) && sprites >= policy.near_animated_cap {
                // Still allow as mid-style mark under sprite cap.
            }
            sprites = sprites.saturating_add(1);
        }
        marks.push(AirPresentationMark {
            mission_id: h.mission_id,
            band,
            geometry,
            flight,
            wind_mps: h.wind_mps,
            heading: h.heading,
            role_mass: h.role_mass,
        });
        let _ = AirPresentationSortieHint::default();
    }

    frame.marks = marks;
    frame.density_only = density_only;
    frame.revision = frame.revision.wrapping_add(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::MissionBook;

    #[test]
    fn far_zoom_collapses_to_density() {
        let mut book = MissionBook::default();
        book.presentation_hints.push(AirPresentationSortieHint {
            mission_id: 1,
            role_mass: true,
            ..Default::default()
        });
        let policy = AirPresentationPolicy::default();
        let zoom = AirPresentationZoom { nearness: 0.05 };
        let mut frame = AirPresentationFrame::default();
        // Manual call body:
        let mass_n = 1;
        let density_high = mass_n > policy.sprite_cap_per_view as usize / 2;
        let band = band_for(zoom.nearness, true, density_high);
        assert_eq!(band, AirPresentationBand::Far);
        let _ = (book, frame);
    }
}
