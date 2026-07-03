//! DES-MINIMAP-VEG-LEGEND-001 / **DES-MINIMAP-VEG-LEGEND-002** — collapsible topology + burn legend.

use bevy_egui::egui;

use crate::engine::states::BaseState;
use crate::gui::minimap_shell::{MinimapOverlayMask, MinimapShellState};

#[derive(Clone, Copy, Debug)]
pub struct TopologyLegendEntry {
    pub glyph: &'static str,
    pub word: &'static str,
    pub hex: &'static str,
}

pub const TOPOLOGY_LEGEND_ENTRIES: [TopologyLegendEntry; 6] = [
    TopologyLegendEntry {
        glyph: "N",
        word: "Network",
        hex: "#4a6fa5",
    },
    TopologyLegendEntry {
        glyph: "C",
        word: "Corridor",
        hex: "#7a6a4a",
    },
    TopologyLegendEntry {
        glyph: "P",
        word: "Patch",
        hex: "#3d8b5f",
    },
    TopologyLegendEntry {
        glyph: "R",
        word: "Ring",
        hex: "#6a5a8a",
    },
    TopologyLegendEntry {
        glyph: "K",
        word: "Cluster",
        hex: "#2f7d4a",
    },
    TopologyLegendEntry {
        glyph: "F",
        word: "Fringe",
        hex: "#8a9a6a",
    },
];

/// Burn scar adjunct rows — DES-MINIMAP-VEG-LEGEND-002 §4.
pub const FIRE_LEGEND_ENTRIES: [TopologyLegendEntry; 3] = [
    TopologyLegendEntry {
        glyph: "S",
        word: "Scar",
        hex: "#3a3a3a",
    },
    TopologyLegendEntry {
        glyph: "B",
        word: "Active burn",
        hex: "#e87830",
    },
    TopologyLegendEntry {
        glyph: "G",
        word: "Regrowth",
        hex: "#6a9a48",
    },
];

fn hex_to_color32(hex: &str) -> egui::Color32 {
    let s = hex.trim_start_matches('#');
    if s.len() != 6 {
        return egui::Color32::GRAY;
    }
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(128);
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(128);
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(128);
    egui::Color32::from_rgb(r, g, b)
}

#[must_use]
pub fn minimap_topology_legend_collapsed_label() -> String {
    format!("▶ {} kinds", TOPOLOGY_LEGEND_ENTRIES.len())
}

#[must_use]
pub fn minimap_topology_legend_expanded_lines() -> Vec<String> {
    TOPOLOGY_LEGEND_ENTRIES
        .iter()
        .map(|e| format!("{} {}", e.glyph, e.word))
        .collect()
}

#[must_use]
pub fn minimap_burn_legend_expanded_lines() -> Vec<String> {
    FIRE_LEGEND_ENTRIES
        .iter()
        .map(|e| format!("{} {}", e.glyph, e.word))
        .collect()
}

#[must_use]
pub fn minimap_burn_legend_visible(ecology_heat: bool, veg_burn_rows: u32) -> bool {
    ecology_heat && veg_burn_rows > 0
}

#[must_use]
pub fn minimap_topology_legend_status_copy(
    overlays: &MinimapOverlayMask,
    ecology_rows: u32,
    map_updating: bool,
) -> Option<String> {
    if map_updating {
        return Some("◐ Map updating…".into());
    }
    if !overlays.ecology_heat {
        return Some("○ Ecology off".into());
    }
    if ecology_rows == 0 {
        return Some("○ No landscape data in view".into());
    }
    None
}

fn draw_legend_grid(ui: &mut egui::Ui, entries: &[TopologyLegendEntry]) {
    egui::Grid::new(egui::Id::new(format!("minimap_legend_grid_{:p}", entries.as_ptr())))
        .num_columns(3)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            for entry in entries {
                let color = hex_to_color32(entry.hex);
                let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 1.0, color);
                ui.label(egui::RichText::new(entry.glyph).small().strong());
                ui.label(egui::RichText::new(entry.word).small());
                ui.end_row();
            }
        });
}

/// GPU compositor path — legend chrome below the Bevy minimap image (not map raster).
pub fn draw_minimap_topology_legend_gpu_chrome(
    ctx: &egui::Context,
    shell: &mut MinimapShellState,
    overlays: &MinimapOverlayMask,
    ecology_rows: u32,
    veg_burn_rows: u32,
    base: BaseState,
) {
    let Some(content) = shell.last_window_rect else {
        return;
    };
    let legend_top = shell
        .last_image_rect
        .map(|r| r.max.y)
        .unwrap_or(content.max.y);
    let available_h = (content.max.y - legend_top).max(0.0);
    if available_h < 20.0 {
        return;
    }
    let est_h: f32 = if shell.topology_legend_expanded {
        72.0
    } else {
        26.0
    };
    let width = content.width().max(120.0);
    let height = est_h.min(available_h);
    crate::render::trace_minimap_size_writer("legend.gpu_chrome", width, height);
    egui::Area::new(egui::Id::new("minimap_topology_legend_gpu"))
        .fixed_pos(egui::pos2(content.min.x, legend_top))
        .show(ctx, |ui| {
            ui.set_width(width);
            draw_minimap_topology_legend_ui(ui, shell, overlays, ecology_rows, veg_burn_rows, base);
        });
}

pub fn draw_minimap_topology_legend_ui(
    ui: &mut egui::Ui,
    shell: &mut MinimapShellState,
    overlays: &MinimapOverlayMask,
    ecology_rows: u32,
    veg_burn_rows: u32,
    base: BaseState,
) {
    let map_updating = shell.compositor_revision != shell.cached_texture_revision;
    if let Some(status) = minimap_topology_legend_status_copy(overlays, ecology_rows, map_updating) {
        ui.label(egui::RichText::new(status).small().weak());
        return;
    }

    let default_open = !matches!(base, BaseState::Simulation);
    if !shell.topology_legend_user_toggled {
        shell.topology_legend_expanded = default_open;
    }

    let header = if shell.topology_legend_expanded {
        "▼ Ecology kinds".to_string()
    } else {
        minimap_topology_legend_collapsed_label()
    };
    let resp = ui
        .add(egui::Button::new(egui::RichText::new(header).small()).min_size(egui::vec2(0.0, 22.0)))
        .interact(egui::Sense::click());
    if resp.clicked() {
        shell.topology_legend_expanded = !shell.topology_legend_expanded;
        shell.topology_legend_user_toggled = true;
    }

    if shell.topology_legend_expanded {
        draw_legend_grid(ui, &TOPOLOGY_LEGEND_ENTRIES);
        if minimap_burn_legend_visible(overlays.ecology_heat, veg_burn_rows) {
            ui.separator();
            ui.label(egui::RichText::new("── Fire read ──").small().weak());
            draw_legend_grid(ui, &FIRE_LEGEND_ENTRIES);
        }
    }
}

#[must_use]
pub fn minimap_topology_legend_ui_witness_green() -> bool {
    TOPOLOGY_LEGEND_ENTRIES.len() >= 6
        && TOPOLOGY_LEGEND_ENTRIES.iter().all(|e| !e.word.is_empty())
        && minimap_topology_legend_expanded_lines().len() == 6
        && FIRE_LEGEND_ENTRIES.len() == 3
        && minimap_burn_legend_expanded_lines().len() == 3
}

#[must_use]
pub fn minimap_burn_legend_ui_wired() -> bool {
    FIRE_LEGEND_ENTRIES
        .iter()
        .any(|e| e.word == "Active burn")
        && minimap_burn_legend_visible(true, 1)
        && !minimap_burn_legend_visible(true, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topology_legend_words_match_design() {
        assert!(minimap_topology_legend_ui_witness_green());
        let lines = minimap_topology_legend_expanded_lines();
        assert!(lines.iter().any(|l| l.contains("Network")));
        assert!(lines.iter().any(|l| l.contains("Patch")));
    }

    #[test]
    fn burn_legend_visible_only_when_burn_rows() {
        assert!(minimap_burn_legend_ui_wired());
        let burn_lines = minimap_burn_legend_expanded_lines();
        assert!(burn_lines.iter().any(|l| l.contains("Active burn")));
    }
}
