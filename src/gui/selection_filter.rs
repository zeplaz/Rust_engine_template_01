//! What a map click or box-drag selects: everything, only buildings, only civilian things
//! (trucks and other vehicles), or only military formations. A strip of four buttons on the map
//! and the `selection_filter_cycle` key switch it.
//!
//! The front controls own the drag box and the formation selection
//! ([`crate::gui::front_controls`]); they publish each finished box ([`FrontDragBox::completed`]).
//! This module turns that box (and clicks) into the other selections, one domain each:
//! - buildings → [`SelectedSites`] + the site card focus ([`HudLogisticsFocus`]);
//! - civilian → [`SelectedVehicles`] + the vehicle card ([`HudVehicleFocus`]);
//! - military off → the formation selection is cleared.
//!
//! Presentation only: nothing here writes sim truth.

use bevy::prelude::*;

use crate::construction::ActiveBuildTool;
use crate::economy::logistics::orders::model::VehicleId;
use crate::gui::front_controls::{build_tool_blocks_front_orders, FrontDragBox, FrontMapCursor, SelectedFormation};
use crate::gui::hud::power_node_hover::footprint_contains_tile;
use crate::gui::hud::simulation_pointer_gate::{simulation_tool_pointer, SimulationMapPointerGate, SimulationToolPointer};
use crate::gui::logistics_focus::HudLogisticsFocus;
use crate::presentation::truck_view::TruckPresentation;
use crate::strategic::{ConstructionSite, SiteFootprint};
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// What the map selects.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectionFilter {
    #[default]
    All,
    Buildings,
    Civilian,
    Military,
}

impl SelectionFilter {
    pub const ALL: [SelectionFilter; 4] =
        [SelectionFilter::All, SelectionFilter::Buildings, SelectionFilter::Civilian, SelectionFilter::Military];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            SelectionFilter::All => "All",
            SelectionFilter::Buildings => "Buildings",
            SelectionFilter::Civilian => "Civilian",
            SelectionFilter::Military => "Military",
        }
    }

    #[must_use]
    pub fn buildings(self) -> bool {
        matches!(self, SelectionFilter::All | SelectionFilter::Buildings)
    }

    #[must_use]
    pub fn civilian(self) -> bool {
        matches!(self, SelectionFilter::All | SelectionFilter::Civilian)
    }

    #[must_use]
    pub fn military(self) -> bool {
        matches!(self, SelectionFilter::All | SelectionFilter::Military)
    }

    #[must_use]
    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|f| *f == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
}

/// Sites in the last box selection (the first one is the site card's focus).
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct SelectedSites(pub Vec<Entity>);

/// Vehicles in the last selection (the first one is the vehicle card's focus).
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct SelectedVehicles(pub Vec<VehicleId>);

/// The vehicle the vehicle card shows.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct HudVehicleFocus(pub Option<VehicleId>);

/// A truck within this many tiles of a click is picked.
const VEHICLE_PICK_TILES: f32 = 0.8;

/// Sites with any footprint tile inside the tile box `lo..=hi`.
#[must_use]
pub fn sites_in_box(sites: &[(Entity, &SiteFootprint)], lo: Vec2, hi: Vec2) -> Vec<Entity> {
    let (l, h) = (lo.floor().as_ivec2(), hi.floor().as_ivec2());
    let mut out: Vec<Entity> = sites
        .iter()
        .filter(|(_, fp)| fp.tiles.iter().any(|t| t.x >= l.x && t.x <= h.x && t.y >= l.y && t.y <= h.y))
        .map(|(e, _)| *e)
        .collect();
    out.sort();
    out
}

/// Vehicles (tile position) inside the tile box.
#[must_use]
pub fn vehicles_in_box(vehicles: &[(VehicleId, Vec2)], lo: Vec2, hi: Vec2) -> Vec<VehicleId> {
    let mut out: Vec<VehicleId> = vehicles
        .iter()
        .filter(|(_, p)| p.x >= lo.x && p.x <= hi.x && p.y >= lo.y && p.y <= hi.y)
        .map(|(id, _)| *id)
        .collect();
    out.sort_by_key(|id| id.0);
    out
}

/// Nearest vehicle to `at` within [`VEHICLE_PICK_TILES`].
#[must_use]
pub fn vehicle_at(vehicles: &[(VehicleId, Vec2)], at: Vec2) -> Option<VehicleId> {
    vehicles
        .iter()
        .map(|(id, p)| (*id, p.distance(at)))
        .filter(|(_, d)| *d <= VEHICLE_PICK_TILES)
        .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0 .0.cmp(&b.0 .0)))
        .map(|(id, _)| id)
}

fn truck_tiles(trucks: &Query<(&TruckPresentation, &Transform)>) -> Vec<(VehicleId, Vec2)> {
    trucks
        .iter()
        .map(|(t, tf)| (t.vehicle_id, tf.translation.truncate() / TILE_METRES_GAMEPLAY))
        .collect()
}

/// The `selection_filter_cycle` key steps All → Buildings → Civilian → Military.
pub fn cycle_selection_filter_system(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<crate::gui::InputBindings>,
    mut filter: ResMut<SelectionFilter>,
) {
    if keys.just_pressed(bindings.selection_filter_cycle) {
        *filter = filter.next();
    }
}

/// Military off: drop whatever formations the front controls selected.
pub fn apply_selection_filter_to_formations_system(filter: Res<SelectionFilter>, mut selected: ResMut<SelectedFormation>) {
    if !filter.military() && !selected.0.is_empty() {
        selected.set(Vec::new());
    }
}

/// A finished box selects the sites in it (buildings allowed); the first is focused.
pub fn box_select_sites_system(
    filter: Res<SelectionFilter>,
    drag: Res<FrontDragBox>,
    sites: Query<(Entity, &SiteFootprint), With<ConstructionSite>>,
    mut selected: ResMut<SelectedSites>,
    mut focus: ResMut<HudLogisticsFocus>,
) {
    let Some((lo, hi, extend)) = drag.completed else {
        return;
    };
    if !filter.buildings() {
        return;
    }
    let list: Vec<(Entity, &SiteFootprint)> = sites.iter().collect();
    let mut v = sites_in_box(&list, lo, hi);
    if extend {
        v.extend(selected.0.iter().copied());
        v.sort();
        v.dedup();
    }
    focus.tracked_entity = v.first().copied();
    selected.0 = v;
}

/// A finished box selects the vehicles in it (civilian allowed); the first one gets the card.
pub fn box_select_vehicles_system(
    filter: Res<SelectionFilter>,
    drag: Res<FrontDragBox>,
    trucks: Query<(&TruckPresentation, &Transform)>,
    mut selected: ResMut<SelectedVehicles>,
    mut focus: ResMut<HudVehicleFocus>,
) {
    let Some((lo, hi, extend)) = drag.completed else {
        return;
    };
    if !filter.civilian() {
        return;
    }
    let mut v = vehicles_in_box(&truck_tiles(&trucks), lo, hi);
    if extend {
        v.extend(selected.0.iter().copied());
        v.sort_by_key(|id| id.0);
        v.dedup();
    }
    focus.0 = v.first().copied();
    selected.0 = v;
}

/// Click on a truck (civilian allowed) → vehicle card; a click elsewhere closes it.
pub fn click_pick_vehicle_system(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<FrontMapCursor>,
    filter: Res<SelectionFilter>,
    tool: Option<Res<ActiveBuildTool>>,
    gate: Option<Res<SimulationMapPointerGate>>,
    trucks: Query<(&TruckPresentation, &Transform)>,
    mut focus: ResMut<HudVehicleFocus>,
) {
    if !buttons.just_released(MouseButton::Left) || !cursor.free {
        return;
    }
    if simulation_tool_pointer(gate.as_deref()) != SimulationToolPointer::Free
        || build_tool_blocks_front_orders(tool.as_deref())
    {
        return;
    }
    let Some(at) = cursor.xy else {
        return;
    };
    let hit = if filter.civilian() { vehicle_at(&truck_tiles(&trucks), at) } else { None };
    if focus.0 != hit {
        focus.0 = hit;
    }
}

/// Does a click at `tile` focus a site? (buildings allowed, tile inside a footprint)
#[must_use]
pub fn site_click_allowed(filter: SelectionFilter, sites: &[(Entity, &SiteFootprint)], tile: IVec2) -> bool {
    filter.buildings() && sites.iter().any(|(_, fp)| footprint_contains_tile(fp, tile))
}

// ───────────────────────── Filter strip (play UI) ─────────────────────────

#[derive(Component)]
pub struct SelectionFilterStripRoot;

#[derive(Component, Clone, Copy)]
pub struct SelectionFilterButton(pub SelectionFilter);

/// Spawn the four-button strip once (top-right, above the cards; movable like a card).
pub fn spawn_selection_filter_strip_system(
    mut commands: Commands,
    existing: Query<(), With<SelectionFilterStripRoot>>,
    palette: Option<Res<crate::gui::style::UiPalette>>,
    mono: Option<Res<crate::gui::style::CmdUiMonoFont>>,
    placements: Res<crate::gui::hud::panel_drag::HudCardPlacements>,
) {
    if !existing.is_empty() {
        return;
    }
    let (Some(palette), Some(mono)) = (palette, mono) else {
        return;
    };
    use crate::gui::style::widgets::{spawn_button, UiButton, UiButtonVariant, UiKit, UiTooltip, GAP};
    let kit = UiKit::new(&palette, mono.0.clone());
    let (anchor, drag, right, top) = placements.card("selection_filter", Vec2::new(12.0, 84.0));
    commands
        .spawn((
            SelectionFilterStripRoot,
            anchor,
            drag,
            Node {
                position_type: PositionType::Absolute,
                right,
                top,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(GAP),
                padding: UiRect::all(Val::Px(GAP)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_border_subtle()),
            bevy::ui::FocusPolicy::Block,
            ZIndex(940),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new("Select:"),
                kit.text_font(crate::gui::style::widgets::FONT_SMALL),
                TextColor(palette.bevy_text_muted()),
            ));
            for f in SelectionFilter::ALL {
                let mut b = UiButton::new(UiButtonVariant::Normal);
                b.selected = f == SelectionFilter::default();
                spawn_button(
                    row,
                    &kit,
                    f.label(),
                    b,
                    (SelectionFilterButton(f), UiTooltip(format!("Click / box-drag selects {} (` cycles)", f.label().to_lowercase()))),
                );
            }
        });
}

/// Strip buttons set the filter; the selected button follows the filter.
pub fn selection_filter_strip_system(
    mut filter: ResMut<SelectionFilter>,
    mut buttons: Query<(&crate::gui::DeprecatedInteraction, &SelectionFilterButton, &mut crate::gui::style::widgets::UiButton)>,
) {
    for (i, b, _) in &buttons {
        if *i == crate::gui::DeprecatedInteraction::Pressed && *filter != b.0 {
            *filter = b.0;
        }
    }
    for (_, b, mut ui) in &mut buttons {
        let sel = b.0 == *filter;
        if ui.selected != sel {
            ui.selected = sel;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::LayerType;

    #[test]
    fn selection_systems_have_disjoint_queries() {
        use crate::gui::style::widgets::test_support::init_ok;
        init_ok(cycle_selection_filter_system);
        init_ok(apply_selection_filter_to_formations_system);
        init_ok(box_select_sites_system);
        init_ok(box_select_vehicles_system);
        init_ok(click_pick_vehicle_system);
        init_ok(spawn_selection_filter_strip_system);
        init_ok(selection_filter_strip_system);
        init_ok(crate::gui::hud::vehicle_info_card::sync_vehicle_info_card_system);
        init_ok(crate::gui::hud::site_map_pick::map_pick_site_focus_system);
    }

    #[test]
    fn filters_cover_their_kinds_and_cycle() {
        assert!(SelectionFilter::All.buildings() && SelectionFilter::All.civilian() && SelectionFilter::All.military());
        assert!(SelectionFilter::Buildings.buildings() && !SelectionFilter::Buildings.military());
        assert!(SelectionFilter::Civilian.civilian() && !SelectionFilter::Civilian.buildings());
        assert!(SelectionFilter::Military.military() && !SelectionFilter::Military.civilian());
        let mut f = SelectionFilter::All;
        for _ in 0..4 {
            f = f.next();
        }
        assert_eq!(f, SelectionFilter::All);
    }

    #[test]
    fn a_box_takes_the_sites_and_trucks_inside_it() {
        let a = SiteFootprint { tiles: vec![IVec2::new(2, 2), IVec2::new(3, 2)], layer: LayerType::Surface };
        let b = SiteFootprint { tiles: vec![IVec2::new(20, 20)], layer: LayerType::Surface };
        let (ea, eb) = (Entity::from_bits(1), Entity::from_bits(2));
        assert_eq!(sites_in_box(&[(ea, &a), (eb, &b)], Vec2::new(3.0, 0.0), Vec2::new(10.0, 10.0)), vec![ea]);
        let trucks = [(VehicleId(1), Vec2::new(5.0, 5.0)), (VehicleId(2), Vec2::new(50.0, 5.0))];
        assert_eq!(vehicles_in_box(&trucks, Vec2::ZERO, Vec2::splat(10.0)), vec![VehicleId(1)]);
        assert_eq!(vehicle_at(&trucks, Vec2::new(5.4, 5.3)), Some(VehicleId(1)));
        assert_eq!(vehicle_at(&trucks, Vec2::new(8.0, 5.0)), None);
    }
}
