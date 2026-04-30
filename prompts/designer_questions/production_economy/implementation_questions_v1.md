# Production & economy — implementation questions `v1`

**Pair with:** [`spec/README.md`](spec/README.md), `power_damage_ui_persistence_v1.md`, `src/systems/production/manifest.rs`, `src/entities/production/*`.

## Power grid

1. **ElectricalNode** component set: which fields are serde vs runtime-only?
2. **Frequency / phase** mismatch penalties — formula in `Serializable` tunable or hardcoded first pass?
3. **Islanding** detection: graph SCC per tick vs event-driven on edge removal?

## Damage & repair

4. **RepairPriority** newtype: validate `1..=100` at boundary (`TryFrom`) — centralize in `entities/damages.rs`?
5. **Repair queue**: single `Resource<GlobalRepairQueue>` vs `RepairQueue` per facility — UI aggregates how?
6. **Component-level damage**: bitmask size; reflection for inspector?

## UI

7. Player **charts** (Bevy): time-series buffer length; sampling cadence tied to sim tick?
8. **Alert** model: ECS components vs dedicated `AlertLog` resource — sort key definitions.

## Persistence

9. Production state in **binary snapshot** — version chunk per `ManufacturingDomain`?
10. Link **repair jobs** to `EntityId` of facility + `BlueprintId` for spare parts — serde shape?

## Factions (overlap)

11. Faction tint on **grid lines** / **ownership** — read from same `FactionBlueprint` as `factions/` folder.

## Manifest & runtime (code-synced — avoid “paper” architecture)

12. **`manufacturing_core`** in `default_production_manifest()` names `runtime_plugin: "ProductionRuntimePlugin"`, but `ProductionRuntimePlugin` (`src/systems/production/runtime.rs`) only adds **Concrete / Aluminum / Power** plugins — **`ManufacturingNode`** (`src/entities/production/core/manufacturing.rs`) has **no** registered systems. Split manifest string vs add `ManufacturingCorePlugin` 📎?
13. **`ProductionManifest`** resource: hot-reload or spawn-time only? Who validates manifest vs actual registered plugins?

## Serialization plugins (`src/systems/production/serialization.rs`)

14. Fill `ConcreteSerializationPlugin::build` (DTO registration + format).
15. Same for `AluminumSerializationPlugin` / `PowerSerializationPlugin` (powerline graph DTO scope).
16. **`ManufacturingBlueprint`** persistence: separate chunk or folded into facility entities 📎?

## Damage types (`src/entities/damages.rs`, `src/systems/damage/damage_system.rs`)

17. **`RoadVehicleDamageInfo` / `BuildingDamageInfo`** are **not** `serde` today — mirror DTO for saves vs derive serde on components 📎?
18. **`DamageSystem`** only runs `apply_road_damage` for **`RoadVehicleDamageInfo`** — where does **building** damage accumulate (production, strategic, separate plugin)?
19. Link **`vehicle_damage.rs`** (if used) to damage pipeline vs dead code 📎?
