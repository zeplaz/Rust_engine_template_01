# Production Migration Matrix v1

> **STATUS:** ✅ **Modular split + manifest** wired in code · ⏳ legacy `sys.rs` files remain on disk, hard-disabled · Serialization plugins mostly **stubs** — extend when save format lands (`prompts/matrix/serialization/serialization_hybrid_migration_matrix_v1.md`). **Paired designer:** `prompts/designer_questions/production_economy/` · **Structured spec:** `production_economy/spec/README.md`.

Version: `v1.0.0`
Scope: concrete + power + aluminum + manufacturing core split into serializable data, ECS runtime systems, and tools/UI boundaries.

> **Prompt use:** Pair `designer_questions/production_economy/` · verify vs `src/entities/production/`, `src/systems/production/` · cite briefly · `ASK:` if save format unset. **Power functional parity:** `prompts/matrix/production/power_legacy_functional_parity_v1.md`.

## Canonical Boundary Rules

- Serializable layer:
  - Configs, enums, blueprints, deterministic tunables.
- ECS runtime layer:
  - Mutable per-entity operational state and frame-update systems.
- Tools/UI layer:
  - Inspection/control resources and editor-facing plugin boundaries only.
- Legacy files:
  - Kept on disk for migration trace, hard-disabled from module graph.

## Domain Matrix

| Domain | Serializable Types | ECS Runtime Types | Runtime Plugin | Serialization Plugin | Tools/UI Hook |
|---|---|---|---|---|---|
| Concrete | `ConcreteProductionConfig`, `ConcreteType` | `CementKilnRuntime`, `AggregateMineRuntime`, `ConcreteMixerRuntime` | `ConcreteRuntimePlugin` | `ConcreteSerializationPlugin` | `ProductionToolsUiPlugin` |
| Power | `PlantDefinition` (JSON), `PowerPlantType`, `PowerDistributionType` | `PowerPlant` (+ `definition_id`), `ElectricalComponent`, `PlantDefinitionRegistry` | `PowerRuntimePlugin` | `PowerSerializationPlugin` | `ProductionToolsUiPlugin` |
| Aluminum | `AluminumProductionConfig`, `FabricationLineType` | `BauxiteMineRuntime`, `AluminaRefineryRuntime`, `AluminumSmelterRuntime`, `AluminumFabricationPlantRuntime` | `AluminumRuntimePlugin` | `AluminumSerializationPlugin` | `ProductionToolsUiPlugin` |
| Manufacturing Core | `ManufacturingBlueprint`, `ManufacturingDomain` | `ManufacturingNode` | ⏳ **no domain `Plugin` yet** — see § **Code vs matrix** | (use domain plugins above) | `ProductionToolsUiPlugin` |

Aggregate: `ProductionSerializationPlugin` in `src/systems/production/serialization.rs` (added to `EnginePlugin` in `engine_with_worldgen.rs`).

## Code vs matrix (do not simplify away)

- **`ProductionRuntimePlugin`** (`src/systems/production/runtime.rs`) registers **`ConcreteRuntimePlugin`**, **`AluminumRuntimePlugin`**, **`PowerRuntimePlugin`** only. It does **not** register systems for **`ManufacturingNode`** / manufacturing-core gameplay — those types exist in `src/entities/production/core/manufacturing.rs` but are **scaffolding**.
- **`default_production_manifest()`** lists a **`manufacturing_core`** domain whose `runtime_plugin` string is `"ProductionRuntimePlugin"` — that names the **aggregator**, not a separate manufacturing-core plugin. Treat manifest row as **registry + intent** until a real plugin wires `ManufacturingNode`.
- **`ProductionSerializationPlugin`** children (`ConcreteSerializationPlugin`, `AluminumSerializationPlugin`, `PowerSerializationPlugin`) are **empty stubs** (`// TODO` in `serialization.rs`) — matrix “serialization plugin” column is **boundary placeholders**, not working save/load.
- **Damage:** `BuildingDamageInfo` / `RoadVehicleDamageInfo` live in `src/entities/damages.rs` (**not serde** today). `DamageSystem` (`src/systems/damage/damage_system.rs`) only schedules **`apply_road_damage`** for road vehicles; building damage accumulation **not hooked** in that plugin.

## Compilation Path Hard-Disable Map

| Legacy File | Disable Method | Replacement |
|---|---|---|
| `src/entities/production/concrete/sys.rs` | not exported by `concrete/mod.rs` | `concrete/components.rs`, `concrete/systems.rs` |
| `src/entities/production/aluminum/production_sys.rs` | removed from `aluminum/mod.rs` exports | `aluminum/components.rs`, `aluminum/systems.rs` |
| `src/systems/production/power_systems.rs` | not included by `systems/production/mod.rs` | `entities/production/power/` (`plant_profile`, `capabilities`, `grid_topology`, `failure_modes`, `systems` / `PowerRuntimePlugin`) |
| `src/systems/production/production_consumption.rs` | not included by `systems/production/mod.rs` | staged replacement under `systems/production/runtime.rs` |

## Agent-Friendly Discovery Registry

Manifest resource:
- `src/systems/production/manifest.rs`
  - `ProductionManifest`
  - `ProductionDomainEntry`
  - `default_production_manifest()`

Runtime wiring:
- `src/systems/production/runtime.rs` inserts default manifest.

## Next Migration Targets

1. Fold common numeric simulation helpers into shared production-core utility module.
2. Add explicit serializable save/load DTOs for production runtime snapshots.
3. Add dedicated tools UI systems for:
   - domain selection,
   - config editing,
   - runtime inspection panels.
4. Resolve any remaining stale imports to old `types_of`/legacy namespaces.

## Prompt Fragment For Subsequent Agent Pass

1. Read `prompts/matrix/production/production_migration_matrix_v1.md`.
2. Keep legacy files non-compiled; only touch canonical modules.
3. Preserve strict serializable vs ECS runtime vs tools/UI boundaries.
4. Extend manifest on each new production domain.
5. Add migration notes when moving or renaming symbols.
