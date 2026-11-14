//! D7 — play chrome for drone AOs: draw tool, quick place, pause/remove, map feedback.
//!
//! Intent only — enqueue via [`MissionIntentQueue`]; MissionBook is the sole sim writer.

use bevy::prelude::*;

use crate::combat::{
    doctrine_preferred_role, DroneAo, DroneMissionKind, DronePackage, DroneRole, FactionDroneDoctrine,
    Formation, MissionBook, MissionIntent, MissionIntentQueue,
};
use crate::construction::{ActiveBuildTool, BuildTool};
use crate::engine::states::BaseState;
use crate::gui::front_controls::{
    FrontDragBox, FrontMapCursor, FrontOrderFeedback, SelectedFormation, PLAYER_SLOT,
};
use crate::gui::options_keybindings_ui::KeybindingsUiState;

/// Screen-pixel threshold matching front box-select.
const DRAG_PX: f32 = 6.0;

fn tool_blocks(tool: &Option<Res<ActiveBuildTool>>) -> bool {
    tool.as_ref()
        .is_some_and(|t| !matches!(t.tool, BuildTool::None))
}

/// Soft tool: when armed, left-drag draws a Drone AO instead of selecting formations.
#[derive(Resource, Debug, Clone)]
pub struct DroneAoTool {
    pub armed: bool,
    pub mission: DroneMissionKind,
    pub role: DroneRole,
    pub drag_start: Option<Vec2>,
    pub drag_end: Option<Vec2>,
    pub next_ao_id: u64,
    pub last_ao_id: Option<u64>,
}

impl Default for DroneAoTool {
    fn default() -> Self {
        Self {
            armed: false,
            mission: DroneMissionKind::Hunt,
            role: DroneRole::FpvStrike,
            drag_start: None,
            drag_end: None,
            next_ao_id: 0,
            last_ao_id: None,
        }
    }
}

impl DroneAoTool {
    fn alloc_id(&mut self) -> u64 {
        self.next_ao_id = self.next_ao_id.wrapping_add(1).max(1);
        self.next_ao_id
    }

    #[must_use]
    pub fn drag_rect(&self) -> Option<(Vec2, Vec2)> {
        let (a, b) = (self.drag_start?, self.drag_end?);
        Some((a.min(b), a.max(b)))
    }

    fn clear_drag(&mut self) {
        self.drag_start = None;
        self.drag_end = None;
    }
}

fn cycle_mission(m: DroneMissionKind) -> DroneMissionKind {
    match m {
        DroneMissionKind::Hunt => DroneMissionKind::Recon,
        DroneMissionKind::Recon => DroneMissionKind::Strike,
        DroneMissionKind::Strike => DroneMissionKind::CuasScreen,
        DroneMissionKind::CuasScreen => DroneMissionKind::SwarmSaturation,
        DroneMissionKind::SwarmSaturation => DroneMissionKind::Hunt,
    }
}

fn cycle_role(r: DroneRole) -> DroneRole {
    match r {
        DroneRole::FpvStrike => DroneRole::ReconLoiter,
        DroneRole::ReconLoiter => DroneRole::AttritCruise,
        DroneRole::AttritCruise => DroneRole::MedAttack,
        DroneRole::MedAttack => DroneRole::Interceptor,
        DroneRole::Interceptor => DroneRole::SwarmMember,
        DroneRole::SwarmMember => DroneRole::FpvStrike,
    }
}

fn mission_for_role(role: DroneRole) -> DroneMissionKind {
    match role {
        DroneRole::ReconLoiter => DroneMissionKind::Recon,
        DroneRole::Interceptor => DroneMissionKind::CuasScreen,
        DroneRole::AttritCruise => DroneMissionKind::Strike,
        DroneRole::SwarmMember => DroneMissionKind::SwarmSaturation,
        _ => DroneMissionKind::Hunt,
    }
}

fn tiles_from_world_rect(lo: Vec2, hi: Vec2) -> (IVec2, IVec2) {
    let min = IVec2::new(lo.x.floor() as i32, lo.y.floor() as i32);
    let max = IVec2::new(hi.x.ceil() as i32 - 1, hi.y.ceil() as i32 - 1);
    (min.min(max), min.max(max))
}

/// Hotkeys: arm draw tool (`U`), Shift+U quick AO on selection, cycle mission/role, pause/remove.
pub fn drone_mission_hotkey_system(
    keys: Res<ButtonInput<KeyCode>>,
    capture: Option<Res<KeybindingsUiState>>,
    tool: Option<Res<ActiveBuildTool>>,
    base: Option<Res<State<BaseState>>>,
    selected: Res<SelectedFormation>,
    doctrine: Res<FactionDroneDoctrine>,
    book: Res<MissionBook>,
    formations: Query<(Entity, &Formation, Option<&DronePackage>)>,
    mut queue: ResMut<MissionIntentQueue>,
    mut fb: ResMut<FrontOrderFeedback>,
    mut ao_tool: ResMut<DroneAoTool>,
) {
    let in_sim = base.as_deref().is_none_or(|b| matches!(b.get(), BaseState::Simulation));
    if !in_sim || capture.as_deref().is_some_and(|c| c.capture_slot.is_some()) || tool_blocks(&tool)
    {
        return;
    }

    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if keys.just_pressed(KeyCode::KeyU) {
        if shift {
            quick_ao_on_selection(
                &selected,
                &doctrine,
                &formations,
                &mut queue,
                &mut fb,
                &mut ao_tool,
            );
        } else {
            ao_tool.armed = !ao_tool.armed;
            ao_tool.clear_drag();
            if ao_tool.armed {
                fb.notify(format!(
                    "drone AO draw on — drag box ({:?}/{:?}); Esc cancels",
                    ao_tool.mission, ao_tool.role
                ));
            } else {
                fb.notify("drone AO draw off");
            }
        }
    }

    if keys.just_pressed(KeyCode::BracketLeft) {
        ao_tool.mission = cycle_mission(ao_tool.mission);
        fb.notify(format!("drone mission {:?}", ao_tool.mission));
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        ao_tool.role = cycle_role(ao_tool.role);
        ao_tool.mission = mission_for_role(ao_tool.role);
        fb.notify(format!("drone role {:?} → {:?}", ao_tool.role, ao_tool.mission));
    }

    if keys.just_pressed(KeyCode::Escape) && ao_tool.armed {
        ao_tool.armed = false;
        ao_tool.clear_drag();
        fb.notify("drone AO draw cancelled");
    }

    if keys.just_pressed(KeyCode::KeyP) {
        if let Some(id) = ao_tool.last_ao_id {
            let currently_paused = book.drone_aos.get(&id).is_some_and(|a| a.paused);
            let paused = !currently_paused;
            match queue.enqueue(MissionIntent::PauseDroneAo { id, paused }) {
                Ok(_) => fb.notify(format!(
                    "drone AO {id} {}",
                    if paused { "paused" } else { "resumed" }
                )),
                Err(_) => fb.notify("mission queue full"),
            }
        } else {
            fb.notify("no drone AO to pause");
        }
    }

    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        if let Some(id) = ao_tool.last_ao_id.take() {
            match queue.enqueue(MissionIntent::RemoveDroneAo { id }) {
                Ok(_) => fb.notify(format!("drone AO {id} removed")),
                Err(_) => {
                    ao_tool.last_ao_id = Some(id);
                    fb.notify("mission queue full");
                }
            }
        } else {
            fb.notify("no drone AO to remove");
        }
    }
}

fn quick_ao_on_selection(
    selected: &SelectedFormation,
    doctrine: &FactionDroneDoctrine,
    formations: &Query<(Entity, &Formation, Option<&DronePackage>)>,
    queue: &mut MissionIntentQueue,
    fb: &mut FrontOrderFeedback,
    ao_tool: &mut DroneAoTool,
) {
    let Some(primary) = selected.primary() else {
        fb.notify("select a formation to place drone AO");
        return;
    };
    let Ok((_, f, pkg)) = formations.get(primary) else {
        return;
    };
    if f.faction_slot != PLAYER_SLOT {
        fb.notify("select a friendly formation");
        return;
    }
    let role = pkg
        .and_then(|p| DroneRole::ALL.into_iter().find(|r| p.count(*r) > 0))
        .unwrap_or_else(|| doctrine_preferred_role(doctrine.get(f.faction_slot)));
    let mission = mission_for_role(role);
    let id = ao_tool.alloc_id();
    let r = 8;
    let ao = DroneAo {
        id,
        faction_slot: f.faction_slot,
        min: f.tile - IVec2::splat(r),
        max: f.tile + IVec2::splat(r),
        mission,
        role,
        priority: 1.0,
        weight: 1.0,
        paused: false,
    };
    match queue.enqueue(MissionIntent::UpsertDroneAo(ao)) {
        Ok(_) => {
            ao_tool.last_ao_id = Some(id);
            ao_tool.role = role;
            ao_tool.mission = mission;
            fb.notify(format!("drone AO {id} ({role:?})"));
        }
        Err(_) => fb.notify("mission queue full"),
    }
}

/// When the drone AO tool is armed, left-drag draws an AO AABB (steals map select).
pub fn drone_ao_draw_system(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<FrontMapCursor>,
    tool: Option<Res<ActiveBuildTool>>,
    base: Option<Res<State<BaseState>>>,
    selected: Res<SelectedFormation>,
    formations: Query<(Entity, &Formation, Option<&DronePackage>)>,
    mut queue: ResMut<MissionIntentQueue>,
    mut fb: ResMut<FrontOrderFeedback>,
    mut ao_tool: ResMut<DroneAoTool>,
    mut front_drag: ResMut<FrontDragBox>,
) {
    let in_sim = base.as_deref().is_none_or(|b| matches!(b.get(), BaseState::Simulation));
    if !in_sim || !ao_tool.armed || tool_blocks(&tool) {
        if !ao_tool.armed {
            ao_tool.clear_drag();
        }
        return;
    }

    // Keep front selection drag from capturing the same stroke.
    *front_drag = FrontDragBox::default();

    if ao_tool.drag_start.is_none() {
        if buttons.just_pressed(MouseButton::Left) && cursor.free {
            if let Some(at) = cursor.xy {
                ao_tool.drag_start = Some(at);
                ao_tool.drag_end = Some(at);
            }
        }
        return;
    }

    if let Some(at) = cursor.xy {
        ao_tool.drag_end = Some(at);
    }

    if buttons.just_released(MouseButton::Left) || !buttons.pressed(MouseButton::Left) {
        let start = ao_tool.drag_start.take();
        let end = ao_tool.drag_end.take();
        let Some(start) = start else { return };
        let end = end.unwrap_or(start);
        let px = cursor.px_per_tile.max(0.05);
        let (lo, hi) = (start.min(end), start.max(end));
        let is_box = (hi - lo).length() * px > DRAG_PX;
        let (min, max) = if is_box {
            tiles_from_world_rect(lo, hi)
        } else {
            // Click: radius-8 around cursor (or selected formation if any).
            let centre = selected
                .primary()
                .and_then(|e| formations.get(e).ok())
                .map(|(_, f, _)| f.tile)
                .unwrap_or_else(|| IVec2::new(start.x.floor() as i32, start.y.floor() as i32));
            (centre - IVec2::splat(8), centre + IVec2::splat(8))
        };

        let (role, mission, faction) = if let Some(primary) = selected.primary() {
            if let Ok((_, f, pkg)) = formations.get(primary) {
                let role = pkg
                    .and_then(|p| DroneRole::ALL.into_iter().find(|r| p.count(*r) > 0))
                    .unwrap_or(ao_tool.role);
                (role, mission_for_role(role), f.faction_slot)
            } else {
                (ao_tool.role, ao_tool.mission, PLAYER_SLOT)
            }
        } else {
            (ao_tool.role, ao_tool.mission, PLAYER_SLOT)
        };

        let id = ao_tool.alloc_id();
        let ao = DroneAo {
            id,
            faction_slot: faction,
            min,
            max: max.max(min),
            mission,
            role,
            priority: 1.0,
            weight: 1.0,
            paused: false,
        };
        match queue.enqueue(MissionIntent::UpsertDroneAo(ao)) {
            Ok(_) => {
                ao_tool.last_ao_id = Some(id);
                ao_tool.role = role;
                ao_tool.mission = mission;
                fb.notify(format!("drone AO {id} ({role:?})"));
            }
            Err(_) => fb.notify("mission queue full"),
        }
    }
}

/// Preview rect while dragging with the drone AO tool (front overlay reads this).
#[must_use]
pub fn drone_ao_preview_rect(tool: &DroneAoTool, px_per_tile: f32) -> Option<(Vec2, Vec2)> {
    let (a, b) = (tool.drag_start?, tool.drag_end?);
    ((b - a).length() * px_per_tile.max(0.05) > DRAG_PX).then(|| (a.min(b), a.max(b)))
}

/// Register D7 chrome resources (called from FrontControlsPlugin).
pub fn register_drone_mission_chrome(app: &mut App) {
    app.init_resource::<DroneAoTool>();
}

/// True when drone draw tool owns left-drag (front select must yield).
#[must_use]
pub fn drone_ao_tool_blocks_select(tool: Option<&DroneAoTool>) -> bool {
    tool.is_some_and(|t| t.armed)
}

/// Book AOs for overlay (player-readable outlines).
#[must_use]
pub fn player_drone_aos<'a>(book: &'a MissionBook) -> impl Iterator<Item = &'a DroneAo> + 'a {
    book.drone_aos
        .values()
        .filter(|a| a.faction_slot == PLAYER_SLOT)
}
