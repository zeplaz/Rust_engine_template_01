//! Auto instrumentation for CLI `--test …` runs — disk analytics, quiet terminal, stall spans, ECS inventory.
//!
//! When [`EngineLaunchArgs::test_mode`] is true (e.g. `cargo run … -- --test vfx`), this module
//! publishes a process-wide latch equivalent to:
//!
//! ```text
//! SIM_ANALYTICS=1 SIM_ANALYTICS_QUIET=1 SIM_ANALYTICS_FRAMES=1 STALL=1
//! ```
//!
//! Manual env vars still work and can override quiet / frame-jsonl off.

use std::sync::atomic::{AtomicBool, Ordering};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde_json::{json, Value};

use crate::engine::test_harness::TestSceneSimChunk;
use crate::engine::{EngineLaunchArgs, TestScene};
use crate::gui::editor::map_editor::MapEditorRoadMarkerV1;
use crate::render::FireSimulationSnapshot;
use crate::strategic::LogisticsGraph;
use crate::systems::transport::TransportTopology;
use crate::terrain::generation::world_generator_enhanced::{TileMarker, WorldGenProgress, WorldMarker};

static ACTIVE: AtomicBool = AtomicBool::new(false);
static QUIET_TERMINAL: AtomicBool = AtomicBool::new(false);
static FRAME_JSONL: AtomicBool = AtomicBool::new(false);
static STALL_SPANS: AtomicBool = AtomicBool::new(false);
static FLUSH_SECS: std::sync::OnceLock<f32> = std::sync::OnceLock::new();

/// Live latch state — readable from systems; atomics mirror this for pre-Startup callers.
#[derive(Resource, Clone, Debug)]
pub struct TestRunInstrumentation {
    pub active: bool,
    pub quiet_terminal: bool,
    pub frame_jsonl: bool,
    pub stall_spans: bool,
    pub flush_secs: f32,
    pub from_test_cli: bool,
    pub test_scene: crate::engine::TestScene,
    pub maneuver: crate::engine::DebugManeuver,
}

impl Default for TestRunInstrumentation {
    fn default() -> Self {
        Self {
            active: false,
            quiet_terminal: false,
            frame_jsonl: false,
            stall_spans: false,
            flush_secs: 5.0,
            from_test_cli: false,
            test_scene: crate::engine::TestScene::None,
            maneuver: crate::engine::DebugManeuver::None,
        }
    }
}

#[must_use]
pub fn env_flag(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

#[must_use]
pub fn instrumentation_active() -> bool {
    env_flag("SIM_ANALYTICS") || env_flag("PERF_DISK") || ACTIVE.load(Ordering::Relaxed)
}

#[must_use]
pub fn instrumentation_quiet_terminal() -> bool {
    instrumentation_active()
        && (env_flag("SIM_ANALYTICS_QUIET") || QUIET_TERMINAL.load(Ordering::Relaxed))
}

#[must_use]
pub fn instrumentation_frame_jsonl() -> bool {
    instrumentation_active()
        && (env_flag("SIM_ANALYTICS_FRAMES") || FRAME_JSONL.load(Ordering::Relaxed))
}

#[must_use]
pub fn instrumentation_stall_spans() -> bool {
    STALL_SPANS.load(Ordering::Relaxed)
}

#[must_use]
pub fn instrumentation_flush_secs() -> f32 {
    std::env::var("SIM_ANALYTICS_FLUSH_SECS")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .filter(|v| *v >= 0.5)
        .unwrap_or_else(|| *FLUSH_SECS.get_or_init(|| 5.0))
}

fn publish_atomics(inst: &TestRunInstrumentation) {
    ACTIVE.store(inst.active, Ordering::Relaxed);
    QUIET_TERMINAL.store(inst.quiet_terminal, Ordering::Relaxed);
    FRAME_JSONL.store(inst.frame_jsonl, Ordering::Relaxed);
    STALL_SPANS.store(inst.stall_spans, Ordering::Relaxed);
    let _ = FLUSH_SECS.set(inst.flush_secs);
}

fn merge_env_overrides(inst: &mut TestRunInstrumentation) {
    if env_flag("SIM_ANALYTICS") || env_flag("PERF_DISK") {
        inst.active = true;
    }
    if env_flag("SIM_ANALYTICS_QUIET") {
        inst.quiet_terminal = true;
    }
    if env_flag("SIM_ANALYTICS_FRAMES") {
        inst.frame_jsonl = true;
    }
    if env_flag("STALL") {
        inst.stall_spans = true;
    }
}

pub fn bootstrap_test_run_instrumentation(
    launch: Res<EngineLaunchArgs>,
    mut inst: ResMut<TestRunInstrumentation>,
    mut fire_cadence: Option<ResMut<crate::render::FireExtractCadence>>,
) {
    let profile = launch.test_instrumentation_profile();
    if profile.active {
        inst.active = true;
        inst.quiet_terminal = profile.quiet_terminal;
        inst.frame_jsonl = profile.frame_jsonl;
        inst.stall_spans = profile.stall_spans;
        inst.flush_secs = profile.flush_secs;
        inst.from_test_cli = true;
        inst.test_scene = launch.test_scene;
        inst.maneuver = launch.maneuver;
    }
    merge_env_overrides(&mut inst);
    publish_atomics(&inst);

    if matches!(launch.test_scene, TestScene::VfxSandbox | TestScene::Visual) {
        if let Some(cadence) = fire_cadence.as_mut() {
            crate::render::FireExtractCadence::clamp_for_runtime(cadence, true);
        }
    }

    if inst.active {
        info!(
            target: "test_instrumentation",
            scene = ?inst.test_scene,
            maneuver = ?inst.maneuver,
            quiet = inst.quiet_terminal,
            frame_jsonl = inst.frame_jsonl,
            stall_spans = inst.stall_spans,
            flush_secs = inst.flush_secs,
            witness = "debug_runs/sim_spectrum_analytics_live.json",
            "TEST_INSTRUMENTATION active (--test harness)"
        );
    }
}

/// Throttled ECS + resource counts merged into sim-spectrum frame snapshots.
#[derive(Resource)]
pub struct EcsResourceInventory {
    sample_every_frames: u32,
    frames_since_sample: u32,
    pub last_json: Option<Value>,
}

impl Default for EcsResourceInventory {
    fn default() -> Self {
        Self {
            sample_every_frames: 15,
            frames_since_sample: 0,
            last_json: None,
        }
    }
}

#[derive(SystemParam)]
struct InventoryProbe<'w, 's> {
    tiles: Query<'w, 's, (), With<TileMarker>>,
    worlds: Query<'w, 's, (), With<WorldMarker>>,
    roads: Query<'w, 's, (), With<MapEditorRoadMarkerV1>>,
    test_chunks: Query<'w, 's, (), With<TestSceneSimChunk>>,
    cameras: Query<'w, 's, (), With<Camera>>,
    transport: Option<Res<'w, TransportTopology>>,
    logistics: Option<Res<'w, LogisticsGraph>>,
    fire_sim: Option<Res<'w, FireSimulationSnapshot>>,
    world_gen: Option<Res<'w, WorldGenProgress>>,
}

fn sample_ecs_resource_inventory(
    mut inventory: ResMut<EcsResourceInventory>,
    probe: InventoryProbe,
) {
    if !instrumentation_active() {
        return;
    }
    inventory.frames_since_sample = inventory.frames_since_sample.saturating_add(1);
    if inventory.frames_since_sample < inventory.sample_every_frames {
        return;
    }
    inventory.frames_since_sample = 0;

    let entities = json!({
        "tiles": probe.tiles.iter().count(),
        "world_markers": probe.worlds.iter().count(),
        "map_editor_roads": probe.roads.iter().count(),
        "test_scene_chunks": probe.test_chunks.iter().count(),
        "cameras": probe.cameras.iter().count(),
    });

    let resources = json!({
        "transport_topology_edges": probe
            .transport
            .as_deref()
            .map(|t| t.neighbors.len())
            .unwrap_or(0),
        "logistics_graph": probe.logistics.as_deref().map(|g| json!({
            "revision": g.revision,
            "nodes": g.nodes.len(),
            "edges": g.edges.len(),
        })),
        "fire_simulation": probe.fire_sim.as_deref().map(|f| json!({
            "instances": f.instances.len(),
            "chunk_heat_cells": f.chunk_heat.len(),
            "stamp_tick": f.stamp.tick,
        })),
        "world_gen": probe.world_gen.as_deref().map(|w| json!({
            "running": w.running,
            "fraction": w.fraction,
            "label": w.label,
        })),
    });

    inventory.last_json = Some(json!({
        "entities": entities,
        "resources": resources,
    }));
}

pub struct TestRunInstrumentationPlugin;

impl Plugin for TestRunInstrumentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TestRunInstrumentation>()
            .init_resource::<EcsResourceInventory>()
            .add_systems(PostStartup, bootstrap_test_run_instrumentation)
            .add_systems(Last, sample_ecs_resource_inventory);

        #[cfg(feature = "test_instrumentation")]
        {
            use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
            app.add_plugins((
                FrameTimeDiagnosticsPlugin::default(),
                EntityCountDiagnosticsPlugin::default(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EngineLaunchArgs;

    #[test]
    fn bootstrap_merges_env_and_cli() {
        let mut inst = TestRunInstrumentation::default();
        let profile = EngineLaunchArgs::from_cli(Some("vfx".into()), false, None)
            .test_instrumentation_profile();
        inst.active = profile.active;
        inst.quiet_terminal = profile.quiet_terminal;
        assert!(inst.active);
        assert!(inst.quiet_terminal);
    }
}
