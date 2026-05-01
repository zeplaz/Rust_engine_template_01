// In-game HUD — site logistics (Workers & Resources / X4 style).
// G3B: prompts/matrix/gap_remediation/runbook/g3b_production_hud_steps_v1.md

use std::collections::HashMap;

use bevy::prelude::*;

use crate::gui::editor::world_gen_ui::ToggleWorldGenUiEvent;

use crate::entities::production::core::{
    resolve_logistics_focus_entity, storage_entities_for_focus, LogisticsSiteMember,
    LogisticsSiteRoot, ResourceConsumer, ResourceProducer, ResourceStorage,
    ResourceStorageCapacity, ResourceType,
};

use super::input_bindings::InputBindings;
use super::logistics_focus::{HudAggregateSettings, HudLogisticsFocus};
use super::options_keybindings_ui::KeybindingsUiState;
use super::DiagnosticsUiState;
use super::FactionToolsState;
use super::LogisticsTargetsPanelState;

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct ResourceDisplay;

#[derive(Component)]
struct LogisticsPickHookAttached;

#[derive(Component, Clone, Copy)]
enum HudToolbarAction {
    Diagnostics,
    FactionTools,
    LogisticsList,
    WorldGenerator,
    KeyBindings,
}

pub struct InGameHudPlugin;

impl Plugin for InGameHudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudLogisticsFocus>()
            .init_resource::<HudAggregateSettings>()
            .add_systems(Startup, spawn_hud)
            .add_systems(
                Update,
                (
                    attach_storage_picking_hooks,
                    cycle_logistics_focus_dev,
                    hud_toolbar_clicks,
                    update_site_logistics_hud,
                ),
            );
    }
}

fn spawn_hud(mut commands: Commands, bindings: Res<InputBindings>) {
    let hint = format!(
        "Site logistics — no focus ({} · {} · toolbar · click storage)",
        InputBindings::format_key(bindings.cycle_logistics_focus),
        InputBindings::format_key(bindings.toggle_logistics_targets_panel),
    );
    let diag_l = format!(
        "Diag ({})",
        InputBindings::format_key(bindings.toggle_diagnostics)
    );
    let fac_l = format!(
        "Faction ({})",
        InputBindings::format_key(bindings.toggle_faction_tools)
    );
    let log_l = format!(
        "Logi ({})",
        InputBindings::format_key(bindings.toggle_logistics_targets_panel)
    );
    let wor_l = format!(
        "World ({})",
        InputBindings::format_key(bindings.toggle_world_generator)
    );
    let key_l = format!(
        "Keys ({})",
        InputBindings::format_key(bindings.toggle_keybindings_options)
    );

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                left: Val::Px(8.0),
                padding: UiRect::all(Val::Px(6.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                max_width: Val::Px(420.0),
                ..default()
            },
            HudRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(4.0),
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                ))
                .with_children(|row| {
                    for (label, action) in [
                        (diag_l, HudToolbarAction::Diagnostics),
                        (fac_l, HudToolbarAction::FactionTools),
                        (log_l, HudToolbarAction::LogisticsList),
                        (wor_l, HudToolbarAction::WorldGenerator),
                        (key_l, HudToolbarAction::KeyBindings),
                    ] {
                        row.spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.18, 0.22, 0.32)),
                            action,
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new(label),
                                TextColor(Color::srgb(0.92, 0.94, 0.98)),
                            ));
                        });
                    }
                });
            parent.spawn((
                Node { ..default() },
                Text::new(hint),
                TextColor(Color::srgb(0.75, 0.85, 1.0)),
                ResourceDisplay,
            ));
        });
}

fn attach_storage_picking_hooks(
    mut commands: Commands,
    q: Query<Entity, (With<ResourceStorage>, Without<LogisticsPickHookAttached>)>,
) {
    for e in q.iter() {
        commands
            .entity(e)
            .insert((Pickable::default(), LogisticsPickHookAttached))
            .observe(on_logistics_storage_clicked);
    }
}

fn on_logistics_storage_clicked(
    mut click: On<Pointer<Click>>,
    mut focus: ResMut<HudLogisticsFocus>,
    roots: Query<(), With<LogisticsSiteRoot>>,
    members: Query<&LogisticsSiteMember>,
) {
    if click.event().event.button != PointerButton::Primary {
        return;
    }
    let entity = click.event().entity;
    let is_hub = roots.get(entity).is_ok();
    let member_of = members.get(entity).ok();
    let resolved = resolve_logistics_focus_entity(entity, member_of, is_hub);
    focus.tracked_entity = Some(resolved);
    click.propagate(false);
}

fn hud_toolbar_clicks(
    interactions: Query<
        (&Interaction, &HudToolbarAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut diag: ResMut<DiagnosticsUiState>,
    mut faction: ResMut<FactionToolsState>,
    mut logistics: ResMut<LogisticsTargetsPanelState>,
    mut keys_ui: ResMut<KeybindingsUiState>,
    mut worldgen_ev: MessageWriter<ToggleWorldGenUiEvent>,
) {
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            HudToolbarAction::Diagnostics => diag.visible = !diag.visible,
            HudToolbarAction::FactionTools => faction.visible = !faction.visible,
            HudToolbarAction::LogisticsList => logistics.visible = !logistics.visible,
            HudToolbarAction::WorldGenerator => {
                worldgen_ev.write(ToggleWorldGenUiEvent);
            }
            HudToolbarAction::KeyBindings => keys_ui.visible = !keys_ui.visible,
        }
    }
}

fn cycle_logistics_focus_dev(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut focus: ResMut<HudLogisticsFocus>,
    roots: Query<(), With<LogisticsSiteRoot>>,
    members: Query<&LogisticsSiteMember>,
    with_storage: Query<Entity, With<ResourceStorage>>,
) {
    if !keys.just_pressed(bindings.cycle_logistics_focus) {
        return;
    }
    let list: Vec<Entity> = with_storage.iter().collect();
    if list.is_empty() {
        focus.tracked_entity = None;
        return;
    }
    let next_raw = match focus.tracked_entity {
        Some(cur) => match list.iter().position(|e| *e == cur) {
            Some(i) => list[(i + 1) % list.len()],
            None => list[0],
        },
        None => list[0],
    };
    let is_hub = roots.get(next_raw).is_ok();
    let m = members.get(next_raw).ok();
    focus.tracked_entity = Some(resolve_logistics_focus_entity(next_raw, m, is_hub));
}

fn merge_amounts_and_caps(
    entities: &[Entity],
    storage_q: &Query<&ResourceStorage>,
    cap_q: &Query<&ResourceStorageCapacity>,
) -> (HashMap<ResourceType, f32>, HashMap<ResourceType, f32>) {
    let mut amounts: HashMap<ResourceType, f32> = HashMap::new();
    let mut caps: HashMap<ResourceType, f32> = HashMap::new();
    for &e in entities {
        if let Ok(s) = storage_q.get(e) {
            for (&ty, &amt) in &s.amounts {
                *amounts.entry(ty).or_insert(0.0) += amt;
            }
        }
        if let Ok(c) = cap_q.get(e) {
            for (&ty, &mx) in &c.max_amounts {
                if mx > 0.0 {
                    *caps.entry(ty).or_insert(0.0) += mx;
                }
            }
        }
    }
    (amounts, caps)
}

fn update_site_logistics_hud(
    time: Res<Time>,
    mut settings: ResMut<HudAggregateSettings>,
    bindings: Res<InputBindings>,
    focus: Res<HudLogisticsFocus>,
    roots: Query<(), With<LogisticsSiteRoot>>,
    storage_entity_q: Query<Entity, With<ResourceStorage>>,
    storage_q: Query<&ResourceStorage>,
    cap_q: Query<&ResourceStorageCapacity>,
    member_q: Query<(Entity, &LogisticsSiteMember)>,
    producer_q: Query<&ResourceProducer>,
    consumer_q: Query<&ResourceConsumer>,
    mut text_q: Query<&mut Text, With<ResourceDisplay>>,
) {
    settings.accumulator += time.delta_secs();
    if settings.accumulator < settings.summary_interval_secs {
        return;
    }
    settings.accumulator = 0.0;

    let summary = match focus.tracked_entity {
        None => format!(
            "Site logistics — no focus\n({} cycle · {} list · toolbar · primary-click pickable storage)",
            InputBindings::format_key(bindings.cycle_logistics_focus),
            InputBindings::format_key(bindings.toggle_logistics_targets_panel),
        ),
        Some(hub_or_single) => {
            let is_hub = roots.get(hub_or_single).is_ok();
            let involved = storage_entities_for_focus(
                hub_or_single,
                is_hub,
                &storage_entity_q,
                &member_q,
            );
            if involved.is_empty() {
                format!(
                    "Site logistics — {:?}\n(no ResourceStorage on this focus)",
                    hub_or_single
                )
            } else {
                let (amounts, caps) = merge_amounts_and_caps(&involved, &storage_q, &cap_q);
                let flow_src = if is_hub {
                    hub_or_single
                } else {
                    involved[0]
                };
                let producer = producer_q.get(flow_src).ok();
                let consumer = consumer_q.get(flow_src).ok();
                format_merged_site_panel(
                    hub_or_single,
                    is_hub,
                    &involved,
                    &amounts,
                    &caps,
                    producer,
                    consumer,
                )
            }
        }
    };

    for mut text in text_q.iter_mut() {
        *text = Text::new(summary.clone());
    }
}

pub fn resource_glyph(ty: ResourceType) -> char {
    match ty {
        ResourceType::Wood => 'W',
        ResourceType::Coal => 'K',
        ResourceType::Oil => 'O',
        ResourceType::RareEarth => 'R',
        ResourceType::Metal => 'M',
        ResourceType::Steel => 'S',
        ResourceType::Concrete => 'C',
        ResourceType::Fertilizer => 'F',
        ResourceType::Chemicals => 'H',
        ResourceType::Electronics => 'E',
        ResourceType::Energy => 'N',
        ResourceType::Fuel => 'u',
        ResourceType::Ammunition => 'A',
        ResourceType::WarSupply => 'G',
        ResourceType::Knowledge => 'Q',
        ResourceType::Labour => 'L',
        ResourceType::Food => 'f',
        ResourceType::Water => 'w',
        ResourceType::Paper => 'P',
        ResourceType::Electricity => 'X',
    }
}

fn ascii_bar(stock: f32, denom: f32, width: usize) -> String {
    if denom <= 0.0 || width == 0 {
        return ".".repeat(width);
    }
    let filled = ((stock / denom).clamp(0.0, 1.0) * width as f32).round() as usize;
    let filled = filled.min(width);
    format!("{}{}", "|".repeat(filled), ".".repeat(width - filled))
}

fn flow_suffix(
    ty: ResourceType,
    producer: Option<&ResourceProducer>,
    consumer: Option<&ResourceConsumer>,
) -> String {
    let mut prod = 0.0f32;
    let mut cons = 0.0f32;
    if let Some(p) = producer {
        if p.resource_type == ty {
            prod = p.production_rate * p.efficiency.clamp(0.0, 1.0);
        }
    }
    if let Some(c) = consumer {
        if let Some(r) = c.consumption_rates.get(&ty) {
            cons = *r;
        }
    }
    if prod > 0.001 || cons > 0.001 {
        format!(" +{:.1}/s −{:.1}/s", prod, cons)
    } else {
        String::new()
    }
}

fn format_merged_site_panel(
    focus: Entity,
    is_hub: bool,
    involved: &[Entity],
    amounts: &HashMap<ResourceType, f32>,
    caps: &HashMap<ResourceType, f32>,
    producer: Option<&ResourceProducer>,
    consumer: Option<&ResourceConsumer>,
) -> String {
    let row_max = amounts
        .values()
        .cloned()
        .fold(0.01f32, |a, b| a.max(b));

    let mut pairs: Vec<(ResourceType, f32)> = amounts
        .iter()
        .filter(|(_, v)| **v > 0.001)
        .map(|(k, v)| (*k, *v))
        .collect();
    pairs.sort_by(|a, b| format!("{:?}", a.0).cmp(&format!("{:?}", b.0)));

    let kind = if is_hub { "hub roll-up" } else { "storage" };
    let header = format!(
        "Site {:?} ({kind}) — {} storages\n",
        focus,
        involved.len()
    );
    if pairs.is_empty() {
        return format!("{header}(empty inventory)");
    }

    let lines: Vec<String> = pairs
        .into_iter()
        .map(|(ty, stock)| {
            let g = resource_glyph(ty);
            let cap = caps.get(&ty).copied().unwrap_or(0.0);
            let denom = if cap > 0.001 {
                cap
            } else {
                row_max
            };
            let bar = ascii_bar(stock, denom, 8);
            let flows = flow_suffix(ty, producer, consumer);
            let cap_hint = if cap > 0.001 {
                format!("/{:.0}", cap)
            } else {
                String::new()
            };
            format!(
                "[{}] {} {:>8.1}{}{}",
                g,
                bar,
                stock,
                cap_hint,
                if flows.is_empty() { String::new() } else { flows }
            )
        })
        .collect();

    format!("{}{}", header, lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_respects_capacity_denominator() {
        let b = ascii_bar(50.0, 100.0, 8);
        assert_eq!(b, "||||....");
    }

    #[test]
    fn flow_shows_producer_and_consumer() {
        use std::collections::HashMap;
        let p = ResourceProducer {
            resource_type: ResourceType::Wood,
            production_rate: 10.0,
            max_production_rate: 10.0,
            energy_consumption: 0.0,
            efficiency: 1.0,
        };
        let mut rates = HashMap::new();
        rates.insert(ResourceType::Wood, 3.0);
        let c = ResourceConsumer {
            resource_types: vec![ResourceType::Wood],
            consumption_rates: rates,
            required_amounts: HashMap::new(),
        };
        let s = flow_suffix(ResourceType::Wood, Some(&p), Some(&c));
        assert!(s.contains("10.0"));
        assert!(s.contains("3.0"));
    }
}
