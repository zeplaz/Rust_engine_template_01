//! CLICK-UX-2 — map click on a construction footprint focuses plant/site info (RO).
//!
//! Yields while a build tool is armed or formation select owns the pointer path.
//! Writes only [`HudLogisticsFocus`] — never sim truth.
//!
//! **Selection is not faction-gated:** any site footprint is pickable (player or foreign).
//! Intel filtering (what the card reveals from sat / sensors / LOS) is a later layer —
//! for DamFlood experiments the dam is player-owned so full dam ops are available.

use bevy::prelude::*;

use crate::construction::{ActiveBuildTool, BuildTool};
use crate::gui::front_controls::{
    build_tool_blocks_front_orders, FrontMapCursor, SelectedFormation,
};
use crate::gui::hud::power_node_hover::footprint_contains_tile;
use crate::gui::hud::simulation_pointer_gate::{
    simulation_tool_pointer, SimulationToolPointer,
};
use crate::gui::logistics_focus::HudLogisticsFocus;
use crate::strategic::{ConstructionSite, SiteFootprint};

/// Prefer the smallest footprint covering the tile (nested pads).
#[must_use]
pub fn pick_construction_site_at_tile(
    sites: &[(Entity, &SiteFootprint)],
    tile: IVec2,
) -> Option<Entity> {
    sites
        .iter()
        .filter(|(_, fp)| footprint_contains_tile(fp, tile))
        .min_by_key(|(_, fp)| fp.tiles.len())
        .map(|(e, _)| *e)
}

/// Idle LMB on a site footprint → logistics/plant focus card.
pub fn map_pick_site_focus_system(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<FrontMapCursor>,
    tool: Option<Res<ActiveBuildTool>>,
    selected: Res<SelectedFormation>,
    sites: Query<(Entity, &SiteFootprint), With<ConstructionSite>>,
    mut focus: ResMut<HudLogisticsFocus>,
    gate: Option<Res<crate::gui::hud::simulation_pointer_gate::SimulationMapPointerGate>>,
) {
    if !buttons.just_released(MouseButton::Left) {
        return;
    }
    if !cursor.free {
        return;
    }
    if simulation_tool_pointer(gate.as_deref()) != SimulationToolPointer::Free {
        return;
    }
    if build_tool_blocks_front_orders(tool.as_deref()) {
        return;
    }
    // Formation click already handled selection; leave focus alone if a unit is selected.
    if !selected.0.is_empty() {
        return;
    }
    let Some(xy) = cursor.xy else {
        return;
    };
    let tile = IVec2::new(xy.x.floor() as i32, xy.y.floor() as i32);
    let list: Vec<(Entity, &SiteFootprint)> = sites.iter().collect();
    // Hit → focus that site (any faction). Miss → clear so cards dismiss.
    focus.tracked_entity = pick_construction_site_at_tile(&list, tile);
}

/// Clear formation selection when a place/demolish tool is armed (mode arbitration).
pub fn clear_formation_selection_when_build_armed(
    tool: Res<ActiveBuildTool>,
    mut selected: ResMut<SelectedFormation>,
) {
    if tool.is_changed() && tool.tool != BuildTool::None && !selected.0.is_empty() {
        selected.set(Vec::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::LayerType;

    #[test]
    fn prefers_smaller_footprint() {
        let big = SiteFootprint {
            tiles: vec![IVec2::ZERO, IVec2::X, IVec2::Y, IVec2::ONE],
            layer: LayerType::Surface,
        };
        let small = SiteFootprint {
            tiles: vec![IVec2::ZERO],
            layer: LayerType::Surface,
        };
        let a = Entity::from_bits(1);
        let b = Entity::from_bits(2);
        let hit = pick_construction_site_at_tile(&[(a, &big), (b, &small)], IVec2::ZERO);
        assert_eq!(hit, Some(b));
    }
}
