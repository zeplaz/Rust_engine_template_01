//! BUILD-READ-SITE-v0-002 — view-only site composition stub (10×8 dashed + zone labels).

use bevy::prelude::*;

use super::build_state::BuildGhostState;
use super::build_tool_authority::ActiveBuildTool;
use super::building_definitions::BuildingDefinitionRegistry;
use super::site_zone_grid::{
    cell_at, cell_fill, primary_loading_cell_count, zone_label_for, SiteZoneCell,
};
use super::pilot_catalog::site_zone_grid_for_arch_dna_preset;
use super::visual_authority::{
    ConstructionVisualRequests, SiteStubBoxRequest, SiteZoneLabelRequest, ZoneTileRequest,
};
use crate::strategic::BuildSiteTile;

pub const SITE_STUB_WIDTH: u32 = 10;
pub const SITE_STUB_DEPTH: u32 = 8;

/// Per-zone label for placement debug overlay.
#[derive(Resource, Clone, Debug, Default)]
pub struct SiteStubOverlayState {
    pub zone_labels: Vec<(BuildSiteTile, String)>,
    pub preset_id: Option<String>,
    pub site_width: u32,
    pub site_depth: u32,
}

fn push_zone_label(
    labels: &mut Vec<(BuildSiteTile, String)>,
    seen: &mut std::collections::HashSet<&'static str>,
    origin: BuildSiteTile,
    dx: u32,
    dz: u32,
    text: &'static str,
) {
    if !seen.insert(text) {
        return;
    }
    labels.push((
        BuildSiteTile {
            x: origin.x + dx,
            z: origin.z + dz,
        },
        text.to_owned(),
    ));
}

/// Fill site stub (dashed border + zone fills + labels) for ARCH-DNA pilot presets.
pub fn sync_site_stub_overlay_requests(
    tool: Res<ActiveBuildTool>,
    ghost: Res<BuildGhostState>,
    registry: Res<BuildingDefinitionRegistry>,
    mut requests: ResMut<ConstructionVisualRequests>,
    mut overlay: ResMut<SiteStubOverlayState>,
) {
    overlay.zone_labels.clear();
    overlay.preset_id = None;
    overlay.site_width = 0;
    overlay.site_depth = 0;

    let Some(intent) = tool.building_intent.as_ref() else {
        return;
    };
    let Some(preset_id) = intent.arch_dna_preset_id.as_deref() else {
        return;
    };
    let Some(grid) = site_zone_grid_for_arch_dna_preset(preset_id) else {
        return;
    };
    let Some(origin) = ghost.origin else {
        return;
    };

    overlay.preset_id = Some(preset_id.to_owned());
    overlay.site_width = grid.width;
    overlay.site_depth = grid.depth;

    requests.site_stub_boxes.push(SiteStubBoxRequest {
        origin,
        width: grid.width,
        depth: grid.depth,
    });

    let mut label_seen = std::collections::HashSet::new();
    for dz in 0..grid.depth {
        for dx in 0..grid.width {
            let Some(cell) = cell_at(&grid, dx, dz) else {
                continue;
            };
            if let Some(color) = cell_fill(cell) {
                let tile = BuildSiteTile {
                    x: origin.x + dx,
                    z: origin.z + dz,
                };
                requests.zone_tiles.push(ZoneTileRequest {
                    center: Vec3::new(tile.x as f32 + 0.5, 0.02, tile.z as f32 + 0.5),
                    color,
                });
            }
            if let Some(label) = zone_label_for(cell) {
                push_zone_label(
                    &mut overlay.zone_labels,
                    &mut label_seen,
                    origin,
                    dx,
                    dz,
                    label,
                );
                requests.site_zone_labels.push(SiteZoneLabelRequest {
                    world: Vec3::new(
                        origin.x as f32 + dx as f32 + 0.5,
                        0.15,
                        origin.z as f32 + dz as f32 + 0.5,
                    ),
                    text: label.to_owned(),
                });
            }
        }
    }

    let _ = registry;
}

#[must_use]
pub fn primary_pct_of_site_stub(occupied_primary_loading: u32, site_cells: u32) -> f32 {
    let denom = site_cells.max(1) as f32;
    occupied_primary_loading as f32 / denom
}

/// BUILD-READ-SITE-v0-002 witness — JSON-backed 10×8 stub + zone labels for pilot.
#[must_use]
pub fn build_read_site_v0_002_witness_green() -> bool {
    build_read_site_v0_002_self_check().is_ok()
}

fn build_read_site_v0_002_self_check() -> Result<(), &'static str> {
    use super::pilot_catalog::PilotCatalog;

    let catalog = PilotCatalog::load_from_disk();
    let grammar_with_site: Vec<_> = catalog
        .grammar_pilots()
        .filter(|p| p.site_json_path.is_some())
        .collect();
    if grammar_with_site.len() < 2 {
        return Err("min_grammar_site_pilots");
    }
    for pilot in grammar_with_site {
        let preset = pilot.arch_dna_preset.as_deref().ok_or("preset")?;
        let grid = site_zone_grid_for_arch_dna_preset(preset).ok_or("site_load")?;
        if grid.width < SITE_STUB_WIDTH / 2 || grid.depth < SITE_STUB_DEPTH / 2 {
            return Err("site_dims");
        }
        let primary = primary_loading_cell_count(&grid);
        let pct = primary_pct_of_site_stub(primary, grid.width * grid.depth);
        if !(0.10..=0.55).contains(&pct) {
            return Err("primary_pct");
        }
        if primary_loading_cell_count(&grid) == 0 {
            return Err("primary_zone");
        }
        if !grid.cells.iter().any(|c| *c == SiteZoneCell::Utility) {
            return Err("utility_yard");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_v0_002_witness_green() {
        assert!(super::build_read_site_v0_002_witness_green());
    }
}
