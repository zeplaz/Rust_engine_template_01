//! Tactical map tile occupation visuals (Syx-style): outlines, state colors, toggleable info.

use bevy::prelude::*;

use crate::construction::building_catalog::FootprintMatrix;
use crate::strategic::BuildSiteTile;

/// HUD / gameplay toggles for construction tile drawing.
#[derive(Resource, Debug, Clone)]
pub struct ConstructionTileVisualSettings {
    /// When false: tile outlines + state colors only (no text/icons).
    pub show_tile_info_labels: bool,
    pub show_occupation_tiles: bool,
    pub show_site_phase_tiles: bool,
}

impl Default for ConstructionTileVisualSettings {
    fn default() -> Self {
        Self {
            show_tile_info_labels: true,
            show_occupation_tiles: true,
            show_site_phase_tiles: true,
        }
    }
}

impl FootprintMatrix {
    /// Local (dx, dz) offsets for occupied cells in row-major `cells`.
    pub fn occupied_local_offsets(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.cells.iter().enumerate().filter_map(|(i, &occ)| {
            if occ == 0 {
                return None;
            }
            let w = self.width.max(1);
            Some((i as u32 % w, i as u32 / w))
        })
    }

    #[must_use]
    pub fn is_occupied_local(&self, dx: u32, dz: u32) -> bool {
        if dx >= self.width || dz >= self.depth {
            return false;
        }
        let i = (dz * self.width + dx) as usize;
        self.cells.get(i).copied().unwrap_or(0) != 0
    }
}

/// Walk grid tiles between two build-site cells (axis steps).
pub fn build_site_tiles_between(a: BuildSiteTile, b: BuildSiteTile) -> Vec<BuildSiteTile> {
    let mut out = Vec::new();
    let mut x = a.x as i32;
    let mut z = a.z as i32;
    let tx = b.x as i32;
    let tz = b.z as i32;
    out.push(BuildSiteTile {
        x: x as u32,
        z: z as u32,
    });
    while x != tx || z != tz {
        if x != tx {
            x += (tx - x).signum();
        } else if z != tz {
            z += (tz - z).signum();
        }
        out.push(BuildSiteTile {
            x: x as u32,
            z: z as u32,
        });
    }
    out
}

pub fn toggle_construction_tile_info_labels(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<crate::gui::InputBindings>,
    mut settings: ResMut<ConstructionTileVisualSettings>,
) {
    if keys.just_pressed(bindings.toggle_construction_tile_labels) {
        settings.show_tile_info_labels = !settings.show_tile_info_labels;
    }
}
