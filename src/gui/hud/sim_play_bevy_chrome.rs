//! Bevy replacements for the play-session cards that still painted with egui.
//!
//! They sit in the existing shell: repair rows in the bottom context tray, the
//! plant card and power hover as small satellites under the status line, the
//! crosshair on the pointer. None of them write the placement grid.

use bevy::prelude::*;

use crate::construction::{
    PowerLineDamageBook, PowerRepairQueue, POWER_REPAIR_PARTS_PER_SEGMENT,
};
use crate::engine::states::BaseState;
use crate::gui::hud::panel_state::HudPanelState;
use crate::gui::hud::plant_focus_card::PlantFocusCardSnapshot;
use crate::gui::hud::power_node_hover::PowerNodeHoverState;
use crate::gui::hud::simulation_pointer_gate::SimulationMapPointerGate;
use crate::gui::hud::simulation_shell_phase2::{
    ContextTrayState, ContextTrayTab, CONTEXT_TRAY_BODY_H_PX,
};
use crate::gui::panel_presence::PanelPresence;
use crate::gui::UiPalette;

#[derive(Component)]
pub struct SimPlayPlantCard;

#[derive(Component)]
pub struct SimPlayPlantCardText;

#[derive(Component)]
pub struct SimPlayPowerHover;

#[derive(Component)]
pub struct SimPlayPowerHoverText;

#[derive(Component)]
pub struct SimPlayCrosshair;

#[derive(Component)]
pub struct ContextTrayPowerRepairList;

#[derive(Component)]
pub(crate) struct PowerRepairQueueAllButton;

#[derive(Component)]
pub(crate) struct PowerRepairCancel(u64);

#[derive(Component)]
pub(crate) struct PowerRepairPriority {
    id: u64,
    delta: i16,
}

#[derive(Default)]
pub(crate) struct RepairListStamp {
    key: u64,
}

pub(crate) fn sync_sim_play_plant_card(
    base: Res<State<BaseState>>,
    card: Res<PlantFocusCardSnapshot>,
    hover: Res<PowerNodeHoverState>,
    mut root: Query<&mut Visibility, With<SimPlayPlantCard>>,
    mut text: Query<&mut Text, With<SimPlayPlantCardText>>,
) {
    let show = matches!(base.get(), BaseState::Simulation) && card.visible && !hover.card_visible;
    for mut vis in &mut root {
        *vis = if show { Visibility::Visible } else { Visibility::Hidden };
    }
    if !show {
        return;
    }
    let status = match card.status {
        crate::gui::hud::plant_focus_card::PlantOperationalStatus::Operational => "Operational",
        crate::gui::hud::plant_focus_card::PlantOperationalStatus::Scram => "SCRAM",
        crate::gui::hud::plant_focus_card::PlantOperationalStatus::Meltdown => "Meltdown",
    };
    let diesel = match card.diesel {
        crate::gui::hud::plant_focus_card::DieselState::Off => "Off",
        crate::gui::hud::plant_focus_card::DieselState::Starting => "Starting",
        crate::gui::hud::plant_focus_card::DieselState::Running => "Running",
        crate::gui::hud::plant_focus_card::DieselState::Failed => "Failed",
    };
    let offsite = if card.offsite_connected { "Connected" } else { "Islanded" };
    let minutes = card
        .minutes_to_next_phase
        .map(|m| format!("\n~{m} min to next phase"))
        .unwrap_or_default();
    let line = format!(
        "{}\nStatus: {status} · Offsite: {offsite}\nDiesels: {diesel}\nDiesel fuel: {:.0}%\nCore heat: {:.0}%{minutes}",
        card.plant_name,
        card.diesel_fuel * 100.0,
        card.core_heat * 100.0,
    );
    for mut t in &mut text {
        *t = Text::new(line.clone());
    }
}

pub(crate) fn sync_sim_play_power_hover(
    base: Res<State<BaseState>>,
    hover: Res<PowerNodeHoverState>,
    gate: Res<SimulationMapPointerGate>,
    mut presence: ResMut<PanelPresence>,
    mut root: Query<(&mut Visibility, &mut Node), With<SimPlayPowerHover>>,
    mut text: Query<&mut Text, With<SimPlayPowerHoverText>>,
) {
    presence.power_hover_root = !root.is_empty();
    let show = matches!(base.get(), BaseState::Simulation)
        && hover.card_visible
        && hover.card.is_some()
        && !(gate.chrome_blocks_pre_egui && !gate.in_play_area);
    for (mut vis, mut node) in &mut root {
        *vis = if show { Visibility::Visible } else { Visibility::Hidden };
        if show {
            node.left = Val::Px(gate.cursor.x + 18.0);
            node.top = Val::Px(gate.cursor.y + 18.0);
        }
    }
    let Some(card) = hover.card.as_ref().filter(|_| show) else {
        return;
    };
    let links = card.links_line.as_deref().unwrap_or("—");
    let yard = card.yard_line.as_deref().unwrap_or("");
    let line = format!(
        "{}  {}\n{}\nLoad {:.0}%\n{}\n{}\n{links}\n{yard}",
        card.title,
        card.voltage_label,
        card.status.label(),
        card.load_pct,
        card.capacity_line,
        card.feeds_line,
    );
    for mut t in &mut text {
        *t = Text::new(line.clone());
    }
}

pub(crate) fn sync_sim_play_crosshair(
    base: Res<State<BaseState>>,
    gate: Res<SimulationMapPointerGate>,
    strip: Option<Res<crate::construction::BuildStripState>>,
    tool: Option<Res<crate::construction::ActiveBuildTool>>,
    mut root: Query<(&mut Visibility, &mut Node), With<SimPlayCrosshair>>,
) {
    let placement = strip.as_ref().is_some_and(|s| {
        crate::gui::hud::simulation_pointer_gate::simulation_build_placement_active(
            s.as_ref(),
            tool.as_deref(),
        )
    });
    // Only the armed placement tool, on the map, with a real cursor. A missing
    // cursor used to pin this node at the screen origin.
    let show = matches!(base.get(), BaseState::Simulation)
        && gate.cursor_known
        && !gate.os_cursor_visible
        && gate.in_play_area
        && !gate.chrome_blocks
        && placement;
    for (mut vis, mut node) in &mut root {
        *vis = if show { Visibility::Visible } else { Visibility::Hidden };
        if show {
            node.left = Val::Px(gate.cursor.x - 7.0);
            node.top = Val::Px(gate.cursor.y - 7.0);
        }
    }
}

fn repair_visible(base: &BaseState, tray: &ContextTrayState) -> bool {
    matches!(base, BaseState::Simulation)
        && tray.active_tab == ContextTrayTab::Logistics
        && tray.panel_state.shows_content()
}

pub(crate) fn sync_context_tray_power_repair_list(
    mut commands: Commands,
    base: Res<State<BaseState>>,
    tray: Res<ContextTrayState>,
    queue: Option<Res<PowerRepairQueue>>,
    palette: Res<UiPalette>,
    mut presence: ResMut<PanelPresence>,
    fonts: Query<&TextFont, With<crate::gui::hud::simulation_shell_phase2::ContextTrayBodyLine>>,
    mut root: Query<(Entity, &mut Visibility, &mut Node), With<ContextTrayPowerRepairList>>,
    mut stamp: Local<RepairListStamp>,
) {
    presence.power_repair_list_root = !root.is_empty();
    let Some(queue) = queue.as_deref() else {
        return;
    };
    let show = repair_visible(base.get(), tray.as_ref());
    let body_h = if tray.panel_state == HudPanelState::Peek {
        36.0
    } else {
        (CONTEXT_TRAY_BODY_H_PX - 28.0).max(24.0)
    };
    let mut key = queue.jobs.len() as u64;
    for job in &queue.jobs {
        key = key
            .wrapping_mul(0x9E37_79B9)
            .wrapping_add(job.id)
            .wrapping_add(job.priority as u64)
            .wrapping_add(job.parts_have as u64);
    }
    key = key.wrapping_add(show as u64);
    let font = fonts
        .iter()
        .next()
        .cloned()
        .unwrap_or_else(|| TextFont::from_font_size(11.0));
    for (entity, mut vis, mut node) in &mut root {
        *vis = if show { Visibility::Visible } else { Visibility::Hidden };
        node.max_height = Val::Px(body_h);
        if !show || key == stamp.key {
            continue;
        }
        commands.entity(entity).despawn_related::<Children>();
        commands.entity(entity).with_children(|list| {
            spawn_repair_rows(list, queue, &palette, &font);
        });
    }
    stamp.key = key;
}

fn spawn_repair_rows(
    list: &mut bevy::ecs::relationship::RelatedSpawnerCommands<ChildOf>,
    queue: &PowerRepairQueue,
    palette: &UiPalette,
    font: &TextFont,
) {
    list.spawn((
        Button,
        Node {
            padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(palette.bevy_hud_panel_fill()),
        BorderColor::all(palette.bevy_border_subtle()),
        PowerRepairQueueAllButton,
    ))
    .with_children(|b| {
        b.spawn((
            Text::new(format!("Queue all damaged · {}", queue.jobs.len())),
            font.clone(),
            TextColor(palette.bevy_accent_gold()),
        ));
    });
    if queue.jobs.is_empty() {
        list.spawn((
            Text::new("○ No power repairs queued"),
            font.clone(),
            TextColor(palette.bevy_text_muted()),
        ));
        return;
    }
    for job in &queue.jobs {
        let parts = if job.parts_ready() {
            format!("parts {}/{}", job.parts_have, job.parts_need)
        } else {
            format!(
                "parts {}/{} · {}",
                job.parts_have,
                job.parts_need,
                job.blocked_reason.as_deref().unwrap_or("need parts")
            )
        };
        list.spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(format!("{}  {parts}  P{}", job.label, job.priority)),
                font.clone(),
                TextColor(palette.bevy_primary_text()),
            ));
            for (label, delta) in [("+", 5i16), ("-", -5)] {
                row.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(palette.bevy_hud_panel_fill()),
                    PowerRepairPriority { id: job.id, delta },
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        font.clone(),
                        TextColor(palette.bevy_secondary_text()),
                    ));
                });
            }
            row.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(palette.bevy_hud_panel_fill()),
                PowerRepairCancel(job.id),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("×"),
                    font.clone(),
                    TextColor(palette.bevy_secondary_text()),
                ));
            });
        });
    }
}

pub(crate) fn context_tray_power_repair_clicks(
    queue: Option<ResMut<PowerRepairQueue>>,
    book: Option<Res<PowerLineDamageBook>>,
    all: Query<&Interaction, (Changed<Interaction>, With<PowerRepairQueueAllButton>)>,
    cancel: Query<(&Interaction, &PowerRepairCancel), Changed<Interaction>>,
    priority: Query<(&Interaction, &PowerRepairPriority), Changed<Interaction>>,
) {
    let (Some(mut queue), Some(book)) = (queue, book) else {
        return;
    };
    if all.iter().any(|i| *i == Interaction::Pressed) {
        queue.queue_all_damaged(book.as_ref(), POWER_REPAIR_PARTS_PER_SEGMENT);
    }
    for (interaction, row) in &cancel {
        if *interaction == Interaction::Pressed {
            queue.cancel(row.0);
        }
    }
    for (interaction, row) in &priority {
        if *interaction == Interaction::Pressed {
            if let Some(job) = queue.jobs.iter_mut().find(|j| j.id == row.id) {
                let p = job.priority as i16 + row.delta;
                job.priority = p.clamp(1, 100) as u8;
            }
        }
    }
}
