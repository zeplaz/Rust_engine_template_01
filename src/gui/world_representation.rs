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
    in_simulation_or_editor_map, map_zoom_alpha, MapCameraDesired, MapCameraSystemSet,
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
    /// Resolves the camera frame and rewrites `lod_map` **in place** (reuses its allocation).
    /// `world_extent` clamps the LOD sweep to the loaded chunk bounding box.
    #[allow(clippy::too_many_arguments)]
    pub fn resolve_into(
        &self,
        global: &GlobalLodState,
        engine: &WorldLodPolicyEngine,
        inputs: LodInputs,
        zones: &[OperationalLodZone],
        bubbles: &[TacticalLodBubble],
        focus_chunk: IVec2,
        interest_radius_chunks: i32,
        chunk_tiles: UVec2,
        world_extent: Option<(IVec2, IVec2)>,
        stamp: SimStepStamp,
        lod_map: &mut WorldLodMap,
    ) -> WorldRepresentationFrame {
        let mut frame = engine.evaluate(inputs, zones);
        frame.bands.global = band_max_fidelity(frame.bands.global, global.default_band);
        frame.visibility = engine.compute_visibility(frame.bands.global);
        frame.resolution = engine.compute_resolution(frame.bands.global);
        frame.sim_step_stamp = stamp;
        build_world_lod_map_into(
            lod_map,
            stamp,
            focus_chunk,
            interest_radius_chunks,
            chunk_tiles,
            world_extent,
            frame.bands.global,
            global.default_band,
            zones,
            bubbles,
        );
        frame
    }
}

/// Sweeps the interest window into `out.cells` **in place** (clear + reuse the existing
/// allocation; never `Vec::new()` per frame). The `dx`/`dy` window is intersected with the
/// world chunk extent so a small world never sweeps the full `(2r+1)^2` block.
#[allow(clippy::too_many_arguments)]
fn build_world_lod_map_into(
    out: &mut WorldLodMap,
    stamp: SimStepStamp,
    focus_chunk: IVec2,
    interest_radius_chunks: i32,
    chunk_tiles: UVec2,
    world_extent: Option<(IVec2, IVec2)>,
    camera_band: WorldLodBand,
    global_floor: WorldLodBand,
    zones: &[OperationalLodZone],
    bubbles: &[TacticalLodBubble],
) {
    out.stamp = stamp;
    let cells = &mut out.cells;
    cells.clear();
    let radius = interest_radius_chunks.max(1);
    let cw = chunk_tiles.x.max(1) as f32;
    let ch = chunk_tiles.y.max(1) as f32;
    // Clamp the sweep window to the actual world chunk extent so a 10×10 world never sweeps
    // ~9k cells. With no loaded chunks (headless / pre-stream) fall back to the radius box.
    let (min_x, max_x, min_y, max_y) = match world_extent {
        Some((min, max)) => (
            (focus_chunk.x - radius).max(min.x),
            (focus_chunk.x + radius).min(max.x),
            (focus_chunk.y - radius).max(min.y),
            (focus_chunk.y + radius).min(max.y),
        ),
        None => (
            focus_chunk.x - radius,
            focus_chunk.x + radius,
            focus_chunk.y - radius,
            focus_chunk.y + radius,
        ),
    };
    for cy in min_y..=max_y {
        for cx in min_x..=max_x {
            let coord = IVec2::new(cx, cy);
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

/// Bounding box (`min..=max`, inclusive chunk coords) of the currently loaded chunks.
/// `None` while no chunks exist (headless / pre-stream) → LOD sweep falls back to the radius box.
#[inline]
fn loaded_chunk_extent(chunks: &Query<(&Chunk, &ChunkCellMatrix)>) -> Option<(IVec2, IVec2)> {
    let mut iter = chunks.iter();
    let first = iter.next()?.0.coord;
    let mut min = first;
    let mut max = first;
    for (chunk, _) in iter {
        min = min.min(chunk.coord);
        max = max.max(chunk.coord);
    }
    Some((min, max))
}

/// Quantize zoom α so scroll wheel does not re-enqueue streaming every tick (0.1 bands).
pub const INTEREST_ZOOM_QUANTUM: f32 = 0.1;

#[inline]
#[must_use]
pub fn quantize_zoom_alpha_for_interest(zoom_alpha: f32) -> f32 {
    let a = zoom_alpha.clamp(0.0, 1.0);
    (a / INTEREST_ZOOM_QUANTUM).round() * INTEREST_ZOOM_QUANTUM
}

#[inline]
#[must_use]
pub fn interest_radius_chunks_from_zoom_alpha(zoom_alpha: f32) -> i32 {
    let a = zoom_alpha.clamp(0.0, 1.0);
    let r = (6.0 + a * 28.0).round() as i32;
    r.clamp(4, 48)
}

/// Dirty-gate fingerprint for [`compute_world_representation_frame`].
///
/// Captures the **real** inputs that change the derived frame / LOD map: sim cadence (heat,
/// agents, bubbles advance per tick), camera pose (quantized so genuine pan/zoom still rebuilds
/// but a perfectly static camera does not re-sweep), the interest radius / focus chunk, the
/// loaded chunk extent, and the tier-2/tier-3 LOD registries. When this is unchanged the
/// `(2r+1)^2` sweep + frame write is skipped — the previously-published frame stays valid.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WorldReprInputFingerprint {
    stamp: SimStepStamp,
    focus_chunk: IVec2,
    interest_radius_chunks: i32,
    /// Camera translation quantized to 0.5 world units (sub-pixel jitter ignored, real pan caught).
    cam_xy_q: IVec2,
    /// Zoom quantized to 1e-3 (band noise ignored, real zoom caught).
    zoom_q: i32,
    world_extent: Option<(IVec2, IVec2)>,
    zones_len: usize,
    /// Cheap order-sensitive digest of the tier-2 zones (band/center/radius/priority).
    zones_digest: u64,
    bubbles_len: usize,
    /// Cheap digest of tier-3 bubbles (center/radius/escalation).
    bubbles_digest: u64,
    default_band: WorldLodBand,
    readability_floor_q: i32,
    readability_density_q: i32,
}

impl WorldReprInputFingerprint {
    #[must_use]
    fn same_except_stamp(self, other: Self) -> bool {
        Self {
            stamp: other.stamp,
            ..self
        } == other
    }
}

#[inline]
fn quantize_f32(v: f32, step: f32) -> i32 {
    (v / step).round() as i32
}

fn digest_zones(zones: &[OperationalLodZone]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for z in zones {
        for part in [
            z.center.x,
            z.center.y,
            z.center.z,
            z.radius,
            z.priority,
            band_fidelity_rank(z.band) as f32,
        ] {
            h ^= u64::from(part.to_bits());
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    h
}

fn digest_bubbles(bubbles: &[TacticalLodBubble]) -> u64 {
    let mut h = 0x84222325cbf29ce4u64;
    for b in bubbles {
        for part in [b.center.x, b.center.y, b.center.z, b.radius] {
            h ^= u64::from(part.to_bits());
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    h
}

pub fn compute_world_representation_frame(
    time: Res<Time>,
    desired: Res<MapCameraDesired>,
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
    mut last_fingerprint: Local<Option<WorldReprInputFingerprint>>,
) {
    let _perf = crate::render::PerfScope::new("upd_world_repr_frame");
    let t0 = std::time::Instant::now();
    let zoom = desired.scale.x;
    let translation = desired.translation.truncate();
    let za = map_zoom_alpha(zoom);

    let stamp = SimStepStamp::from_tick(*sim, *sim_time);
    let sz = tiles_per_chunk(&chunks);
    let cw = sz.x.max(1) as f32;
    let ch = sz.y.max(1) as f32;
    let world_extent = loaded_chunk_extent(&chunks);
    let interest_radius_chunks =
        interest_radius_chunks_from_zoom_alpha(quantize_zoom_alpha_for_interest(za));
    let focus_chunk = IVec2::new(
        (desired.translation.x / cw).floor() as i32,
        (desired.translation.y / ch).floor() as i32,
    );

    // DIRTY-GATE (perf: fire/LOD per-frame stall). Skip the full `(2r+1)^2` sweep + frame write
    // when no real input changed. Gates on ALL inputs the frame depends on so a legitimately
    // changed frame still rebuilds: sim cadence, camera pan/zoom, focus chunk, interest radius,
    // loaded world extent, and the tier-2/tier-3 LOD registries.
    let fingerprint = WorldReprInputFingerprint {
        stamp,
        focus_chunk,
        interest_radius_chunks,
        cam_xy_q: IVec2::new(quantize_f32(translation.x, 0.5), quantize_f32(translation.y, 0.5)),
        zoom_q: quantize_f32(zoom, 1.0e-3),
        world_extent,
        zones_len: zones.zones.len(),
        zones_digest: digest_zones(&zones.zones),
        bubbles_len: tactical.bubbles.len(),
        bubbles_digest: digest_bubbles(&tactical.bubbles),
        default_band: global.default_band,
        readability_floor_q: quantize_f32(global.readability_zoom_floor, 1.0e-3),
        readability_density_q: quantize_f32(global.readability_screen_density, 1.0e-3),
    };
    if *last_fingerprint == Some(fingerprint) {
        if let Some(perf) = perf.as_mut() {
            crate::render::record_frame_perf_ms(
                perf,
                t0.elapsed().as_secs_f32() * 1000.0,
                crate::render::FramePerfSlot::WorldRepr,
            );
        }
        return;
    }
    if crate::engine::ux_spike_active()
        && last_fingerprint
            .is_some_and(|prev| prev.same_except_stamp(fingerprint))
    {
        return;
    }
    *last_fingerprint = Some(fingerprint);

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

    let mut next = resolver.resolve_into(
        &global,
        &engine,
        inputs,
        &zones.zones,
        &tactical.bubbles,
        focus_chunk,
        interest_radius_chunks,
        sz,
        world_extent,
        stamp,
        &mut lod_map,
    );

    next.zoom = zoom;
    next.focus_chunk = focus_chunk;
    next.interest_radius_chunks = interest_radius_chunks;
    next.gameplay_importance = gameplay.gameplay_importance;

    *frame = next;
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

/// PT-4-003 — production iso atlas demotes PG-2 mesh extract when any site is covered.
pub fn sync_procedural_tile_primary_active(
    registry: Option<Res<crate::construction::procedural::TileAtlasRegistry>>,
    sites: Query<(
        &crate::strategic::PlannedSite,
        Option<&crate::strategic::ProceduralBuildingSpec>,
    )>,
    mut flag: ResMut<crate::construction::procedural::ProceduralTilePrimaryActive>,
) {
    flag.0 = false;
    let Some(registry) = registry else {
        return;
    };
    for (planned, spec) in &sites {
        let entry = crate::gui::map_tile_atlas_stamp::resolve_atlas_entry_for_planned_site(
            &registry,
            planned,
            spec.as_deref(),
        );
        if entry.is_some_and(crate::construction::procedural::production_atlas_covers_assembly) {
            flag.0 = true;
            return;
        }
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
    tile_primary: Option<Res<crate::construction::procedural::ProceduralTilePrimaryActive>>,
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
    if tile_primary.as_deref().is_some_and(|f| f.0)
        && std::env::var_os("RUST_ENGINE_FORCE_PG2_MESHES").is_none()
    {
        policy.procedural_module_meshes = false;
    }
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
        .init_resource::<crate::render::extraction::ProceduralModuleSceneCatalog>()
        .init_resource::<crate::render::extraction::ProceduralModuleVisualPolicy>()
        .init_resource::<crate::render::extraction::ProceduralBuildExtract>()
        .init_resource::<crate::construction::ProceduralAssemblyRequest>()
        .add_systems(
            Startup,
            (
                crate::construction::procedural::init_procedural_module_registry,
                crate::construction::procedural::init_style_pack_registry,
                crate::construction::procedural::init_variant_catalog,
                crate::render::extraction::load_procedural_module_scenes,
            )
                .chain(),
        )
        .init_resource::<crate::construction::ProceduralTilePrimaryActive>()
        .add_systems(
            Update,
            (
                decay_tactical_lod_bubbles,
                refresh_lod_zone_registry,
                crate::render::stall_checkpoint_before_world_repr,
                sync_procedural_tile_primary_active,
                compute_world_representation_frame,
                witness_stage5_lod_band_log_after_world_representation,
                apply_representation_result,
                crate::render::extraction::sync_procedural_module_visual_policy,
                crate::construction::sync_procedural_assembly_request_from_sites,
                crate::render::extraction::extract_procedural_build_assembly,
                crate::construction::spawn_procedural_build_on_site_operational
                    .after(crate::construction::advance_site_construction_tick_system)
                    .after(crate::render::extraction::extract_procedural_build_assembly),
                crate::render::stall_checkpoint_post_world_repr,
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
    fn interest_radius_stable_within_zoom_quantum_band() {
        let a = interest_radius_chunks_from_zoom_alpha(quantize_zoom_alpha_for_interest(0.14));
        let b = interest_radius_chunks_from_zoom_alpha(quantize_zoom_alpha_for_interest(0.16));
        assert_eq!(a, b);
        let c = interest_radius_chunks_from_zoom_alpha(quantize_zoom_alpha_for_interest(0.24));
        assert_ne!(a, c);
    }

    #[test]
    fn lod_map_clamps_sweep_to_world_extent() {
        // 10×10-chunk world (coords 0..=9), radius 48 → unclamped would sweep ~9409 cells.
        let extent = Some((IVec2::ZERO, IVec2::new(9, 9)));
        let mut map = WorldLodMap::default();
        build_world_lod_map_into(
            &mut map,
            SimStepStamp::default(),
            IVec2::new(5, 5),
            48,
            UVec2::new(32, 32),
            extent,
            WorldLodBand::Strategic,
            WorldLodBand::Strategic,
            &[],
            &[],
        );
        // Clamped to the 10×10 world = 100 cells, not (2*48+1)^2 = 9409.
        assert_eq!(map.cells.len(), 100);
        // No loaded chunks → falls back to the radius box (small radius here).
        let mut unbounded = WorldLodMap::default();
        build_world_lod_map_into(
            &mut unbounded,
            SimStepStamp::default(),
            IVec2::ZERO,
            2,
            UVec2::new(32, 32),
            None,
            WorldLodBand::Strategic,
            WorldLodBand::Strategic,
            &[],
            &[],
        );
        assert_eq!(unbounded.cells.len(), 25); // (2*2+1)^2
    }

    #[test]
    fn lod_map_reuses_allocation_in_place() {
        let extent = Some((IVec2::ZERO, IVec2::new(9, 9)));
        let mut map = WorldLodMap::default();
        build_world_lod_map_into(
            &mut map,
            SimStepStamp::default(),
            IVec2::new(5, 5),
            48,
            UVec2::new(32, 32),
            extent,
            WorldLodBand::Strategic,
            WorldLodBand::Strategic,
            &[],
            &[],
        );
        let cap_after_first = map.cells.capacity();
        assert!(cap_after_first >= 100);
        // Second build into the same buffer must not require a fresh allocation.
        build_world_lod_map_into(
            &mut map,
            SimStepStamp::new(1, 1000),
            IVec2::new(5, 5),
            48,
            UVec2::new(32, 32),
            extent,
            WorldLodBand::Strategic,
            WorldLodBand::Strategic,
            &[],
            &[],
        );
        assert_eq!(map.cells.len(), 100);
        assert!(map.cells.capacity() >= cap_after_first);
        assert_eq!(map.stamp, SimStepStamp::new(1, 1000));
    }

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
