//! **Options → Key bindings** Bevy window (`InputBindings` defaults: F1 opens this panel).

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use super::input_bindings::InputBindings;
use super::panel_presence::PanelPresence;
use super::style::{CmdUiMonoFont, UiPalette};

#[derive(Resource, Debug, Clone, Default)]
pub struct KeybindingsUiState {
    pub visible: bool,
    pub capture_slot: Option<BindingSlot>,
    pub last_io_message: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingSlot {
    ToggleOptions,
    Diagnostics,
    AiExplainability,
    PressureComposer,
    FactionTools,
    CycleLogistics,
    LogisticsTargets,
    ScenarioScriptPanel,
    StrategicStripCompact,
    CommandLeftStack,
    StrategicOverlayCongestion,
    StrategicOverlayEw,
    CycleBuildPlanningTool,
    ConfirmBuildPlacement,
    WorldGenerator,
    AgentPermissions,
    EguiUiScale,
    SimulationPause,
    StartGameplayRecording,
    StopGameplayRecording,
    CancelCapture,
    MapPanNorth,
    MapPanSouth,
    MapPanWest,
    MapPanEast,
    MapPanFast,
    MapMouseGrip,
    MapZoomIn,
    MapZoomOut,
    MapRotateCcw,
    MapRotateCw,
    MapToggleEdgeScroll,
    MapRecenterWorld,
    MapResetZoom,
    MapFrameWorld,
    MapCycleCameraMode,
}

#[derive(Component)]
pub struct KeybindingsBevyRoot;

#[derive(Component)]
struct KeybindingsRowButton(BindingSlot);

#[derive(Component)]
struct KeybindingsBodyStamp;

#[derive(Default)]
struct KeybindingsRebuildStamp {
    key: u64,
}

pub struct KeybindingsOptionsPlugin;

impl Plugin for KeybindingsOptionsPlugin {
    fn build(&self, app: &mut App) {
        let bindings =
            InputBindings::try_load_from_ron_path(&InputBindings::default_input_bindings_ron_path())
                .unwrap_or_default();
        app.insert_resource(bindings)
            .init_resource::<KeybindingsUiState>()
            .add_systems(
                Update,
                (
                    toggle_keybindings_options_window,
                    apply_binding_capture,
                    sync_keybindings_bevy_panel,
                    keybindings_row_clicks,
                )
                    .chain(),
            );
    }
}

fn toggle_keybindings_options_window(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut state: ResMut<KeybindingsUiState>,
) {
    if state.capture_slot.is_some() {
        return;
    }
    if keys.just_pressed(bindings.toggle_keybindings_options) {
        state.visible = !state.visible;
    }
}

fn apply_binding_capture(
    keys: Res<ButtonInput<KeyCode>>,
    mut bindings: ResMut<InputBindings>,
    mut state: ResMut<KeybindingsUiState>,
) {
    let Some(slot) = state.capture_slot else {
        return;
    };

    let cancel = bindings.cancel_keybinding_capture;
    if keys.just_pressed(cancel) {
        state.capture_slot = None;
        return;
    }

    for k in keys.get_just_pressed() {
        let k = *k;
        if k == cancel {
            continue;
        }
        assign_binding(&mut bindings, slot, k);
        state.capture_slot = None;
        state.last_io_message = Some(format!("bound {} → {}", slot_label(slot), InputBindings::format_key(k)));
        return;
    }
}

fn assign_binding(bindings: &mut InputBindings, slot: BindingSlot, k: KeyCode) {
    match slot {
        BindingSlot::ToggleOptions => bindings.toggle_keybindings_options = k,
        BindingSlot::Diagnostics => bindings.toggle_diagnostics = k,
        BindingSlot::AiExplainability => bindings.toggle_ai_explainability = k,
        BindingSlot::PressureComposer => bindings.toggle_pressure_composer = k,
        BindingSlot::FactionTools => bindings.toggle_faction_tools = k,
        BindingSlot::CycleLogistics => bindings.cycle_logistics_focus = k,
        BindingSlot::LogisticsTargets => bindings.toggle_logistics_targets_panel = k,
        BindingSlot::ScenarioScriptPanel => bindings.toggle_scenario_script_panel = k,
        BindingSlot::StrategicStripCompact => bindings.toggle_strategic_hud_strip_compact = k,
        BindingSlot::CommandLeftStack => bindings.toggle_command_left_stack = k,
        BindingSlot::StrategicOverlayCongestion => {
            bindings.toggle_strategic_overlay_routing_congestion = k
        }
        BindingSlot::StrategicOverlayEw => bindings.toggle_strategic_overlay_ew_denial = k,
        BindingSlot::CycleBuildPlanningTool => bindings.cycle_build_planning_tool = k,
        BindingSlot::ConfirmBuildPlacement => bindings.confirm_build_placement = k,
        BindingSlot::WorldGenerator => bindings.toggle_world_generator = k,
        BindingSlot::AgentPermissions => bindings.toggle_agent_permissions = k,
        BindingSlot::EguiUiScale => bindings.toggle_egui_ui_scale = k,
        BindingSlot::SimulationPause => bindings.toggle_simulation_pause = k,
        BindingSlot::StartGameplayRecording => bindings.start_gameplay_recording = k,
        BindingSlot::StopGameplayRecording => bindings.stop_gameplay_recording = k,
        BindingSlot::CancelCapture => bindings.cancel_keybinding_capture = k,
        BindingSlot::MapPanNorth => bindings.map_pan_north = k,
        BindingSlot::MapPanSouth => bindings.map_pan_south = k,
        BindingSlot::MapPanWest => bindings.map_pan_west = k,
        BindingSlot::MapPanEast => bindings.map_pan_east = k,
        BindingSlot::MapPanFast => bindings.map_pan_fast_modifier = k,
        BindingSlot::MapMouseGrip => bindings.map_mouse_grip = k,
        BindingSlot::MapZoomIn => bindings.map_zoom_in = k,
        BindingSlot::MapZoomOut => bindings.map_zoom_out = k,
        BindingSlot::MapRotateCcw => bindings.map_rotate_ccw = k,
        BindingSlot::MapRotateCw => bindings.map_rotate_cw = k,
        BindingSlot::MapToggleEdgeScroll => bindings.map_toggle_edge_scroll = k,
        BindingSlot::MapRecenterWorld => bindings.map_recenter_world = k,
        BindingSlot::MapResetZoom => bindings.map_reset_zoom = k,
        BindingSlot::MapFrameWorld => bindings.map_frame_world = k,
        BindingSlot::MapCycleCameraMode => bindings.map_cycle_camera_mode = k,
    }
}

fn slot_label(slot: BindingSlot) -> &'static str {
    match slot {
        BindingSlot::ToggleOptions => "Options / keybindings",
        BindingSlot::Diagnostics => "Diagnostics (F3)",
        BindingSlot::AiExplainability => "AI explainability",
        BindingSlot::PressureComposer => "Pressure composer",
        BindingSlot::FactionTools => "Faction tools",
        BindingSlot::CycleLogistics => "Cycle logistics focus",
        BindingSlot::LogisticsTargets => "Logistics targets",
        BindingSlot::ScenarioScriptPanel => "Scenario script",
        BindingSlot::StrategicStripCompact => "Strategic strip compact",
        BindingSlot::CommandLeftStack => "Command left stack",
        BindingSlot::StrategicOverlayCongestion => "Overlay congestion",
        BindingSlot::StrategicOverlayEw => "Overlay EW",
        BindingSlot::CycleBuildPlanningTool => "Cycle build tool",
        BindingSlot::ConfirmBuildPlacement => "Confirm placement",
        BindingSlot::WorldGenerator => "World generator",
        BindingSlot::AgentPermissions => "Agent permissions",
        BindingSlot::EguiUiScale => "UI scale",
        BindingSlot::SimulationPause => "Simulation pause",
        BindingSlot::StartGameplayRecording => "Start capture",
        BindingSlot::StopGameplayRecording => "Stop capture",
        BindingSlot::CancelCapture => "Cancel capture / rebind",
        BindingSlot::MapPanNorth => "Map pan N",
        BindingSlot::MapPanSouth => "Map pan S",
        BindingSlot::MapPanWest => "Map pan W",
        BindingSlot::MapPanEast => "Map pan E",
        BindingSlot::MapPanFast => "Map pan fast",
        BindingSlot::MapMouseGrip => "Map mouse grip",
        BindingSlot::MapZoomIn => "Map zoom in",
        BindingSlot::MapZoomOut => "Map zoom out",
        BindingSlot::MapRotateCcw => "Map rotate CCW",
        BindingSlot::MapRotateCw => "Map rotate CW",
        BindingSlot::MapToggleEdgeScroll => "Map edge scroll",
        BindingSlot::MapRecenterWorld => "Map recenter",
        BindingSlot::MapResetZoom => "Map reset zoom",
        BindingSlot::MapFrameWorld => "Map frame world",
        BindingSlot::MapCycleCameraMode => "Map camera mode",
    }
}

fn slot_key(bindings: &InputBindings, slot: BindingSlot) -> KeyCode {
    match slot {
        BindingSlot::ToggleOptions => bindings.toggle_keybindings_options,
        BindingSlot::Diagnostics => bindings.toggle_diagnostics,
        BindingSlot::AiExplainability => bindings.toggle_ai_explainability,
        BindingSlot::PressureComposer => bindings.toggle_pressure_composer,
        BindingSlot::FactionTools => bindings.toggle_faction_tools,
        BindingSlot::CycleLogistics => bindings.cycle_logistics_focus,
        BindingSlot::LogisticsTargets => bindings.toggle_logistics_targets_panel,
        BindingSlot::ScenarioScriptPanel => bindings.toggle_scenario_script_panel,
        BindingSlot::StrategicStripCompact => bindings.toggle_strategic_hud_strip_compact,
        BindingSlot::CommandLeftStack => bindings.toggle_command_left_stack,
        BindingSlot::StrategicOverlayCongestion => {
            bindings.toggle_strategic_overlay_routing_congestion
        }
        BindingSlot::StrategicOverlayEw => bindings.toggle_strategic_overlay_ew_denial,
        BindingSlot::CycleBuildPlanningTool => bindings.cycle_build_planning_tool,
        BindingSlot::ConfirmBuildPlacement => bindings.confirm_build_placement,
        BindingSlot::WorldGenerator => bindings.toggle_world_generator,
        BindingSlot::AgentPermissions => bindings.toggle_agent_permissions,
        BindingSlot::EguiUiScale => bindings.toggle_egui_ui_scale,
        BindingSlot::SimulationPause => bindings.toggle_simulation_pause,
        BindingSlot::StartGameplayRecording => bindings.start_gameplay_recording,
        BindingSlot::StopGameplayRecording => bindings.stop_gameplay_recording,
        BindingSlot::CancelCapture => bindings.cancel_keybinding_capture,
        BindingSlot::MapPanNorth => bindings.map_pan_north,
        BindingSlot::MapPanSouth => bindings.map_pan_south,
        BindingSlot::MapPanWest => bindings.map_pan_west,
        BindingSlot::MapPanEast => bindings.map_pan_east,
        BindingSlot::MapPanFast => bindings.map_pan_fast_modifier,
        BindingSlot::MapMouseGrip => bindings.map_mouse_grip,
        BindingSlot::MapZoomIn => bindings.map_zoom_in,
        BindingSlot::MapZoomOut => bindings.map_zoom_out,
        BindingSlot::MapRotateCcw => bindings.map_rotate_ccw,
        BindingSlot::MapRotateCw => bindings.map_rotate_cw,
        BindingSlot::MapToggleEdgeScroll => bindings.map_toggle_edge_scroll,
        BindingSlot::MapRecenterWorld => bindings.map_recenter_world,
        BindingSlot::MapResetZoom => bindings.map_reset_zoom,
        BindingSlot::MapFrameWorld => bindings.map_frame_world,
        BindingSlot::MapCycleCameraMode => bindings.map_cycle_camera_mode,
    }
}

const BINDING_SLOTS: &[BindingSlot] = &[
    BindingSlot::ToggleOptions,
    BindingSlot::Diagnostics,
    BindingSlot::PressureComposer,
    BindingSlot::FactionTools,
    BindingSlot::LogisticsTargets,
    BindingSlot::CycleLogistics,
    BindingSlot::SimulationPause,
    BindingSlot::WorldGenerator,
    BindingSlot::CommandLeftStack,
    BindingSlot::ConfirmBuildPlacement,
    BindingSlot::MapPanNorth,
    BindingSlot::MapPanSouth,
    BindingSlot::MapPanWest,
    BindingSlot::MapPanEast,
    BindingSlot::MapZoomIn,
    BindingSlot::MapZoomOut,
    BindingSlot::CancelCapture,
];

fn bindings_stamp(bindings: &InputBindings, state: &KeybindingsUiState) -> u64 {
    let mut key = state.visible as u64;
    key = key.wrapping_add(
        state
            .capture_slot
            .and_then(|s| BINDING_SLOTS.iter().position(|x| *x == s))
            .map(|i| i as u64 + 1)
            .unwrap_or(0),
    );
    for slot in BINDING_SLOTS {
        key = key
            .wrapping_mul(0x9E37_79B9)
            .wrapping_add(format!("{:?}", slot_key(bindings, *slot)).len() as u64);
        for b in format!("{:?}", slot_key(bindings, *slot)).bytes() {
            key = key.wrapping_mul(31).wrapping_add(b as u64);
        }
    }
    key
}

fn sync_keybindings_bevy_panel(
    mut commands: Commands,
    state: Res<KeybindingsUiState>,
    bindings: Res<InputBindings>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<KeybindingsBevyRoot>>,
    mut stamp: Local<KeybindingsRebuildStamp>,
) {
    presence.keybindings_root = !roots.is_empty();
    if !state.visible {
        for e in &roots {
            commands.entity(e).despawn();
        }
        presence.keybindings_root = false;
        stamp.key = 0;
        return;
    }

    let next = bindings_stamp(bindings.as_ref(), state.as_ref());
    if !roots.is_empty() && stamp.key == next {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }

    let tf = TextFont::from_font_size(12.0).with_font(mono.0.clone());
    let capture_msg = state
        .capture_slot
        .map(|s| format!("Press a key for «{}» (Esc cancels)", slot_label(s)))
        .or_else(|| state.last_io_message.clone())
        .unwrap_or_else(|| "Click a row to rebind".into());

    commands
        .spawn((
            KeybindingsBevyRoot,
            KeybindingsBodyStamp,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(56.0),
                right: Val::Px(16.0),
                width: Val::Px(340.0),
                max_height: Val::Px(480.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::ZERO,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_wire_magenta()),
            FocusPolicy::Block,
            ZIndex(970),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Key bindings"),
                tf.clone(),
                TextColor(palette.bevy_primary_text()),
            ));
            root.spawn((
                Text::new(capture_msg),
                tf.clone(),
                TextColor(palette.bevy_text_muted()),
            ));
            for slot in BINDING_SLOTS {
                let label = format!(
                    "{}  [{}]",
                    slot_label(*slot),
                    InputBindings::format_key(slot_key(bindings.as_ref(), *slot))
                );
                root.spawn((
                    KeybindingsRowButton(*slot),
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(palette.bevy_button_idle()),
                    BorderColor::all(palette.bevy_wire_magenta()),
                    FocusPolicy::Block,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(label),
                        tf.clone(),
                        TextColor(palette.bevy_primary_text()),
                    ));
                });
            }
        });
    stamp.key = next;
    presence.keybindings_root = true;
}

fn keybindings_row_clicks(
    mut state: ResMut<KeybindingsUiState>,
    interactions: Query<(&Interaction, &KeybindingsRowButton), Changed<Interaction>>,
) {
    for (interaction, row) in &interactions {
        if *interaction == Interaction::Pressed {
            state.capture_slot = Some(row.0);
            state.last_io_message = None;
        }
    }
}
