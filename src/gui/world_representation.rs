//! Global world representation anchor (**theme-3**): [`WorldLodPolicyEngine`] → [`WorldRepresentationFrame`].
//! LOD is decided once per frame from [`LodInputs`]; projection and compute graphs only read the frame.
//!
//! Spine: `prompts/guides/base_visual_world_representation_v1.md`.

use bevy::prelude::*;

use super::representation_policy::{
    build_representation_inputs, build_representation_result, LodZoneClass, RepresentationResult,
};
use super::view_representation::{CameraVisualState, VisualBudgetSettings, VisualCadence};
use crate::compute::AgentFrame;
use crate::gui::lod_zone_authoring::refresh_lod_zone_registry;
use crate::gui::map_camera::{
    in_simulation_or_editor_map, map_zoom_alpha, MapCameraDesired, MapCameraSettings,
    MapCameraSystemSet,
};
use crate::render::{ChunkFireHeat, FireSimulationSnapshot, Stage5LodBandLogWitness};
use crate::systems::sim_control::{SimControlSystemSet, SimStepStamp, SimTick, SimTimeMicros};
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

/// Schedule anchor: LOD policy runs after camera smooth and sim tick.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorldRepresentationSystemSet {
    ComputeFrame,
}

/// LOD band output (not a single “quality enum” — contextual policy result).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WorldLodBand {
    LocalTactical,
    Operational,
    Strategic,
    Macro,
}

impl WorldLodBand {
    #[must_use]
    pub const fn short_label(self) -> &'static str {
        match self {
            Self::LocalTactical => "LT",
            Self::Operational => "OP",
            Self::Strategic => "ST",
            Self::Macro => "MC",
        }
    }
}

/// Tier 1 baseline band for the whole world (nothing falls below this floor).
#[derive(Resource, Clone, Copy, Debug)]
pub struct GlobalLodState {
    pub default_band: WorldLodBand,
    /// VA3: written by [`crate::gui::sync_tile_readability_witness`], read in LOD evaluate.
    pub readability_zoom_floor: f32,
    pub readability_screen_density: f32,
}

impl Default for GlobalLodState {
    fn default() -> Self {
        Self {
            default_band: WorldLodBand::Strategic,
            readability_zoom_floor: 0.0,
            readability_screen_density: 1.0,
        }
    }
}

pub type LodZoneId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LodZoneSource {
    JumpPoint,
    Settlement,
    CombatFront,
    LogisticsHub,
    MissionArea,
    PlayerFocus,
}

/// Tier 2 persistent gameplay-interest region (chunk-scale semantics).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OperationalLodZone {
    pub zone_id: LodZoneId,
    pub class: LodZoneClass,
    pub center: Vec3,
    pub radius: f32,
    pub band: WorldLodBand,
    pub priority: f32,
    pub source: LodZoneSource,
}

/// Tier 3 transient micro bubble (projectiles, local combat spikes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TacticalEscalation {
    Rendering,
    Simulation,
    ProjectileTracking,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TacticalLodBubble {
    pub center: Vec3,
    pub radius: f32,
    pub ttl_seconds: f32,
    pub escalation: TacticalEscalation,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct TacticalLodBubbleRegistry {
    pub bubbles: Vec<TacticalLodBubble>,
}

/// Authoring surface for tier-2 zones (gameplay / editor injects; policy engine only reads).
#[derive(Resource, Clone, Debug, Default)]
pub struct LodZoneRegistry {
    pub zones: Vec<OperationalLodZone>,
}

/// Spatial LOD field sampled by render/compute/AI (camera sample stays on [`WorldRepresentationFrame`]).
#[derive(Resource, Clone, Debug, Default)]
pub struct WorldLodMap {
    pub stamp: SimStepStamp,
    pub cells: Vec<LodCell>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LodCell {
    pub coord: IVec2,
    pub render_band: WorldLodBand,
    pub compute_band: WorldLodBand,
    pub importance: f32,
}

impl WorldLodMap {
    #[must_use]
    pub fn cell_at(&self, coord: IVec2) -> Option<&LodCell> {
        self.cells.iter().find(|cell| cell.coord == coord)
    }

    #[must_use]
    pub fn render_band_at(&self, coord: IVec2, fallback: WorldLodBand) -> WorldLodBand {
        self.cell_at(coord)
            .map(|cell| cell.render_band)
            .unwrap_or(fallback)
    }

    #[must_use]
    pub fn compute_band_at(&self, coord: IVec2, fallback: WorldLodBand) -> WorldLodBand {
        self.cell_at(coord)
            .map(|cell| cell.compute_band)
            .unwrap_or(fallback)
    }
}

/// Composes tier-1/2/3 inputs into camera frame + spatial map.
#[derive(Resource, Debug, Default)]
pub struct WorldRepresentationResolver;

#[inline]
fn band_fidelity_rank(band: WorldLodBand) -> u8 {
    match band {
        WorldLodBand::LocalTactical => 0,
        WorldLodBand::Operational => 1,
        WorldLodBand::Strategic => 2,
        WorldLodBand::Macro => 3,
    }
}

#[inline]
fn band_max_fidelity(a: WorldLodBand, b: WorldLodBand) -> WorldLodBand {
    if band_fidelity_rank(a) <= band_fidelity_rank(b) {
        a
    } else {
        b
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldLodBands {
    pub global: WorldLodBand,
}

impl Default for WorldLodBands {
    fn default() -> Self {
        Self {
            global: WorldLodBand::LocalTactical,
        }
    }
}

/// Which representation channels are active at this band.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldVisibilityMask {
    pub fire_instances: bool,
    pub fire_chunk_heat: bool,
    pub pathfinding_field: bool,
    pub agent_steering: bool,
}

impl Default for WorldVisibilityMask {
    fn default() -> Self {
        Self {
            fire_instances: true,
            fire_chunk_heat: true,
            pathfinding_field: true,
            agent_steering: true,
        }
    }
}

/// Resolution / cadence hints for GPU and CPU projection (registry scaling hooks later).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldResolutionPolicy {
    pub fire_instance_cap: usize,
    pub chunk_heat_bin: i32,
    pub compute_dispatch_hz: f32,
}

impl Default for WorldResolutionPolicy {
    fn default() -> Self {
        Self {
            fire_instance_cap: usize::MAX,
            chunk_heat_bin: 1,
            compute_dispatch_hz: 60.0,
        }
    }
}

/// Per-frame output: single LOD decision + camera snapshot for spatial consumers.
#[derive(Resource, Clone, Copy, Debug)]
pub struct WorldRepresentationFrame {
    pub bands: WorldLodBands,
    pub visibility: WorldVisibilityMask,
    pub resolution: WorldResolutionPolicy,
    pub zoom: f32,
    pub focus_chunk: IVec2,
    pub interest_radius_chunks: i32,
    pub sim_step_stamp: SimStepStamp,
    pub gameplay_importance: f32,
}

impl WorldRepresentationFrame {
    #[inline]
    #[must_use]
    pub fn global_band(self) -> WorldLodBand {
        self.bands.global
    }
}

impl Default for WorldRepresentationFrame {
    fn default() -> Self {
        Self {
            bands: WorldLodBands::default(),
            visibility: WorldVisibilityMask::default(),
            resolution: WorldResolutionPolicy::default(),
            zoom: 1.0,
            focus_chunk: IVec2::ZERO,
            interest_radius_chunks: 8,
            sim_step_stamp: SimStepStamp::default(),
            gameplay_importance: 0.0,
        }
    }
}

/// Signals that may feed LOD policy (camera + gameplay; no subsystem may set bands directly).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LodInputs {
    pub camera_position: Vec2,
    pub camera_distance: f32,
    pub camera_velocity: f32,
    pub screen_density: f32,
    pub zoom_level: f32,
    pub gameplay_importance: f32,
    pub ai_density: f32,
    pub combat_intensity: f32,
    pub event_priority: f32,
}

impl LodInputs {
    /// Combined gameplay urgency used for tactical override (gameplay beats camera-only collapse).
    #[must_use]
    pub fn merged_importance(&self) -> f32 {
        self.gameplay_importance
            .max(self.combat_intensity)
            .max(self.event_priority)
            .clamp(0.0, 1.0)
    }

    /// Scales camera-driven LOD scale toward tactical when off-screen activity is high.
    #[must_use]
    pub fn camera_scale_modifier(&self) -> f32 {
        let gameplay = self.merged_importance();
        let ai = self.ai_density.clamp(0.0, 1.0);
        (1.0 - gameplay * 0.85 - ai * 0.1).clamp(0.25, 1.0)
    }
}

impl Default for LodInputs {
    fn default() -> Self {
        Self {
            camera_position: Vec2::ZERO,
            camera_distance: 0.0,
            camera_velocity: 0.0,
            screen_density: 1.0,
            zoom_level: 0.5,
            gameplay_importance: 0.0,
            ai_density: 0.0,
            combat_intensity: 0.0,
            event_priority: 0.0,
        }
    }
}

/// Read-only gameplay-side signals sampled into [`LodInputs`] each frame.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct LodGameplaySignals {
    pub gameplay_importance: f32,
    pub ai_density: f32,
    pub combat_intensity: f32,
    pub event_priority: f32,
}

#[must_use]
pub fn gather_lod_gameplay_signals(
    fire_chunk_heat: &[ChunkFireHeat],
    agents: &AgentFrame,
    tactical: &TacticalLodBubbleRegistry,
) -> LodGameplaySignals {
    let burning_chunks = fire_chunk_heat.len();
    let max_heat = fire_chunk_heat
        .iter()
        .map(|row| row.heat)
        .fold(0.0f32, f32::max);
    let chunk_pressure = (burning_chunks as f32 / 48.0).clamp(0.0, 1.0);
    let combat_intensity = (chunk_pressure * 0.55 + max_heat.clamp(0.0, 1.0) * 0.45).clamp(0.0, 1.0);
    let ai_density = (agents.agent_count as f32 / 256.0).clamp(0.0, 1.0);
    let event_priority = if tactical.bubbles.is_empty() {
        0.0
    } else {
        0.85
    };
    let gameplay_importance = combat_intensity
        .max(ai_density * 0.35)
        .max(event_priority * 0.9);
    LodGameplaySignals {
        gameplay_importance,
        ai_density,
        combat_intensity,
        event_priority,
    }
}

/// Camera-side state for velocity and last sample (inputs only, not LOD output).
#[derive(Clone, Copy, Debug)]
pub struct CameraLodState {
    pub previous_translation: Vec2,
    pub initialized: bool,
}

impl Default for CameraLodState {
    fn default() -> Self {
        Self {
            previous_translation: Vec2::ZERO,
            initialized: false,
        }
    }
}

/// Global weights and thresholds for [`WorldLodPolicyEngine::evaluate`].
#[derive(Clone, Copy, Debug)]
pub struct LodGlobalRules {
    pub zoom_weight: f32,
    pub distance_weight: f32,
    pub velocity_weight: f32,
    pub operational_scale: f32,
    pub strategic_scale: f32,
    pub macro_scale: f32,
    pub importance_override_threshold: f32,
}

impl Default for LodGlobalRules {
    fn default() -> Self {
        Self {
            zoom_weight: 0.6,
            distance_weight: 0.3,
            velocity_weight: 0.1,
            operational_scale: 0.3,
            strategic_scale: 1.0,
            macro_scale: 2.5,
            importance_override_threshold: 0.75,
        }
    }
}

/// Deterministic LOD decision graph (one evaluation per frame per camera).
#[derive(Resource, Clone, Debug)]
pub struct WorldLodPolicyEngine {
    pub camera: CameraLodState,
    pub global_rules: LodGlobalRules,
}

impl Default for WorldLodPolicyEngine {
    fn default() -> Self {
        Self {
            camera: CameraLodState::default(),
            global_rules: LodGlobalRules::default(),
        }
    }
}

impl WorldLodPolicyEngine {
    pub fn evaluate(
        &self,
        inputs: LodInputs,
        zones: &[OperationalLodZone],
    ) -> WorldRepresentationFrame {
        let rules = self.global_rules;
        let scale = (inputs.zoom_level.clamp(0.0, 4.0) * rules.zoom_weight
            + inputs.camera_distance.clamp(0.0, 4.0) * rules.distance_weight
            + inputs.camera_velocity.clamp(0.0, 4.0) * rules.velocity_weight)
            * inputs.camera_scale_modifier();

        let mut band = if scale < rules.operational_scale {
            WorldLodBand::LocalTactical
        } else if scale < rules.strategic_scale {
            WorldLodBand::Operational
        } else if scale < rules.macro_scale {
            WorldLodBand::Strategic
        } else {
            WorldLodBand::Macro
        };

        let (zone_band, zone_importance) =
            self.apply_zone_overrides(band, inputs.camera_position, zones);
        band = zone_band;
        band = self.apply_importance_override(
            band,
            inputs.merged_importance().max(zone_importance),
        );

        WorldRepresentationFrame {
            bands: WorldLodBands { global: band },
            visibility: self.compute_visibility(band),
            resolution: self.compute_resolution(band),
            zoom: 1.0,
            focus_chunk: IVec2::ZERO,
            interest_radius_chunks: 8,
            sim_step_stamp: SimStepStamp::default(),
            gameplay_importance: 0.0,
        }
    }

    fn apply_zone_overrides(
        &self,
        band: WorldLodBand,
        camera_position: Vec2,
        zones: &[OperationalLodZone],
    ) -> (WorldLodBand, f32) {
        let mut out = band;
        let mut importance = 0.0f32;
        for zone in zones {
            if camera_position.distance(zone.center.truncate()) > zone.radius {
                continue;
            }
            out = band_max_fidelity(out, zone.band);
            importance = importance.max(zone.priority.clamp(0.0, 1.0));
        }
        (out, importance)
    }

    fn apply_importance_override(&self, band: WorldLodBand, importance_mask: f32) -> WorldLodBand {
        if importance_mask < self.global_rules.importance_override_threshold {
            return band;
        }
        match band {
            WorldLodBand::Macro => WorldLodBand::Strategic,
            WorldLodBand::Strategic => WorldLodBand::Operational,
            WorldLodBand::Operational => WorldLodBand::LocalTactical,
            WorldLodBand::LocalTactical => WorldLodBand::LocalTactical,
        }
    }

    pub(crate) fn compute_visibility(&self, band: WorldLodBand) -> WorldVisibilityMask {
        match band {
            WorldLodBand::LocalTactical => WorldVisibilityMask {
                fire_instances: true,
                fire_chunk_heat: true,
                pathfinding_field: true,
                agent_steering: true,
            },
            WorldLodBand::Operational => WorldVisibilityMask {
                fire_instances: true,
                fire_chunk_heat: true,
                pathfinding_field: true,
                agent_steering: true,
            },
            WorldLodBand::Strategic => WorldVisibilityMask {
                // Keep fire instance **extraction** on at strategic zoom; throttle cost via
                // [`WorldResolutionPolicy::fire_instance_cap`] + projection top-heat cap, not a hard bool gate.
                fire_instances: true,
                fire_chunk_heat: true,
                pathfinding_field: false,
                agent_steering: false,
            },
            WorldLodBand::Macro => WorldVisibilityMask {
                fire_instances: true,
                fire_chunk_heat: true,
                pathfinding_field: false,
                agent_steering: false,
            },
        }
    }

    pub(crate) fn compute_resolution(&self, band: WorldLodBand) -> WorldResolutionPolicy {
        const OPERATIONAL_FIRE_INSTANCE_CAP: usize = 48;
        match band {
            WorldLodBand::LocalTactical => WorldResolutionPolicy {
                fire_instance_cap: usize::MAX,
                chunk_heat_bin: 1,
                compute_dispatch_hz: 60.0,
            },
            WorldLodBand::Operational => WorldResolutionPolicy {
                fire_instance_cap: OPERATIONAL_FIRE_INSTANCE_CAP,
                chunk_heat_bin: 2,
                compute_dispatch_hz: 20.0,
            },
            WorldLodBand::Strategic => WorldResolutionPolicy {
                fire_instance_cap: 24,
                chunk_heat_bin: 1,
                compute_dispatch_hz: 10.0,
            },
            WorldLodBand::Macro => WorldResolutionPolicy {
                fire_instance_cap: 8,
                chunk_heat_bin: 4,
                compute_dispatch_hz: 1.0,
            },
        }
    }
}

/// Band → visibility mask (shared by camera frame and spatial map sampling).
#[must_use]
pub fn visibility_for_band(band: WorldLodBand) -> WorldVisibilityMask {
    WorldLodPolicyEngine::default().compute_visibility(band)
}

/// Band → resolution policy (shared by camera frame and spatial map sampling).
#[must_use]
pub fn resolution_for_band(band: WorldLodBand) -> WorldResolutionPolicy {
    WorldLodPolicyEngine::default().compute_resolution(band)
}

impl WorldRepresentationResolver {
    pub fn resolve(
        &self,
        global: &GlobalLodState,
        engine: &WorldLodPolicyEngine,
        inputs: LodInputs,
        zones: &[OperationalLodZone],
        bubbles: &[TacticalLodBubble],
        focus_chunk: IVec2,
        interest_radius_chunks: i32,
        chunk_tiles: UVec2,
        stamp: SimStepStamp,
    ) -> (WorldRepresentationFrame, WorldLodMap) {
        let mut frame = engine.evaluate(inputs, zones);
        frame.bands.global = band_max_fidelity(frame.bands.global, global.default_band);
        frame.visibility = engine.compute_visibility(frame.bands.global);
        frame.resolution = engine.compute_resolution(frame.bands.global);
        frame.sim_step_stamp = stamp;
        let map = build_world_lod_map(
            stamp,
            focus_chunk,
            interest_radius_chunks,
            chunk_tiles,
            frame.bands.global,
            global.default_band,
            zones,
            bubbles,
        );
        (frame, map)
    }
}

fn build_world_lod_map(
    stamp: SimStepStamp,
    focus_chunk: IVec2,
    interest_radius_chunks: i32,
    chunk_tiles: UVec2,
    camera_band: WorldLodBand,
    global_floor: WorldLodBand,
    zones: &[OperationalLodZone],
    bubbles: &[TacticalLodBubble],
) -> WorldLodMap {
    let mut cells = Vec::new();
    let radius = interest_radius_chunks.max(1);
    let cw = chunk_tiles.x.max(1) as f32;
    let ch = chunk_tiles.y.max(1) as f32;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let coord = focus_chunk + IVec2::new(dx, dy);
            let sample_world = Vec2::new(
                (coord.x as f32 + 0.5) * cw,
                (coord.y as f32 + 0.5) * ch,
            );
            let mut render_band = band_max_fidelity(camera_band, global_floor);
            let mut compute_band = render_band;
            let mut importance = 0.0f32;
            for zone in zones {
                if sample_world.distance(zone.center.truncate()) > zone.radius {
                    continue;
                }
                render_band = band_max_fidelity(render_band, zone.band);
                compute_band = band_max_fidelity(compute_band, zone.band);
                importance = importance.max(zone.priority.clamp(0.0, 1.0));
            }
            for bubble in bubbles {
                if sample_world.distance(bubble.center.truncate()) > bubble.radius {
                    continue;
                }
                render_band = band_max_fidelity(render_band, WorldLodBand::LocalTactical);
                compute_band = band_max_fidelity(compute_band, WorldLodBand::LocalTactical);
                importance = 1.0;
            }
            cells.push(LodCell {
                coord,
                render_band,
                compute_band,
                importance,
            });
        }
    }
    WorldLodMap { stamp, cells }
}

fn decay_tactical_lod_bubbles(time: Res<Time>, mut bubbles: ResMut<TacticalLodBubbleRegistry>) {
    let dt = time.delta_secs();
    bubbles.bubbles.retain_mut(|bubble| {
        bubble.ttl_seconds -= dt;
        bubble.ttl_seconds > 0.0
    });
}

#[inline]
fn tiles_per_chunk(chunks: &Query<(&Chunk, &ChunkCellMatrix)>) -> UVec2 {
    chunks
        .iter()
        .next()
        .map(|(_, m)| m.size)
        .unwrap_or(UVec2::new(32, 32))
}

#[inline]
fn interest_radius_chunks_from_zoom_alpha(zoom_alpha: f32) -> i32 {
    let a = zoom_alpha.clamp(0.0, 1.0);
    let r = (6.0 + a * 28.0).round() as i32;
    r.clamp(4, 48)
}

pub fn compute_world_representation_frame(
    time: Res<Time>,
    desired: Res<MapCameraDesired>,
    _settings: Res<MapCameraSettings>,
    sim: Res<SimTick>,
    sim_time: Res<SimTimeMicros>,
    chunks: Query<(&Chunk, &ChunkCellMatrix)>,
    fire_sim: Res<FireSimulationSnapshot>,
    agents: Res<AgentFrame>,
    tactical: Res<TacticalLodBubbleRegistry>,
    global: Res<GlobalLodState>,
    zones: Res<LodZoneRegistry>,
    resolver: Res<WorldRepresentationResolver>,
    mut engine: ResMut<WorldLodPolicyEngine>,
    mut frame: ResMut<WorldRepresentationFrame>,
    mut lod_map: ResMut<WorldLodMap>,
    mut perf: Option<ResMut<crate::render::FramePerf>>,
) {
    let t0 = std::time::Instant::now();
    let zoom = desired.scale.x;
    let translation = desired.translation.truncate();
    let za = map_zoom_alpha(zoom);
    let velocity = if engine.camera.initialized {
        (translation - engine.camera.previous_translation).length() / time.delta_secs().max(1e-4)
    } else {
        0.0
    };
    engine.camera.previous_translation = translation;
    engine.camera.initialized = true;

    let gameplay = gather_lod_gameplay_signals(&fire_sim.chunk_heat, &agents, &tactical);
    let mut inputs = LodInputs {
        camera_position: translation,
        camera_distance: translation.length() * 1e-4,
        camera_velocity: velocity * 1e-3,
        screen_density: 1.0,
        zoom_level: za,
        gameplay_importance: gameplay.gameplay_importance,
        ai_density: gameplay.ai_density,
        combat_intensity: gameplay.combat_intensity,
        event_priority: gameplay.event_priority,
    };
    crate::gui::apply_tile_readability_lod_bias(global.as_ref(), &mut inputs);

    let stamp = SimStepStamp::from_tick(*sim, *sim_time);
    let sz = tiles_per_chunk(&chunks);
    let cw = sz.x.max(1) as f32;
    let ch = sz.y.max(1) as f32;
    let interest_radius_chunks = interest_radius_chunks_from_zoom_alpha(za);
    let focus_chunk = IVec2::new(
        (desired.translation.x / cw).floor() as i32,
        (desired.translation.y / ch).floor() as i32,
    );

    let (mut next, map) = resolver.resolve(
        &global,
        &engine,
        inputs,
        &zones.zones,
        &tactical.bubbles,
        focus_chunk,
        interest_radius_chunks,
        sz,
        stamp,
    );

    next.zoom = zoom;
    next.focus_chunk = focus_chunk;
    next.interest_radius_chunks = interest_radius_chunks;
    next.gameplay_importance = gameplay.gameplay_importance;

    *frame = next;
    *lod_map = map;
    if let Some(perf) = perf.as_mut() {
        crate::render::record_frame_perf_ms(
            perf,
            t0.elapsed().as_secs_f32() * 1000.0,
            crate::render::FramePerfSlot::WorldRepr,
        );
    }
}

/// LOD band transition log + [`Stage5LodBandLogWitness`] (separate system: keeps `compute_world_representation_frame`
/// within Bevy’s `.chain()` system-param limit).
pub fn witness_stage5_lod_band_log_after_world_representation(
    frame: Res<WorldRepresentationFrame>,
    mut last_logged_band: Local<Option<WorldLodBand>>,
    mut lod_band_witness: ResMut<Stage5LodBandLogWitness>,
) {
    let band = frame.global_band();
    if last_logged_band.map(|b| b != band).unwrap_or(true) {
        let za = map_zoom_alpha(frame.zoom);
        if crate::render::frame_perf_verbose() {
            info!(
                target: "world_representation::lod",
                "WorldRepresentation: zoom={:.3} zoom_α={:.3} → LOD band {:?} ({})",
                frame.zoom,
                za,
                band,
                band.short_label()
            );
        }
        lod_band_witness.lod_band_log_emissions = lod_band_witness.lod_band_log_emissions.saturating_add(1);
        *last_logged_band = Some(band);
    }
}

pub fn apply_representation_result(
    sim: Res<SimTick>,
    sim_time: Res<SimTimeMicros>,
    frame: Res<WorldRepresentationFrame>,
    zones: Res<LodZoneRegistry>,
    camera: Res<CameraVisualState>,
    budgets: Res<VisualBudgetSettings>,
    cadence: Res<VisualCadence>,
    construction_phase: Option<Res<crate::construction::site_phase_tile_instances::ConstructionPhaseGpuChannel>>,
    graph: Option<Res<crate::strategic::LogisticsGraph>>,
    mut policy: ResMut<RepresentationResult>,
) {
    let stamp = SimStepStamp::from_tick(*sim, *sim_time);
    let mut inputs = build_representation_inputs(&camera, &zones, &budgets, &cadence, stamp);
    inputs.overlay_policy.construction_phase = construction_phase
        .as_deref()
        .is_some_and(|c| c.active);
    inputs.overlay_policy.logistics = graph
        .as_deref()
        .is_some_and(|g| !g.edges.is_empty());
    *policy = build_representation_result(&frame, &inputs);
}

pub(crate) fn register_world_representation_frame(app: &mut App) {
    app.configure_sets(
        Update,
        WorldRepresentationSystemSet::ComputeFrame
            .after(MapCameraSystemSet::Smooth)
            .after(crate::gui::ViewAuthoritySystemSet::SyncViewManager)
            .after(SimControlSystemSet::AdvanceSimTick)
            .after(crate::render::extraction::FireVisualFrameSet::BuildProfiles)
            .before(crate::render::extraction::FireVisualFrameSet::ProjectGpu),
    );
    app.init_resource::<WorldLodPolicyEngine>()
        .init_resource::<GlobalLodState>()
        .init_resource::<LodZoneRegistry>()
        .init_resource::<TacticalLodBubbleRegistry>()
        .init_resource::<WorldRepresentationResolver>()
        .init_resource::<WorldLodMap>()
        .init_resource::<WorldRepresentationFrame>()
        .init_resource::<Stage5LodBandLogWitness>()
        .init_resource::<RepresentationResult>()
        .add_systems(
            Update,
            (
                decay_tactical_lod_bubbles,
                refresh_lod_zone_registry,
                compute_world_representation_frame,
                witness_stage5_lod_band_log_after_world_representation,
                apply_representation_result,
            )
                .chain()
                .in_set(WorldRepresentationSystemSet::ComputeFrame)
                .run_if(in_simulation_or_editor_map),
        );
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::MinimalPlugins;

    #[test]
    fn representation_resolver_resources_register() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        register_world_representation_frame(&mut app);
        assert!(app.world().contains_resource::<WorldRepresentationFrame>());
        assert!(app.world().contains_resource::<RepresentationResult>());
    }

    #[test]
    fn policy_scale_partitions_bands() {
        let engine = WorldLodPolicyEngine::default();
        let low = engine.evaluate(
            LodInputs {
                zoom_level: 0.1,
                camera_distance: 0.0,
                camera_velocity: 0.0,
                screen_density: 1.0,
                ..Default::default()
            },
            &[],
        );
        assert_eq!(low.global_band(), WorldLodBand::LocalTactical);

        let high = engine.evaluate(
            LodInputs {
                zoom_level: 4.0,
                camera_distance: 2.0,
                camera_velocity: 0.0,
                screen_density: 1.0,
                ..Default::default()
            },
            &[],
        );
        assert_eq!(high.global_band(), WorldLodBand::Macro);
    }

    #[test]
    fn importance_override_steps_toward_tactical() {
        let engine = WorldLodPolicyEngine::default();
        let base = engine.evaluate(
            LodInputs {
                zoom_level: 1.0,
                camera_distance: 2.0,
                camera_velocity: 0.0,
                screen_density: 1.0,
                ..Default::default()
            },
            &[],
        );
        assert_eq!(base.global_band(), WorldLodBand::Strategic);

        let boosted = engine.evaluate(
            LodInputs {
                zoom_level: 1.0,
                camera_distance: 2.0,
                camera_velocity: 0.0,
                screen_density: 1.0,
                event_priority: 1.0,
                ..Default::default()
            },
            &[],
        );
        assert!(
            band_fidelity_rank(boosted.global_band()) < band_fidelity_rank(base.global_band())
        );
    }

    #[test]
    fn band_monotonic_in_zoom_level() {
        let engine = WorldLodPolicyEngine::default();
        let mut prev = engine
            .evaluate(
                LodInputs {
                    zoom_level: 0.0,
                    ..Default::default()
                },
                &[],
            )
            .global_band();
        for i in 1..=200 {
            let t = i as f32 / 200.0;
            let b = engine
                .evaluate(
                    LodInputs {
                        zoom_level: t,
                        ..Default::default()
                    },
                    &[],
                )
                .global_band();
            let ord = |x: WorldLodBand| match x {
                WorldLodBand::LocalTactical => 0,
                WorldLodBand::Operational => 1,
                WorldLodBand::Strategic => 2,
                WorldLodBand::Macro => 3,
            };
            assert!(
                ord(b) >= ord(prev),
                "band regressed at zoom_level={t}: {:?} after {:?}",
                b,
                prev
            );
            prev = b;
        }
    }

    #[test]
    fn gameplay_importance_pulls_macro_zoom_toward_tactical() {
        let engine = WorldLodPolicyEngine::default();
        let macro_only = engine.evaluate(
            LodInputs {
                zoom_level: 4.0,
                camera_distance: 2.0,
                ..Default::default()
            },
            &[],
        );
        assert_eq!(macro_only.global_band(), WorldLodBand::Macro);

        let boosted = engine.evaluate(
            LodInputs {
                zoom_level: 4.0,
                camera_distance: 2.0,
                gameplay_importance: 1.0,
                combat_intensity: 1.0,
                ..Default::default()
            },
            &[],
        );
        assert!(
            band_fidelity_rank(boosted.global_band()) < band_fidelity_rank(macro_only.global_band())
        );
    }

    #[test]
    fn gather_lod_gameplay_signals_from_fire_and_agents() {
        let chunk_heat = vec![crate::render::sim_visual_extract::ChunkFireHeat {
            chunk: IVec2::ZERO,
            heat: 0.9,
            smoke: 0.2,
        }];
        let agents = AgentFrame {
            agent_count: 128,
            ..Default::default()
        };
        let signals = gather_lod_gameplay_signals(&chunk_heat, &agents, &TacticalLodBubbleRegistry::default());
        assert!(signals.combat_intensity > 0.4);
        assert!(signals.ai_density > 0.0);
        assert!(signals.gameplay_importance > 0.4);
    }

    #[test]
    fn zone_inside_camera_raises_band_fidelity() {
        let engine = WorldLodPolicyEngine::default();
        let macro_inputs = LodInputs {
            zoom_level: 4.0,
            camera_distance: 2.0,
            ..Default::default()
        };
        let zones = [OperationalLodZone {
            zone_id: 1,
            class: LodZoneClass::Camera,
            center: Vec3::ZERO,
            radius: 64.0,
            band: WorldLodBand::LocalTactical,
            priority: 0.9,
            source: LodZoneSource::PlayerFocus,
        }];
        let boosted = engine.evaluate(macro_inputs, &zones);
        assert_eq!(boosted.global_band(), WorldLodBand::LocalTactical);
    }
}
