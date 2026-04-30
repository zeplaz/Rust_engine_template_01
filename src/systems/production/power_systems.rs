// `systems/production/power_systems.rs` — **historical path** only.
//
// All active power simulation now lives under:
//   `src/entities/production/power/`
//
// Layout (Bevy 0.18):
//   - `plant_definition.rs` / `plant_registry.rs` — serde schema + `PlantDefinitionRegistry` (embedded
//     `assets/config/power/plant_definitions.json`). `PowerPlant.definition_id` selects a row.
//   - `plant_profile.rs` — `PlantArchetype` per `PowerPlantType` (fallback when `definition_id` is empty).
//   - `capabilities.rs` — marker components (`SteamCycle`, `ContainmentBuilding`, `VariableRenewable`) +
//     `attach_power_plant_capabilities` on `Added<PowerPlant>`. Failures are **scoped** to markers, not every fuel type.
//   - `grid_topology.rs` — `rebuild_electrical_grid_topology`, `recalculate_grid_totals_from_members`,
//     `purge_removed_power_components_from_grids`, `purge_stale_grid_references`, `emit_grid_overload_signals`
//     (`GridOverloadEvent` message for overload parity).
//   - `failure_modes.rs` — placeholders for steam / nuclear / renewable derates (queries use `With<SteamCycle>` etc.).
//   - `systems.rs` — `PowerRuntimePlugin` chains the above with electrical clamp + power output; `run_if` uses
//     `SimControlState::should_tick()` (pause/step). See `prompts/matrix/production/power_legacy_functional_parity_v1.md`.

//
// **Not** ported as-is from the pre-0.11 file:
//   - `update_load_system` used `ElectricalLoad` + `ProductionComponent` types that no longer exist in this crate;
//     tie loads to `ElectricalComponent` + your manufacturing blueprint when that data model is ready.
//   - The old `power_plant_operational_system` used `ResMut<State<OperationalStatus>>` (one global “state” for all plants).
//     Per-plant status is **`PowerPlant.status`** (`OperationalStatus` on the entity).
//
// Do not re-add a second plugin here — use `PowerRuntimePlugin` (`ProductionRuntimePlugin` already registers it).
