//! DF-4 — play UI dam control card (multi-channel release → site [`DamControl`]).
//! Player panel for any dam in the game — not DamFlood-only.
//!
//! Writes [`crate::construction::DamControl`] on the focused dam; construction mirrors
//! that into [`crate::systems::water::BarrierControlIntent`] → SurfaceWaterField →
//! PixelWorld / GPU field / foam jets.

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::construction::{DamBody, DamControl};
use crate::gui::logistics_focus::HudLogisticsFocus;
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

#[derive(Component)]
pub struct DamOpsCardRoot;

#[derive(Component)]
pub struct DamOpsCardLabel;

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
fn dam_ops_text(
    ctrl: &DamControl,
    body: Option<&DamBody>,
    focus_turbine: u8,
    field: Option<&SurfaceWaterField>,
    hydro: Option<&HydroGenerationState>,
) -> String {
    let inflow = field.map(|f| f.inflow_m3s + f.rain_runoff_m3s).unwrap_or(0.0);
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
        "DAM OPS — click dam, then use keys (no mouse knobs yet)\n\
         ─────────────────────────────\n\
         Natural inflow: {}  (river before the dam)\n\
         Release now:    {}  (low {} + spill {}{})\n\
         Pass-through:   {:.0}% of natural\n\
         {}\
         ─────────────────────────────\n\
         CHANNEL          KEYS          NOW\n\
         Low outlet       [  ]          {:.0}%\n\
         Mid outlet       ;  '          {:.0}%\n\
         Spillway gate    -  =          {:.0}%\n\
         {}\
         ── Presets (number row) ──\n\
         1 SHUT all  ·  2 LOW only  ·  3 PASS  ·  4 FLOOD  ·  5 AUGMENT lost turb\n\
         Tab select T#  ·  , / . setpoint  ·  / trip-restore\n\
         Alt+D hide card · water viz follows these setpoints",
        fmt_m3s(inflow),
        fmt_m3s(release),
        fmt_m3s(q_out),
        fmt_m3s(q_spill),
        extras,
        pass_pct,
        balance,
        ctrl.outlet_open01 * 100.0,
        ctrl.mid_outlet_open01 * 100.0,
        ctrl.gate_open01 * 100.0,
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

/// Draw dam ops readout when visible (Bevy UI). Rebuild only on text / visibility change.
pub fn rebuild_dam_ops_card_system(
    mut commands: Commands,
    mut state: ResMut<DamOpsCardState>,
    focus: Res<HudLogisticsFocus>,
    mut dams: Query<(Entity, &ConstructionSite, &DamBody, &mut DamControl)>,
    field: Option<Res<SurfaceWaterField>>,
    hydro: Option<Res<HydroGenerationState>>,
    existing: Query<Entity, With<DamOpsCardRoot>>,
    mut label: Query<&mut Text, With<DamOpsCardLabel>>,
) {
    if !state.visible {
        for e in &existing {
            commands.entity(e).despawn();
        }
        state.last_text.clear();
        return;
    }

    let Some((_, body, ctrl)) = focused_dam(&focus, &mut dams) else {
        for e in &existing {
            commands.entity(e).despawn();
        }
        state.last_text.clear();
        state.visible = false;
        return;
    };

    let text = dam_ops_text(
        ctrl.as_ref(),
        Some(body),
        state.focus_turbine,
        field.as_deref(),
        hydro.as_deref(),
    );

    if let Ok(mut t) = label.single_mut() {
        if state.last_text != text {
            t.0 = text.clone();
            state.last_text = text;
        }
        return;
    }

    for e in &existing {
        commands.entity(e).despawn();
    }
    state.last_text = text.clone();
    commands.spawn((
        DamOpsCardRoot,
        DamOpsCardLabel,
        Text::new(text),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(12.0),
            top: Val::Px(88.0),
            max_width: Val::Px(400.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.06, 0.09, 0.11, 0.88)),
        TextFont::from_font_size(13.0),
        TextColor(Color::srgb(0.86, 0.92, 0.90)),
        FocusPolicy::Pass,
        Pickable::IGNORE,
        ZIndex(960),
    ));
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

/// Adjust setpoints on the focused dam's [`DamControl`] (production authority).
pub fn dam_ops_card_adjust_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DamOpsCardState>,
    focus: Res<HudLogisticsFocus>,
    mut dams: Query<(Entity, &ConstructionSite, &DamBody, &mut DamControl)>,
    field: Option<Res<SurfaceWaterField>>,
    mut scenario_intent: Option<ResMut<crate::scenario::dam_flood::DamControlIntent>>,
) {
    if !state.visible {
        return;
    }
    let Some((_, _body, mut ctrl)) = focused_dam(&focus, &mut dams) else {
        return;
    };
    let step = 0.05_f32;
    let head = field
        .as_ref()
        .map(|f| (f.pool.stage_m - f.pool.bed_min_m).max(0.5))
        .unwrap_or(10.0);
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

    // Release presets.
    if keys.just_pressed(KeyCode::Digit1) || keys.just_pressed(KeyCode::Numpad1) {
        ctrl.outlet_open01 = 0.0;
        ctrl.mid_outlet_open01 = 0.0;
        ctrl.gate_open01 = 0.0;
        for t in &mut ctrl.turbines {
            t.setpoint_m3s = 0.0;
        }
    }
    if keys.just_pressed(KeyCode::Digit2) || keys.just_pressed(KeyCode::Numpad2) {
        ctrl.outlet_open01 = 0.15;
        ctrl.mid_outlet_open01 = 0.0;
        ctrl.gate_open01 = 0.0;
    }
    if keys.just_pressed(KeyCode::Digit3) || keys.just_pressed(KeyCode::Numpad3) {
        ctrl.outlet_open01 = 0.45;
        ctrl.mid_outlet_open01 = 0.25;
        ctrl.gate_open01 = 0.15;
    }
    if keys.just_pressed(KeyCode::Digit4) || keys.just_pressed(KeyCode::Numpad4) {
        ctrl.outlet_open01 = 1.0;
        ctrl.mid_outlet_open01 = 0.85;
        ctrl.gate_open01 = 0.85;
    }
    // Augment: cover lost turbine Q with mid (+ low if needed).
    if keys.just_pressed(KeyCode::Digit5) || keys.just_pressed(KeyCode::Numpad5) {
        let commanded: f32 = ctrl.turbines.iter().map(|t| t.setpoint_m3s.max(0.0)).sum();
        let effective: f32 = ctrl.turbines.iter().map(|t| t.effective_m3s()).sum();
        let lost = (commanded - effective).max(0.0);
        if lost > 0.5 {
            let need = open_for_flow(lost, head, mid_area);
            ctrl.mid_outlet_open01 = ctrl.mid_outlet_open01.max(need);
            if need > 0.95 {
                ctrl.outlet_open01 = ctrl
                    .outlet_open01
                    .max(open_for_flow(lost * 0.5, head, mid_area * 1.2));
            }
        }
    }

    if keys.just_pressed(KeyCode::BracketLeft) {
        ctrl.outlet_open01 = (ctrl.outlet_open01 - step).clamp(0.0, 1.0);
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        ctrl.outlet_open01 = (ctrl.outlet_open01 + step).clamp(0.0, 1.0);
    }
    if keys.just_pressed(KeyCode::Semicolon) {
        ctrl.mid_outlet_open01 = (ctrl.mid_outlet_open01 - step).clamp(0.0, 1.0);
    }
    if keys.just_pressed(KeyCode::Quote) {
        ctrl.mid_outlet_open01 = (ctrl.mid_outlet_open01 + step).clamp(0.0, 1.0);
    }
    if keys.just_pressed(KeyCode::Minus) {
        ctrl.gate_open01 = (ctrl.gate_open01 - step).clamp(0.0, 1.0);
    }
    if keys.just_pressed(KeyCode::Equal) {
        ctrl.gate_open01 = (ctrl.gate_open01 + step).clamp(0.0, 1.0);
    }
    if keys.just_pressed(KeyCode::Tab) {
        state.focus_turbine =
            (state.focus_turbine + 1) % DAM_TURBINE_UNIT_COUNT as u8;
    }
    let ti = (state.focus_turbine as usize).min(DAM_TURBINE_UNIT_COUNT - 1);
    if keys.just_pressed(KeyCode::Comma) {
        ctrl.turbines[ti].setpoint_m3s = (ctrl.turbines[ti].setpoint_m3s - 5.0).max(0.0);
    }
    if keys.just_pressed(KeyCode::Period) {
        ctrl.turbines[ti].setpoint_m3s = (ctrl.turbines[ti].setpoint_m3s + 5.0).min(120.0);
    }
    if keys.just_pressed(KeyCode::Slash) {
        // Operator isolate / restore selected unit (damage caps still apply via derate).
        if ctrl.turbines[ti].available01 > 0.5 {
            ctrl.turbines[ti].available01 = 0.0;
        } else {
            ctrl.turbines[ti].available01 = 1.0;
        }
    }
    sync_ctrl_aggregate(&mut ctrl);

    // Keep scenario intent mirrored so DamFlood books / steps stay consistent.
    if let Some(intent) = scenario_intent.as_mut() {
        intent.gate_open01 = ctrl.gate_open01;
        intent.outlet_open01 = ctrl.outlet_open01;
        intent.mid_outlet_open01 = ctrl.mid_outlet_open01;
        intent.turbines = ctrl.turbines;
        intent.focus_turbine = state.focus_turbine;
        intent.sync_turbine_aggregate();
    }
}

pub struct DamOpsCardPlugin;

impl Plugin for DamOpsCardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DamOpsCardState>().add_systems(
            Update,
            (
                toggle_dam_ops_card_system,
                dam_ops_card_adjust_system.after(toggle_dam_ops_card_system),
                rebuild_dam_ops_card_system.after(dam_ops_card_adjust_system),
            )
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
        let text = dam_ops_text(&ctrl, None, 0, Some(&field), None);
        assert!(text.contains("Natural inflow"));
        assert!(text.contains("Release now"));
        assert!(text.contains("Held vs natural") || text.contains("Pass-through"));
        assert!(text.contains("Low outlet"));
        assert!(text.contains("Mid outlet"));
        assert!(text.contains("Presets"));
    }
}
