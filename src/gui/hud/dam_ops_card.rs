//! DF-4 — dam operations: the dam section of the site card (multi-channel release → site
//! [`DamControl`]). Player controls for any dam in the game — not DamFlood-only.
//!
//! The section lives inside the site card (`plant_info_card`: one movable card per focused site,
//! no second overlapping card). Widgets (preset buttons, channel steppers + sliders, turbine unit
//! buttons, online check) and the keyboard shortcuts both send [`DamAction`]s; one system applies
//! them.
//!
//! Writes [`crate::construction::DamControl`] on the focused dam; construction mirrors
//! that into [`crate::systems::water::BarrierControlIntent`] → SurfaceWaterField →
//! PixelWorld / GPU field / foam jets.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::construction::{DamBody, DamControl};
use crate::gui::logistics_focus::HudLogisticsFocus;
use crate::gui::DeprecatedInteraction;
use crate::gui::style::widgets::{
    spawn_button, spawn_note, spawn_section_header, spawn_stepper, spawn_toggle, ui_slider_drag, ToggleState, UiButton, UiButtonVariant, UiKit, UiSlider, UiTooltip, FONT_SMALL, GAP,
};
use crate::strategic::{ConstructionSite, SiteArchetype};
use crate::systems::water::{HydroGenerationState, SurfaceWaterField, DAM_TURBINE_UNIT_COUNT};

#[derive(Resource, Debug, Clone)]
pub struct DamOpsCardState {
    pub visible: bool,
    /// Last drawn text — skip rebuild when unchanged (despawn/respawn every frame ate clicks).
    pub last_text: String,
    /// Turbine slot the ,/. keys adjust.
    pub focus_turbine: u8,
}

impl Default for DamOpsCardState {
    fn default() -> Self {
        Self {
            visible: false,
            last_text: String::new(),
            focus_turbine: 0,
        }
    }
}


/// Toggle with Alt+D. Opens only while map focus is a Dam; clears when focus leaves / empties.
pub fn toggle_dam_ops_card_system(
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<HudLogisticsFocus>,
    sites: Query<&ConstructionSite>,
    mut state: ResMut<DamOpsCardState>,
    mut last_focus: Local<Option<Entity>>,
) {
    if focus.tracked_entity != *last_focus {
        *last_focus = focus.tracked_entity;
        let on_dam = focus.tracked_entity.is_some_and(|e| {
            sites.get(e).is_ok_and(|s| s.archetype == SiteArchetype::Dam)
        });
        if on_dam {
            state.visible = true;
        } else if state.visible {
            state.visible = false;
            state.last_text.clear();
        }
    }
    if !(keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight)) {
        return;
    }
    if keys.just_pressed(KeyCode::KeyD) {
        state.visible = !state.visible;
        if !state.visible {
            state.last_text.clear();
        }
    }
}

#[must_use]
fn fmt_m3s(q: f32) -> String {
    if q >= 100.0 {
        format!("{q:.0} m³/s")
    } else if q >= 10.0 {
        format!("{q:.1} m³/s")
    } else {
        format!("{q:.2} m³/s")
    }
}

#[must_use]
fn fmt_storage_m3(v: f64) -> String {
    if v >= 1.0e6 {
        format!("{:.2} Mm³", v / 1.0e6)
    } else if v >= 1.0e3 {
        format!("{:.1} dam³", v / 1.0e3)
    } else {
        format!("{v:.0} m³")
    }
}

/// Operator-facing balance from live field + site control (production path).
fn dam_status_text(
    ctrl: &DamControl,
    body: Option<&DamBody>,
    focus_turbine: u8,
    field: Option<&SurfaceWaterField>,
    hydro: Option<&HydroGenerationState>,
) -> String {
    // River inflow of the map. (The rain falling on the whole map is not this reservoir's inflow.)
    let inflow = field.map(|f| f.inflow_m3s).unwrap_or(0.0);
    let q_out = field.map(|f| f.outlet_flow_m3s).unwrap_or(0.0);
    let q_mid = field.map(|f| f.mid_outlet_flow_m3s).unwrap_or(0.0);
    let q_spill = field.map(|f| f.spillway_flow_m3s).unwrap_or(0.0);
    let q_over = field.map(|f| f.overtopping_flow_m3s).unwrap_or(0.0);
    let q_turb = field
        .map(|f| f.turbine_flow_m3s)
        .unwrap_or_else(|| ctrl.turbines.iter().map(|t| t.effective_m3s()).sum());
    let q_breach = field.map(|f| f.breach_flow_m3s).unwrap_or(0.0);
    let release = q_out + q_mid + q_spill + q_over + q_turb + q_breach;
    let held_back = (inflow - release).max(0.0);
    let drawdown = (release - inflow).max(0.0);
    let stage = field.map(|f| f.pool.stage_m).unwrap_or(0.0);
    let bed = field.map(|f| f.pool.bed_min_m).unwrap_or(0.0);
    let crest = field
        .map(crate::systems::water::SurfaceWaterField::channel_crest_m)
        .unwrap_or_else(|| body.map(|b| b.crest_m).unwrap_or(0.0));
    let head = (stage - bed).max(0.0);
    let freeboard = if crest > 0.0 {
        (crest - stage).max(0.0)
    } else {
        0.0
    };
    let storage = field.map(|f| f.pool.storage_m3).unwrap_or(0.0);
    let pass_pct = if inflow > 1.0 {
        ((release / inflow) * 100.0).clamp(0.0, 999.0)
    } else {
        0.0
    };
    let mw = hydro.map(|h| h.power_mw).unwrap_or(0.0);
    let commanded_turb: f32 = ctrl.turbines.iter().map(|t| t.setpoint_m3s.max(0.0)).sum();
    let lost_turb = (commanded_turb - q_turb).max(0.0);

    let mut extras = String::new();
    if q_mid > 0.5 {
        extras.push_str(&format!(" + mid {}", fmt_m3s(q_mid)));
    }
    if q_turb > 0.5 {
        extras.push_str(&format!(" + turb {}", fmt_m3s(q_turb)));
    }
    if q_over > 0.5 {
        extras.push_str(&format!(" + OVERTOP {}", fmt_m3s(q_over)));
    }
    if q_breach > 0.5 {
        extras.push_str(&format!(" + breach {}", fmt_m3s(q_breach)));
    }

    let hazard = if q_over > 0.5 {
        "⚠ OVERTOPPING — crest sheet eroding structure; open releases or lose the dam\n"
            .to_string()
    } else if crest > 0.0 && freeboard < 2.0 && held_back > 1.0 {
        format!("⚠ Freeboard only {freeboard:.1} m — rain/inflow filling; release soon\n")
    } else if lost_turb > 1.0 {
        format!(
            "⚠ Turbine loss {} — augment with mid/low outlets (preset 5)\n",
            fmt_m3s(lost_turb)
        )
    } else {
        String::new()
    };

    let balance = if held_back > 1.0 {
        format!(
            "Held vs natural: {} held  (would pass {} without dam)\n\
             Reservoir:       {}  · stage {:.1} m (head {:.1} m)\n\
             {}\
             {}",
            fmt_m3s(held_back),
            fmt_m3s(inflow),
            fmt_storage_m3(storage),
            stage,
            head,
            if crest > 0.0 {
                format!("Crest / freeboard: {crest:.1} m · {freeboard:.1} m freeboard\n")
            } else {
                String::new()
            },
            hazard
        )
    } else if drawdown > 1.0 {
        format!(
            "Held vs natural: releasing {} more than inflow\n\
             Reservoir:       {}  · stage {:.1} m (head {:.1} m)\n\
             {}",
            fmt_m3s(drawdown),
            fmt_storage_m3(storage),
            stage,
            head,
            hazard
        )
    } else {
        format!(
            "Held vs natural: near pass-through (≈ natural river)\n\
             Reservoir:       {}  · stage {:.1} m (head {:.1} m)\n\
             {}",
            fmt_storage_m3(storage),
            stage,
            head,
            hazard
        )
    };

    let has_hydro = body.is_some_and(|b| b.hydro_site.is_some())
        || commanded_turb > 0.0
        || mw > 0.0;
    let mut turb_lines = String::new();
    if has_hydro {
        for (i, t) in ctrl.turbines.iter().enumerate() {
            let mark = if i as u8 == focus_turbine { '>' } else { ' ' };
            let status = if t.available01 < 0.05 {
                "TRIP"
            } else if t.available01 < 0.95 {
                "DERATE"
            } else {
                "ok"
            };
            turb_lines.push_str(&format!(
                "{mark}T{} set {:.0} m³/s [{status}]\n",
                i + 1,
                t.setpoint_m3s
            ));
        }
        turb_lines.push_str(&format!(
            "Hydro live:     {} → {:.1} MW\n",
            fmt_m3s(q_turb),
            mw
        ));
    } else {
        turb_lines.push_str("Turbines:       none (flood-control / reservoir dam)\n");
    }

    format!(
        "Natural inflow: {}  (river before the dam)\n\
         Release now:    {}  (low {} + spill {}{})\n\
         Pass-through:   {:.0}% of natural\n\
         {}\
         {}",
        fmt_m3s(inflow),
        fmt_m3s(release),
        fmt_m3s(q_out),
        fmt_m3s(q_spill),
        extras,
        pass_pct,
        balance,
        turb_lines.trim_end(),
    )
}

fn focused_dam<'a>(
    focus: &HudLogisticsFocus,
    dams: &'a mut Query<(Entity, &ConstructionSite, &DamBody, &mut DamControl)>,
) -> Option<(Entity, &'a DamBody, Mut<'a, DamControl>)> {
    let e = focus.tracked_entity?;
    let Ok((entity, site, body, ctrl)) = dams.get_mut(e) else {
        return None;
    };
    if site.archetype != SiteArchetype::Dam {
        return None;
    }
    Some((entity, body, ctrl))
}

fn sync_ctrl_aggregate(ctrl: &mut DamControl) {
    ctrl.turbine_setpoint_m3s = ctrl.turbines.iter().map(|t| t.effective_m3s()).sum();
}

/// Estimate mid/low open needed to pass ~`q_m3s` at current head (rough orifice inverse).
fn open_for_flow(q_m3s: f32, head_m: f32, area_m2: f32) -> f32 {
    let h = head_m.max(0.5);
    let area = area_m2.max(0.5);
    let full = 0.6 * area * (2.0 * 9.81 * h).sqrt();
    if full <= 1e-3 {
        return 1.0;
    }
    (q_m3s / full).clamp(0.0, 1.0)
}

// ───────────────────────── Controls: one action path for keys and widgets ─────────────────────────

/// A dam release channel the card adjusts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamChannel {
    Low,
    Mid,
    Spill,
    /// The selected turbine unit's setpoint.
    Turbine,
}

impl DamChannel {
    pub const ALL: [DamChannel; 4] = [DamChannel::Low, DamChannel::Mid, DamChannel::Spill, DamChannel::Turbine];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            DamChannel::Low => "Low outlet",
            DamChannel::Mid => "Mid outlet",
            DamChannel::Spill => "Spillway gates",
            DamChannel::Turbine => "Turbine setpoint",
        }
    }
}

/// Largest turbine setpoint a unit takes (m³/s).
pub const TURBINE_MAX_M3S: f32 = 120.0;

/// One operator command on the focused dam (keys and card widgets both send these).
#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub enum DamAction {
    /// 1 shut all · 2 low only · 3 pass · 4 flood release · 5 augment lost turbine flow.
    Preset(u8),
    /// Step a channel (fraction of its range, signed).
    Nudge(DamChannel, f32),
    /// Set a channel to a fraction of its range.
    Set(DamChannel, f32),
    SelectTurbine(u8),
    /// Trip / restore the selected turbine unit.
    ToggleTurbine,
}

/// Fraction (0..1) of a channel's range now.
#[must_use]
pub fn channel_fraction(ctrl: &DamControl, focus_turbine: u8, ch: DamChannel) -> f32 {
    match ch {
        DamChannel::Low => ctrl.outlet_open01,
        DamChannel::Mid => ctrl.mid_outlet_open01,
        DamChannel::Spill => ctrl.gate_open01,
        DamChannel::Turbine => {
            let ti = (focus_turbine as usize).min(DAM_TURBINE_UNIT_COUNT - 1);
            ctrl.turbines[ti].setpoint_m3s / TURBINE_MAX_M3S
        }
    }
    .clamp(0.0, 1.0)
}

fn set_channel(ctrl: &mut DamControl, focus_turbine: u8, ch: DamChannel, f: f32) {
    let f = f.clamp(0.0, 1.0);
    match ch {
        DamChannel::Low => ctrl.outlet_open01 = f,
        DamChannel::Mid => ctrl.mid_outlet_open01 = f,
        DamChannel::Spill => ctrl.gate_open01 = f,
        DamChannel::Turbine => {
            let ti = (focus_turbine as usize).min(DAM_TURBINE_UNIT_COUNT - 1);
            ctrl.turbines[ti].setpoint_m3s = (f * TURBINE_MAX_M3S).round();
        }
    }
}

/// Apply one action to a dam's control (head / mid-outlet area feed the augment preset).
pub fn apply_dam_action(ctrl: &mut DamControl, focus_turbine: &mut u8, action: DamAction, head_m: f32, mid_area_m2: f32) {
    match action {
        DamAction::Preset(1) => {
            ctrl.outlet_open01 = 0.0;
            ctrl.mid_outlet_open01 = 0.0;
            ctrl.gate_open01 = 0.0;
            for t in &mut ctrl.turbines {
                t.setpoint_m3s = 0.0;
            }
        }
        DamAction::Preset(2) => {
            ctrl.outlet_open01 = 0.15;
            ctrl.mid_outlet_open01 = 0.0;
            ctrl.gate_open01 = 0.0;
        }
        DamAction::Preset(3) => {
            ctrl.outlet_open01 = 0.45;
            ctrl.mid_outlet_open01 = 0.25;
            ctrl.gate_open01 = 0.15;
        }
        DamAction::Preset(4) => {
            ctrl.outlet_open01 = 1.0;
            ctrl.mid_outlet_open01 = 0.85;
            ctrl.gate_open01 = 0.85;
        }
        // Augment: cover lost turbine flow with the mid outlet (+ low if needed).
        DamAction::Preset(5) => {
            let commanded: f32 = ctrl.turbines.iter().map(|t| t.setpoint_m3s.max(0.0)).sum();
            let effective: f32 = ctrl.turbines.iter().map(|t| t.effective_m3s()).sum();
            let lost = (commanded - effective).max(0.0);
            if lost > 0.5 {
                let need = open_for_flow(lost, head_m, mid_area_m2);
                ctrl.mid_outlet_open01 = ctrl.mid_outlet_open01.max(need);
                if need > 0.95 {
                    ctrl.outlet_open01 = ctrl.outlet_open01.max(open_for_flow(lost * 0.5, head_m, mid_area_m2 * 1.2));
                }
            }
        }
        DamAction::Preset(_) => {}
        DamAction::Nudge(ch, d) => {
            let f = channel_fraction(ctrl, *focus_turbine, ch);
            set_channel(ctrl, *focus_turbine, ch, f + d);
        }
        DamAction::Set(ch, f) => set_channel(ctrl, *focus_turbine, ch, f),
        DamAction::SelectTurbine(i) => *focus_turbine = i.min(DAM_TURBINE_UNIT_COUNT as u8 - 1),
        DamAction::ToggleTurbine => {
            let ti = (*focus_turbine as usize).min(DAM_TURBINE_UNIT_COUNT - 1);
            ctrl.turbines[ti].available01 = if ctrl.turbines[ti].available01 > 0.5 { 0.0 } else { 1.0 };
        }
    }
    sync_ctrl_aggregate(ctrl);
}

/// Keyboard shortcuts (same actions as the card's widgets) while the dam section is open.
pub fn dam_keys_to_actions_system(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<DamOpsCardState>,
    mut out: MessageWriter<DamAction>,
) {
    if !state.visible {
        return;
    }
    let step = 0.05;
    let digits = [
        (KeyCode::Digit1, KeyCode::Numpad1, 1),
        (KeyCode::Digit2, KeyCode::Numpad2, 2),
        (KeyCode::Digit3, KeyCode::Numpad3, 3),
        (KeyCode::Digit4, KeyCode::Numpad4, 4),
        (KeyCode::Digit5, KeyCode::Numpad5, 5),
    ];
    for (a, b, n) in digits {
        if keys.just_pressed(a) || keys.just_pressed(b) {
            out.write(DamAction::Preset(n));
        }
    }
    let nudges = [
        (KeyCode::BracketLeft, DamChannel::Low, -step),
        (KeyCode::BracketRight, DamChannel::Low, step),
        (KeyCode::Semicolon, DamChannel::Mid, -step),
        (KeyCode::Quote, DamChannel::Mid, step),
        (KeyCode::Minus, DamChannel::Spill, -step),
        (KeyCode::Equal, DamChannel::Spill, step),
        (KeyCode::Comma, DamChannel::Turbine, -5.0 / TURBINE_MAX_M3S),
        (KeyCode::Period, DamChannel::Turbine, 5.0 / TURBINE_MAX_M3S),
    ];
    for (k, ch, d) in nudges {
        if keys.just_pressed(k) {
            out.write(DamAction::Nudge(ch, d));
        }
    }
    if keys.just_pressed(KeyCode::Tab) {
        out.write(DamAction::SelectTurbine((state.focus_turbine + 1) % DAM_TURBINE_UNIT_COUNT as u8));
    }
    if keys.just_pressed(KeyCode::Slash) {
        out.write(DamAction::ToggleTurbine);
    }
}

/// Apply queued actions to the focused dam's [`DamControl`] (production authority) and mirror the
/// scenario intent.
pub fn apply_dam_actions_system(
    mut actions: MessageReader<DamAction>,
    mut state: ResMut<DamOpsCardState>,
    focus: Res<HudLogisticsFocus>,
    mut dams: Query<(Entity, &ConstructionSite, &DamBody, &mut DamControl)>,
    field: Option<Res<SurfaceWaterField>>,
    mut scenario_intent: Option<ResMut<crate::scenario::dam_flood::DamControlIntent>>,
) {
    if actions.is_empty() {
        return;
    }
    let Some((_, _body, mut ctrl)) = focused_dam(&focus, &mut dams) else {
        actions.clear();
        return;
    };
    let head = field.as_ref().map(|f| (f.pool.stage_m - f.pool.bed_min_m).max(0.5)).unwrap_or(10.0);
    let mid_area = field
        .as_ref()
        .map(|f| {
            if f.hydraulics.mid_outlet_area_m2 > 0.0 {
                f.hydraulics.mid_outlet_area_m2
            } else {
                f.hydraulics.outlet_area_m2 * 0.55
            }
        })
        .unwrap_or(2.0);
    let mut focus_turbine = state.focus_turbine;
    for a in actions.read() {
        apply_dam_action(&mut ctrl, &mut focus_turbine, *a, head, mid_area);
    }
    if state.focus_turbine != focus_turbine {
        state.focus_turbine = focus_turbine;
    }
    if let Some(intent) = scenario_intent.as_mut() {
        intent.gate_open01 = ctrl.gate_open01;
        intent.outlet_open01 = ctrl.outlet_open01;
        intent.mid_outlet_open01 = ctrl.mid_outlet_open01;
        intent.turbines = ctrl.turbines;
        intent.focus_turbine = focus_turbine;
        intent.sync_turbine_aggregate();
    }
}

// ───────────────────────── The dam section of the site card ─────────────────────────

/// The dam section inside the site card (spawned while a dam is focused).
#[derive(Component)]
pub struct DamCardSection;

#[derive(Component)]
pub struct DamStatusText;

#[derive(Component, Clone, Copy)]
pub struct DamPresetButton(pub u8);

#[derive(Component, Clone, Copy)]
pub struct DamChannelStep(pub DamChannel, pub f32);

#[derive(Component, Clone, Copy)]
pub struct DamChannelReadout(pub DamChannel);

#[derive(Component, Clone, Copy)]
pub struct DamChannelSlider(pub DamChannel);

#[derive(Component, Clone, Copy)]
pub struct DamTurbineButton(pub u8);

#[derive(Component)]
pub struct DamTurbineOnlineToggle;

const PRESETS: [(u8, &str); 5] = [(1, "Shut"), (2, "Low"), (3, "Pass"), (4, "Flood"), (5, "Augment")];

fn readout(ctrl: &DamControl, focus_turbine: u8, ch: DamChannel) -> String {
    let f = channel_fraction(ctrl, focus_turbine, ch);
    match ch {
        DamChannel::Turbine => format!("{:.0} m³/s", f * TURBINE_MAX_M3S),
        _ => format!("{:.0} %", f * 100.0),
    }
}

/// Spawn / remove the dam section inside the site card as dam focus comes and goes.
#[allow(clippy::too_many_arguments)]
pub fn sync_dam_card_section_system(
    mut commands: Commands,
    state: Res<DamOpsCardState>,
    focus: Res<HudLogisticsFocus>,
    mut dams: Query<(Entity, &ConstructionSite, &DamBody, &mut DamControl)>,
    card: Query<Entity, With<crate::gui::hud::plant_info_card::FacilityInfoCardRoot>>,
    sections: Query<Entity, With<DamCardSection>>,
    palette: Option<Res<crate::gui::style::UiPalette>>,
    mono: Option<Res<crate::gui::style::CmdUiMonoFont>>,
) {
    let want = state.visible && focused_dam(&focus, &mut dams).is_some();
    if !want {
        for e in &sections {
            commands.entity(e).despawn();
        }
        return;
    }
    if !sections.is_empty() {
        return;
    }
    let (Ok(card), Some(palette), Some(mono)) = (card.single(), palette, mono) else {
        return;
    };
    let Some((_, _, ctrl)) = focused_dam(&focus, &mut dams) else {
        return;
    };
    let kit = UiKit::new(&palette, mono.0.clone());
    let ft = state.focus_turbine;
    commands.entity(card).with_children(|root| {
        root.spawn((
            DamCardSection,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(GAP),
                margin: UiRect::top(Val::Px(6.0)),
                padding: UiRect::top(Val::Px(6.0)),
                border: UiRect::top(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(palette.bevy_border_subtle()),
        ))
        .with_children(|sec| {
            spawn_section_header(sec, &kit, "DAM OPERATIONS");
            sec.spawn((DamStatusText, Text::new(""), kit.text_font(FONT_SMALL), TextColor(palette.bevy_primary_text())));
            spawn_section_header(sec, &kit, "Release presets");
            sec.spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(GAP), row_gap: Val::Px(GAP), ..default() })
                .with_children(|row| {
                    for (n, label) in PRESETS {
                        spawn_button(row, &kit, label, UiButton::new(UiButtonVariant::Normal), (DamPresetButton(n), UiTooltip(preset_tip(n).into())));
                    }
                });
            spawn_section_header(sec, &kit, "Channels");
            for ch in [DamChannel::Low, DamChannel::Mid, DamChannel::Spill] {
                spawn_stepper(
                    sec,
                    &kit,
                    ch.label(),
                    DamChannelStep(ch, -0.05),
                    DamChannelStep(ch, 0.05),
                    DamChannelReadout(ch),
                    &readout(&ctrl, ft, ch),
                    DamChannelSlider(ch),
                    channel_fraction(&ctrl, ft, ch),
                );
            }
            spawn_section_header(sec, &kit, "Turbines");
            sec.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(GAP), ..default() })
                .with_children(|row| {
                    for i in 0..DAM_TURBINE_UNIT_COUNT as u8 {
                        let mut b = UiButton::new(UiButtonVariant::Normal);
                        b.selected = i == ft;
                        spawn_button(row, &kit, &format!("T{}", i + 1), b, DamTurbineButton(i));
                    }
                });
            spawn_stepper(
                sec,
                &kit,
                DamChannel::Turbine.label(),
                DamChannelStep(DamChannel::Turbine, -5.0 / TURBINE_MAX_M3S),
                DamChannelStep(DamChannel::Turbine, 5.0 / TURBINE_MAX_M3S),
                DamChannelReadout(DamChannel::Turbine),
                &readout(&ctrl, ft, DamChannel::Turbine),
                DamChannelSlider(DamChannel::Turbine),
                channel_fraction(&ctrl, ft, DamChannel::Turbine),
            );
            let online = ctrl.turbines[(ft as usize).min(DAM_TURBINE_UNIT_COUNT - 1)].available01 > 0.5;
            spawn_toggle(sec, &kit, "Selected unit online", online, DamTurbineOnlineToggle);
            spawn_note(sec, &kit, "Keys: 1-5 presets · [ ] low · ; ' mid · - = spill · Tab unit · , . setpoint · / trip", true, ());
        });
    });
}

fn preset_tip(n: u8) -> &'static str {
    match n {
        1 => "Close every outlet, gate and turbine (fill the reservoir)",
        2 => "Low outlet only (small steady release)",
        3 => "Pass roughly the natural river",
        4 => "Open everything (flood release)",
        _ => "Cover lost turbine flow with the mid / low outlets",
    }
}

/// Card widgets → [`DamAction`]s (buttons on press, sliders while held).
#[allow(clippy::type_complexity)]
pub fn dam_card_widgets_to_actions_system(
    presets: Query<(&DeprecatedInteraction, &DamPresetButton), Changed<DeprecatedInteraction>>,
    steps: Query<(&DeprecatedInteraction, &DamChannelStep), Changed<DeprecatedInteraction>>,
    turbines: Query<(&DeprecatedInteraction, &DamTurbineButton), Changed<DeprecatedInteraction>>,
    online: Query<&DeprecatedInteraction, (Changed<DeprecatedInteraction>, With<DamTurbineOnlineToggle>)>,
    sliders: Query<(&DamChannelSlider, &DeprecatedInteraction, &RelativeCursorPosition)>,
    mut out: MessageWriter<DamAction>,
) {
    for (i, p) in &presets {
        if *i == DeprecatedInteraction::Pressed {
            out.write(DamAction::Preset(p.0));
        }
    }
    for (i, s) in &steps {
        if *i == DeprecatedInteraction::Pressed {
            out.write(DamAction::Nudge(s.0, s.1));
        }
    }
    for (i, t) in &turbines {
        if *i == DeprecatedInteraction::Pressed {
            out.write(DamAction::SelectTurbine(t.0));
        }
    }
    for i in &online {
        if *i == DeprecatedInteraction::Pressed {
            out.write(DamAction::ToggleTurbine);
        }
    }
    for (s, i, cursor) in &sliders {
        if let Some(f) = ui_slider_drag(i, cursor) {
            out.write(DamAction::Set(s.0, f));
        }
    }
}

/// Mirror the dam's state into the section (status text, readouts, sliders, unit buttons).
#[allow(clippy::type_complexity)]
pub fn refresh_dam_card_section_system(
    mut state: ResMut<DamOpsCardState>,
    focus: Res<HudLogisticsFocus>,
    mut dams: Query<(Entity, &ConstructionSite, &DamBody, &mut DamControl)>,
    readings: (Option<Res<SurfaceWaterField>>, Option<Res<HydroGenerationState>>),
    mut status: Query<&mut Text, (With<DamStatusText>, Without<DamChannelReadout>)>,
    mut readouts: Query<(&DamChannelReadout, &mut Text), Without<DamStatusText>>,
    mut sliders: Query<(&DamChannelSlider, &mut UiSlider)>,
    mut toggles: (Query<(&DamTurbineButton, &mut UiButton)>, Query<&mut ToggleState, With<DamTurbineOnlineToggle>>),
) {
    let ft = state.focus_turbine;
    let Some((_, body, ctrl)) = focused_dam(&focus, &mut dams) else {
        return;
    };
    let (field, hydro) = readings;
    let text = dam_status_text(&ctrl, Some(body), ft, field.as_deref(), hydro.as_deref());
    if state.last_text != text {
        for mut t in &mut status {
            t.0.clone_from(&text);
        }
        state.last_text = text;
    }
    for (r, mut t) in &mut readouts {
        let s = readout(&ctrl, ft, r.0);
        if t.0 != s {
            t.0 = s;
        }
    }
    for (s, mut slider) in &mut sliders {
        let f = channel_fraction(&ctrl, ft, s.0);
        if (slider.fraction - f).abs() > 1e-4 {
            slider.fraction = f;
        }
    }
    let (buttons, onl) = &mut toggles;
    for (b, mut ui) in buttons.iter_mut() {
        let sel = b.0 == ft;
        if ui.selected != sel {
            ui.selected = sel;
        }
    }
    let online = ctrl.turbines[(ft as usize).min(DAM_TURBINE_UNIT_COUNT - 1)].available01 > 0.5;
    for mut t in onl.iter_mut() {
        if t.0 != online {
            t.0 = online;
        }
    }
}

pub struct DamOpsCardPlugin;

impl Plugin for DamOpsCardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DamOpsCardState>().add_message::<DamAction>().add_systems(
            Update,
            (
                toggle_dam_ops_card_system,
                dam_keys_to_actions_system,
                dam_card_widgets_to_actions_system,
                apply_dam_actions_system,
                sync_dam_card_section_system,
                refresh_dam_card_section_system,
            )
                .chain()
                .after(crate::gui::hud::plant_info_card::sync_facility_info_card_system)
                .run_if(crate::gui::ui_gates::in_simulation_or_editor),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::water::DamTurbineUnit;

    #[test]
    fn balance_text_names_inflow_release_and_held() {
        let ctrl = DamControl {
            gate_open01: 0.1,
            outlet_open01: 0.2,
            mid_outlet_open01: 0.1,
            turbine_setpoint_m3s: 40.0,
            turbines: {
                let mut t = [DamTurbineUnit::default(); DAM_TURBINE_UNIT_COUNT];
                t[0].setpoint_m3s = 40.0;
                t
            },
        };
        let mut field = SurfaceWaterField::default();
        field.wired = true;
        field.inflow_m3s = 220.0;
        field.outlet_flow_m3s = 40.0;
        field.spillway_flow_m3s = 10.0;
        field.pool.stage_m = 18.0;
        field.pool.bed_min_m = 2.0;
        field.pool.storage_m3 = 2.5e6;
        let text = dam_status_text(&ctrl, None, 0, Some(&field), None);
        assert!(text.contains("Natural inflow"));
        assert!(text.contains("Release now"));
        assert!(text.contains("Held vs natural") || text.contains("Pass-through"));
    }

    #[test]
    fn dam_card_systems_have_disjoint_queries() {
        use crate::gui::style::widgets::test_support::init_ok;
        init_ok(dam_keys_to_actions_system);
        init_ok(dam_card_widgets_to_actions_system);
        init_ok(apply_dam_actions_system);
        init_ok(sync_dam_card_section_system);
        init_ok(refresh_dam_card_section_system);
        init_ok(crate::gui::hud::panel_drag::place_hud_cards_system);
        init_ok(crate::gui::hud::plant_info_card::sync_facility_info_card_system);
        init_ok(crate::gui::hud::panel_drag::drag_hud_panels_system);
    }

    #[test]
    fn widgets_and_keys_share_one_action_path() {
        let mut ctrl = DamControl {
            gate_open01: 0.0,
            outlet_open01: 0.0,
            mid_outlet_open01: 0.0,
            turbine_setpoint_m3s: 0.0,
            turbines: [DamTurbineUnit::default(); DAM_TURBINE_UNIT_COUNT],
        };
        let mut ft = 0;
        apply_dam_action(&mut ctrl, &mut ft, DamAction::Preset(4), 10.0, 2.0);
        assert!(ctrl.outlet_open01 > 0.9 && ctrl.gate_open01 > 0.8, "flood preset opens up");
        apply_dam_action(&mut ctrl, &mut ft, DamAction::Set(DamChannel::Spill, 0.3), 10.0, 2.0);
        assert!((ctrl.gate_open01 - 0.3).abs() < 1e-5, "slider sets the spillway");
        apply_dam_action(&mut ctrl, &mut ft, DamAction::Nudge(DamChannel::Spill, -0.5), 10.0, 2.0);
        assert_eq!(ctrl.gate_open01, 0.0, "steps clamp at shut");
        apply_dam_action(&mut ctrl, &mut ft, DamAction::SelectTurbine(1), 10.0, 2.0);
        apply_dam_action(&mut ctrl, &mut ft, DamAction::Set(DamChannel::Turbine, 0.5), 10.0, 2.0);
        assert_eq!(ft, 1);
        assert_eq!(ctrl.turbines[1].setpoint_m3s, TURBINE_MAX_M3S * 0.5);
        assert_eq!(ctrl.turbines[0].setpoint_m3s, 0.0, "only the selected unit moves");
        apply_dam_action(&mut ctrl, &mut ft, DamAction::ToggleTurbine, 10.0, 2.0);
        assert!(ctrl.turbines[1].available01 < 0.5, "trip");
        apply_dam_action(&mut ctrl, &mut ft, DamAction::Preset(1), 10.0, 2.0);
        assert!(ctrl.turbines.iter().all(|t| t.setpoint_m3s == 0.0), "shut stops every unit");
    }
}
