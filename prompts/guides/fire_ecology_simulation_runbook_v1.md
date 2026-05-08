# Fire ecology simulation runbook `v1`

> **STATUS:** Draft **v1** — design scaffold. Implementation **Pending**.

Version: `v1.0.0`  
Parent: [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md) · [`flora_ecology_runbook_v1.md`](flora_ecology_runbook_v1.md) (vegetation / ecology) · [`weather_simulation_runbook_v1.md`](weather_simulation_runbook_v1.md) · [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md)

---

## 1. Purpose

Create a scalable fire simulation system integrating:

- vegetation ecology
- weather
- terrain
- structures
- military / environmental destruction
- suppression
- economy / logistics
- GPU visualization

while preserving:

- chunk scalability
- deterministic gameplay
- LOD simulation
- ECS authority separation

---

## 2. Design philosophy

**Fire is NOT:**

- particle effects only

**Fire IS:**

- energy propagation
- fuel consumption
- atmospheric interaction
- terrain coupling
- infrastructure damage
- ecology succession

Visual particles are only a rendered consequence.

---

## 3. Core architecture

```
CPU ECS
 ├── Fire propagation authority
 ├── Fuel consumption
 ├── Structure damage
 ├── Ecology transitions
 ├── Smoke generation rates
 ├── Suppression logic
 └── Strategic consequences
            │
            ▼
GPU derived fields
 ├── smoke transport
 ├── embers
 ├── ash
 ├── flame particles
 ├── heat haze
 ├── volumetric fog
 └── lighting
```

---

## 4. Simulation layers

| Layer | Responsibility |
|:---|:---|
| **Fire state** | ignition / spread / extinguish |
| **Fuel model** | biomass / material energy |
| **Weather coupling** | wind / humidity / rain |
| **Terrain coupling** | slope / elevation / barriers |
| **Structure coupling** | burnability / damage |
| **Ecology coupling** | succession / regrowth |
| **VFX layer** | GPU rendering only |

---

## 5. Spatial scale rules

### Strategic scale (chunk fields)

Primary truth.

```rust
pub struct FireFieldCell {
    pub heat: f32,
    pub active_flame: f32,
    pub smoldering: f32,

    pub fuel_remaining: f32,
    pub ignition_probability: f32,

    pub moisture: f32,
    pub oxygen_factor: f32,

    pub smoke_output: f32,
    pub ember_output: f32,

    pub suppression_strength: f32,

    pub last_burn_time: u64,
}
```

**Resolution example:** 1–8 m cells inside chunk overlays — **not** per-tree globally.

### Local scale (high-detail ECS)

Activated near:

- players
- combat
- structures
- vehicles
- active suppression

Here instantiate:

- tree entities
- localized flame fronts
- collapsing structures
- detailed debris
- firefighters
- hose streams
- localized oxygen pockets

---

## 6. Fuel system

### Fuel categories

```rust
pub enum FuelType {
    Grass,
    Shrub,
    ForestLitter,
    YoungForest,
    MatureForest,

    Peat,
    DrySoil,

    WoodStructure,
    ReinforcedWood,
    FuelStorage,

    Oil,
    Chemical,
    Electrical,

    UrbanDebris,
}
```

### Fuel properties

```rust
pub struct FuelProfile {
    pub ignition_temp: f32,
    pub burn_rate: f32,
    pub energy_density: f32,

    pub moisture_absorption: f32,
    pub ember_generation: f32,
    pub smoke_generation: f32,

    pub reignition_chance: f32,

    pub suppression_resistance: f32,
}
```

---

## 7. Vegetation coupling

From ecology fields:

```rust
pub struct VegetationCell {
    pub biomass: f32,
    pub species_mix: SpeciesMix,
    pub moisture_retention: f32,
    pub shade_factor: f32,
    pub age_class: f32,

    pub fire_risk: f32,
    pub regrowth_rate: f32,
}
```

Fire consumes:

- biomass
- canopy
- litter
- root stability

which feeds back into:

- erosion
- hydrology
- flood risk
- dust generation

---

## 8. Weather coupling

### Wind

Wind heavily biases spread direction.

```rust
pub struct WindSample {
    pub velocity: Vec2,
    pub turbulence: f32,
    pub humidity: f32,
}
```

**Spread factor:**

`base_spread × wind_alignment × fuel_factor × slope_factor × humidity_factor`

### Rain

Rain reduces:

- ignition chance
- ember survival
- smoke density
- flame intensity

Heavy rain can:

- terminate crown fires
- create steam / smoke bursts

### Drought

Drought raises:

- ignition probability
- spread velocity
- ember generation
- underground smoldering

---

## 9. Terrain coupling

### Slope

Fire spreads uphill faster: **upslope spread multiplier** derived from terrain gradient and wind alignment.

### Terrain materials

| Material | Fire behavior |
|:---|:---|
| rock | barrier |
| wetland | suppressive |
| dry grass | fast spread |
| dense forest | sustained burn |
| sand | low spread |
| snow | suppressive |
| urban rubble | intermittent |

### Firebreaks

Supported naturally:

| Feature | Effect |
|:---|:---|
| roads | partial barrier |
| rivers | major barrier |
| trenches | strong barrier |
| cleared zones | suppression |
| concrete | block |
| burned ground | low fuel |

---

## 10. Structure fire system

### Building materials

```rust
pub enum BuildingMaterial {
    Wood,
    Brick,
    Concrete,
    Steel,
    Composite,
}
```

### Structural fire state

```rust
pub struct FireDamageState {
    pub ignition_level: f32,
    pub structural_integrity: f32,
    pub internal_temperature: f32,

    pub smoke_production: f32,
    pub collapse_risk: f32,
}
```

### Secondary hazards

Structures may generate:

- toxic smoke
- fuel explosions
- electrical fires
- ammunition cookoff
- infrastructure outages

---

## 11. Military / conflict interaction

Remain **simulation-oriented** rather than ideology-oriented.

### Tactical fire usage

Possible gameplay / system interactions:

| Action | Simulation result |
|:---|:---|
| forest burning | remove concealment |
| scorched terrain | deny movement / resources |
| smoke generation | visibility reduction |
| incendiary attack | infrastructure disruption |
| fuel depot ignition | chain explosions |
| trench fire | area denial |
| controlled burn | defensive line |

### Escalation / political systems

Strategic simulation can track:

```rust
pub struct IncidentProfile {
    pub civilian_damage: f32,
    pub ecological_damage: f32,
    pub infrastructure_damage: f32,

    pub treaty_violation_score: f32,
    pub media_visibility: f32,
}
```

This enables diplomacy effects, sanctions, reputation shifts, escalation pressure — without hardcoding ideology.

---

## 12. Smoke system

### CPU truth

CPU only tracks **smoke generation rate** — not individual particles.

### GPU smoke field

Compute sim:

```wgsl
struct SmokeCell {
    density: f32,
    velocity: vec2<f32>,
    temperature: f32,
}
```

GPU handles: advection, turbulence, diffusion, rendering.

---

## 13. Ember system

### Hybrid model

- **CPU:** ember intensity scalar, spread probability
- **GPU:** actual ember particles

### Ember spot fires

```
active fire
  → generates embers
  → wind transport
  → random landing
  → ignition checks
```

Critical for: crown fires, urban spread, wildfire storms.

---

## 14. Fire progression states

```rust
pub enum FireState {
    Ignition,
    Growing,
    Sustained,
    CrownFire,
    Smoldering,
    BurnedOut,
}
```

---

## 15. Ecology recovery

After burn:

`burn severity → soil damage → erosion increase → regrowth suppression → succession reset`

### Burn severity

```rust
pub enum BurnSeverity {
    Light,
    Moderate,
    Severe,
    Sterilized,
}
```

---

## 16. Chunk scheduling

Fire uses adaptive tick rates.

| Situation | Tick rate |
|:---|:---|
| dormant chunk | low |
| smoldering | medium |
| active fire | high |
| near player | very high |

Integrated with [`chunk_scheduler_runbook_v1.md`](chunk_scheduler_runbook_v1.md).

---

## 17. GPU VFX integration

### Hanabi — excellent candidates

- sparks
- embers
- debris bursts
- local smoke emitters
- flame tongues
- ignition bursts
- explosions
- burning vehicle FX

### Custom GPU systems — when and why

| System | Reason |
|:---|:---|
| smoke fields | persistent world sim |
| atmospheric haze | large-scale |
| heat distortion | fullscreen |
| ash transport | chunked persistent |
| wind transport | shared field |
| fog / fire coupling | global |

---

## 18. Recommended GPU pipeline

```
CPU fire fields
    ↓
GPU upload
    ↓
compute:
    smoke advection
    ember transport
    heat shimmer
    ash transport
    volumetric accumulation
    ↓
Hanabi local emitters
    ↓
composite render passes
```

---

## 19. Data-oriented chunk layout

```rust
pub struct ChunkFireOverlay {
    pub heat: Vec<f32>,
    pub fuel: Vec<f32>,
    pub moisture: Vec<f32>,
    pub smoke_output: Vec<f32>,
    pub suppression: Vec<f32>,
}
```

SOA layout preferred.

---

## 20. Long-term extensions

Future-compatible:

- underground fires
- peat fires
- chemical contamination
- radiation fires
- oxygen-deprived fires
- volcanic ignition
- lightning storms
- dynamic weather fronts
- firefighting logistics
- water bomber systems
- evacuation simulation

---

## 21. Recommended immediate milestones

### Phase 1

Implement CPU field simulation:

- ignition
- spread
- fuel
- moisture
- wind bias
- roads / rivers barriers

### Phase 2

Add GPU smoke:

- density field
- advection
- fullscreen compositing

### Phase 3

Add Hanabi local FX:

- embers
- sparks
- debris
- flame bursts

### Phase 4

Add ecology recovery:

- regrowth
- erosion coupling
- hydrology changes

---

## 22. Critical design rule

The **authoritative** fire simulation must **never** depend on:

- particles
- renderer state
- framerate
- visibility
- GPU readback

All gameplay truth remains:

- CPU deterministic
- chunk authoritative
- serializable
- replay-safe

GPU exists to visualize the consequences at scale.
