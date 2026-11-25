//! RD-7 freight-yard section of the site card: the yard's trains (busy / idle / held at a signal),
//! its stock of train materials and a "Build train" button (one order request; the order sim
//! debits the materials or rejects). Presentation only.

use bevy::prelude::*;

use crate::economy::logistics::orders::model::{CargoKind, SiteId};
use crate::economy::logistics::orders::plugin::{LogisticsOrderRequest, LogisticsOrderSiteId, LogisticsOrdersSnapshotRes};
use crate::economy::logistics::orders::sim::LogisticsOrderCommand;
use crate::economy::logistics::orders::snapshot::{LogisticsOrdersSnapshot, SiteRole};
use crate::gui::logistics_focus::HudLogisticsFocus;
use crate::gui::style::widgets::{spawn_button, spawn_section_header, UiButton, UiButtonVariant, UiKit, FONT_SMALL, GAP};
use crate::gui::DeprecatedInteraction;

#[derive(Component)]
pub struct YardCardSection(pub SiteId);

#[derive(Component)]
pub struct YardCardText;

#[derive(Component)]
pub struct YardBuildTrainButton;

/// The focused site's order-sim id when it is a freight yard.
fn focused_yard(focus: &HudLogisticsFocus, ids: &Query<&LogisticsOrderSiteId>, snap: &LogisticsOrdersSnapshot) -> Option<SiteId> {
    let id = ids.get(focus.tracked_entity?).ok()?.0;
    snap.site(id).filter(|s| s.role == SiteRole::Yard).map(|s| s.id)
}

/// Yard readout lines.
#[must_use]
pub fn yard_lines(snap: &LogisticsOrdersSnapshot, yard: SiteId) -> String {
    let rail = &crate::infrastructure::road_tier_table().rail;
    let trains: Vec<_> = snap.vehicles.iter().filter(|v| v.rail && (v.at == yard || v.from == yard || v.to == yard)).collect();
    let busy = trains.iter().filter(|v| v.order.is_some()).count();
    let stock = |tag: &str| snap.site(yard).and_then(|s| s.stock.iter().find(|r| r.cargo == tag)).map_or(0, |r| r.on_hand);
    let cost = rail.train_cost();
    let cost_txt = cost.iter().map(|(t, n)| format!("{n} {t}")).collect::<Vec<_>>().join(" + ");
    format!(
        "Trains here: {} ({} busy)\nStock: {} Steel · {} Machinery\nA new train ({} wagons): {}",
        trains.len(),
        busy,
        stock("Steel"),
        stock("Machinery"),
        rail.build_wagons,
        cost_txt
    )
}

/// Spawn / remove / refresh the yard section inside the site card.
#[allow(clippy::too_many_arguments)]
pub fn sync_yard_card_section_system(
    mut commands: Commands,
    focus: Res<HudLogisticsFocus>,
    ids: Query<&LogisticsOrderSiteId>,
    snapshot: Option<Res<LogisticsOrdersSnapshotRes>>,
    card: Query<Entity, With<crate::gui::hud::plant_info_card::FacilityInfoCardRoot>>,
    sections: Query<(Entity, &YardCardSection)>,
    mut texts: Query<&mut Text, With<YardCardText>>,
    style: (Option<Res<crate::gui::style::UiPalette>>, Option<Res<crate::gui::style::CmdUiMonoFont>>),
) {
    let Some(snapshot) = snapshot else {
        return;
    };
    let yard = focused_yard(&focus, &ids, &snapshot.0);
    for (e, s) in &sections {
        if Some(s.0) != yard {
            commands.entity(e).despawn();
        }
    }
    let Some(yard) = yard else {
        return;
    };
    let body = yard_lines(&snapshot.0, yard);
    if sections.iter().any(|(_, s)| s.0 == yard) {
        for mut t in &mut texts {
            if t.0 != body {
                t.0.clone_from(&body);
            }
        }
        return;
    }
    let (Ok(card), (Some(palette), Some(mono))) = (card.single(), style) else {
        return;
    };
    let kit = UiKit::new(&palette, mono.0.clone());
    commands.entity(card).with_children(|root| {
        root.spawn((
            YardCardSection(yard),
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(GAP),
                margin: UiRect::top(Val::Px(6.0)),
                ..default()
            },
        ))
        .with_children(|sec| {
            spawn_section_header(sec, &kit, "FREIGHT YARD");
            sec.spawn((YardCardText, Text::new(body), kit.text_font(FONT_SMALL), TextColor(palette.bevy_primary_text())));
            spawn_button(sec, &kit, "Build train", UiButton::new(UiButtonVariant::Normal), YardBuildTrainButton);
        });
    });
}

/// "Build train": one order request for the focused yard (the order sim checks the materials).
pub fn yard_build_train_button_system(
    buttons: Query<&DeprecatedInteraction, (Changed<DeprecatedInteraction>, With<YardBuildTrainButton>)>,
    sections: Query<&YardCardSection>,
    snapshot: Option<Res<LogisticsOrdersSnapshotRes>>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
) {
    if !buttons.iter().any(|i| *i == DeprecatedInteraction::Pressed) {
        return;
    }
    let (Some(snapshot), Ok(section)) = (snapshot, sections.single()) else {
        return;
    };
    let Some(owner) = snapshot.0.site(section.0).map(|s| s.owner) else {
        return;
    };
    let rail = &crate::infrastructure::road_tier_table().rail;
    requests.write(LogisticsOrderRequest(LogisticsOrderCommand::BuildTrain {
        owner,
        home: section.0,
        wagons: rail.build_wagons,
        wagon_capacity: rail.wagon_capacity,
        speed_mps: rail.speed_mps(),
        cost: rail.train_cost().into_iter().map(|(t, n)| (CargoKind::new(t), n)).collect(),
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yard_section_systems_have_disjoint_queries() {
        use crate::gui::style::widgets::test_support::init_ok;
        init_ok(sync_yard_card_section_system);
        init_ok(yard_build_train_button_system);
    }
}
