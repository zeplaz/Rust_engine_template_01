Right now your system is good for macro ecology pressure, but it cannot yet represent:

crown fires vs grass fires
ammo cookoff chains
fuel depot explosions
lithium battery thermal runaway
concrete heat-spall
steel weakening
toxic chemical plumes
smoke persistence
post-fire contamination
urban wildfire spread

The current ChunkEcology.fire_risk should become only one upstream contributor into a deeper combustion model.

You want a hierarchy like:

Biome
  -> Ecology
      -> Fuel Composition
          -> Burn Behavior
              -> Fire State
                  -> Smoke / Toxicity / Heat
                      -> Logistics + AI + Visibility
1. Add Physical Fuel Taxonomy

Instead of generic "forest burns", define combustible classes.

terrain/fire/fuel.rs
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FuelMaterialKind {
    Grass,
    Brush,
    Timber,
    Peat,

    Coal,
    Oil,
    Diesel,
    Gasoline,

    WoodStructure,
    ConcreteStructure,
    SteelStructure,

    Ammunition,
    ChemicalOxidizer,
    BatteryLithium,
    Plastic,

    Rubber,
    Fabric,
}

#[derive(Clone, Copy, Debug)]
pub struct FuelMaterialDef {
    pub ignition_temp_c: f32,

    /// Heat energy output.
    pub burn_energy: f32,

    /// Seconds to fully consume at baseline oxygen.
    pub burn_duration: f32,

    /// Smoke generation.
    pub smoke_density: f32,

    /// Toxicity.
    pub toxic_output: f32,

    /// Explosion pressure.
    pub explosive_force: f32,

    /// Persistent ground contamination.
    pub contamination: f32,

    /// Can self-propagate without external fuel.
    pub thermal_runaway: bool,

    /// Structural weakening.
    pub structural_damage_rate: f32,
}

pub fn fuel_material_def(kind: FuelMaterialKind) -> FuelMaterialDef {
    match kind {
        FuelMaterialKind::Grass => FuelMaterialDef {
            ignition_temp_c: 230.0,
            burn_energy: 0.35,
            burn_duration: 12.0,
            smoke_density: 0.15,
            toxic_output: 0.0,
            explosive_force: 0.0,
            contamination: 0.0,
            thermal_runaway: false,
            structural_damage_rate: 0.0,
        },

        FuelMaterialKind::Timber => FuelMaterialDef {
            ignition_temp_c: 300.0,
            burn_energy: 0.7,
            burn_duration: 120.0,
            smoke_density: 0.5,
            toxic_output: 0.1,
            explosive_force: 0.0,
            contamination: 0.0,
            thermal_runaway: false,
            structural_damage_rate: 0.4,
        },

        FuelMaterialKind::Gasoline => FuelMaterialDef {
            ignition_temp_c: 210.0,
            burn_energy: 1.0,
            burn_duration: 40.0,
            smoke_density: 0.8,
            toxic_output: 0.3,
            explosive_force: 0.85,
            contamination: 0.5,
            thermal_runaway: false,
            structural_damage_rate: 1.0,
        },

        FuelMaterialKind::BatteryLithium => FuelMaterialDef {
            ignition_temp_c: 120.0,
            burn_energy: 0.95,
            burn_duration: 400.0,
            smoke_density: 1.0,
            toxic_output: 0.95,
            explosive_force: 0.25,
            contamination: 0.7,
            thermal_runaway: true,
            structural_damage_rate: 0.8,
        },

        FuelMaterialKind::Ammunition => FuelMaterialDef {
            ignition_temp_c: 170.0,
            burn_energy: 0.9,
            burn_duration: 60.0,
            smoke_density: 0.4,
            toxic_output: 0.2,
            explosive_force: 1.0,
            contamination: 0.1,
            thermal_runaway: true,
            structural_damage_rate: 1.0,
        },

        _ => FuelMaterialDef {
            ignition_temp_c: 300.0,
            burn_energy: 0.5,
            burn_duration: 60.0,
            smoke_density: 0.4,
            toxic_output: 0.0,
            explosive_force: 0.0,
            contamination: 0.0,
            thermal_runaway: false,
            structural_damage_rate: 0.2,
        },
    }
}
2. Forest Representation Must Become Fuel Layers

Right now forest is only:

Forest -> green color

That is insufficient.

You need:

Ground fuel
Shrub layer
Canopy layer
Dead biomass
Moisture
Species mix
Add Vegetation Fuel Structure
#[derive(Clone, Debug)]
pub struct VegetationFuelLayer {
    pub live_biomass: f32,
    pub dead_biomass: f32,
    pub moisture: f32,

    pub ignition_bias: f32,

    pub fuel_kind: FuelMaterialKind,
}

#[derive(Component, Clone, Debug)]
pub struct ChunkFuelProfile {
    pub grass: VegetationFuelLayer,
    pub brush: VegetationFuelLayer,
    pub canopy: VegetationFuelLayer,

    pub peat_depth: f32,

    pub suppression_difficulty: f32,
}
3. Fire Must Be Cell-Based Eventually

Chunk scalar heat is fine for strategic sim.

But actual wildfire behavior needs:

temperature field
fuel load
oxygen/wind
spread vectors
ember transport
spot ignition

You can keep chunk-level authority while embedding coarse fire cells.

Chunk Fire Grid
pub struct FireCell {
    pub temperature: f32,

    pub active_flame: bool,

    pub fuel_remaining: f32,

    pub smoke_density: f32,

    pub toxic_density: f32,

    pub ember_pressure: f32,
}
4. Buildings Need Burn Profiles

Not all structures burn equally.

building/fire_profile.rs
#[derive(Clone, Debug)]
pub struct StructureFireProfile {
    pub primary_material: FuelMaterialKind,

    pub fuel_load: f32,

    pub ignition_resistance: f32,

    pub collapse_threshold: f32,

    pub internal_pressure_risk: f32,

    pub emits_toxic_smoke: bool,

    pub explosion_chain: bool,
}
Example Profiles
pub fn fuel_depot_profile() -> StructureFireProfile {
    StructureFireProfile {
        primary_material: FuelMaterialKind::Gasoline,
        fuel_load: 1.0,
        ignition_resistance: 0.1,
        collapse_threshold: 0.2,
        internal_pressure_risk: 1.0,
        emits_toxic_smoke: true,
        explosion_chain: true,
    }
}

pub fn ammo_dump_profile() -> StructureFireProfile {
    StructureFireProfile {
        primary_material: FuelMaterialKind::Ammunition,
        fuel_load: 0.9,
        ignition_resistance: 0.2,
        collapse_threshold: 0.1,
        internal_pressure_risk: 1.0,
        emits_toxic_smoke: false,
        explosion_chain: true,
    }
}

pub fn lithium_battery_warehouse() -> StructureFireProfile {
    StructureFireProfile {
        primary_material: FuelMaterialKind::BatteryLithium,
        fuel_load: 1.0,
        ignition_resistance: 0.05,
        collapse_threshold: 0.15,
        internal_pressure_risk: 0.4,
        emits_toxic_smoke: true,
        explosion_chain: false,
    }
}
5. Smoke Needs Strategic Gameplay Impact

Smoke should affect:

AI visibility
thermal imaging
helicopter operations
civilian panic
logistics
pathfinding risk
oxygen hazards indoors
artillery spotting
Add Atmospheric Smoke
#[derive(Component, Clone, Copy, Debug)]
pub struct ChunkSmokeField {
    pub density: f32,
    pub toxicity: f32,
    pub visibility_penalty: f32,
}
6. Logistics Should Consider Smoke + Fire

Your current pathfinding is already ready for this.

Extend:

pub struct LogisticsEnvironmentSample {
    pub fire_heat: f32,
    pub fire_risk: f32,
    pub biomass: f32,

    pub smoke_density: f32,
    pub toxicity: f32,

    pub explosion_risk: f32,
}

Then:

cost_mul *= 1.0 + smoke_density * 0.4;
cost_mul *= 1.0 + explosion_risk * 2.0;
7. Terrain Preview Colors Need Dynamic Fire Overlay

Current preview colors are static.

You need blended overlays:

forest + drought
forest + active flame
forest + ash
urban + smoke
fuel depot + explosion hazard
Dynamic Preview
pub fn blend_fire_overlay(
    base: [u8; 4],
    heat: f32,
    smoke: f32,
) -> [u8; 4] {
    let mut r = base[0] as f32;
    let mut g = base[1] as f32;
    let mut b = base[2] as f32;

    r += heat * 180.0;
    g *= 1.0 - smoke * 0.7;
    b *= 1.0 - smoke * 0.85;

    [
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
        255,
    ]
}
8. Mission Builder Importance

Mission builders need:

"flammable region"
"volatile storage"
"dry season preset"
"urban firestorm risk"
"chemical hazard map"
"civilian evacuation pressure"

So expose fire ecology as editable scenario layers.

Editor Layer Types
pub enum ScenarioHazardLayer {
    WildfireRisk,
    FuelStorage,
    AmmoStorage,
    ChemicalHazard,
    SmokeZone,
    EvacuationRisk,
}
9. Future Critical Upgrade: Ember Simulation

Most wildfire spread is not adjacent flames.

It is:

wind carries embers
embers ignite new spot fires
spot fires merge

So eventually:

pub struct EmberParticle {
    pub position: Vec2,
    pub heat: f32,
    pub lifetime: f32,
}

Chunk-level probabilistic ember transport is enough initially.

Recommended Architecture

Keep:

ChunkEcology

as strategic ecosystem state.

Add:

ChunkFuelProfile
ChunkSmokeField
ChunkFireGrid
StructureFireProfile
FuelMaterialDef

This gives:

forests
industrial disasters
urban fires
military depot explosions
persistent smoke warfare
environmental collapse
civilian disaster scenarios