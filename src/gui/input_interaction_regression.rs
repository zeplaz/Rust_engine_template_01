//! **Interaction regression suite** (fleet `interaction`) — headless input injection against the
//! real construction / picking / menu systems. Each test is named after its ledger row in
//! `src/dev/interaction_bug_ledger_v1.md`.
//!
//! Run: `cargo test -p proc_A_dine01 --lib interaction_regression`.
//!
//! [`InteractionHarness`] drives a `MinimalPlugins` app with a primary window, a valid tactical
//! map hole and a free pointer gate. `step()` emulates one frame boundary (update + input clear,
//! which `InputPlugin` would do in `PreUpdate`).

use bevy::ecs::message::MessageReader;
use bevy::ecs::schedule::ScheduleConfigs;
use bevy::ecs::system::ScheduleSystem;
use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::construction::{
    ActiveBuildTool, BuildCommandActor, BuildGhostState, BuildPlacementMode, BuildPlacementPreview,
    BuildStripState, BuildTool, ConstructionHistory, PendingConstructionQueue, PlaceFeedbackPulse,
    StagedPlacementMode, ToolContext,
};
use crate::construction::ActiveToolSession;
use crate::engine::states::BaseState;
use crate::gui::hud::simulation_pointer_gate::{
    add_pointer_gate_systems, PointerGateSet, SimulationMapPointerGate,
};
use crate::gui::hud::simulation_shell_phase2::ContextTrayState;
use crate::gui::{
    CommandLeftStackState, InputBindings, MainWorldCameraOrthoTrace, MapCameraDesiredRes,
    MinimapShellState, SimulationMapViewport,
};
use crate::strategic::CommitConstructionSiteEvent;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// Window logical size used by every harness (scale factor 1 → logical == physical).
pub const HARNESS_WINDOW: Vec2 = Vec2::new(1280.0, 720.0);

#[derive(Resource, Default)]
pub struct CommitEventCount(pub u32);

fn count_commit_events(
    mut reader: MessageReader<CommitConstructionSiteEvent>,
    mut count: ResMut<CommitEventCount>,
) {
    for _ in reader.read() {
        count.0 = count.0.saturating_add(1);
    }
}

pub struct InteractionHarness {
    pub app: App,
}

impl InteractionHarness {
    /// Bare harness: window + map hole covering the whole window + ortho 1 world unit / px
    /// centered on world (256, 256) of a 512² world.
    #[must_use]
    pub fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<InputBindings>()
            .init_resource::<MapCameraDesiredRes>()
            .init_resource::<CommitEventCount>()
            .add_message::<CommitConstructionSiteEvent>();
        let mut params = WorldGenParams::default();
        params.width = 512;
        params.height = 512;
        app.insert_resource(params);
        let mut vp = SimulationMapViewport::default();
        vp.valid = true;
        vp.min = Vec2::ZERO;
        vp.max = HARNESS_WINDOW;
        vp.window_logical = HARNESS_WINDOW;
        app.insert_resource(vp);
        app.insert_resource(MainWorldCameraOrthoTrace {
            fixed_width: HARNESS_WINDOW.x,
            fixed_height: HARNESS_WINDOW.y,
            view_pixels: HARNESS_WINDOW,
            authority_zoom: 1.0,
            desired_zoom: 1.0,
            camera_center: Vec2::new(256.0, 256.0),
        });
        app.insert_resource(SimulationMapPointerGate {
            in_play_area: true,
            wheel_play_area: true,
            window_logical: HARNESS_WINDOW,
            ..Default::default()
        });
        let mut window = Window::default();
        window
            .resolution
            .set(HARNESS_WINDOW.x, HARNESS_WINDOW.y);
        app.world_mut().spawn((window, PrimaryWindow));
        Self { app }
    }

    /// Harness with the building placement chain as scheduled in `BuildPlanningPlugin`
    /// (pick → second-LMB commit → cancel), minus validation (preview is set by the test).
    #[must_use]
    pub fn building_placement() -> Self {
        Self::building_placement_with_gate(None)
    }

    /// As [`Self::building_placement`], optionally with the real pointer-gate systems
    /// ([`PointerGateSet`]) scheduled in `base` state; consumers run after the publish set,
    /// exactly as in `BuildPlanningPlugin`.
    #[must_use]
    pub fn building_placement_with_gate(gate_base: Option<BaseState>) -> Self {
        let mut h = Self::new();
        if let Some(base) = gate_base {
            h.app
                .add_plugins(bevy::state::app::StatesPlugin)
                .insert_state(base)
                .init_resource::<CommandLeftStackState>()
                .init_resource::<MinimapShellState>()
                .init_resource::<ContextTrayState>();
            add_pointer_gate_systems(&mut h.app);
        }
        let actor = h.app.world_mut().spawn_empty().id();
        h.app
            .insert_resource(BuildCommandActor(actor))
            .init_resource::<BuildStripState>()
            .init_resource::<ActiveBuildTool>()
            .init_resource::<BuildGhostState>()
            .init_resource::<BuildPlacementPreview>()
            .init_resource::<PendingConstructionQueue>()
            .init_resource::<StagedPlacementMode>()
            .init_resource::<ActiveToolSession>()
            .init_resource::<crate::construction::BuildingDefinitionRegistry>()
            .init_resource::<ConstructionHistory>()
            .init_resource::<PlaceFeedbackPulse>();
        let chain = (
            crate::construction::build_pick_ghost_tile_system,
            crate::construction::build_place_on_second_lmb_system,
            count_commit_events,
        )
            .chain();
        if gate_base.is_some() {
            h.add_update(chain.after(PointerGateSet::Publish));
        } else {
            h.add_update(chain);
        }
        h
    }

    /// Spawn a laid-out UI panel root (`marker`) covering `min..max` in window physical px.
    pub fn spawn_panel<M: Component>(&mut self, marker: M, min: Vec2, max: Vec2) {
        let size = max - min;
        self.app.world_mut().spawn((
            marker,
            ComputedNode {
                size,
                ..Default::default()
            },
            UiGlobalTransform::from_translation(min + size * 0.5),
        ));
    }

    pub fn add_update(&mut self, systems: ScheduleConfigs<ScheduleSystem>) -> &mut Self {
        self.app.add_systems(Update, systems);
        self
    }

    pub fn set_cursor(&mut self, logical: Vec2) {
        let mut q = self
            .app
            .world_mut()
            .query_filtered::<&mut Window, With<PrimaryWindow>>();
        let mut w = q.single_mut(self.app.world_mut()).expect("primary window");
        w.set_physical_cursor_position(Some(DVec2::new(logical.x as f64, logical.y as f64)));
        let mut gate = self.app.world_mut().resource_mut::<SimulationMapPointerGate>();
        gate.cursor = logical;
        gate.cursor_known = true;
    }

    pub fn press_mouse(&mut self, b: MouseButton) {
        self.app
            .world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(b);
    }

    pub fn release_mouse(&mut self, b: MouseButton) {
        self.app
            .world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(b);
    }

    pub fn press_key(&mut self, k: KeyCode) {
        self.app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(k);
    }

    pub fn release_key(&mut self, k: KeyCode) {
        self.app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(k);
    }

    /// One frame: run `Update`, then clear `just_*` like `InputPlugin` does next `PreUpdate`.
    pub fn step(&mut self) {
        self.app.update();
        self.app.world_mut().resource_mut::<ButtonInput<KeyCode>>().clear();
        self.app
            .world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
    }

    /// Press + step + release + step.
    pub fn click(&mut self, b: MouseButton) {
        self.press_mouse(b);
        self.step();
        self.release_mouse(b);
        self.step();
    }

    #[must_use]
    pub fn commits(&self) -> u32 {
        self.app.world().resource::<CommitEventCount>().0
    }

    pub fn res<R: Resource>(&self) -> &R {
        self.app.world().resource::<R>()
    }

    pub fn res_mut<R: Resource<Mutability = bevy::ecs::component::Mutable>>(&mut self) -> Mut<'_, R> {
        self.app.world_mut().resource_mut::<R>()
    }

    /// Screen point (logical px) over world XY for the default ortho (1 world unit / px).
    #[must_use]
    pub fn screen_for_world(&self, world: Vec2) -> Vec2 {
        let o = *self.res::<MainWorldCameraOrthoTrace>();
        let vp = *self.res::<SimulationMapViewport>();
        crate::gui::input_pick::TacticalMapPickFrame::from_rendered_ortho(&vp, &o)
            .expect("harness frame")
            .world_xy_to_screen(world)
    }

    /// Arm a two-click building tool with a valid preview.
    pub fn arm_factory(&mut self) {
        use crate::construction::BuildingArchetypeId;
        self.res_mut::<BuildStripState>().active = ToolContext::Industry;
        let intent = crate::construction::intent_from_archetype(
            BuildingArchetypeId::Factory,
            self.res::<crate::construction::BuildingDefinitionRegistry>(),
        );
        let mut tool = self.res_mut::<ActiveBuildTool>();
        tool.tool = BuildTool::Building(BuildingArchetypeId::Factory);
        // The pick system holds the ghost until a catalog row sets the intent.
        tool.building_intent = Some(intent);
        drop(tool);
        self.allow_commit();
    }

    pub fn allow_commit(&mut self) {
        let mut p = self.res_mut::<BuildPlacementPreview>();
        p.report.valid = true;
        p.report.allows_commit = true;
    }
}

impl Default for InteractionHarness {
    fn default() -> Self {
        Self::new()
    }
}

// ── INT-005: two-click place FSM ─────────────────────────────────────────────

#[test]
fn int_005_first_lmb_locks_without_committing() {
    let mut h = InteractionHarness::building_placement();
    h.arm_factory();
    let at = h.screen_for_world(Vec2::new(100.5, 200.5));
    h.set_cursor(at);
    h.step();
    assert_eq!(h.res::<BuildGhostState>().placement_mode, BuildPlacementMode::Place);

    h.click(MouseButton::Left);
    assert_eq!(h.commits(), 0, "first LMB must only lock the ghost (Adjust), not commit");
    assert_eq!(h.res::<BuildGhostState>().placement_mode, BuildPlacementMode::Adjust);

    h.allow_commit();
    h.click(MouseButton::Left);
    assert_eq!(h.commits(), 1, "second LMB commits the locked ghost");
}

#[test]
fn int_005_locked_origin_survives_cursor_motion() {
    let mut h = InteractionHarness::building_placement();
    h.arm_factory();
    h.set_cursor(h.screen_for_world(Vec2::new(10.5, 10.5)));
    h.click(MouseButton::Left);
    let locked = h.res::<BuildGhostState>().origin;
    h.set_cursor(h.screen_for_world(Vec2::new(40.5, 60.5)));
    h.step();
    assert_eq!(h.res::<BuildGhostState>().origin, locked);
}

// ── R-H4 / R-H5: panel rect blocking through the real gate systems ───────────

/// Build a gated harness, arm the factory and park the cursor on world (100.5, 200.5).
fn gated_factory(base: BaseState) -> (InteractionHarness, Vec2) {
    let mut h = InteractionHarness::building_placement_with_gate(Some(base));
    h.arm_factory();
    let at = h.screen_for_world(Vec2::new(100.5, 200.5));
    h.set_cursor(at);
    (h, at)
}

fn physical_rect_around(p: Vec2, half: f32) -> (Vec2, Vec2) {
    (p - Vec2::splat(half), p + Vec2::splat(half))
}

#[test]
fn panel_open_cursor_outside_rect_reaches_placement() {
    let (mut h, at) = gated_factory(BaseState::Simulation);
    // Panel on the far right; the cursor is nowhere near it.
    h.spawn_panel(
        crate::gui::diagnostics_ui::DiagnosticsBevyRoot,
        Vec2::new(980.0, 60.0),
        Vec2::new(1260.0, 400.0),
    );
    assert!(at.x < 900.0);
    h.step();
    h.click(MouseButton::Left);
    assert_eq!(h.res::<BuildGhostState>().placement_mode, BuildPlacementMode::Adjust);
    h.allow_commit();
    h.click(MouseButton::Left);
    assert_eq!(h.commits(), 1);
}

#[test]
fn panel_open_cursor_over_rect_blocks_placement() {
    let (mut h, at) = gated_factory(BaseState::Simulation);
    let (min, max) = physical_rect_around(at, 40.0);
    h.spawn_panel(crate::gui::diagnostics_ui::DiagnosticsBevyRoot, min, max);
    h.step();
    h.click(MouseButton::Left);
    assert_eq!(h.res::<BuildGhostState>().placement_mode, BuildPlacementMode::Place);
    h.click(MouseButton::Left);
    assert_eq!(h.commits(), 0);
}

#[test]
fn editor_dock_present_cursor_off_dock_reaches_placement() {
    let (mut h, at) = gated_factory(BaseState::Editor);
    h.spawn_panel(
        crate::gui::hud::dock_shell::EditorDockBevyRoot,
        Vec2::new(8.0, 40.0),
        Vec2::new(288.0, 400.0),
    );
    assert!(at.x > 300.0);
    h.step();
    h.click(MouseButton::Left);
    assert_eq!(h.res::<BuildGhostState>().placement_mode, BuildPlacementMode::Adjust);
}

#[test]
fn panel_spawned_this_frame_blocks_first_click() {
    let (mut h, _at) = gated_factory(BaseState::Simulation);
    h.step();
    // Spawned, not laid out yet (`ComputedNode` empty): the first click must not pass through.
    h.app
        .world_mut()
        .spawn((crate::gui::diagnostics_ui::DiagnosticsBevyRoot, Node::default()));
    h.press_mouse(MouseButton::Left);
    h.step();
    assert_eq!(h.res::<BuildGhostState>().placement_mode, BuildPlacementMode::Place);
}

#[test]
fn admin_zone_drag_press_on_chrome_ignored_and_map_drag_survives_panel() {
    use crate::construction::ZoneTool;
    use crate::strategic::settlement::{AdminZoneDrag, DistrictBook, TownBook, TownId};
    let mut h = InteractionHarness::new();
    h.app
        .insert_resource(ActiveBuildTool {
            tool: BuildTool::Zone(ZoneTool::ResidentialLow),
            ..Default::default()
        })
        .init_resource::<AdminZoneDrag>()
        .init_resource::<DistrictBook>()
        .insert_resource(TownBook {
            default_town: Some(TownId("t".into())),
            ..Default::default()
        });
    h.add_update(crate::construction::admin_zone_drag_input_system.into_configs());
    h.press_key(KeyCode::ControlLeft);
    h.set_cursor(Vec2::new(300.0, 300.0));

    // Press over chrome: no drag starts.
    h.res_mut::<SimulationMapPointerGate>().chrome_blocks = true;
    h.press_mouse(MouseButton::Left);
    h.step();
    assert!(!h.res::<AdminZoneDrag>().armed);
    h.release_mouse(MouseButton::Left);
    h.step();

    // Press on the map: drag starts. Then the cursor crosses a panel: drag keeps growing.
    h.res_mut::<SimulationMapPointerGate>().chrome_blocks = false;
    h.press_mouse(MouseButton::Left);
    h.step();
    assert!(h.res::<AdminZoneDrag>().armed);
    let before = h.res::<AdminZoneDrag>().vertices.len();
    h.res_mut::<SimulationMapPointerGate>().chrome_blocks = true;
    h.set_cursor(Vec2::new(900.0, 600.0));
    h.step();
    assert!(h.res::<AdminZoneDrag>().vertices.len() > before);
}
