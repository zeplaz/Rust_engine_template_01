//! Player controls for the front lane (tactical map): selection, orders, hotkeys, order feedback and
//! the selected-formation readout.
//!
//! Everything goes through the one funnel the AI also uses, [`DeploymentOrderQueue::enqueue`]; nothing
//! here writes sim state. Drawing lives in `render::world_map_front_mesh`, which reads the resources
//! below ([`SelectedFormation`], [`FrontDragBox`], [`FrontOrderFeedback`], [`FrontOverlaySettings`]).
//!
//! Mouse (TacticalFlat2D only; the map plane does not exist in PixelWorld3D, so the pointer is never
//! `free` there): left click selects one of the player's formations, shift-click toggles, left-drag draws
//! a box that selects every formation inside (shift extends), clicking empty ground clears; right click
//! orders the whole selection (enemy formation / enemy-held front cell = Attack, Threatened pressure
//! point of ours = Reinforce, else Move with the group spread around the target).
//!
//! Keys (rebindable in Options -> Key bindings, `InputBindings::front_*`): hold, fortify (the theater the
//! formation stands in, else the nearest of its faction), cycle selection, overlay and per-layer toggles.
//! Nothing fires while a key is being rebound, a build tool is armed, or (mouse) chrome owns the pointer.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::combat::{
    DeploymentKind, DeploymentOrder, DeploymentOrderId, DeploymentOrderQueue, DeploymentOutcomes, DeploymentReject,
    Formation, Posture, PressureKind, PressurePointBook, TheaterBook, TheaterId,
};
use crate::construction::{cursor_world_on_map, ActiveBuildTool, BuildTool};
use crate::engine::states::BaseState;
use crate::gui::hud::simulation_pointer_gate::{
    simulation_tool_pointer, PointerGateSet, SimulationMapPointerGate, SimulationToolPointer,
};
use crate::gui::input_bindings::InputBindings;
use crate::gui::options_keybindings_ui::KeybindingsUiState;
use crate::gui::style::widgets::{
    panel_bundle, set_text_if_changed, spawn_button, spawn_header, spawn_label_row, spawn_note, ui_button_fired, UiButton,
    UiButtonVariant, UiKit, UiTooltip,
};
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::gui::{MainWorldCameraOrthoTrace, MapCameraDesiredRes, SimulationMapViewport, WorldPresentationMode};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// The slot whose formations the local player commands.
pub const PLAYER_SLOT: u8 = 0;

/// A drag shorter than this many screen pixels is a click.
const DRAG_PX: f32 = 6.0;

/// Ordering label for systems that read the controls' output the same frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FrontControlsSet;

/// Which front overlay layers the map draws. All on by default.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrontOverlaySettings {
    pub fronts: bool,
    pub theaters: bool,
    pub pressure: bool,
    pub formations: bool,
    pub labels: bool,
}

impl Default for FrontOverlaySettings {
    fn default() -> Self {
        Self { fronts: true, theaters: true, pressure: true, formations: true, labels: true }
    }
}

impl FrontOverlaySettings {
    #[must_use]
    pub fn any(&self) -> bool {
        self.fronts || self.theaters || self.pressure || self.formations || self.labels
    }

    /// Everything off if anything is on, else everything on.
    pub fn toggle_all(&mut self) {
        let on = !self.any();
        *self = Self { fronts: on, theaters: on, pressure: on, formations: on, labels: on };
    }
}

/// The player's selected formations (sorted by entity, so order issue is deterministic).
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectedFormation(pub Vec<Entity>);

impl SelectedFormation {
    #[must_use]
    pub fn primary(&self) -> Option<Entity> {
        self.0.first().copied()
    }

    #[must_use]
    pub fn contains(&self, e: Entity) -> bool {
        self.0.contains(&e)
    }

    pub fn set(&mut self, mut v: Vec<Entity>) {
        v.sort();
        v.dedup();
        if self.0 != v {
            self.0 = v;
        }
    }

    pub fn toggle(&mut self, e: Entity) {
        let mut v = self.0.clone();
        if let Some(i) = v.iter().position(|x| *x == e) {
            v.remove(i);
        } else {
            v.push(e);
        }
        self.set(v);
    }
}

/// Cursor on the map plane, published each frame after the pointer gate.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct FrontMapCursor {
    /// Map-plane position in tile units (`x`, row).
    pub xy: Option<Vec2>,
    /// The gate lets the map see the pointer (not over chrome / panels).
    pub free: bool,
    /// Screen pixels per tile (symbol hit radius and the drag threshold derive from it).
    pub px_per_tile: f32,
}

/// An in-progress left-drag (map-plane tile coordinates).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct FrontDragBox {
    pub start: Option<Vec2>,
    pub end: Option<Vec2>,
    pub px_per_tile: f32,
}

impl FrontDragBox {
    /// The rectangle to draw, once the drag is long enough to be a box.
    #[must_use]
    pub fn rect(&self) -> Option<(Vec2, Vec2)> {
        let (a, b) = (self.start?, self.end?);
        ((b - a).length() * self.px_per_tile.max(0.05) > DRAG_PX).then(|| (a.min(b), a.max(b)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarkerState {
    /// Queued, the executor has not looked at it yet.
    Pending,
    Accepted,
    Rejected(DeploymentReject),
}

/// Brief marker for one issued order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrderMarker {
    pub id: DeploymentOrderId,
    pub kind: DeploymentKind,
    /// Where the order points (the target tile centre; the formation itself for Hold / Fortify).
    pub target: Vec2,
    /// Where the formation was when ordered.
    pub from: Vec2,
    pub state: MarkerState,
    pub age_s: f32,
}

impl OrderMarker {
    fn lifetime_s(&self) -> f32 {
        match self.state {
            MarkerState::Pending => 5.0,
            MarkerState::Accepted => 1.4,
            MarkerState::Rejected(_) => 3.5,
        }
    }
}

/// Short human text for a refusal.
#[must_use]
pub fn reject_text(r: DeploymentReject) -> &'static str {
    match r {
        DeploymentReject::NoSuchFormation => "formation is gone",
        DeploymentReject::NotOwner => "not your formation",
        DeploymentReject::Broken => "broken, cannot take orders",
        DeploymentReject::UnknownTheater => "no such theater",
        DeploymentReject::NothingToHold => "nothing of yours to hold there",
    }
}

/// Order markers and the last notice, for the map and the readout.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct FrontOrderFeedback {
    pub markers: Vec<OrderMarker>,
    /// `(text, age_s)`
    pub notice: Option<(String, f32)>,
}

const NOTICE_LIFETIME_S: f32 = 4.0;
const MAX_MARKERS: usize = 64;

impl FrontOrderFeedback {
    fn notify(&mut self, text: impl Into<String>) {
        self.notice = Some((text.into(), 0.0));
    }
}

/// Enqueue one order and start its marker. A refused enqueue (full queue) becomes a notice.
fn issue(queue: &mut DeploymentOrderQueue, fb: &mut FrontOrderFeedback, order: DeploymentOrder, target: Vec2, from: Vec2) {
    match queue.enqueue(order) {
        Ok(id) => {
            if fb.markers.len() >= MAX_MARKERS {
                fb.markers.remove(0);
            }
            fb.markers.push(OrderMarker { id, kind: order.kind, target, from, state: MarkerState::Pending, age_s: 0.0 });
        }
        Err(e) => fb.notify(format!("order not queued: {e:?}")),
    }
}

fn tile_center(t: IVec2) -> Vec2 {
    t.as_vec2() + Vec2::splat(0.5)
}

// ---------------------------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------------------------

/// What a right click on `target` means for a formation of `slot`.
#[must_use]
pub fn order_kind_for_target(
    slot: u8,
    target: IVec2,
    theaters: &TheaterBook,
    pressure: Option<&PressurePointBook>,
    hostile_formation_tiles: &[IVec2],
) -> DeploymentKind {
    let near = |a: IVec2, b: IVec2| (a - b).as_vec2().length() <= 1.5;
    if hostile_formation_tiles.iter().any(|t| near(*t, target)) {
        return DeploymentKind::Attack(target);
    }
    let enemy_front = theaters.theaters.iter().flat_map(|t| t.cells.iter()).any(|c| c.tile == target && c.owner != slot);
    if enemy_front {
        return DeploymentKind::Attack(target);
    }
    if let Some(p) = pressure.and_then(|b| {
        b.points
            .iter()
            .filter(|p| p.side == slot && p.kind == PressureKind::Threatened && near(p.tile, target))
            .min_by(|a, b| (a.tile - target).as_vec2().length().total_cmp(&(b.tile - target).as_vec2().length()))
    }) {
        return DeploymentKind::Reinforce { theater: p.theater, tile: p.tile };
    }
    DeploymentKind::Move(target)
}

/// Where the `i`-th of a group stands around a Move target (0 = the target itself).
#[must_use]
pub fn spread_offset(i: usize) -> IVec2 {
    const RING: [(i32, i32); 9] = [(0, 0), (1, 0), (0, 1), (-1, 0), (0, -1), (1, 1), (-1, 1), (-1, -1), (1, -1)];
    let (x, y) = RING[i % 9];
    IVec2::new(x, y) * (1 + 2 * (i / 9) as i32)
}

/// The theater a Fortify order should name for this formation: the one it stands in (area of
/// responsibility), else the nearest theater of its faction.
#[must_use]
pub fn theater_for_fortify(f: &Formation, theaters: &TheaterBook) -> Option<TheaterId> {
    let mine = || theaters.theaters.iter().filter(|t| t.pair.0 == f.faction_slot || t.pair.1 == f.faction_slot);
    mine()
        .find(|t| t.aor_contains(f.tile, theaters.depth_tiles))
        .or_else(|| {
            mine().min_by(|a, b| {
                let d = |t: &crate::combat::Theater| (t.centroid - f.tile.as_vec2()).length();
                d(a).total_cmp(&d(b)).then(a.id.cmp(&b.id))
            })
        })
        .map(|t| t.id)
}

/// One line for the readout: what the formation is doing.
#[must_use]
pub fn describe_order(f: &Formation) -> String {
    let at = |t: IVec2| format!("({}, {})", t.x, t.y);
    match (f.posture, f.target) {
        (Posture::Retreat, _) => "broken, falling back".to_string(),
        (Posture::Attack, Some(t)) => format!("attack {}", at(t)),
        (Posture::Move, Some(t)) if f.fortify => format!("moving to dig in at {}", at(t)),
        (Posture::Move, Some(t)) => format!("move to {}", at(t)),
        (_, Some(t)) => format!("heading to {}", at(t)),
        (_, None) if f.fortify && f.dug_in >= 1.0 => "dug in".to_string(),
        (_, None) if f.fortify => "digging in".to_string(),
        (_, None) if f.engaged => "holding, in contact".to_string(),
        (_, None) => "holding".to_string(),
    }
}

/// Player formations whose tile centre lies inside the box.
#[must_use]
pub fn formations_in_box(formations: &[(Entity, Formation)], lo: Vec2, hi: Vec2) -> Vec<Entity> {
    formations
        .iter()
        .filter(|(_, f)| f.faction_slot == PLAYER_SLOT)
        .filter(|(_, f)| {
            let c = tile_center(f.tile);
            c.x >= lo.x && c.x <= hi.x && c.y >= lo.y && c.y <= hi.y
        })
        .map(|(e, _)| *e)
        .collect()
}

fn own_sorted(formations: &Query<(Entity, &Formation)>) -> Vec<Entity> {
    let mut v: Vec<Entity> = formations.iter().filter(|(_, f)| f.faction_slot == PLAYER_SLOT).map(|(e, _)| e).collect();
    v.sort();
    v
}

// ---------------------------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------------------------

fn px_per_tile(trace: &MainWorldCameraOrthoTrace) -> f32 {
    if trace.fixed_width > 0.0 && trace.view_pixels.x > 0.0 {
        trace.view_pixels.x / trace.fixed_width
    } else {
        1.0
    }
}

/// Publish the map-plane cursor (after the pointer gate has decided who owns the pointer).
#[allow(clippy::too_many_arguments)]
pub fn update_front_map_cursor_system(
    window: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Option<Res<MapCameraDesiredRes>>,
    map_vp: Option<Res<SimulationMapViewport>>,
    params: Option<Res<WorldGenParams>>,
    gate: Option<Res<SimulationMapPointerGate>>,
    trace: Option<Res<MainWorldCameraOrthoTrace>>,
    mode: Option<Res<WorldPresentationMode>>,
    mut cursor: ResMut<FrontMapCursor>,
) {
    let mut next = FrontMapCursor::default();
    // PixelWorld3D has no flat map plane for tile picks: the pointer is never free there.
    if mode.as_deref().is_none_or(|m| m.is_tactical()) {
        next.px_per_tile = trace.as_deref().map_or(1.0, px_per_tile);
        next.free = gate.as_deref().is_some_and(|g| g.in_play_area)
            && simulation_tool_pointer(gate.as_deref()) == SimulationToolPointer::Free;
        if let (Ok(w), Some(desired), Some(map_vp), Some(params)) = (window.single(), desired, map_vp, params) {
            next.xy = cursor_world_on_map(w, authority.as_deref(), desired.as_ref(), map_vp.as_ref(), params.as_ref())
                .map(|v| Vec2::new(v.x, v.z));
        }
    }
    cursor.set_if_neq(next);
}

fn tool_blocks(tool: &Option<Res<ActiveBuildTool>>) -> bool {
    tool.as_deref().is_some_and(|t| t.tool != BuildTool::None)
}

fn shift_down(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
}

/// Selection, box-drag and right-click orders.
#[allow(clippy::too_many_arguments)]
pub fn front_mouse_system(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    cursor: Res<FrontMapCursor>,
    tool: Option<Res<ActiveBuildTool>>,
    theaters: Res<TheaterBook>,
    pressure: Res<PressurePointBook>,
    formations: Query<(Entity, &Formation)>,
    mut selected: ResMut<SelectedFormation>,
    mut drag: ResMut<FrontDragBox>,
    mut queue: ResMut<DeploymentOrderQueue>,
    mut fb: ResMut<FrontOrderFeedback>,
) {
    // Dead or captured formations leave the selection.
    let live: Vec<Entity> = selected
        .0
        .iter()
        .copied()
        .filter(|e| formations.get(*e).is_ok_and(|(_, f)| f.faction_slot == PLAYER_SLOT))
        .collect();
    selected.set(live);

    let blocked = tool_blocks(&tool);
    let shift = shift_down(&keys);

    // --- left: click / box ---
    if drag.start.is_none() {
        if buttons.just_pressed(MouseButton::Left) && cursor.free && !blocked {
            if let Some(at) = cursor.xy {
                *drag = FrontDragBox { start: Some(at), end: Some(at), px_per_tile: cursor.px_per_tile };
            }
        }
    } else {
        if let Some(at) = cursor.xy {
            drag.end = Some(at);
        }
        drag.px_per_tile = cursor.px_per_tile;
        if buttons.just_released(MouseButton::Left) || !buttons.pressed(MouseButton::Left) {
            let d = *drag;
            *drag = FrontDragBox::default();
            let Some(start) = d.start else { return };
            let all: Vec<(Entity, Formation)> = formations.iter().map(|(e, f)| (e, f.clone())).collect();
            if let Some((lo, hi)) = d.rect() {
                let mut v = formations_in_box(&all, lo, hi);
                if shift {
                    v.extend(selected.0.iter().copied());
                }
                selected.set(v);
            } else {
                let reach = (symbol_reach_tiles(d.px_per_tile)).max(0.8);
                let hit = all
                    .iter()
                    .filter(|(_, f)| f.faction_slot == PLAYER_SLOT)
                    .map(|(e, f)| (*e, (tile_center(f.tile) - start).length()))
                    .filter(|(_, dist)| *dist <= reach)
                    .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.to_bits().cmp(&b.0.to_bits())));
                match (hit, shift) {
                    (Some((e, _)), true) => selected.toggle(e),
                    (Some((e, _)), false) => selected.set(vec![e]),
                    (None, true) => {}
                    (None, false) => selected.set(Vec::new()),
                }
            }
        }
        return;
    }

    // --- right: order the selection ---
    if buttons.just_pressed(MouseButton::Right) && cursor.free && !blocked {
        let Some(at) = cursor.xy else { return };
        if selected.0.is_empty() {
            return;
        }
        let tile = IVec2::new(at.x.floor() as i32, at.y.floor() as i32);
        let all: Vec<(Entity, &Formation)> = formations.iter().collect();
        let mut group: Vec<(Entity, &Formation)> =
            selected.0.iter().filter_map(|e| all.iter().find(|(x, _)| x == e).copied()).collect();
        group.sort_by_key(|(e, _)| *e);
        for (i, (e, f)) in group.iter().enumerate() {
            let hostile: Vec<IVec2> =
                all.iter().filter(|(_, o)| o.faction_slot != f.faction_slot).map(|(_, o)| o.tile).collect();
            let mut kind = order_kind_for_target(f.faction_slot, tile, &theaters, Some(&pressure), &hostile);
            if let DeploymentKind::Move(t) = kind {
                kind = DeploymentKind::Move(t + spread_offset(i));
            }
            let target = match kind {
                DeploymentKind::Move(t) | DeploymentKind::Attack(t) => tile_center(t),
                DeploymentKind::Reinforce { tile, .. } => tile_center(tile),
                DeploymentKind::Hold | DeploymentKind::Fortify(_) => tile_center(f.tile),
            };
            issue(&mut queue, &mut fb, DeploymentOrder { formation: *e, kind, issued_by: f.faction_slot }, target, tile_center(f.tile));
        }
    }
}

/// Screen-sized symbol reach in tiles (matches the drawn symbol edge at this zoom).
#[must_use]
pub fn symbol_reach_tiles(px_per_tile: f32) -> f32 {
    (20.0 / px_per_tile.max(0.05)).clamp(0.9, 12.0) * 0.7
}

fn hold_selected(sel: &SelectedFormation, formations: &Query<(Entity, &Formation)>, queue: &mut DeploymentOrderQueue, fb: &mut FrontOrderFeedback) {
    if sel.0.is_empty() {
        fb.notify("nothing selected");
        return;
    }
    for e in &sel.0 {
        if let Ok((_, f)) = formations.get(*e) {
            let at = tile_center(f.tile);
            issue(queue, fb, DeploymentOrder { formation: *e, kind: DeploymentKind::Hold, issued_by: f.faction_slot }, at, at);
        }
    }
}

fn fortify_selected(
    sel: &SelectedFormation,
    formations: &Query<(Entity, &Formation)>,
    theaters: &TheaterBook,
    queue: &mut DeploymentOrderQueue,
    fb: &mut FrontOrderFeedback,
) {
    if sel.0.is_empty() {
        fb.notify("nothing selected");
        return;
    }
    for e in &sel.0 {
        let Ok((_, f)) = formations.get(*e) else { continue };
        let Some(theater) = theater_for_fortify(f, theaters) else {
            fb.notify("no front to dig in on yet");
            continue;
        };
        let at = tile_center(f.tile);
        issue(queue, fb, DeploymentOrder { formation: *e, kind: DeploymentKind::Fortify(theater), issued_by: f.faction_slot }, at, at);
    }
}

/// Hotkeys: hold, fortify, cycle selection, overlay and layer toggles.
#[allow(clippy::too_many_arguments)]
pub fn front_key_commands_system(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Option<Res<InputBindings>>,
    capture: Option<Res<KeybindingsUiState>>,
    tool: Option<Res<ActiveBuildTool>>,
    base: Option<Res<State<BaseState>>>,
    theaters: Res<TheaterBook>,
    formations: Query<(Entity, &Formation)>,
    mut selected: ResMut<SelectedFormation>,
    mut settings: ResMut<FrontOverlaySettings>,
    mut queue: ResMut<DeploymentOrderQueue>,
    mut fb: ResMut<FrontOrderFeedback>,
) {
    // Editor text fields and the rebind capture own the keyboard; the keys are play-mode controls.
    let in_sim = base.as_deref().is_none_or(|b| matches!(b.get(), BaseState::Simulation));
    if !in_sim || capture.as_deref().is_some_and(|c| c.capture_slot.is_some()) || tool_blocks(&tool) {
        return;
    }
    let b = bindings.as_deref().cloned().unwrap_or_default();
    let mut s = *settings;
    if keys.just_pressed(b.front_toggle_overlay) {
        s.toggle_all();
    }
    if keys.just_pressed(b.front_toggle_fronts) {
        s.fronts = !s.fronts;
    }
    if keys.just_pressed(b.front_toggle_theaters) {
        s.theaters = !s.theaters;
    }
    if keys.just_pressed(b.front_toggle_pressure) {
        s.pressure = !s.pressure;
    }
    if keys.just_pressed(b.front_toggle_formations) {
        s.formations = !s.formations;
    }
    if keys.just_pressed(b.front_toggle_labels) {
        s.labels = !s.labels;
    }
    settings.set_if_neq(s);

    if keys.just_pressed(b.front_cycle_selection) {
        let own = own_sorted(&formations);
        if own.is_empty() {
            fb.notify("no formations to select");
        } else {
            let next = match selected.primary().and_then(|p| own.iter().position(|e| *e == p)) {
                Some(i) => own[(i + 1) % own.len()],
                None => own[0],
            };
            selected.set(vec![next]);
        }
    }
    if keys.just_pressed(b.front_hold) {
        hold_selected(&selected, &formations, &mut queue, &mut fb);
    }
    if keys.just_pressed(b.front_fortify) {
        fortify_selected(&selected, &formations, &theaters, &mut queue, &mut fb);
    }
}

/// Resolve markers against the executor's outcomes and age them out.
pub fn update_order_feedback_system(
    time: Res<Time>,
    outcomes: Option<Res<DeploymentOutcomes>>,
    mut fb: ResMut<FrontOrderFeedback>,
) {
    if fb.markers.is_empty() && fb.notice.is_none() {
        return;
    }
    let dt = time.delta_secs();
    let mut next = fb.clone();
    for m in &mut next.markers {
        m.age_s += dt;
        if m.state == MarkerState::Pending {
            if let Some(r) = outcomes.as_deref().and_then(|o| o.recent.iter().rev().find(|r| r.id == m.id)) {
                m.state = match r.result {
                    Ok(()) => MarkerState::Accepted,
                    Err(e) => {
                        // Age restarts so the refusal is visible for its full lifetime.
                        m.age_s = 0.0;
                        MarkerState::Rejected(e)
                    }
                };
            }
        }
    }
    // Surface the newest refusal as a notice.
    if let Some(e) = next.markers.iter().rev().find_map(|m| match m.state {
        MarkerState::Rejected(e) if m.age_s <= 0.01 => Some(e),
        _ => None,
    }) {
        next.notice = Some((format!("order refused: {}", reject_text(e)), 0.0));
    }
    next.markers.retain(|m| m.age_s < m.lifetime_s());
    if let Some((_, age)) = &mut next.notice {
        *age += dt;
    }
    if next.notice.as_ref().is_some_and(|(_, age)| *age > NOTICE_LIFETIME_S) {
        next.notice = None;
    }
    fb.set_if_neq(next);
}

// ---------------------------------------------------------------------------------------------
// Selected-formation readout (Bevy UI)
// ---------------------------------------------------------------------------------------------

/// Root of the readout panel. Registered with the pointer gate so clicks on it never reach the map.
#[derive(Component)]
pub struct FrontSelectionPanelRoot;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Field {
    Title,
    Kind,
    Strength,
    Morale,
    Supply,
    Posture,
    DugIn,
    Order,
    Notice,
}

#[derive(Component)]
struct ReadoutValue(Field);

#[derive(Component, Clone, Copy)]
enum ReadoutButton {
    Hold,
    Fortify,
    Clear,
}

fn pct(v: f32) -> String {
    format!("{:.0}%", v.clamp(0.0, 1.0) * 100.0)
}

/// Texts of every readout field for the selection (`None` when there is nothing to show).
fn readout_texts(
    sel: &SelectedFormation,
    formations: &Query<(Entity, &Formation)>,
    notice: Option<&str>,
) -> Option<BTreeMap<Field, String>> {
    let primary = sel.primary().and_then(|e| formations.get(e).ok()).map(|(_, f)| f)?;
    let n = sel.0.len();
    let mut m = BTreeMap::new();
    m.insert(Field::Title, if n > 1 { format!("{n} formations (first shown)") } else { "Formation".to_string() });
    m.insert(Field::Kind, format!("{:?} · slot {}", primary.kind, primary.faction_slot));
    m.insert(Field::Strength, pct(primary.strength));
    m.insert(Field::Morale, pct(primary.morale));
    m.insert(Field::Supply, pct(primary.supply));
    m.insert(Field::Posture, format!("{:?}{}", primary.posture, if primary.engaged { " · in contact" } else { "" }));
    m.insert(Field::DugIn, pct(primary.dug_in));
    m.insert(Field::Order, describe_order(primary));
    m.insert(Field::Notice, notice.unwrap_or("").to_string());
    Some(m)
}

#[allow(clippy::too_many_arguments)]
pub fn sync_front_selection_panel(
    mut commands: Commands,
    base: Option<Res<State<BaseState>>>,
    mode: Option<Res<WorldPresentationMode>>,
    selected: Res<SelectedFormation>,
    fb: Res<FrontOrderFeedback>,
    formations: Query<(Entity, &Formation)>,
    palette: Option<Res<UiPalette>>,
    mono: Option<Res<CmdUiMonoFont>>,
    roots: Query<Entity, With<FrontSelectionPanelRoot>>,
    mut values: Query<(&ReadoutValue, &mut Text)>,
) {
    let tactical_sim = base.as_deref().is_none_or(|b| matches!(b.get(), BaseState::Simulation))
        && mode.as_deref().is_none_or(|m| m.is_tactical());
    let texts = tactical_sim
        .then(|| readout_texts(&selected, &formations, fb.notice.as_ref().map(|(t, _)| t.as_str())))
        .flatten();
    let Some(texts) = texts else {
        for e in &roots {
            commands.entity(e).despawn();
        }
        return;
    };
    if roots.is_empty() {
        let (Some(palette), Some(mono)) = (palette, mono) else { return };
        let kit = UiKit::new(&palette, mono.0.clone());
        let mut node = Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(16.0),
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-150.0)),
            width: Val::Px(300.0),
            ..default()
        };
        node.align_self = AlignSelf::FlexEnd;
        commands.spawn((FrontSelectionPanelRoot, panel_bundle(&palette, node), ZIndex(940))).with_children(|root| {
            spawn_header(root, &kit, "Formation", ReadoutValue(Field::Title), None::<ReadoutButton>);
            for (label, field) in [
                ("Kind", Field::Kind),
                ("Strength", Field::Strength),
                ("Morale", Field::Morale),
                ("Supply", Field::Supply),
                ("Posture", Field::Posture),
                ("Dug in", Field::DugIn),
                ("Order", Field::Order),
            ] {
                spawn_label_row(root, &kit, label, "-", ReadoutValue(field));
            }
            spawn_note(root, &kit, "", true, ReadoutValue(Field::Notice));
            root.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(4.0), ..default() }).with_children(|row| {
                for (label, tip, which) in [
                    ("Hold", "Hold position (hotkey in Options -> Key bindings)", ReadoutButton::Hold),
                    ("Dig in", "Fortify the theater this formation stands in", ReadoutButton::Fortify),
                    ("Clear", "Deselect", ReadoutButton::Clear),
                ] {
                    spawn_button(row, &kit, label, UiButton::new(UiButtonVariant::Normal), which)
                        .insert(UiTooltip(tip.to_string()));
                }
            });
        });
        return; // texts fill in next frame, once the nodes exist
    }
    for (v, mut t) in &mut values {
        if let Some(s) = texts.get(&v.0) {
            set_text_if_changed(&mut t, s);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn front_panel_buttons_system(
    interactions: Query<(&Interaction, &UiButton, &ReadoutButton), Changed<Interaction>>,
    theaters: Res<TheaterBook>,
    formations: Query<(Entity, &Formation)>,
    mut selected: ResMut<SelectedFormation>,
    mut queue: ResMut<DeploymentOrderQueue>,
    mut fb: ResMut<FrontOrderFeedback>,
) {
    for (interaction, button, which) in &interactions {
        if !ui_button_fired(interaction, button) {
            continue;
        }
        match which {
            ReadoutButton::Hold => hold_selected(&selected, &formations, &mut queue, &mut fb),
            ReadoutButton::Fortify => fortify_selected(&selected, &formations, &theaters, &mut queue, &mut fb),
            ReadoutButton::Clear => selected.set(Vec::new()),
        }
    }
}

pub struct FrontControlsPlugin;

impl Plugin for FrontControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrontOverlaySettings>()
            .init_resource::<SelectedFormation>()
            .init_resource::<FrontMapCursor>()
            .init_resource::<FrontDragBox>()
            .init_resource::<FrontOrderFeedback>()
            .init_resource::<DeploymentOrderQueue>()
            .init_resource::<TheaterBook>()
            .init_resource::<PressurePointBook>()
            .add_systems(
                Update,
                (
                    update_front_map_cursor_system,
                    front_mouse_system,
                    front_key_commands_system,
                    front_panel_buttons_system,
                    update_order_feedback_system,
                )
                    .chain()
                    .in_set(FrontControlsSet)
                    .after(PointerGateSet::Publish),
            )
            .add_systems(Update, sync_front_selection_panel.after(FrontControlsSet));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{OutcomeRecord, UnitKind};
    use crate::gui::style::widgets::test_support::init_ok;

    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<InputBindings>()
            .init_resource::<DeploymentOutcomes>()
            .insert_resource(Time::<()>::default())
            .add_plugins(FrontControlsPlugin);
        // Run the controls without the window-dependent cursor system: tests publish the cursor.
        app
    }

    fn spawn(app: &mut App, slot: u8, x: i32, y: i32) -> Entity {
        app.world_mut().spawn(Formation::new(slot, UnitKind::Infantry, IVec2::new(x, y))).id()
    }

    fn set_cursor(app: &mut App, at: Vec2, free: bool) {
        *app.world_mut().resource_mut::<FrontMapCursor>() = FrontMapCursor { xy: Some(at), free, px_per_tile: 20.0 };
    }

    fn step(app: &mut App) {
        // The cursor system needs a window; run only the controls that read the published cursor.
        let mut schedule = Schedule::default();
        schedule.add_systems((front_mouse_system, front_key_commands_system, update_order_feedback_system).chain());
        schedule.run(app.world_mut());
        let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        mouse.clear();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().clear();
    }

    fn press(app: &mut App, b: MouseButton, at: Vec2) {
        set_cursor(app, at, true);
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(b);
        step(app);
    }

    fn release(app: &mut App, b: MouseButton, at: Vec2) {
        set_cursor(app, at, true);
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().release(b);
        step(app);
    }

    fn click(app: &mut App, b: MouseButton, at: Vec2) {
        press(app, b, at);
        release(app, b, at);
    }

    fn queued(app: &App) -> Vec<DeploymentOrder> {
        app.world().resource::<DeploymentOrderQueue>().iter().map(|(_, o)| *o).collect()
    }

    #[test]
    fn click_shift_click_and_box_select_then_one_order_each() {
        let mut app = app();
        let a = spawn(&mut app, 0, 10, 10);
        let b = spawn(&mut app, 0, 12, 10);
        let c = spawn(&mut app, 0, 40, 40);
        let _enemy = spawn(&mut app, 1, 11, 11);
        click(&mut app, MouseButton::Left, Vec2::new(10.5, 10.5));
        assert_eq!(app.world().resource::<SelectedFormation>().0, vec![a]);
        // Shift-click adds, shift-click again removes.
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ShiftLeft);
        click(&mut app, MouseButton::Left, Vec2::new(12.5, 10.5));
        assert_eq!(app.world().resource::<SelectedFormation>().0.len(), 2);
        click(&mut app, MouseButton::Left, Vec2::new(12.5, 10.5));
        assert_eq!(app.world().resource::<SelectedFormation>().0, vec![a]);
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::ShiftLeft);
        // Box over a and b only (the enemy inside is not selectable).
        press(&mut app, MouseButton::Left, Vec2::new(8.0, 8.0));
        release(&mut app, MouseButton::Left, Vec2::new(14.0, 14.0));
        let mut want = vec![a, b];
        want.sort();
        assert_eq!(app.world().resource::<SelectedFormation>().0, want);
        assert!(!app.world().resource::<SelectedFormation>().contains(c));
        // One order per selected formation; moves are spread.
        click(&mut app, MouseButton::Right, Vec2::new(20.5, 20.5));
        let orders = queued(&app);
        assert_eq!(orders.len(), 2);
        let targets: Vec<_> = orders.iter().map(|o| o.kind).collect();
        assert_ne!(targets[0], targets[1]);
        assert!(orders.iter().all(|o| matches!(o.kind, DeploymentKind::Move(t) if (t - IVec2::new(20, 20)).abs().max_element() <= 1)));
        assert_eq!(app.world().resource::<FrontOrderFeedback>().markers.len(), 2);
    }

    #[test]
    fn keys_hold_cycle_and_toggle_layers_and_respect_rebind_capture() {
        let mut app = app();
        let a = spawn(&mut app, 0, 5, 5);
        let b = spawn(&mut app, 0, 6, 5);
        let b_keys = InputBindings::default();
        let tap = |app: &mut App, k: KeyCode| {
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(k);
            step(app);
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(k);
        };
        tap(&mut app, b_keys.front_cycle_selection);
        let first = app.world().resource::<SelectedFormation>().primary().unwrap();
        tap(&mut app, b_keys.front_cycle_selection);
        let second = app.world().resource::<SelectedFormation>().primary().unwrap();
        assert_ne!(first, second);
        assert!([a, b].contains(&first) && [a, b].contains(&second));
        tap(&mut app, b_keys.front_hold);
        assert!(queued(&app).iter().any(|o| o.kind == DeploymentKind::Hold && o.formation == second));
        tap(&mut app, b_keys.front_toggle_pressure);
        assert!(!app.world().resource::<FrontOverlaySettings>().pressure);
        tap(&mut app, b_keys.front_toggle_overlay);
        assert!(!app.world().resource::<FrontOverlaySettings>().any(), "one layer was off, others on -> all off");
        tap(&mut app, b_keys.front_toggle_overlay);
        assert!(app.world().resource::<FrontOverlaySettings>().fronts);
        // While a binding is being captured, keys do nothing.
        app.insert_resource(KeybindingsUiState { capture_slot: Some(crate::gui::options_keybindings_ui::BindingSlot::FrontHold), ..default() });
        let before = queued(&app).len();
        tap(&mut app, b_keys.front_hold);
        assert_eq!(queued(&app).len(), before);
    }

    #[test]
    fn refused_order_flashes_and_notifies() {
        let mut app = app();
        let a = spawn(&mut app, 0, 5, 5);
        app.world_mut().resource_mut::<SelectedFormation>().set(vec![a]);
        click(&mut app, MouseButton::Right, Vec2::new(9.5, 5.5));
        let id = app.world().resource::<FrontOrderFeedback>().markers[0].id;
        // The executor refuses it.
        app.world_mut().resource_mut::<DeploymentOutcomes>().recent.push(OutcomeRecord {
            id,
            formation: a,
            kind: DeploymentKind::Hold,
            issued_by: 0,
            result: Err(DeploymentReject::Broken),
            at_micros: 0,
        });
        step(&mut app);
        let fb = app.world().resource::<FrontOrderFeedback>();
        assert!(matches!(fb.markers[0].state, MarkerState::Rejected(DeploymentReject::Broken)));
        assert!(fb.notice.as_ref().is_some_and(|(t, _)| t.contains("refused")), "{:?}", fb.notice);
    }

    #[test]
    fn gate_closed_or_enemy_only_means_no_selection_or_orders() {
        let mut app = app();
        let _a = spawn(&mut app, 0, 5, 5);
        let e = spawn(&mut app, 1, 30, 5);
        set_cursor(&mut app, Vec2::new(5.5, 5.5), false);
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        step(&mut app);
        assert!(app.world().resource::<SelectedFormation>().0.is_empty());
        click(&mut app, MouseButton::Left, Vec2::new(30.5, 5.5));
        assert!(!app.world().resource::<SelectedFormation>().contains(e));
    }

    #[test]
    fn pure_helpers() {
        assert_eq!(spread_offset(0), IVec2::ZERO);
        assert_ne!(spread_offset(1), spread_offset(2));
        let mut f = Formation::new(0, UnitKind::Infantry, IVec2::new(1, 1));
        assert_eq!(describe_order(&f), "holding");
        f.fortify = true;
        assert_eq!(describe_order(&f), "digging in");
        f.dug_in = 1.0;
        assert_eq!(describe_order(&f), "dug in");
        f.posture = Posture::Retreat;
        assert!(describe_order(&f).contains("broken"));
        let mut s = FrontOverlaySettings::default();
        s.toggle_all();
        assert!(!s.any());
    }

    #[test]
    fn systems_have_disjoint_queries() {
        init_ok(front_mouse_system);
        init_ok(front_key_commands_system);
        init_ok(update_order_feedback_system);
        init_ok(front_panel_buttons_system);
        init_ok(sync_front_selection_panel);
        init_ok(update_front_map_cursor_system);
    }
}
