//! Engine-wide **sim interest** frame — sole writer for fidelity / cadence hints.
//!
//! PLAN-REF-BOOKS Slice A3 / §3d: one immutable-per-tick resource; domain systems
//! `Res` only. Never written by View/UI. Camera pose is an optional **hint**.

use bevy::prelude::*;

use crate::combat::MissionBook;
use crate::render::view_runtime::{ViewProjectionAuthority, ViewSurfaceId};
use crate::systems::fire::ChunkSurfaceFire;
use crate::systems::sim_control::{SimStepClock, SimTick};
use crate::terrain::generation::Chunk;

/// Named set — use `.after(AttentionSet::Build)`, never string anchors.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttentionSet {
    /// Sole writer of [`SimAttentionFrame`] for this tick.
    Build,
}

/// Fidelity tier for a faction / chunk / theater slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttentionTier {
    /// Full MissionBook / order-sim / spatial audio.
    Full,
    /// Aggregates / cheap auto-supply / ambient bed only.
    Aggregate,
    /// Skip or minimal work.
    Dormant,
}

impl AttentionTier {
    #[must_use]
    pub fn allows_sortie_pulse(self) -> bool {
        !matches!(self, AttentionTier::Dormant)
    }

    #[must_use]
    pub fn allows_full_ai(self) -> bool {
        matches!(self, AttentionTier::Full)
    }

    /// Order-sim / MFG work multiplier this step (1 / 0.5 / 0).
    #[must_use]
    pub fn work_scale(self) -> f32 {
        match self {
            AttentionTier::Full => 1.0,
            AttentionTier::Aggregate => 0.5,
            AttentionTier::Dormant => 0.0,
        }
    }
}

/// One focus blob (player camera, AO, fire theater, construction).
#[derive(Clone, Copy, Debug)]
pub struct SimAttentionFocus {
    pub world_xz: Vec2,
    pub radius_world: f32,
    pub tier: AttentionTier,
}

/// Immutable for the rest of the tick after [`AttentionSet::Build`].
#[derive(Resource, Clone, Debug)]
pub struct SimAttentionFrame {
    pub tick: u64,
    pub sim_steps: u64,
    pub foci: Vec<SimAttentionFocus>,
    /// Chunk coord nearest player WorldMain pose (when available).
    pub focus_chunk: Option<IVec2>,
    pub focus_radius_chunks: i32,
    pub player_theater_active: bool,
    /// Soft budgets for anytime / load-balance consumers.
    pub budget_path_units: u32,
    pub budget_faction_slots: u32,
}

impl Default for SimAttentionFrame {
    fn default() -> Self {
        Self {
            tick: 0,
            sim_steps: 0,
            foci: Vec::new(),
            focus_chunk: None,
            focus_radius_chunks: 2,
            player_theater_active: false,
            budget_path_units: 64,
            budget_faction_slots: 4,
        }
    }
}

impl SimAttentionFrame {
    /// Chebyshev chunk distance to player focus; `None` focus → Aggregate.
    #[must_use]
    pub fn tier_for_chunk(&self, coord: IVec2) -> AttentionTier {
        let Some(fc) = self.focus_chunk else {
            return AttentionTier::Aggregate;
        };
        let d = (coord - fc).abs().max_element();
        if d <= self.focus_radius_chunks {
            AttentionTier::Full
        } else if d <= self.focus_radius_chunks * 3 {
            AttentionTier::Aggregate
        } else {
            AttentionTier::Dormant
        }
    }

    #[must_use]
    pub fn in_player_focus_chunks(&self, coord: IVec2) -> bool {
        matches!(self.tier_for_chunk(coord), AttentionTier::Full)
    }

    /// Faction/theater slot fidelity (Slice B2): round-robin Full slots by sim step.
    #[must_use]
    pub fn tier_for_faction_slot(&self, faction_slot: u8) -> AttentionTier {
        if !self.player_theater_active && self.foci.is_empty() {
            return AttentionTier::Aggregate;
        }
        let slots = self.budget_faction_slots.max(1);
        let active = (self.sim_steps % u64::from(slots)) as u8;
        if faction_slot == active {
            AttentionTier::Full
        } else if faction_slot == active.wrapping_add(1) % slots as u8 {
            AttentionTier::Aggregate
        } else {
            AttentionTier::Dormant
        }
    }
}

/// World units per chunk step for focus→chunk mapping (aligned with fire extract spacing).
const CHUNK_SPACING_WORLD: f32 = 64.0;

pub fn build_sim_attention_frame(
    mut frame: ResMut<SimAttentionFrame>,
    tick: Res<SimTick>,
    clock: Res<SimStepClock>,
    view_auth: Option<Res<ViewProjectionAuthority>>,
    mission_book: Option<Res<MissionBook>>,
    fire_chunks: Query<(&Chunk, &ChunkSurfaceFire)>,
    // DH-03b: RO dense heat — elevates foci / path budget; never writes heat.
    dense_heat: Option<Res<crate::compute::DenseHeatPlayField>>,
    heat_field: Option<Res<crate::compute::HeatDiffusionFieldBuffers>>,
) {
    frame.tick = tick.0;
    frame.sim_steps = clock.steps;
    frame.foci.clear();
    frame.focus_chunk = None;
    frame.player_theater_active = false;
    frame.focus_radius_chunks = 2;
    frame.budget_faction_slots = 4;

    if let Some(auth) = view_auth.as_ref() {
        if let Some(surface) = auth.surface(ViewSurfaceId::WorldMain) {
            let xz = surface.camera.translation;
            frame.foci.push(SimAttentionFocus {
                world_xz: xz,
                radius_world: CHUNK_SPACING_WORLD * 2.0,
                tier: AttentionTier::Full,
            });
            frame.focus_chunk = Some(IVec2::new(
                (xz.x / CHUNK_SPACING_WORLD).floor() as i32,
                (xz.y / CHUNK_SPACING_WORLD).floor() as i32,
            ));
            frame.player_theater_active = true;
        }
    }
    // FIDELITY-1 B6: soft path/compute budget scales with player theater.
    frame.budget_path_units = if frame.player_theater_active { 96 } else { 32 };

    // Active air theaters (MissionBook RO) — Aggregate/Full foci for cadence consumers.
    if let Some(book) = mission_book.as_ref() {
        for z in book.air_op_zones.values() {
            if z.paused {
                continue;
            }
            let c = z.centroid_tile();
            let xz = Vec2::new(c.x as f32, c.y as f32) * (CHUNK_SPACING_WORLD / 16.0);
            frame.foci.push(SimAttentionFocus {
                world_xz: xz,
                radius_world: CHUNK_SPACING_WORLD * 1.5,
                tier: AttentionTier::Full,
            });
            if frame.focus_chunk.is_none() {
                frame.focus_chunk = Some(IVec2::new(
                    (xz.x / CHUNK_SPACING_WORLD).floor() as i32,
                    (xz.y / CHUNK_SPACING_WORLD).floor() as i32,
                ));
            }
        }
        for ao in book.drone_aos.values() {
            if ao.paused {
                continue;
            }
            let c = ao.centroid_tile();
            let xz = Vec2::new(c.x as f32, c.y as f32) * (CHUNK_SPACING_WORLD / 16.0);
            frame.foci.push(SimAttentionFocus {
                world_xz: xz,
                radius_world: CHUNK_SPACING_WORLD,
                tier: AttentionTier::Aggregate,
            });
        }
    }

    // Hot fire chunks elevate interest rings (does not steal ChunkSimLod writer).
    for (chunk, fire) in &fire_chunks {
        if fire.heat > 0.12 {
            let xz = Vec2::new(chunk.coord.x as f32, chunk.coord.y as f32) * CHUNK_SPACING_WORLD;
            frame.foci.push(SimAttentionFocus {
                world_xz: xz,
                radius_world: CHUNK_SPACING_WORLD,
                tier: AttentionTier::Full,
            });
        }
    }

    // DH-03b: dense heat RO — hot cells add Aggregate foci; active heat bumps path budget.
    let mut dense_hot = 0u32;
    if let Some(play) = dense_heat.as_deref() {
        if let Some(grid) = play.grid.as_ref() {
            for y in 0..grid.height {
                for x in 0..grid.width {
                    let i = grid.idx(x, y);
                    if grid.heat[i] <= 0.12 {
                        continue;
                    }
                    dense_hot = dense_hot.saturating_add(1);
                    let chunk = grid.origin + IVec2::new(x as i32, y as i32);
                    let xz = Vec2::new(chunk.x as f32, chunk.y as f32) * CHUNK_SPACING_WORLD;
                    frame.foci.push(SimAttentionFocus {
                        world_xz: xz,
                        radius_world: CHUNK_SPACING_WORLD,
                        tier: AttentionTier::Aggregate,
                    });
                }
            }
        }
    } else if let Some(field) = heat_field.as_deref() {
        for cell in field.read_cells() {
            if cell.heat <= 0.12 {
                continue;
            }
            dense_hot = dense_hot.saturating_add(1);
            let xz = Vec2::new(cell.chunk.x as f32, cell.chunk.y as f32) * CHUNK_SPACING_WORLD;
            frame.foci.push(SimAttentionFocus {
                world_xz: xz,
                radius_world: CHUNK_SPACING_WORLD,
                tier: AttentionTier::Aggregate,
            });
        }
    }
    if dense_hot > 0 {
        frame.budget_path_units = frame.budget_path_units.saturating_add(16).min(128);
    }
}

pub struct SimAttentionPlugin;

impl Plugin for SimAttentionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimAttentionFrame>().add_systems(
            Update,
            build_sim_attention_frame.in_set(AttentionSet::Build),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_for_chunk_uses_chebyshev_rings() {
        let mut f = SimAttentionFrame::default();
        f.focus_chunk = Some(IVec2::ZERO);
        f.focus_radius_chunks = 1;
        assert_eq!(f.tier_for_chunk(IVec2::ZERO), AttentionTier::Full);
        assert_eq!(f.tier_for_chunk(IVec2::new(1, 0)), AttentionTier::Full);
        assert_eq!(f.tier_for_chunk(IVec2::new(2, 0)), AttentionTier::Aggregate);
        assert_eq!(f.tier_for_chunk(IVec2::new(4, 0)), AttentionTier::Dormant);
    }

    #[test]
    fn faction_slot_round_robin_and_work_scale() {
        let mut f = SimAttentionFrame::default();
        f.player_theater_active = true;
        f.sim_steps = 0;
        f.budget_faction_slots = 4;
        assert!(f.tier_for_faction_slot(0).allows_full_ai());
        assert!(f.tier_for_faction_slot(1).allows_sortie_pulse());
        assert!(!f.tier_for_faction_slot(2).allows_sortie_pulse());
        assert_eq!(AttentionTier::Full.work_scale(), 1.0);
        assert_eq!(AttentionTier::Aggregate.work_scale(), 0.5);
        assert_eq!(AttentionTier::Dormant.work_scale(), 0.0);
    }

    #[test]
    fn build_system_writes_frame_without_view() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SimTick>();
        app.init_resource::<SimStepClock>();
        app.init_resource::<SimAttentionFrame>();
        app.add_systems(Update, build_sim_attention_frame);
        app.update();
        let frame = app.world().resource::<SimAttentionFrame>();
        assert!(frame.foci.is_empty());
        assert!(frame.focus_chunk.is_none());
    }
}
