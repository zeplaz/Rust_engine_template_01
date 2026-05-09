# Asset → simulation ownership matrix `v1`

> **Runbook:** [`../../guides/asset_system_audit_runbook_v1.md`](../../guides/asset_system_audit_runbook_v1.md) · **Step pack:** [`runbook/s0_steps_v1.md`](runbook/s0_steps_v1.md)

Version: `v1.2.3`  
**STATUS:** **Partial** — inventory pass from repo (2026-05-06); S0 step pack verified same day; rows below cite **primary** owners + anchors.

---

## How to use

For each row: one **primary** runtime owner, **sim layer**, lifecycle, persistence. **Anchor** is evidence (Rust path or asset); update when code moves. **No row** ⇒ do not build dependent simulation without adding the row first.

**Drift warning:** Python `src/utils/asset_tools/src/config/asset_config.py` lists such as `RESOURCE_TYPES` can be **richer** than Rust `ResourceType` (`src/entities/types/p_enumz.rs`). Treat that as **S8 after S0** alignment work, not as engine truth until reflected in Rust or an explicit codegen bridge.

---

## Matrix

| Asset / taxonomy | Primary runtime owner | Sim layer | Static / dynamic | Persistent? | Anchor (code / data) | Status |
|:---|:---|:---|:---|:---|:---|:---|
| Terrain **MaterialRegistry** | `MaterialUnificationPlugin` / material pass systems | ontology → derived | both | registry ids / hashes yes | `src/systems/terrain/material_plugin.rs` (`MaterialRegistryLoader`); `src/terrain/material/registry.rs` | **Applied** |
| Terrain **TagRegistry** | same | ontology | static (assets) | yes | `src/terrain/material/tags.rs`; registration `material_plugin.rs` | **Applied** |
| Terrain **RuleSet** / **WorldProfile** | same | ontology + derived | both | yes | `src/terrain/material/rules.rs`, `profile.rs` | **Applied** |
| **TerrainFamilyRegistry** | terrain + preview / biome aggregate | ontology | static (assets) | family names yes | `src/terrain/family.rs` | **Applied** |
| **MobilityProfileRegistry** | terrain + mobility interpretation | derived consumer | static (assets) | yes | `src/terrain/mobility/mod.rs` | **Applied** |
| **ChunkDependency** / **ChunkDirty** | material chunk pipeline | derived + scheduler hook | dynamic | chunk entity state | `src/terrain/material/dependency.rs`; `material_plugin.rs` (`rebuild_dirty_chunks`) | **Partial** (terrain only) |
| **ResourceType** (Rust enum) | production + logistics + vehicle cargo | economy / industry | both | partial | `src/entities/types/p_enumz.rs`; consumers `prod_comps.rs`, `entities/vehicles/runtime.rs` | **Applied** |
| **ConcreteType** + kiln / mine / mixer **runtime** | `ConcreteRuntimePlugin` | industry | both | `ConcreteProductionConfig` + entities TBD | `src/entities/production/concrete/components.rs`, `systems.rs` via `ProductionRuntimePlugin` | **Partial** |
| **Aluminum** runtime | `AluminumRuntimePlugin` | industry | both | TBD | `src/entities/production/aluminum/systems.rs` via `ProductionRuntimePlugin` | **Partial** |
| **PlantDefinition** / **PlantDefinitionRegistry** | `PowerRuntimePlugin` | industry + infrastructure | static defs | definition ids yes | `src/entities/production/power/plant_definition.rs`, `plant_registry.rs`, `systems.rs`; JSON `assets/config/power/plant_definitions.json` | **Partial** |
| **ManufacturingNode** | `ManufacturingCorePlugin` | industry | dynamic | TBD | `src/entities/production/core/manufacturing_plugin.rs`, `manufacturing.rs` | **Partial** (scaffold) |
| **SegmentMembership** | factions / agents / serialization | strategic | both | yes | `src/entities/types/e_flagz.rs` | **Partial** |
| Transport cost / nav export | `TransportSimulationPlugin` + `NavigationSchedulePlugin` | logistics + infrastructure | dynamic | caches often recompute | `src/systems/transport/`, `src/systems/navigation/` | **Partial** |
| Road vehicle tools / cargo rules | `RoadVehicleToolsUiPlugin` + vehicle runtime | agents / logistics | both | partial | `src/entities/vehicles/` | **Partial** |
| Damage | `DamageSystem` | overlay / decay | dynamic | situational | `src/systems/damage/` | **Partial** |
| **Weather** (chunk layer) | `WeatherPlugin` | weather / climate | dynamic per chunk | TBD | `src/systems/weather/chunk_weather.rs` (`ChunkWeather`, `WeatherSimDiagnostics`); [`weather_simulation_runbook_v1.md`](../../guides/weather_simulation_runbook_v1.md) | **Partial** |
| Legacy **trucks.dat** / **base_terrains.dat** | `io/serialization` (legacy paths) | legacy | static files | file-backed | `src/io/serialization/deserializers.rs` | **Legacy** — prefer registries + components |
| Vehicle **fuel_type** / petroleum split (designer) | *not fully in Rust enum yet* | ontology / industry | — | — | Tooling: `asset config.py` `FUEL_TYPES`; engine alignment **Pending** | **Pending** |
| Petroleum **crude / fractions** (designer) | *stubs / docs* | industry | — | — | Runbook: `petroleum_industry_simulation_runbook_v1.md` | **Pending** |
| **Macro regions** (Voronoi skeleton) | `world_generator_enhanced` finalize + `despawn_generated_world_entities` | strategic grouping / saves partition | static after gen | raster optional in saves | `MacroRegion`, `MacroRegionRaster`, `TileRegionIndex`, `RegionMarker` — `src/terrain/generation/world_generator_enhanced.rs` | **Partial** |
| **ChunkStrategicOverlay** (operational SOA) | `StrategicFieldsPlugin` | strategic **fields** (control, threat, logistics scalars per cell) | dynamic | TBD | `src/strategic/mod.rs`, `src/strategic/plugin.rs` (`ensure_chunk_strategic_overlays`) | **Partial** |
| **LogisticsGraph** (Resource) + net → field | `logistics_net_inject_into_overlays` | sparse **graph** coupling into chunk logistics scalars | dynamic | TBD | `src/strategic/logistics_net.rs`; `LogisticsNode.anchor` → `ChunkCellKey` | **Partial** |

---

## Appendix A — `ResourceType` (Rust) vs Python `RESOURCE_TYPES` (S0-S03)

**Engine enum:** `src/entities/types/p_enumz.rs` (`ResourceType`). **Tool list:** `src/utils/asset_tools/src/config/asset_config.py` (`RESOURCE_TYPES`).

| Category | Detail |
|:---|:---|
| **Python-only** (no `ResourceType` variant today) | `CrudeOil`, `NaturalGas`, `Refined_*`, `Petrochemical_Intermediates`, and other petroleum-split strings in `RESOURCE_TYPES`. |
| **Rust-only** (not in Python `RESOURCE_TYPES`) | *None* — `Energy` added to Python (2026-05-06, S8 parallel) to match `ResourceType::Energy`. |
| **Overlapping names** (both sides) | `Wood`, `Coal`, `Oil`, `RareEarth`, `Metal`, `Steel`, `Concrete`, `Fertilizer`, `Chemicals`, `Electronics`, `Ammunition`, `WarSupply`, `Knowledge`, `Labour`, `Food`, `Water`, `Paper`, `Electricity`, `Energy`, `Fuel`. |
| **Intent** | Extend Rust **or** generate Python from Rust **or** document “authoring-only” extras until petroleum migration lands — see S8 after S0. |

Loader registration parity (S0-S01): `material_plugin.rs` registers exactly these six loaders — `TerrainFamilyRegistryLoader`, `MobilityProfileRegistryLoader`, `MaterialRegistryLoader`, `TagRegistryLoader`, `RuleSetLoader`, `WorldProfileLoader` — matching the terrain rows in the matrix above.

---

## Cross-links

- [`../../guides/simulation_expansion_orchestrator_v1.md`](../../guides/simulation_expansion_orchestrator_v1.md)  
- [`../../guides/chunk_scheduler_runbook_v1.md`](../../guides/chunk_scheduler_runbook_v1.md) + [`runbook/s1_steps_v1.md`](runbook/s1_steps_v1.md) + [`chunk_scheduler_gap_table_v1.md`](chunk_scheduler_gap_table_v1.md)  
- [`runbook/s2_steps_v1.md`](runbook/s2_steps_v1.md) + [`../../guides/weather_simulation_runbook_v1.md`](../../guides/weather_simulation_runbook_v1.md)  
- [`python_asset_tools_a0_inventory_v1.md`](python_asset_tools_a0_inventory_v1.md)

---

## Document history

- **2026-05-06:** Initial matrix scaffold (S0 kickoff).
- **2026-05-06:** `v1.1.0` — code anchor inventory; terrain + production + transport rows; drift note vs Python resources.
- **2026-05-06:** `v1.2.0` — `AluminumRuntimePlugin` row; appendix A (ResourceType vs Python); loader registration verification note.
- **2026-05-06:** `v1.2.1` — **Weather** row (`WeatherPlugin` scaffold); Appendix A: Python `Energy` aligned with Rust.
- **2026-05-06:** `v1.2.2` — `ChunkWeather` implementation anchor.
- **2026-05-06:** `v1.2.3` — **Macro regions** + **ChunkStrategicOverlay** + **LogisticsGraph** rows (strategic field + nets spine).
