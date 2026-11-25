//! Vehicle info card (Bevy UI): the truck the player clicked or box-selected
//! ([`HudVehicleFocus`]): owner, what it is doing, its cargo and load, where it is going and its
//! capacity and speed. Presentation only: reads the order sim. Movable like every info card.

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::economy::logistics::orders::model::{Vehicle, VehiclePhase};
use crate::economy::logistics::orders::plugin::LogisticsOrderSimRes;
use crate::gui::hud::panel_drag::HudCardPlacements;
use crate::gui::selection_filter::{HudVehicleFocus, SelectedVehicles};
use crate::gui::style::{CmdUiMonoFont, UiPalette};

#[derive(Component)]
pub struct VehicleInfoCardRoot;

#[derive(Component)]
pub struct VehicleInfoCardText;

/// Card lines for one vehicle (site names from the order sim).
#[must_use]
pub fn vehicle_card_lines(v: &Vehicle, site_name: &dyn Fn(u32) -> String, selected: usize) -> Vec<String> {
    let doing = match &v.phase {
        VehiclePhase::Idle => "idle".to_string(),
        VehiclePhase::ToPickup(l) => format!("driving to pick up ({:.0} %)", l.fraction() * 100.0),
        VehiclePhase::Loading(_) => "loading".to_string(),
        VehiclePhase::Hauling(l) => format!("hauling ({:.0} %)", l.fraction() * 100.0),
        VehiclePhase::Unloading(_) => "unloading".to_string(),
    };
    let mut lines = vec![
        format!("TRUCK #{}  · owner {}", v.id.0, v.owner.0),
        format!("Doing:     {doing}"),
    ];
    match &v.assignment {
        Some(a) => {
            lines.push(format!("Cargo:     {} × {} of {}", a.cargo.as_str(), a.load, v.capacity_units));
            lines.push(format!("Route:     {} → {}", site_name(a.source.0), site_name(a.sink.0)));
        }
        None => lines.push(format!("Cargo:     empty (capacity {})", v.capacity_units)),
    }
    lines.push(format!("Home:      {}  · speed {:.0} km/h", site_name(v.home.0), v.speed_mps * 3.6));
    if selected > 1 {
        lines.push(format!("Selected:  {selected} vehicles (box) — showing the first"));
    }
    lines
}

/// Show / update / close the vehicle card.
#[allow(clippy::too_many_arguments)]
pub fn sync_vehicle_info_card_system(
    mut commands: Commands,
    mut focus: ResMut<HudVehicleFocus>,
    selected: Res<SelectedVehicles>,
    sim: Option<Res<LogisticsOrderSimRes>>,
    roots: Query<Entity, With<VehicleInfoCardRoot>>,
    mut texts: Query<&mut Text, With<VehicleInfoCardText>>,
    style: (Option<Res<UiPalette>>, Option<Res<CmdUiMonoFont>>),
    placements: Res<HudCardPlacements>,
) {
    let vehicle = focus.0.and_then(|id| sim.as_ref().and_then(|s| s.sim.vehicles.get(&id).cloned()));
    let Some(v) = vehicle else {
        if focus.0.is_some() {
            focus.0 = None; // the truck is gone (scrapped, destroyed)
        }
        for e in &roots {
            commands.entity(e).despawn();
        }
        return;
    };
    let names = sim.as_ref().map(|s| s.sim.sites.clone()).unwrap_or_default();
    let site_name = |id: u32| {
        names
            .get(&crate::economy::logistics::orders::model::SiteId(id))
            .cloned()
            .unwrap_or_else(|| format!("site {id}"))
    };
    let body = vehicle_card_lines(&v, &site_name, selected.0.len()).join("\n");
    if let Some(mut text) = texts.iter_mut().next() {
        if text.0 != body {
            text.0 = body;
        }
        return;
    }
    let (Some(palette), Some(mono)) = (style.0, style.1) else {
        return;
    };
    // Left of the site card by default, so both can be open.
    let (anchor, drag, right, top) = placements.card("vehicle_card", Vec2::new(400.0, 120.0));
    commands
        .spawn((
            VehicleInfoCardRoot,
            anchor,
            drag,
            Node {
                position_type: PositionType::Absolute,
                top,
                right,
                width: Val::Px(320.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_wire_magenta()),
            FocusPolicy::Block,
            ZIndex(950),
        ))
        .with_children(|root| {
            root.spawn((
                VehicleInfoCardText,
                Text::new(body),
                TextFont::from_font_size(11.5).with_font(mono.0.clone()),
                TextColor(palette.bevy_primary_text()),
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::logistics::orders::model::{Assignment, CargoClassMask, CargoKind, Leg, OrderId, OwnerId, SiteId, VehicleId};

    #[test]
    fn the_card_says_what_the_truck_carries_and_where() {
        let v = Vehicle {
            id: VehicleId(7),
            owner: OwnerId::PLAYER,
            class_mask: CargoClassMask(1),
            capacity_units: 40,
            speed_mps: 12.0,
            home: SiteId(1),
            at: SiteId(1),
            phase: VehiclePhase::Hauling(Leg::new(10)),
            assignment: Some(Assignment { order: OrderId(3), source: SiteId(1), sink: SiteId(2), cargo: CargoKind::new("Gravel"), load: 40 }),
        };
        let name = |id: u32| if id == 1 { "HQ".to_string() } else { "Works 2".to_string() };
        let text = vehicle_card_lines(&v, &name, 3).join("\n");
        assert!(text.contains("TRUCK #7"));
        assert!(text.contains("Gravel × 40 of 40"));
        assert!(text.contains("HQ → Works 2"));
        assert!(text.contains("hauling"));
        assert!(text.contains("3 vehicles"));
    }
}
