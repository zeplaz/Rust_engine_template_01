//! Site zone grid loader — BUILD-READ-SITE-v0-002 (`site_zone_grid_v1` JSON).

use std::path::Path;

use bevy_egui::egui;
use serde::Deserialize;

use super::ghost_visual::footprint_valid_color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SiteZoneCell {
    Void,
    Primary,
    Loading,
    Utility,
    Rail,
    Service,
    Parking,
}

#[derive(Debug, Clone)]
pub struct SiteZoneGrid {
    pub site_id: String,
    pub width: u32,
    pub depth: u32,
    pub cells: Vec<SiteZoneCell>,
}

#[derive(Debug, Deserialize)]
struct SiteZoneGridV1 {
    site_id: String,
    width: u32,
    depth: u32,
    cells: Vec<String>,
}

fn parse_zone_cell(name: &str) -> SiteZoneCell {
    match name {
        "primary" => SiteZoneCell::Primary,
        "loading" => SiteZoneCell::Loading,
        "utility" => SiteZoneCell::Utility,
        "rail" => SiteZoneCell::Rail,
        "service" => SiteZoneCell::Service,
        "parking" => SiteZoneCell::Parking,
        "buffer" | _ => SiteZoneCell::Void,
    }
}

#[must_use]
pub fn load_site_zone_grid_from_path(path: &Path) -> Option<SiteZoneGrid> {
    let text = std::fs::read_to_string(path).ok()?;
    let raw: SiteZoneGridV1 = serde_json::from_str(&text).ok()?;
    if raw.width == 0 || raw.depth == 0 {
        return None;
    }
    let expected = (raw.width * raw.depth) as usize;
    if raw.cells.len() != expected {
        return None;
    }
    Some(SiteZoneGrid {
        site_id: raw.site_id,
        width: raw.width,
        depth: raw.depth,
        cells: raw.cells.iter().map(|s| parse_zone_cell(s)).collect(),
    })
}

#[must_use]
pub fn cell_at(grid: &SiteZoneGrid, dx: u32, dz: u32) -> Option<SiteZoneCell> {
    if dx >= grid.width || dz >= grid.depth {
        return None;
    }
    let idx = (dz * grid.width + dx) as usize;
    grid.cells.get(idx).copied()
}

#[must_use]
pub fn cell_fill(cell: SiteZoneCell) -> Option<egui::Color32> {
    match cell {
        SiteZoneCell::Void => None,
        SiteZoneCell::Primary | SiteZoneCell::Loading => {
            let c = footprint_valid_color();
            Some(egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 90))
        }
        SiteZoneCell::Utility => Some(egui::Color32::from_rgba_unmultiplied(200, 140, 40, 38)),
        SiteZoneCell::Rail => Some(egui::Color32::from_rgba_unmultiplied(136, 136, 136, 64)),
        SiteZoneCell::Service => Some(egui::Color32::from_rgba_unmultiplied(80, 200, 220, 48)),
        SiteZoneCell::Parking => Some(egui::Color32::from_rgba_unmultiplied(68, 68, 68, 30)),
    }
}

#[must_use]
pub fn zone_label_for(cell: SiteZoneCell) -> Option<&'static str> {
    match cell {
        SiteZoneCell::Rail => Some("Rail"),
        SiteZoneCell::Utility => Some("Yard"),
        SiteZoneCell::Service => Some("Svc"),
        SiteZoneCell::Parking => Some("Park"),
        SiteZoneCell::Loading => Some("Load"),
        _ => None,
    }
}

#[must_use]
pub fn primary_loading_cell_count(grid: &SiteZoneGrid) -> u32 {
    grid.cells
        .iter()
        .filter(|c| matches!(c, SiteZoneCell::Primary | SiteZoneCell::Loading))
        .count() as u32
}

/// BUILD-GRAMMAR-SITE-ZONE-001 — site occupancy 15–40% of footprint matrix.
#[must_use]
pub fn site_occupancy_pct(grid: &SiteZoneGrid) -> f32 {
    let total = grid.cells.len().max(1) as f32;
    let occupied = grid
        .cells
        .iter()
        .filter(|c| !matches!(c, SiteZoneCell::Void))
        .count() as f32;
    occupied / total
}

#[must_use]
pub fn site_zone_occupancy_witness_green(grid: &SiteZoneGrid) -> bool {
    let pct = site_occupancy_pct(grid);
    (0.15..=0.40).contains(&pct)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::pilot_catalog::site_zone_grid_for_arch_dna_preset;

    #[test]
    fn grammar_pilot_site_json_loads() {
        use crate::construction::pilot_catalog::PilotCatalog;

        let catalog = PilotCatalog::load_from_disk();
        for pilot in catalog.grammar_pilots().filter(|p| p.site_json_path.is_some()) {
            let preset = pilot
                .arch_dna_preset
                .as_deref()
                .expect("grammar preset");
            let grid =
                site_zone_grid_for_arch_dna_preset(preset).expect("site grid");
            assert!(grid.width >= 4 && grid.depth >= 4, "pilot={}", pilot.id);
            assert!(primary_loading_cell_count(&grid) > 0);
        }
    }
}
