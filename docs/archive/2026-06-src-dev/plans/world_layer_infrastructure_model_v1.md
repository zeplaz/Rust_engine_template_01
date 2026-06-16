# World layers and infrastructure model `v1`

| Field | Value |
|:---|:---|
| **ID** | **WORLD-INFRA-LAYERS-001** |
| **Date** | 2026-06-02 |
| **Status** | **SIGNED — design authority** |
| **Exec (coders)** | [`plan_infrastructure_world_layers_exec_001_v1.md`](plan_infrastructure_world_layers_exec_001_v1.md) |
| **Aligns with** | [`docs/archive/2026-06-prompts-guides/matrix/matrix/transport/road_rail_migration_matrix_v1.md`](../../docs/archive/2026-06-prompts-guides/matrix/matrix/transport/road_rail_migration_matrix_v1.md) (R1–R10), [`transport_code_implementation_plan_v1.md`](../../docs/reference/designer_questions/transport/transport_code_implementation_plan_v1.md) |
| **Runtime today** | R8 [`TransportNetworkSnapshot`](../../src/systems/transport/snapshot.rs), [`CorridorClass`](../../src/systems/transport/types.rs), strategic [`CorridorType`](../../src/strategic/runbook_rounds.rs), [`NetworkType`](../../src/strategic/spatial_network.rs) |
| **Legacy to retire** | `TerrainFeatures { road, track }` ([`bevy_terrain.rs`](../../src/terrain/bevy_terrain.rs)), map snapshot `road: bool`, private ECS road stubs ([`legacy_transport_stubs.rs`](../../src/entities/structure/legacy_transport_stubs.rs)) |

---

## 1. Principle

**Tiles answer one question:** what **terrain** exists here (family, moisture, elevation, cover, etc.).

**Infrastructure is graph-based:** nodes, edges, and typed networks. Vehicles, utilities, and logistics **attach to networks**, not to tile booleans.

Do not model “roads” and “rail” as separate boolean layers on terrain. Model **transport corridors** (and parallel **utility networks**) with shared graph vocabulary.

---

## 2. World stack (six layers)

| Layer | Holds | Examples |
|:---|:---|:---|
| **Terrain** | Per-cell / chunk physical surface | family, hydrology, slope, cover |
| **Infrastructure** | Networks (see §3) | road graph, rail graph, utilities, waterways |
| **Settlement** | Places and zoning | town, district, building footprint |
| **Economic** | Flows and markets | demand, trade routes, activation |
| **Political** | Control and policy | ownership, tariffs, jurisdiction |
| **Military** | Supply and movement | corridors, depots, interdiction |

**Infrastructure** is not a flat tile flag. It is split into **independent networks** that may share geometry but not data structures:

- Road network  
- Rail network  
- Utility network (power, water, sewer, gas, telecom)  
- Waterway network  
- Air network (future)  
- Logistics network (strategic graph / convoy capacity — may alias transport edges)

---

## 3. Target types (authoritative sketch)

### 3.1 Tile — terrain only

```rust
/// Per-tile or per-chunk terrain payload — NO road/rail booleans.
pub struct TileTerrain {
    pub family: TerrainFamilyId,
    // moisture, slope, cover tags, etc.
}
```

**Deprecate:** `TerrainFeatures { road, track }`, editor snapshot `road: bool` — replace with **derived overlay** from graph bake (visual only) or remove.

### 3.2 Infrastructure occupancy (optional index)

A tile/chunk may hold **references** to infrastructure for rendering and queries, not ownership of sim state:

```rust
#[derive(Debug, Default)]
pub struct TileInfrastructureIndex {
    /// Ids into network registries — not inline segment data.
    pub transport_edge_ids: Vec<TransportEdgeId>,
    pub utility_edge_ids: Vec<UtilityEdgeId>,
    pub structure_refs: Vec<StructureRef>,
}
```

Full segment payloads live on **edges** in network resources, not duplicated per tile.

### 3.3 Transport — corridor-first

```rust
pub enum TransportLink {
    Road(RoadSegment),
    Rail(RailSegment),
    Canal(CanalSegment),
    Pipeline(PipelineSegment),
    Footpath(FootpathSegment),
}

/// Strategic / planning family — superset of sim [`CorridorClass`].
pub enum CorridorType {
    Road,
    Rail,
    Canal,
    Pipeline,
    AirRoute,
    ShippingLane,
    Hyperloop, // future — profile-gated
}
```

**Vehicles and agents** interact with **corridors** (edges + profile + field state), not with `road: bool`.

Align existing code:

| This design | Repo today |
|:---|:---|
| `CorridorType` | [`strategic::runbook_rounds::corridor::CorridorType`](../../src/strategic/runbook_rounds.rs) + [`systems::transport::CorridorClass`](../../src/systems/transport/types.rs) — **merge vocabulary** in one external registry; stop string `profile.contains("rail")` as long-term truth |
| Graph node/edge | R8 `TransportNodeRecord` / `TransportEdgeRecord` + runtime `TransportTopology` |
| Edge field (congestion, damage) | `EdgeFieldState` — already hybrid-field ready |

### 3.4 Road segment (profile payload)

```rust
pub struct RoadSegment {
    pub road_type: RoadType,
    pub lanes: u8,
    pub speed_limit: u16,
    pub surface: SurfaceType,
    pub owner: OwnerId,
}

pub enum RoadType {
    DirtTrack,
    LocalStreet,
    CollectorRoad,
    ArterialRoad,
    Highway,
    Expressway,
}

pub enum SurfaceType {
    Dirt,
    Gravel,
    Asphalt,
    Concrete,
    Cobblestone,
}
```

**Implementation note (matrix R4):** these enums belong in **RON profiles** (`RoadProfile` id → struct), not as the only material path. Tags `["road","paved","high_friction"]` resolve to terrain **MaterialId** for rendering. Sim uses profile fields; renderer uses tags.

### 3.5 Rail segment

```rust
pub struct RailSegment {
    pub gauge: RailGauge,
    pub electrification: Electrification,
    pub tracks: u8,
    pub max_speed: f32,
}

pub enum RailGauge {
    Narrow600mm,
    Narrow762mm,
    Cape1067mm,
    Meter1000mm,
    Irish1600mm,
    Broad1676mm,
    Standard1435mm,
    Russian1520mm,
}

pub enum Electrification {
    None,
    ThirdRail,
    AC25KV,
    AC15KV,
    DC750V,
    DC1500V,
    DC3000V,
}
```

**Implementation note (matrix R5):** replace legacy [`GaugeType`](../../src/entities/structure/legacy_transport_stubs.rs) stub with `RailProfile` in assets; stricter curvature than road profiles.

### 3.6 Utilities — separate networks

```rust
pub enum UtilityLink {
    Water(WaterPipe),
    Sewer(SewerPipe),
    Power(PowerLine),
    Telecom(FiberLine),
    Gas(GasPipe),
}

pub struct PowerLine {
    pub voltage: VoltageClass,
    pub capacity_mw: f32,
}

pub enum VoltageClass {
    Lv,
    Mv,
    Hv,
    Ehv,
}
```

Align with [`NetworkType::Power`](../../src/strategic/spatial_network.rs) / `Fluid` / `Data` — utilities are **not** `TransportLink` variants for sim; they share spatial indexing but separate solvers (power flow, pipe pressure).

### 3.7 Buildings connect to networks

**Invalid:** `building.has_power`.

**Target:**

```rust
pub struct UtilityConnection {
    pub network_id: NetworkId,
    pub demand: f32,
}
```

Building components hold `Vec<UtilityConnection>` per utility family; activation/economy reads demand from connections, not tile flags.

---

## 4. Graph model (town → port chain)

```
Town (Settlement node)
  |
RoadNode (TransportNode)
  |
RoadEdge (profile: CollectorRoad, EdgeFieldState)
  |
HighwayJunction (TransportNode, degree > 2)
  |
RailTerminal (TransportNode + allowed_agents: ["train"])
  |
Port (Settlement + Maritime edge class)
```

**Persistence (R8):** store **control points + profile id** on edges; bake subdivides to segments **after** editor confirm — already specified in migration matrix §2.

**Navigation (G5):** export `{ edge_id, cost, allowed_agents }` from graph — [`NavExportEdge`](../../src/systems/transport/types.rs), not tile traversal.

---

## 5. What changes in this repo

**Full program (6 epics, ~12–14 weeks, 25 PRs):** [`plan_infrastructure_world_layers_exec_001_v1.md`](plan_infrastructure_world_layers_exec_001_v1.md) — **this is the coder workboard**; sections below are summary only.

| Epic | Theme |
|:---:|:---|
| **0** | Profiles + deprecate tile flags |
| **1** | Graph core R1–R3 |
| **2** | Authoring R9 + construction |
| **3** | Persistence R8/M5 |
| **4** | Utility networks |
| **5** | Settlement + logistics on graph |
| **6** | Materials R4, nav R7, overlay R10 |

---

## 6. Relationship to construction

Today: [`ConstructionType::RoadSegment`](../../src/construction/construction_pipeline.rs) places tile markers — acceptable **authoring** step.

Target: confirm → append `TransportEdgeRecord` to snapshot → hydrate → markers become **views** on graph ids.

Corridor phases in R8 `TransportConstructionRecord` already exist — use for Planned → Operational, not tile paint.

---

## 7. Anti-patterns (explicit)

| Anti-pattern | Why |
|:---|:---|
| `if tile.road { … }` for routing | No strategic graph; breaks towns/regions/logistics |
| Duplicate rail + road bool on same tile | Cannot represent grade separation, bridges, stacks |
| `has_power` on building | Hides network outages, graph cuts, capacity |
| Baking only segments to save | Loses spline edit; breaks deterministic reload (R8) |
| One enum for “all infrastructure” | Utilities and transport need different solvers |

---

## 8. Open decisions

| ID | Question | Default bias |
|:---|:---|:---|
| **INFRA-D-01** | Single `NetworkGraph` resource vs per-family graphs | Per-family graphs, shared spatial index |
| **INFRA-D-02** | `TransportLink` enum vs profile-only edges | Profile-only at runtime; enum in authoring API only |
| **INFRA-D-03** | Canal / hyperloop in v1 | Schema reserve in `CorridorType`; no sim until profile exists |

---

## 9. Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Initial direction from product sketch; mapped to R8 + migration matrix |
