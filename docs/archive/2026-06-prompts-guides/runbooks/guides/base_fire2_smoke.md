ChunkWeather
ChunkEcology
ChunkSurfaceFire
LogisticsEnvironmentSample

The next step is creating a shared:

Atmospheric Field Authority

that drives:

rendering
particles
AI visibility
thermal signatures
spotting
sensors
lighting
gameplay hazards
wind transport

from one coherent source.

TARGET ARCHITECTURE
Simulation Layer
----------------
ChunkWeather
ChunkEcology
ChunkSurfaceFire
ChunkSmokeField
ChunkAtmosphere

↓ generates

Atmospheric Cells
-----------------
Smoke Density
Fog Density
Heat Distortion
Toxicity
Ash
Ember Pressure

↓ drives

Rendering
----------
Volumetric fog
Ground haze
Smoke columns
Heat shimmer
Light scattering
Fire glow

↓ drives

Particles
----------
Embers
Ash
Sparks
Smoke wisps
Debris
Explosion plumes

↓ drives

Gameplay
---------
Vision attenuation
Thermal masking
Pathfinding risk
Civilian panic
Aircraft danger
Sensor degradation
1. CORE ATMOSPHERIC FIELD

You need one unified field resource.

atmosphere/field.rs
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct AtmosphereCell {
    /// Visual smoke opacity.
    pub smoke_density: f32,

    /// Natural fog/moisture haze.
    pub fog_density: f32,

    /// Dangerous inhalation/toxic load.
    pub toxicity: f32,

    /// Air temperature distortion.
    pub heat_distortion: f32,

    /// Floating ash density.
    pub ash_density: f32,

    /// Fire transport pressure.
    pub ember_density: f32,

    /// Combined visibility attenuation.
    pub visibility: f32,
}

#[derive(Resource)]
pub struct AtmosphereField {
    pub size: UVec2,
    pub cells: Vec<AtmosphereCell>,
}

impl AtmosphereField {
    #[inline]
    pub fn idx(&self, x: u32, y: u32) -> usize {
        (y * self.size.x + x) as usize
    }

    pub fn cell(&self, x: u32, y: u32) -> AtmosphereCell {
        self.cells[self.idx(x, y)]
    }

    pub fn cell_mut(&mut self, x: u32, y: u32) -> &mut AtmosphereCell {
        let i = self.idx(x, y);
        &mut self.cells[i]
    }
}
2. ATMOSPHERIC SIMULATION PASS

This combines:

weather
fire
ecology
terrain
wind

into final atmosphere.

atmosphere/sim.rs
pub fn atmosphere_field_update(
    mut field: ResMut<AtmosphereField>,
    chunk_q: Query<(
        &Chunk,
        &ChunkWeather,
        &ChunkEcology,
        Option<&ChunkSurfaceFire>,
    )>,
) {
    for (chunk, wx, eco, fire) in &chunk_q {
        let heat = fire.map(|f| f.heat).unwrap_or(0.0);

        let smoke_gen =
            heat * eco.biomass * (1.0 + eco.fire_risk);

        let fog =
            wx.fog_density
            + wx.rain_intensity * 0.2;

        let toxicity =
            smoke_gen * 0.45;

        let ember_density =
            heat * wx.wind_speed * eco.biomass;

        let visibility =
            1.0
            - smoke_gen * 0.7
            - fog * 0.45;

        let cx = chunk.coord.x as u32;
        let cy = chunk.coord.y as u32;

        let c = field.cell_mut(cx, cy);

        c.smoke_density =
            smoke_gen.clamp(0.0, 1.0);

        c.fog_density =
            fog.clamp(0.0, 1.0);

        c.toxicity =
            toxicity.clamp(0.0, 1.0);

        c.ember_density =
            ember_density.clamp(0.0, 1.0);

        c.visibility =
            visibility.clamp(0.05, 1.0);

        c.heat_distortion =
            heat * 0.8;

        c.ash_density =
            smoke_gen * 0.35;
    }
}
3. WIND ADVECTION

Smoke cannot remain static.

You need directional movement.

smoke_transport.rs
pub fn advect_smoke(
    time: Res<Time>,
    mut field: ResMut<AtmosphereField>,
    wind: Res<GlobalWind>,
) {
    let dt = time.delta_secs();

    let size = field.size;

    let old = field.cells.clone();

    for y in 0..size.y {
        for x in 0..size.x {

            let fx =
                x as f32
                - wind.direction.x * wind.speed * dt;

            let fy =
                y as f32
                - wind.direction.y * wind.speed * dt;

            let sx = fx.floor() as i32;
            let sy = fy.floor() as i32;

            if sx < 0 || sy < 0 {
                continue;
            }

            if sx >= size.x as i32 || sy >= size.y as i32 {
                continue;
            }

            let src =
                old[(sy as u32 * size.x + sx as u32) as usize];

            let dst =
                field.cell_mut(x, y);

            dst.smoke_density =
                src.smoke_density * 0.985;

            dst.toxicity =
                src.toxicity * 0.992;

            dst.ash_density =
                src.ash_density * 0.98;
        }
    }
}
4. PARTICLE SYSTEM INTEGRATION

DO NOT spawn particles directly from every fire entity.

That scales horribly.

Instead:

AtmosphereField
    ↓
Emission Zones
    ↓
GPU/CPU Particle Emitters
Fire Particle Emitter
#[derive(Component)]
pub struct FireEmitter {
    pub intensity: f32,
    pub smoke_rate: f32,
    pub ember_rate: f32,
}
Spawn Emitters From Fire Heat
pub fn sync_fire_emitters(
    mut commands: Commands,
    q: Query<(Entity, &ChunkSurfaceFire)>,
) {
    for (e, fire) in &q {

        commands.entity(e).insert(
            FireEmitter {
                intensity: fire.heat,
                smoke_rate: fire.heat * 45.0,
                ember_rate: fire.heat * 8.0,
            }
        );
    }
}
5. PARTICLE TYPES

You need layered particles.

particle/types.rs
pub enum AtmosphereParticleKind {
    Smoke,
    Ash,
    Ember,
    Spark,
    Dust,
    ToxicGas,
    Steam,
}
Shared Particle Component
#[derive(Component)]
pub struct AtmosphereParticle {
    pub velocity: Vec3,
    pub lifetime: f32,
    pub density: f32,
    pub temperature: f32,
    pub kind: AtmosphereParticleKind,
}
6. GPU INSTANCING

DO NOT use sprite entities for massive smoke.

You need:

low-count ECS controllers
high-count GPU instances

Especially for:

smoke
embers
ash

Use:

custom material
instanced quads
or compute-driven particles

Eventually:

bevy_hanabi

or custom WGSL pipeline.

7. VISUAL LAYERS

You need multiple rendering layers.

A. Ground Smoke

Low altitude.

fog banks
creeping wildfire smoke
urban haze
B. Smoke Columns

Vertical billowing.

industrial fires
fuel depot plume
forest crown fire
C. Heat Distortion

Screen-space refraction.

wildfire shimmer
jet exhaust
burning metal
D. Ashfall

Slow drifting particles.

reduced visibility
post-fire atmosphere
nuclear winter style events
8. VISIBILITY INTEGRATION

This is critical.

Your AI and gameplay must use the SAME field.

vision.rs
pub fn visibility_between(
    a: Vec2,
    b: Vec2,
    atmosphere: &AtmosphereField,
) -> f32 {
    let mut vis = 1.0;

    // sample along line
    // accumulate smoke/fog attenuation

    vis
}
9. FIRE LIGHTING

Fire should emit:

dynamic orange lighting
flicker
smoke shadowing
nighttime visibility
Fire Light Emission (sim metadata)

Chunk entities carry [`FireLightEmission`](crate::systems::fire::FireLightEmission): radius, base/current intensity, flicker — **not** a [`PointLight`]. Render pulls [`RequestLocalLight`](crate::render::RequestLocalLight) via [`extract_fire_light_emission_to_requests`](crate::render::extract_fire_lights::extract_fire_light_emission_to_requests) into the pooled local light path.

```rust
#[derive(Component)]
pub struct FireLightEmission {
    pub radius: f32,
    pub base_intensity: f32,
    pub current_intensity: f32,
    pub flicker_strength: f32,
    pub flicker_phase: f32,
}
```

Intensity tied to:

fire.heat
* biomass
* fuel_energy

10. MISSION BUILDER OVERLAYS

Expose atmosphere overlays.

Overlays
Smoke density
Visibility
Toxicity
Heat map
Ember spread
Fog pressure
11. RENDER STRATEGY

You already have:

Bevy UI = player HUD
egui = dev/editor

Do same separation here.

PLAYER VIEW

Stylized + performant.

smoke volumes
atmospheric haze
dramatic fire
DEV VIEW

Debug overlays:

grid smoke density
wind vectors
ember probability
particle counts
visibility heatmap

Use egui debug windows + Bevy gizmos.

12. CRITICAL FUTURE FEATURE

You eventually want:

Atmospheric Pressure Fronts

for:

wildfire weather
smoke inversion layers
chemical gas drift
sandstorms
artillery smoke
battlefield obscuration

This becomes a strategic simulation layer.

Recommended Immediate Next Modules as concepts

Build next:
if not already ingerated 
atmosphere/
    field.rs
    sim.rs
    transport.rs
    visibility.rs

fire/
    fuel.rs
    emitter.rs

particles/
    atmosphere_particles.rs

Then plan for and build out :

render/
    smoke_volume_pipeline.rs
    heat_distortion.wgsl
    volumetric_fog.wgsl

part 2
test fire system 
Purpose:
- Validate atmospheric field simulation
- Validate smoke/fog rendering hooks
- Validate fire → atmosphere → particles pipeline
- Validate logistics visibility penalties
- Validate future wildfire scaling architecture

This scene is NOT gameplay-first.
It is a systems validation sandbox.

---

# 1. Goals

The test scene must demonstrate:

- Forest fire spread
- Smoke accumulation
- Wind-driven smoke transport
- Fog layering
- Ember particle emission
- Dynamic fire lighting
- Visibility degradation
- Different burn materials
- Atmospheric overlays
- Performance instrumentation

---

# 2. Required Scene Features

The scene should contain:

| Region | Purpose |
|---|---|
| Dense forest | Crown fire testing |
| Grassland | Fast low-intensity fire |
| Wetland | Fire suppression area |
| Fuel depot | Explosion + toxic smoke |
| Ammo dump | Cookoff chain |
| Lithium battery storage | Thermal runaway |
| Urban block | Structure fire propagation |
| Fog valley | Visibility tests |
| Elevated ridge | Smoke column visualization |

---

# 3. World Layout

Recommended map size:

```text
256 x 256 tiles
```

Chunk size:

```text
32 x 32
```

---

# 4. Suggested Layout

```text
####################################################

Mountain Ridge
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Dense Forest
TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT

Grassland
....................................................

Fuel Depot
FFFFF

Ammo Dump
AAAAA

Urban Zone
UUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUU

Fog Basin
~~~~~~~~~~~~~~~

Wetland
WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW

####################################################
```

Legend:

| Symbol | Meaning |
|---|---|
| T | Forest |
| . | Grassland |
| F | Fuel depot |
| A | Ammo storage |
| U | Urban |
| ~ | Fog basin |
| W | Wetland |

---

# 5. Create Terrain Families

Add these terrain families:

```rust
Forest
DenseForest
Grassland
Wetland
Urban
Industrial
Mountain
FogBasin
```

---

# 6. Material Definitions

Every material must define:

```rust
burn_energy
smoke_output
toxic_output
traction_mod
thermal_mass
```

---

# Example

```rust
MaterialDef {
    name: "PineForest".into(),

    preview_color: [20, 90, 20, 255],

    sim: hashmap! {
        "burn_energy" => 0.85,
        "smoke_output" => 0.75,
        "toxic_output" => 0.1,
        "traction_mod" => 1.1,
    }
}
```

---

# 7. Add Fuel Profiles

Spawn chunk fuel profiles.

---

# Example

```rust
commands.spawn((
    ChunkFuelProfile {
        grass: VegetationFuelLayer {
            live_biomass: 0.3,
            dead_biomass: 0.2,
            moisture: 0.1,
            ignition_bias: 0.8,
            fuel_kind: FuelMaterialKind::Grass,
        },

        brush: VegetationFuelLayer {
            live_biomass: 0.5,
            dead_biomass: 0.4,
            moisture: 0.15,
            ignition_bias: 0.9,
            fuel_kind: FuelMaterialKind::Brush,
        },

        canopy: VegetationFuelLayer {
            live_biomass: 0.9,
            dead_biomass: 0.6,
            moisture: 0.2,
            ignition_bias: 1.0,
            fuel_kind: FuelMaterialKind::Timber,
        },

        peat_depth: 0.2,

        suppression_difficulty: 0.75,
    }
));
```

---

# 8. Add Ignition Sources

Spawn multiple fire sources.

---

# Forest Fire

```rust
ChunkSurfaceFire {
    heat: 0.65,
}
```

---

# Fuel Depot Fire

```rust
ChunkSurfaceFire {
    heat: 1.0,
}
```

---

# Ammo Cookoff

```rust
ChunkSurfaceFire {
    heat: 0.85,
}
```

---

# 9. Wind Setup

Create strong directional wind.

```rust
GlobalWind {
    direction: Vec2::new(1.0, 0.25).normalize(),
    speed: 12.0,
}
```

Expected result:

- smoke drifts east
- embers leap forest gaps
- valley fog compresses smoke

---

# 10. Atmosphere Resource

Initialize atmosphere field.

```rust
commands.insert_resource(
    AtmosphereField {
        size: UVec2::new(256, 256),
        cells: vec![
            AtmosphereCell::default();
            256 * 256
        ],
    }
);
```

---

# 11. Visualization Modes

Add runtime visualization modes.

Keyboard:

| Key | Mode |
|---|---|
| F1 | Terrain |
| F2 | Fire Heat |
| F3 | Smoke Density |
| F4 | Fog Density |
| F5 | Toxicity |
| F6 | Visibility |
| F7 | Ember Pressure |
| F8 | Logistics Cost |
| F9 | AI Visibility |

---

# 12. Tile Overlay Rendering

Add overlay renderer.

---

# Example

```rust
pub fn atmosphere_overlay_color(
    cell: AtmosphereCell,
    mode: OverlayMode,
) -> [u8; 4] {
    match mode {

        OverlayMode::Smoke => {
            let v =
                (cell.smoke_density * 255.0) as u8;

            [v, v, v, 255]
        }

        OverlayMode::Toxicity => {
            let g =
                (cell.toxicity * 255.0) as u8;

            [0, g, 0, 255]
        }

        OverlayMode::Visibility => {
            let r =
                ((1.0 - cell.visibility) * 255.0) as u8;

            [r, 0, 0, 255]
        }

        _ => [255, 0, 255, 255]
    }
}
```

---

# 13. Debug UI (egui)

Create dev panel.

---

# Required Metrics

```text
Total Active Fires
Smoke Particle Count
Atmosphere Update Time
Particle Spawn Rate
Visibility Mean
Max Toxicity
Fire Spread Rate
Chunk Updates/sec
```

---

# Example

```rust
egui::Window::new("Atmosphere Debug")
    .show(ctx, |ui| {

        ui.label(format!(
            "Active Fires: {}",
            metrics.active_fires
        ));

        ui.label(format!(
            "Smoke Density Avg: {:.2}",
            metrics.mean_smoke
        ));

        ui.label(format!(
            "Particle Count: {}",
            metrics.particle_count
        ));
    });
```

---

# 14. Particle Validation

The scene must visibly demonstrate:

| Effect | Requirement |
|---|---|
| Smoke columns | Vertical rise |
| Embers | Wind drift |
| Ash | Slow atmospheric fall |
| Sparks | Short-lived burst |
| Heat shimmer | Near intense fires |
| Toxic plume | Green/yellow tint |

---

# 15. Lighting Validation

At night:

- fire illuminates terrain
- smoke partially occludes light
- fog diffuses fire glow

---

# 16. Gameplay Validation

Test:

| Feature | Expected |
|---|---|
| AI LOS | Reduced in smoke |
| Logistics | Avoids active fire |
| Visibility | Degrades in fog |
| Thermal zones | Hotspots detectable |
| Urban fire | Slower spread |
| Grass fire | Rapid spread |

---

# 17. Performance Targets

Initial target:

| Metric | Goal |
|---|---|
| Active fire chunks | 256+ |
| Smoke particles | 20k+ |
| Stable FPS | 60 |
| Atmosphere update | <2 ms |
| Pathfinding update | <1 ms |

---

# 18. ECS Scheduling

Recommended order:

```text
Weather Update
→ Ecology Update
→ Fire Update
→ Atmosphere Field + Advect
→ Emitters (fuel-aware)
→ Particles
→ Coupling
→ VisualExtract (sim → render snapshots)
→ RenderPrep (shader handles / GPU bridge)
→ UI
```

---

# 19. Field ownership & unified fuel row

Simulation should grow by **who owns which field**, not by feature-named mega-systems.

| Layer / field | Primary writers (CPU today) | Primary readers |
|---|---|---|
| Terrain substrate | materialization / hydrology | ecology, fire ignition |
| Ecology / vegetation | `EcologyPlugin`, vegetation integrators | fuel profile, `FireFuelField` |
| Fuel strata | `chunk_fuel_profile_tick` | combustion, surface fire |
| Fire heat / overlay | `FirePlugin` (surface + overlay ticks) | smoke field, emitters, atmosphere blend |
| Smoke chunk grid | `ChunkSmokeField` | atmosphere field fill, GPU debug |
| Global atmosphere | `AtmospherePlugin` (fill, advect, blend) | logistics sample, visibility, render prep |
| Particles / mesh VFX | future extraction → render | **no** authoritative smoke mass |
| **VisualExtract** | `publish_sim_visual_extract` (smoke) + `FireVisualFramePlugin` → `extract_fire_visual_frame` (fire) | `SimChunkSmokeVisualExtract`; CPU `FireVisualFrame` → `SharedOverlayFieldBuffers` (heat map only from frame); render `ExtractResource` + `FireVisualGpuInstanceStorage`; `SimFireEmitterVisualExtract` (main-world mirror, not extracted) |

**Unified fuel row:** [`FuelLayer`](../../src/terrain/fire/fuel_layer.rs) is a single normalized struct (surface / shrub / canopy fuel, moisture, volatility, toxic smoke, burn temperature, ember proxy). Use [`FuelLayer::from_vegetation_strata`](../../src/terrain/fire/fuel_layer.rs) via [`ChunkFuelProfile::to_fuel_layer`](../../src/systems/fire/chunk_fuel_profile.rs) for wildland aggregates; use presets (`FuelLayer::forest`, `fuel_dump`, `battery_facility`, `concrete_building`) for industrial / structure scenarios until per-cell fuel grids land. Helpers `visual_fire_height` and `ember_rate_base` are intentionally cheap hooks for render and emitter tuning—wind still applied outside.

Particles remain **garnish**; authoritative smoke stays **field-first** (`ChunkSmokeField` → `AtmosphereField`).

---

# 20. Success Criteria

The test scene succeeds when:

- Smoke visibly advects with wind
- Fog and smoke blend correctly
- Different fuel types visibly behave differently
- Visibility overlays match rendered world
- Fire spreads according to ecology/fuel
- Pathfinding avoids hazards
- FPS remains stable under stress
- Debug overlays match simulation state

---

# 21. Future Expansion Hooks

This scene should later support:

- rainstorms
- aircraft retardant drops
- artillery smoke
- chemical gas release
- nuclear ash
- sandstorms
- civilian evacuation simulation
- firefighting vehicles
- dynamic weather fronts

---

# 22. Recommended Initial Milestone

Implement in this order:

```text
1. Terrain
2. Chunk atmosphere field
3. Fire heat overlay
4. Smoke transport
5. Basic particles
6. Visibility penalties
7. Debug overlays
8. Dynamic lighting
9. Material-specific fires
10. Explosion chains
```