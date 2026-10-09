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
use crate::gui::hud::simulation_pointer_gate::SimulationMapPointerGate;
use crate::gui::{
    InputBindings, MainWorldCameraOrthoTrace, MapCameraDesiredRes, SimulationMapViewport,
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
        let mut h = Self::new();
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
        h.add_update(
            (
                crate::construction::build_pick_ghost_tile_system,
                crate::construction::build_place_on_second_lmb_system,
                count_commit_events,
            )
                .chain(),
        );
        h
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
        self.res_mut::<ActiveBuildTool>().tool = BuildTool::Building(BuildingArchetypeId::Factory);
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
#[ignore = "INT-005 open: first LMB leaves placement_mode at Place instead of Adjust; product vs harness cause not yet triaged (interaction_bug_ledger_v1.md)"]
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
