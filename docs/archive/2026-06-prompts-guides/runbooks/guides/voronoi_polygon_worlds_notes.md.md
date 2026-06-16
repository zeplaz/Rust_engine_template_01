# Voronoi / Polygon World Generation Notes
## Gameplay-Driven Region Worlds for Large Scale Simulation
from http://www-cs-students.stanford.edu/~amitp/game-programming/polygon-map-generation/
Status:
- Research + design reference
- NOT direct implementation spec
- Used to guide:
  - worldgen
  - region partitioning
  - AI reasoning
  - logistics
  - terrain semantics
  - LOD streaming
  - pathfinding
  - political regions
  - strategic simulation

Primary inspiration:
- Amit Patel / Red Blob Games polygon map generation work
- Expanded toward:
  - logistics simulation
  - warfare
  - infrastructure
  - dynamic ecology
  - large scale streaming worlds

## Engine hooks (Rust template)

Lightweight wiring in this repo—not a full Layer 0 strategic graph yet, but aligned with “semantics + structure before pure tile noise”:

- **`src/terrain/generation/polygon_world_semantics.rs`** — [`MacroStrategicKind`](../../src/terrain/generation/polygon_world_semantics.rs), [`classify_strategic_tile`](../../src/terrain/generation/polygon_world_semantics.rs), and [`apply_strategic_field_nudge`](../../src/terrain/generation/polygon_world_semantics.rs). Each generated tile can carry a strategic component for AI / logistics / future coarse sim; optional nudges adjust moisture and temperature after fractal sampling.
- **`WorldGenParams::strategic_field_coupling`** in **`src/terrain/generation/world_generator_enhanced.rs`** — World Generator UI slider (e.g. 0–0.35). **`0`** = classify only (default, same climate as before aside from tagging); larger values bias fields toward the strategic kind, then biome classification runs on the adjusted values. The hydrology moisture grid uses these post-nudge values for consistency with tiles.
- **`src/terrain/voronoi_enhanced.rs`** — Voronoi site placement and variants (regular, centroidal, weighted, etc.). Tiles still attach to the nearest site for ECS hierarchy; **Voronoi cells do not sculpt height**—that’s unchanged noise + passes.

A full macro graph (region adjacency, Delaunay duals, watershed hierarchies as first-class objects) remains future work; these paths are the current anchors from this guide to runnable code.

---

# Core Principle

Do NOT generate the world as "noise-first tiles".

Instead:

World generation should primarily generate:
- regions
- relationships
- topology
- flow systems
- strategic structures
- terrain semantics

Then:
- rasterize into chunk/tile detail layers
- generate micro detail separately
- preserve high level graph structures

The simulation truth is NOT the tile.
The simulation truth is the layered world graph.

---

# Why This Fits Our Engine

Our engine targets:
- 100k+ field simulation
- 10k detailed simulation bubbles
- heavy LOD systems
- logistics + economy + war
- dynamic infrastructure
- AI operational reasoning
- chunk streaming
- hybrid navigation

Tile-only worlds become:
- too expensive
- too noisy
- hard for AI reasoning
- difficult for strategic simulation

Polygon / graph-first worlds provide:
- stable regions
- semantic terrain
- scalable pathfinding
- natural logistics partitioning
- efficient streaming domains
- meaningful AI territories

---

# World Structure Layers

The world should be represented as multiple linked layers.

## Layer 0 — Strategic Region Graph

Large polygons / macro regions.

Examples:
- mountain basin
- valley system
- industrial corridor
- floodplain
- desert plateau
- river watershed
- urban district
- island chain

Used for:
- AI planning
- strategic logistics
- warfare fronts
- economy balancing
- influence
- weather cells
- migration
- ownership

Scale:
- ~500 to 50,000 regions

Simulation rate:
- very slow
- low frequency

---

## Layer 1 — Mobility / Infrastructure Graph

Connected graph:
- roads
- rail
- pipelines
- rivers
- sea routes
- power transmission
- telecom
- drone corridors

Used for:
- convoy routing
- throughput
- supply
- rerouting
- congestion
- accidents
- infrastructure damage

Primary navigation layer.

---

## Layer 2 — Chunk Simulation Grid

Chunk-based terrain simulation.

Examples:
- moisture
- vegetation
- erosion
- flooding
- fire spread
- local traffic
- local economy

Chunk owns:
- terrain facts
- derived metrics
- overlays
- local object density

Truth layer for:
- environment simulation

NOT:
- strategic AI

---

## Layer 3 — High Detail Bubble Simulation

Local detailed simulation:
- cities
- battles
- player interaction
- construction
- local vehicles
- crowd systems

Only active near:
- player
- active wars
- major infrastructure
- important AI zones

---

# Key Design Principle

## TERRAIN FACTS ≠ NAVIGATION VERDICTS

Do NOT encode:
- traversable
- non_traversable

Instead encode:
- slope
- wetness
- rockiness
- vegetation density
- water depth
- temperature
- surface stability
- erosion
- snow
- mud
- debris

Then:
mobility profiles interpret those facts.

This is critical.

A swamp:
- may block trucks
- slow infantry
- permit hovercraft
- permit drones
- permit amphibious vehicles

Simulation stays extensible.

---

# Region Generation Philosophy

Traditional generators:
- generate elevation noise
- derive gameplay afterward

Our direction:
- generate gameplay structure first
- derive visuals afterward

Examples:
- strategic mountain barriers
- logistics corridors
- natural choke points
- flood basins
- resource belts
- trade coastlines
- industrial river systems

Noise is secondary detail.

---

# Voronoi / Polygon Use

Voronoi regions are useful because they create:
- recognizable territories
- stable partitions
- natural adjacency graphs
- scalable simulation regions

Potential uses:
- climate cells
- ownership regions
- AI planning zones
- terrain sectors
- utility districts
- streaming sectors
- political boundaries
- ecology domains

---

# Delaunay + Voronoi Duality

Important concept.

We can use:
- Voronoi regions for area ownership
- Delaunay edges for connectivity

Examples:
- roads
- rail
- river routing
- migration
- trade
- military corridors

This creates:
- clean strategic graph systems

---

# Rivers

Rivers should be:
- graph systems
- watershed systems
- logistics systems

NOT:
- purely visual splines

Rivers affect:
- trade
- flooding
- agriculture
- power generation
- transport
- warfare
- bridges
- industry

Rivers become:
- infrastructure corridors

---

# Watersheds

Watersheds are EXTREMELY important.

They define:
- ecology
- agriculture
- flood risk
- settlement patterns
- logistics corridors
- hydroelectric potential
- strategic regions

Watershed hierarchy can become:
- AI planning hierarchy
- weather hierarchy
- ecology hierarchy

---

# Mountains

Mountains should NOT just be:
- high elevation noise

They should create:
- barriers
- weather effects
- isolation
- mining regions
- defensive positions
- logistics constraints
- avalanches
- tunnel opportunities

---

# Roads / Logistics

Roads should emerge from:
- terrain constraints
- settlements
- industry
- trade demand
- military pressure

NOT:
- arbitrary decorative splines

Road graph should support:
- lane abstraction
- throughput
- congestion
- crashes
- weather disruption
- bridge collapse
- rerouting
- convoy scheduling

---

# Hybrid Navigation

We are NOT using:
- pure lane graph
- pure grid nav

We use:
- graph-first navigation
- local field movement fallback

Primary:
- roads
- rails
- infrastructure graph

Fallback:
- local terrain field movement
- offroad movement
- emergency reroutes
- getting stuck
- collisions
- accidents

This supports:
- vehicles leaving roads
- crash events
- weather failures
- military maneuvers

---

# Soft Reservation System

Reservation system should be:
- probabilistic
- soft constrained

NOT:
- deterministic lockstep traffic

Collisions remain possible.

Factors:
- stress
- weather
- visibility
- road quality
- congestion
- AI behavior
- speed
- maintenance
- damage

Supports:
- emergent accidents
- disasters
- convoy pileups
- wartime chaos

---

# LOD Philosophy

LOD is simulation authority partitioning.

NOT just rendering.

Different layers own:
- different fidelity
- different update rates
- different truth sources

Examples:
- strategic logistics:
  low frequency
- local battle:
  high frequency

---

# Chunk Streaming

Chunks should stream:
- independently
- asynchronously
- by relevance

Relevance factors:
- player distance
- AI interest
- logistics importance
- military activity
- economy activity
- weather events

---

# Dynamic Overlay Layers

World facts:
- mostly persistent

Derived overlays:
- recomputed
- transient
- non-save-authoritative

Examples:
- mobility costs
- mud
- snow buildup
- traffic density
- fire spread risk
- temporary flooding
- contamination

These belong in:
- dynamic overlays

NOT:
- baked terrain tags

---

# AI Implications

AI should reason on:
- regions
- corridors
- supply networks
- influence graphs
- terrain semantics

NOT:
- individual tiles first

Tiles are tactical detail.
Graphs are strategic truth.

---

# Rasterization Philosophy

Polygons are NOT final render format.

They are:
- simulation structures

Later:
- rasterize into chunk fields
- generate detail textures
- spawn foliage
- carve roads
- produce decals
- generate clutter

---

# Ecology

Vegetation should emerge from:
- moisture
- temperature
- slope
- soil
- sunlight
- fire history
- grazing
- flooding

NOT:
- biome painting only

---

# Infrastructure Damage

Infrastructure should behave physically.

Examples:
- bridge collapse
- flooded rail
- pipeline rupture
- blocked export terminal
- transformer overload
- refinery shutdown

Effects propagate through:
- logistics graph
- economy
- military readiness

---

# Oil / Resource Simulation

Resources should have:
- grades
- qualities
- processing constraints

Examples:
- sweet crude
- sour crude
- heavy oil
- oil sands
- shale oil

Refineries specialize.

Outputs:
- diesel
- gasoline
- jet fuel
- kerosene
- chemical feedstock
- military fuel

Demand shifts:
- seasons
- war
- economy
- doctrine

Infrastructure disruptions:
- terminal destruction
- pipeline sabotage
- refinery overload

Can back-pressure extraction systems.

---

# Concrete / Material Chains

Concrete should be:
- industrially simulated

Inputs:
- aggregate
- water
- cement
- additives
- energy

Properties:
- cure time
- weather sensitivity
- strength
- cost
- durability

Construction quality should matter.

---

# Weather

Weather should operate on:
- large regional cells
- fronts
- pressure systems
- watersheds
- terrain interaction

Weather affects:
- power
- transport
- visibility
- agriculture
- combat
- flooding
- infrastructure decay
- construction speed

---

# Preview / Debug Philosophy

Everything important should have:
- overlays
- previews
- heatmaps
- ghost planning views

Examples:
- road preview
- logistics preview
- slope
- moisture
- traffic
- congestion
- weather
- flood risk
- power coverage
- signal range

Visualization is mandatory for debugging large simulations.

---

# ECS Philosophy

Simulation modules should stay independent.

Core graph systems:
- should not depend on gameplay systems

Gameplay systems:
- attach overlays/components/resources

Use:
- indices
- handles
- IDs
- sparse overlays

Avoid:
- hard coupling

---

# Important Engine Direction

We are building:
- a simulation-first engine

NOT:
- a decorative terrain renderer

The terrain exists to:
- support simulation
- support logistics
- support warfare
- support economy
- support infrastructure
- support AI reasoning

Visual fidelity is layered on top.

---

# Recommended Immediate Direction

Priority order:

1. Terrain facts system
2. Chunk SoA storage
3. Region graph generation
4. Watershed generation
5. Mobility evaluator
6. Logistics graph
7. Streaming authority
8. Overlay systems
9. AI strategic regions
10. High detail local simulation

---

# Major Warning

Do NOT:
- hardcode gameplay into terrain tags
- overfit to tiles
- couple visuals to simulation truth
- bake mobility into materials
- make AI reason directly on raster detail

Those choices kill scalability later.

---

# Core Engine Identity

The engine should think in:
- regions
- flows
- overlays
- graphs
- logistics
- influence
- terrain semantics

NOT:
- isolated tiles
- binary passability
- static terrain classes
