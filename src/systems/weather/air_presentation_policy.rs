//! Air / drone **presentation** fidelity policy (D8 scaffold; readable from D1 MissionBook extract).
//!
//! Defaults stay cheap (symbology / sprites / cosmetic wind-lean paths). Budgets and fidelity
//! enums are the upgrade knobs — raise them later for denser sprites, sim-synced paths, or
//! optional voxel/mesh heroes **without** rewriting MissionBook or combat resolve.
//!
//! **Hard rule:** presentation never writes sim. Higher fidelity only consumes richer snapshots.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Zoom / density band for what to draw (D8 extract picks one per view).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AirPresentationBand {
    #[default]
    Far,
    Mid,
    Near,
    Hero,
}

/// How closely on-screen flight matches MissionBook / envelope truth.
///
/// Upgrade path (low → high cost):
/// 1. [`TemplateCosmetic`] — seeded loops + wind lean (default).
/// 2. [`EnvelopeDriven`] — path samples link/flight/sensor gates from the book snapshot.
/// 3. [`SimSynced`] — presentation samples the same sortie timeline the sim advanced (still read-only).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FlightPathFidelity {
    /// Cheap canned / wind-lean templates. Default for mass FPV/swarm.
    #[default]
    TemplateCosmetic,
    /// Paths respect envelope abort / heading from MissionBook fields (no extra sim entities).
    EnvelopeDriven,
    /// Paths follow sim sortie ticks published on the presentation snapshot.
    SimSynced,
}

/// Geometry tier allowed under current budgets.
///
/// Mass FPV/swarm defaults to [`MarksOrSprites`]. [`VoxelOrMesh`] is opt-in when
/// `voxel_hero_budget` / role scarcity allows — never implied for every swarm member.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AirGeometryTier {
    /// AO tint, APP-6 marks, count badges.
    SymbologyOnly,
    /// Batched 2D sprites / dots (default mid zoom).
    #[default]
    MarksOrSprites,
    /// Rare animated mesh or voxel heroes (cruise / manned / zoomed interceptor).
    VoxelOrMesh,
}

/// Tunable caps — raise when perf budgets allow; MissionBook extract must clamp to these.
#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AirPresentationPolicy {
    /// Max batched sprite/dot instances per view (overflow → density / symbology).
    pub sprite_cap_per_view: u32,
    /// Max mesh/voxel hero instances per view (0 = heroes disabled).
    pub voxel_hero_budget: u32,
    /// Max simultaneous animated near-band sprites (FPV/interceptor spotlight).
    pub near_animated_cap: u32,
    /// Default flight path fidelity for mass roles (FPV / swarm).
    pub mass_flight_fidelity: FlightPathFidelity,
    /// Default flight path fidelity for scarce roles (cruise / interceptor / manned pool).
    pub scarce_flight_fidelity: FlightPathFidelity,
    /// Max geometry tier for mass FPV/swarm (raise to [`AirGeometryTier::VoxelOrMesh`] only with budget).
    pub mass_geometry_tier: AirGeometryTier,
    /// Max geometry tier for scarce / hero air.
    pub scarce_geometry_tier: AirGeometryTier,
    /// When true, D8 may emit voxel/mesh for mass roles up to `voxel_hero_budget`
    /// (off by default — enables future quality without a redesign).
    pub allow_mass_voxel_upgrade: bool,
}

impl Default for AirPresentationPolicy {
    fn default() -> Self {
        Self {
            sprite_cap_per_view: 256,
            voxel_hero_budget: 4,
            near_animated_cap: 24,
            mass_flight_fidelity: FlightPathFidelity::TemplateCosmetic,
            scarce_flight_fidelity: FlightPathFidelity::EnvelopeDriven,
            mass_geometry_tier: AirGeometryTier::MarksOrSprites,
            scarce_geometry_tier: AirGeometryTier::VoxelOrMesh,
            allow_mass_voxel_upgrade: false,
        }
    }
}

impl AirPresentationPolicy {
    /// Geometry allowed for a mass (swarm/FPV) draw under current policy.
    #[must_use]
    pub fn mass_geometry(&self) -> AirGeometryTier {
        if self.allow_mass_voxel_upgrade
            && self.voxel_hero_budget > 0
            && matches!(self.mass_geometry_tier, AirGeometryTier::VoxelOrMesh)
        {
            AirGeometryTier::VoxelOrMesh
        } else if matches!(self.mass_geometry_tier, AirGeometryTier::VoxelOrMesh) {
            // Tier raised but upgrade flag off → stay on sprites.
            AirGeometryTier::MarksOrSprites
        } else {
            self.mass_geometry_tier
        }
    }

    /// Effective flight fidelity for mass roles (never above policy knob).
    #[must_use]
    pub fn mass_flight(&self) -> FlightPathFidelity {
        self.mass_flight_fidelity
    }

    /// Effective flight fidelity for scarce roles.
    #[must_use]
    pub fn scarce_flight(&self) -> FlightPathFidelity {
        self.scarce_flight_fidelity
    }

    /// Raise mass presentation toward sim when budgets allow (idempotent helper for future tools/UI).
    pub fn scale_mass_quality(&mut self, sprites: u32, near: u32, flight: FlightPathFidelity) {
        self.sprite_cap_per_view = self.sprite_cap_per_view.max(sprites);
        self.near_animated_cap = self.near_animated_cap.max(near);
        self.mass_flight_fidelity = flight;
    }

    /// Opt into mass voxel/mesh heroes under an explicit budget (future quality path).
    pub fn enable_mass_voxel_upgrade(&mut self, hero_budget: u32) {
        self.allow_mass_voxel_upgrade = true;
        self.voxel_hero_budget = self.voxel_hero_budget.max(hero_budget);
        self.mass_geometry_tier = AirGeometryTier::VoxelOrMesh;
    }
}

/// Snapshot contract MissionBook (D1+) must fill so D8 can upgrade fidelity without new sim writers.
///
/// Fields are additive: cosmetic templates only need `mission_id` + wind; `SimSynced` needs
/// `sortie_tick` / sample points. Leaving higher fields at defaults keeps D8 cheap.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AirPresentationSortieHint {
    pub mission_id: u64,
    pub role_mass: bool,
    pub wind_mps: bevy::math::Vec2,
    pub heading: bevy::math::Vec2,
    /// Envelope / sim-synced path samples (metres XY); empty ⇒ template-only.
    pub path_samples_m: Vec<bevy::math::Vec2>,
    /// Sim sortie tick when fidelity is [`FlightPathFidelity::SimSynced`].
    pub sortie_tick: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_keep_mass_off_voxels() {
        let p = AirPresentationPolicy::default();
        assert_eq!(p.mass_geometry(), AirGeometryTier::MarksOrSprites);
        assert_eq!(p.mass_flight(), FlightPathFidelity::TemplateCosmetic);
        assert!(!p.allow_mass_voxel_upgrade);
    }

    #[test]
    fn upgrade_path_can_enable_mass_voxel_and_sim_paths() {
        let mut p = AirPresentationPolicy::default();
        p.scale_mass_quality(1024, 64, FlightPathFidelity::SimSynced);
        assert_eq!(p.mass_flight(), FlightPathFidelity::SimSynced);
        assert_eq!(p.sprite_cap_per_view, 1024);

        p.enable_mass_voxel_upgrade(16);
        assert!(p.allow_mass_voxel_upgrade);
        assert_eq!(p.mass_geometry(), AirGeometryTier::VoxelOrMesh);
        assert!(p.voxel_hero_budget >= 16);
    }

    #[test]
    fn voxel_tier_without_flag_stays_sprites() {
        let mut p = AirPresentationPolicy::default();
        p.mass_geometry_tier = AirGeometryTier::VoxelOrMesh;
        assert_eq!(p.mass_geometry(), AirGeometryTier::MarksOrSprites);
    }
}
