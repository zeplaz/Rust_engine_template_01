//! ECS components for operational construction sites (P2-A).

use bevy::prelude::*;

use super::resources::{FootprintTiles, SiteConstructionPhase, SiteId};
use crate::strategic::build_order::BuildSiteTile;
use crate::strategic::spatial_network::LayerType;

pub use crate::construction::SiteStageProgress;

// -----------------------------------------------------------------------------
// Site identity & lifecycle
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SiteArchetype {
    CivilHousing,
    Factory,
    PowerPlant,
    RailDepot,
    MilitaryBase,
    RadarSite,
    SensorPost,
    TrenchLine,
    BunkerComplex,
    FuelDepot,
    WaterPlant,
}

/// Bitmask of required / connected [`crate::strategic::spatial_network::NetworkType`] lanes (expand as graphs mature).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NetworkMask(pub u32);

#[derive(Component, Debug, Clone)]
pub struct ConstructionSite {
    pub site_id: u64,
    pub owner: Entity,
    pub archetype: SiteArchetype,
    pub phase: SiteConstructionPhase,
    pub operational_readiness: f32,
}

#[derive(Component, Debug, Clone)]
pub struct SiteFootprint {
    pub tiles: Vec<IVec2>,
    pub layer: LayerType,
}

/// Authoritative sparse weights for a committed parametric site.
#[derive(Component, Debug, Clone)]
pub struct SiteWeightedFootprint {
    pub weights: Vec<(IVec2, f32)>,
}

/// Scale exponents applied at industrial activation (Phase 4 hooks).
#[derive(Component, Debug, Clone, Copy)]
pub struct BuildingScaleParams {
    pub scale_factor: f32,
    pub effective_scale: f32,
}

/// PG-3 — procedural assembly request derived at commit; read by render extract + tile atlas stamp.
#[derive(Component, Debug, Clone)]
pub struct ProceduralBuildingSpec(pub crate::construction::procedural::ProceduralBuildingRequest);

#[derive(Component, Debug, Clone)]
pub struct SiteResourceManifest {
    pub concrete_required: f32,
    pub steel_required: f32,
    pub fuel_required: f32,
    pub machinery_required: f32,

    pub delivered_concrete: f32,
    pub delivered_steel: f32,
    pub delivered_fuel: f32,
    pub delivered_machinery: f32,
}

impl Default for SiteResourceManifest {
    fn default() -> Self {
        Self {
            concrete_required: 0.0,
            steel_required: 0.0,
            fuel_required: 0.0,
            machinery_required: 0.0,
            delivered_concrete: 0.0,
            delivered_steel: 0.0,
            delivered_fuel: 0.0,
            delivered_machinery: 0.0,
        }
    }
}

impl SiteResourceManifest {
    /// Aggregate delivered / required in [0, 1]; 1 = fully supplied for construction phase.
    #[inline]
    pub fn delivered_ratio(&self) -> f32 {
        let mut num = 0u32;
        let mut acc = 0.0f32;
        if self.concrete_required > 0.0 {
            num += 1;
            acc += (self.delivered_concrete / self.concrete_required).clamp(0.0, 1.0);
        }
        if self.steel_required > 0.0 {
            num += 1;
            acc += (self.delivered_steel / self.steel_required).clamp(0.0, 1.0);
        }
        if self.fuel_required > 0.0 {
            num += 1;
            acc += (self.delivered_fuel / self.fuel_required).clamp(0.0, 1.0);
        }
        if self.machinery_required > 0.0 {
            num += 1;
            acc += (self.delivered_machinery / self.machinery_required).clamp(0.0, 1.0);
        }
        if num == 0 {
            return 1.0;
        }
        acc / num as f32
    }
}

#[derive(Component, Debug, Clone)]
pub struct SiteNetworkAttachment {
    pub required_networks: NetworkMask,
    pub connected_networks: NetworkMask,
}

impl Default for SiteNetworkAttachment {
    fn default() -> Self {
        Self {
            required_networks: NetworkMask(0),
            connected_networks: NetworkMask(0),
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct SiteTerrainValidation {
    pub slope_ok: bool,
    pub hydrology_ok: bool,
    pub geology_ok: bool,
    pub flood_risk: f32,
}

impl Default for SiteTerrainValidation {
    fn default() -> Self {
        Self {
            slope_ok: true,
            hydrology_ok: true,
            geology_ok: true,
            flood_risk: 0.0,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct SiteOperationalStats {
    pub workforce_ratio: f32,
    pub supply_ratio: f32,
    pub power_ratio: f32,
    pub integrity: f32,
}

impl Default for SiteOperationalStats {
    fn default() -> Self {
        Self {
            workforce_ratio: 0.0,
            supply_ratio: 0.0,
            power_ratio: 0.0,
            integrity: 1.0,
        }
    }
}

/// Player / AI **planned** site — inspectable; pairs with [`ConstructionSite`] after commit.
#[derive(Component, Clone, Debug)]
pub struct PlannedSite {
    pub site_id: SiteId,
    pub origin: BuildSiteTile,
    pub footprint: FootprintTiles,
    pub archetype: SiteArchetype,
    pub layer: LayerType,
    pub catalog_id: Option<String>,
    pub placement: Option<super::parametric::CommittedPlacementSnapshot>,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct SiteConstructionRate {
    pub labor_efficiency: f32,
    pub machinery_efficiency: f32,
    pub weather_penalty: f32,
}

impl Default for SiteConstructionRate {
    fn default() -> Self {
        Self {
            labor_efficiency: 1.0,
            machinery_efficiency: 1.0,
            weather_penalty: 1.0,
        }
    }
}

/// Marks entities that contribute fields into [`crate::strategic::ChunkStrategicOverlay`] (P2-E).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ZoneEmitter {
    pub supply_strength: f32,
    pub fire_control_strength: f32,
    pub sensor_strength: f32,
    pub civil_authority_strength: f32,
}
