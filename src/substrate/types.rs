//! Full `WorldChunkState` domain shapes (WSS-PLAN-002) — not truncated stubs.

use bevy::math::{IVec2, Vec2};
use bevy::prelude::{Component, UVec2};
use serde::{Deserialize, Serialize};

use crate::substrate::slab::ChunkKey;

/// Default cell grid for skeleton hydrate/tests when no [`ChunkCellMatrix`] is present.
/// Production hydrate copies `matrix.size` → `(size.x * size.y)`.
pub const SUBSTRATE_SKELETON_CELL_GRID: UVec2 = UVec2::new(32, 32);

#[inline]
pub fn substrate_cell_count(size: UVec2) -> usize {
    (size.x * size.y) as usize
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeologyClass {
    #[default]
    Unknown,
    Sedimentary,
    Igneous,
    Metamorphic,
}

#[derive(Clone, Debug, Default)]
pub struct TerrainState {
    pub height: Vec<f32>,
    pub material_ids: Vec<u16>,
    pub geology_class: GeologyClass,
    pub biome_id: u16,
    pub porosity: Vec<f32>,
    pub hardness: Vec<f32>,
}

#[derive(Clone, Debug, Default)]
pub struct HydrologyState {
    pub water_depth: Vec<f32>,
    pub flow_velocity: Vec<Vec2>,
    pub sediment: Vec<f32>,
    pub salinity: Vec<f32>,
    pub saturation: Vec<f32>,
    pub ocean_mask: Vec<u8>,
    pub river_mask: Vec<u8>,
    pub lake_mask: Vec<u8>,
}

/// Fast local weather scalars — mirrors ECS [`ChunkWeather`] fields for future dual-write.
#[derive(Clone, Copy, Debug, Default)]
pub struct ChunkWeatherLocal {
    pub rain_intensity: f32,
    pub fog_density: f32,
    pub snow_depth: f32,
    pub wind_speed: f32,
    pub lightning_risk: f32,
    pub visibility_factor: f32,
    pub soil_moisture: f32,
}

impl ChunkWeatherLocal {
    #[must_use]
    pub fn from_chunk_weather_default() -> Self {
        Self {
            visibility_factor: 1.0,
            soil_moisture: 0.45,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ClipmapSampleRef {
    pub level_indices: [u32; 4],
    pub weights: [f32; 4],
}

#[derive(Clone, Debug, Default)]
pub struct AtmosphereState {
    pub local: ChunkWeatherLocal,
    pub clipmap_sample: ClipmapSampleRef,
}

#[derive(Clone, Debug, Default)]
pub struct ContaminationState {
    pub airborne: Vec<f32>,
    pub soil: Vec<f32>,
    pub waterborne: Vec<f32>,
    pub bioactive: Vec<f32>,
    pub radiation: Vec<f32>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AtmosphereCoupling {
    pub wind_transport: f32,
    pub humidity_binding: f32,
    pub thermal_exchange: f32,
}

#[derive(Clone, Debug, Default)]
pub struct DeformationState {
    pub height_delta: Vec<f32>,
    pub compaction: Vec<f32>,
    pub landslide_risk: Vec<f32>,
    pub last_mutation_tick: u64,
}

#[derive(Clone, Debug, Default)]
pub struct EcologyState {
    pub biomass: Vec<f32>,
    pub fuel_load: Vec<f32>,
    pub vegetation_class: Vec<u8>,
    pub fire_risk: Vec<f32>,
    pub stress: Vec<f32>,
}

#[derive(Clone, Debug, Default)]
pub struct ThermalState {
    pub surface_heat: Vec<f32>,
    pub subsurface_heat: Vec<f32>,
    pub ash_cover: Vec<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChunkActivationReason {
    FireFront,
    FloodSolve,
    Construction,
    Combat,
    PlayerProximity,
    HydrologyEvent,
}

/// Hot-region ECS mirror entity — slab remains authoritative for persist (WSS-SLAB-PR-3).
#[derive(Component, Clone, Copy, Debug)]
pub struct ActiveChunkRuntime {
    pub key: ChunkKey,
    pub activation_reason: ChunkActivationReason,
    pub deactivate_after_ticks: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DynamicOverlaySlice {
    pub mud: Vec<f32>,
    pub snow_accum: Vec<f32>,
    pub danger: Vec<f32>,
    pub congestion: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct WorldChunkState {
    pub key: ChunkKey,
    pub terrain: TerrainState,
    pub hydrology: HydrologyState,
    pub ecology: EcologyState,
    pub atmosphere: AtmosphereState,
    pub contamination: ContaminationState,
    pub coupling: AtmosphereCoupling,
    pub deformation: DeformationState,
    pub thermal: ThermalState,
    pub dynamic: DynamicOverlaySlice,
    pub sim_lod: u8,
    pub version: u32,
}

impl WorldChunkState {
    #[must_use]
    pub fn new_empty(key: ChunkKey, cell_count: usize) -> Self {
        let zf = || vec![0.0_f32; cell_count];
        let zu8 = || vec![0_u8; cell_count];
        Self {
            key,
            terrain: TerrainState {
                height: zf(),
                material_ids: vec![0; cell_count],
                porosity: zf(),
                hardness: zf(),
                ..Default::default()
            },
            hydrology: HydrologyState {
                water_depth: zf(),
                flow_velocity: vec![Vec2::ZERO; cell_count],
                sediment: zf(),
                salinity: zf(),
                saturation: zf(),
                ocean_mask: zu8(),
                river_mask: zu8(),
                lake_mask: zu8(),
            },
            ecology: EcologyState {
                biomass: zf(),
                fuel_load: zf(),
                vegetation_class: zu8(),
                fire_risk: zf(),
                stress: zf(),
            },
            atmosphere: AtmosphereState {
                local: ChunkWeatherLocal::from_chunk_weather_default(),
                ..Default::default()
            },
            contamination: ContaminationState {
                airborne: zf(),
                soil: zf(),
                waterborne: zf(),
                bioactive: zf(),
                radiation: zf(),
            },
            coupling: AtmosphereCoupling::default(),
            deformation: DeformationState {
                height_delta: zf(),
                compaction: zf(),
                landslide_risk: zf(),
                ..Default::default()
            },
            thermal: ThermalState {
                surface_heat: zf(),
                subsurface_heat: zf(),
                ash_cover: zf(),
            },
            dynamic: DynamicOverlaySlice {
                mud: zf(),
                snow_accum: zf(),
                danger: zf(),
                congestion: zf(),
            },
            sim_lod: 0,
            version: 1,
        }
    }

    #[must_use]
    pub fn cell_grid_matches_terrain(&self) -> bool {
        let n = self.terrain.height.len();
        n > 0
            && self.terrain.material_ids.len() == n
            && self.hydrology.water_depth.len() == n
            && self.hydrology.ocean_mask.len() == n
    }
}

/// Skeleton hydrate from chunk coord only (PR-1 test / witness refresh).
pub fn hydrate_skeleton_chunk(registry: &mut crate::substrate::registry::WorldSubstrateRegistry, coord: IVec2) {
    let key = ChunkKey::from(coord);
    let n = substrate_cell_count(SUBSTRATE_SKELETON_CELL_GRID);
    let state = WorldChunkState::new_empty(key, n);
    registry.chunks.insert(key, state);
    registry.chunks.set_resident(key, true);
}
