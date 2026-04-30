## Repo Boundary Matrix v1

> **STATUS:** ✅ Step 1 inventory + boundaries **largely applied** · ⏳ occasional file re-audit · ⚠️ footer “read order” updated for nested `prompts/matrix/<subsystem>/` (2026-04-27).
>
> **Prompt use:** Pair with target subsystem’s `designer_questions/…` and its matrix · `rg`/read before moving types across Serializable/ECS/ToolsUI · cite paths · `ASK:` if layer ambiguous. **Navigation / factions** have no dedicated matrix folder — see `prompts/matrix/README.md` § *without a dedicated folder*.

Version: `v1.0.0`
Audience: downstream agents (including smaller models) performing boundary-preserving refactors.
Scope: classifies every active source file in `src/` into one of:

- `Serializable` — pure data, configs, enums, DTOs (must be `Serialize/Deserialize` if persistence is required).
- `ECSRuntime` — Bevy `Component`/`Resource` mutable runtime state and per-frame systems.
- `ToolsUI` — editor/tooling-facing UI plugins, panels, preview layers (egui).
- `LegacyMigration` — kept on disk for traceability, hard-disabled from active module graph.

Strict rules:
- `Serializable` MUST NOT depend on `ECSRuntime` or `ToolsUI`.
- `ECSRuntime` MAY depend on `Serializable`.
- `ToolsUI` MAY depend on `Serializable`, MAY read `ECSRuntime`, MUST NOT mutate `ECSRuntime` outside dedicated apply systems.
- `LegacyMigration` MUST NOT be referenced by any active module.

---

### Step 1 — Repo-wide inventory (current state)

#### Engine + entry points

| Path | Layer | Notes |
|---|---|---|
| `src/lib.rs` | Glue | Module root; safe re-exports only. |
| `src/main.rs` | Glue | Binary entry. |
| `src/bin/world_generator.rs` | ToolsUI | Standalone tools binary, OK. |
| `src/engine/mod.rs` | Glue | |
| `src/engine/engine.rs` | LegacyMigration | Older `EnginePlugin`; replaced by `engine_with_worldgen.rs`. |
| `src/engine/engine_with_worldgen.rs` | ECSRuntime | Wires runtime + tools plugin trio. |
| `src/engine/states.rs` | Serializable | Bevy `States` enums; consider adding `Serialize` later. |
| `src/engine/transitions.rs` | ECSRuntime | |
| `src/engine/sets.rs` | ECSRuntime | |
| `src/engine/utils.rs` | Mixed | Audit needed. |
| `src/engine/lmodels/*` | Mixed | Mostly research/legacy logic models; audit per file. |

#### Terrain + worldgen

| Path | Layer | Notes |
|---|---|---|
| `src/terrain/mod.rs` | Glue | |
| `src/terrain/biome.rs` | Serializable | Canonical terrain/biome model + classifier. |
| `src/terrain/ecology.rs` | Serializable | Flora/Crop/Flower types + suitability. |
| `src/terrain/bevy_terrain.rs` | Serializable + LegacyMigration | New legacy enums + features kept; old `TerrainType` aliased. |
| `src/terrain/locational.rs` | Audit | |
| `src/terrain/tiles.rs` | ECSRuntime | |
| `src/terrain/tools.rs` | ToolsUI | |
| `src/terrain/voronoi.rs` | Serializable | Pure functions. |
| `src/terrain/voronoi_enhanced.rs` | Serializable | Pure functions. |
| `src/terrain/world.rs` | ECSRuntime | `World`/`GeoRegion`. |
| `src/terrain/generation/mod.rs` | Glue | |
| `src/terrain/generation/geo_plugin.rs` | ECSRuntime | |
| `src/terrain/generation/bevy_terrain_gen.rs` | Serializable | Generation models. |
| `src/terrain/generation/world_generator.rs` | LegacyMigration | Old skeleton. |
| `src/terrain/generation/world_generator_enhanced.rs` | ECSRuntime + Serializable | Mixed; classifier fully delegates to `terrain::biome`. |
| `src/terrain/generation/world_generation_plugin.rs` | ECSRuntime + ToolsUI | Provides runtime + tools trio. |
| `src/bevysubengines/world_generator_plugin.rs` | ECSRuntime + ToolsUI | Subengine path; serializable `WorldData`. |
| `src/bevysubengines/mod.rs` | Glue | |

#### Production (already split via Production Migration Matrix v1)

| Path | Layer | Notes |
|---|---|---|
| `src/entities/production/mod.rs` | Glue | |
| `src/entities/production/core/mod.rs` | Glue | |
| `src/entities/production/core/manufacturing.rs` | Serializable + ECSRuntime | Blueprint + node bridge. |
| `src/entities/production/core/production_care.rs` | LegacyMigration | Audit imports (broken refs). |
| `src/entities/production/core/production_utils.rs` | LegacyMigration | Audit imports (broken refs). |
| `src/entities/production/concrete/mod.rs` | Glue | |
| `src/entities/production/concrete/components.rs` | Serializable + ECSRuntime | Config + runtime split. |
| `src/entities/production/concrete/systems.rs` | ECSRuntime | `ConcreteRuntimePlugin`. |
| `src/entities/production/concrete/sys.rs` | LegacyMigration | Hard-disabled from `mod.rs`. |
| `src/entities/production/aluminum/mod.rs` | Glue | |
| `src/entities/production/aluminum/components.rs` | Serializable + ECSRuntime | Config + runtime split. |
| `src/entities/production/aluminum/systems.rs` | ECSRuntime | `AluminumRuntimePlugin`. |
| `src/entities/production/aluminum/production_sys.rs` | LegacyMigration | Hard-disabled. |
| `src/entities/production/power/mod.rs` | Glue | |
| `src/entities/production/power/components.rs` | ECSRuntime | |
| `src/entities/production/power/power_states.rs` | Serializable | Enums fully serializable. |
| `src/entities/production/power/systems.rs` | ECSRuntime | `PowerRuntimePlugin`. |
| `src/entities/production/prod_comps.rs` | LegacyMigration | Broken legacy components. |
| `src/systems/production/mod.rs` | Glue | |
| `src/systems/production/manifest.rs` | Serializable | Domain registry. |
| `src/systems/production/runtime.rs` | ECSRuntime | Aggregates runtime trio. |
| `src/systems/production/tools_ui.rs` | ToolsUI | Tools state + plugin. |
| `src/systems/production/power_systems.rs` | LegacyMigration | Disabled. |
| `src/systems/production/production_consumption.rs` | LegacyMigration | Disabled. |

#### Entities (general)

| Path | Layer | Notes |
|---|---|---|
| `src/entities/mod.rs` | Glue | |
| `src/entities/prelude.rs` | LegacyMigration | References missing modules (`e_componets`, `strukturave`). |
| `src/entities/types.rs` | LegacyMigration | References missing `types_aliases`. |
| `src/entities/components.rs` | LegacyMigration | Audit imports. |
| `src/entities/entity.rs` | LegacyMigration | Audit imports. |
| `src/entities/types/*` | Serializable | Enums + flags; some `Serialize`-ready. |
| `src/entities/structure/*` | LegacyMigration | Broken imports. |
| `src/entities/vehicles/*` | LegacyMigration | Broken imports. |

#### Systems (game)

| Path | Layer | Notes |
|---|---|---|
| `src/systems/mod.rs` | Glue | |
| `src/systems/agents/*` | ECSRuntime | Permissions/manager/multiplayer. |
| `src/systems/collision/mod.rs` | ECSRuntime | |
| `src/systems/damage/*` | ECSRuntime | |
| `src/systems/navigation/*` | ECSRuntime | |

#### GUI

| Path | Layer | Notes |
|---|---|---|
| `src/gui/mod.rs` | Glue | |
| `src/gui/main_menu.rs` | ToolsUI | |
| `src/gui/splash.rs` | ToolsUI | |
| `src/gui/in_game_ui.rs` | ToolsUI | (in-game runtime UI subset). |
| `src/gui/ui_windows.rs` | ToolsUI | |
| `src/gui/agent_permissions_ui.rs` | ToolsUI | |
| `src/gui/gui_assets.rs` | ToolsUI | |
| `src/gui/gui_sets.rs` | ToolsUI | |
| `src/gui/components/*` | ECSRuntime | |
| `src/gui/editor/world_gen_ui.rs` | ToolsUI | |
| `src/gui/editor/world_preview.rs` | ToolsUI | |

#### IO

| Path | Layer | Notes |
|---|---|---|
| `src/io/mod.rs` | Glue | |
| `src/io/mouse.rs` | ECSRuntime | |
| `src/io/templates.rs` | LegacyMigration | Broken imports. |
| `src/io/serialization/*` | Serializable | Deserializers/resource loaders. |

#### Misc

| Path | Layer | Notes |
|---|---|---|
| `src/idgen.rs` | Serializable | |
| `src/core/*` | Serializable | |
| `src/traits/*` | Serializable | Trait definitions only. |
| `src/render/*` | ECSRuntime | |
| `src/utils/*` | Mixed | Audit `events.rs` for layer split. |
| `src/events/*` | ECSRuntime | Bevy events. |

---

### Step 2 — Import-graph conformance (back-edges flagged)

| File | Bad Import | Reason | Fix |
|---|---|---|---|
| `src/entities/production/concrete/sys.rs` | `crate::production::core::resources::ResourceType` | Wrong root path | Already legacy; do not fix. |
| `src/entities/production/aluminum/production_sys.rs` | `crate::production::core::resources::ResourceType` | Same | Legacy; ignore. |
| `src/systems/production/power_systems.rs` | `crate::systems::production::production_consumption::*`, `crate::entities::types_of::*`, `crate::entities::production::power_comps::*` | Stale namespace + duplicate enum sources | Legacy; ignore. |
| `src/systems/production/production_consumption.rs` | `crate::entities::e_componets::*`, `crate::entities::damages::*`, `crate::entities::e_states::*`, `crate::entities::e_flag_types::*` | Missing modules | Legacy; ignore. |
| `src/entities/prelude.rs` | `e_componets`, `e_infos`, `e_states`, `types_aliases`, `types_of`, `strukturave` | None of these modules exist | Demote `prelude.rs` to legacy migration shim. |
| `src/entities/types.rs` | `types_of`, `types_aliases` | Missing | Migrate or legacy-mark. |
| `src/entities/structure/components.rs` | `crate::entities::types_of::*` | Missing | Legacy. |
| `src/entities/components.rs` | `prelude::*` (broken) | Cascades | Legacy. |
| `src/io/templates.rs` | `crate::io::deserialzers::derezers::*` | Typo + missing | Legacy. |
| `src/entities/vehicles/road_vehicles.rs` | `crate::entity::*`, `crate::resource::*` | Wrong roots | Legacy. |

Direction rule violations to enforce going forward:
- `Serializable` files importing from `bevy::ecs`, `Component`, `Resource`, `egui` must be relocated.
- `ToolsUI` files writing to ECS state outside explicit apply systems must be refactored.

---

### Step 3 — Data contract normalization (Serialize/Deserialize audit)

Already canonical:
- `terrain::biome::*` — fully serializable.
- `terrain::ecology::*` — fully serializable.
- `entities::production::*::components.rs` — config types serializable, runtime types are ECS-only.
- `entities::production::power::power_states.rs` — fully serializable.
- `entities::production::core::manufacturing.rs` — blueprints serializable.
- `systems::production::manifest.rs` — registry resource.

Pending (not blocking):
- `engine::states::*` — add `Serialize/Deserialize` for save systems.
- `WorldGenParams` (subengine) — already serializable.
- `WorldData` (subengine) — already serializable.
- `RegionMethod` vs `RegionMethodType` — collapse to one canonical serializable enum (tracked in terrain biome matrix).

---

### Step 4 — Plugin boundary trios

Each domain MUST expose:
- `<Domain>RuntimePlugin` (ECSRuntime).
- `<Domain>ToolsUiPlugin` (ToolsUI). (May be shared aggregate `ProductionToolsUiPlugin` until per-domain panels added.)
- Optional `<Domain>SerializationPlugin` (if save/load needed).

Current state:

| Domain | Runtime | ToolsUI | Serialization |
|---|---|---|---|
| World generation | `WorldGenerationInGamePlugin` | `WorldGenerationToolsUiPlugin` | `WorldGeneratorSubenginePlugin` (save/load events) |
| Concrete | `ConcreteRuntimePlugin` | `ProductionToolsUiPlugin` (shared) | TODO |
| Aluminum | `AluminumRuntimePlugin` | `ProductionToolsUiPlugin` (shared) | TODO |
| Power | `PowerRuntimePlugin` | `ProductionToolsUiPlugin` (shared) | TODO |
| Manufacturing core | (covered by `ProductionRuntimePlugin`) | `ProductionToolsUiPlugin` (shared) | TODO |
| Agents | `AgentSystemsPlugin` | `AgentPermissionsUiPlugin` | TODO |
| Damage | `DamageSystem` | TODO | TODO |
| Navigation | (per-system) | TODO | TODO |
| Render | (per-plugin) | n/a | n/a |

Convention going forward:
- New domains MUST appear in `ProductionManifest::domains` (or equivalent registry) so agent tools can discover them.

---

### Step 5 — Legacy retirement queue + symbol map

Hard-disabled (kept on disk, not in module graph):
- `src/entities/production/concrete/sys.rs`
- `src/entities/production/aluminum/production_sys.rs`
- `src/systems/production/power_systems.rs`
- `src/systems/production/production_consumption.rs`
- `src/entities/production/prod_comps.rs` (recommend marking)
- `src/engine/engine.rs` (legacy alt EnginePlugin path)

Recommended next disables (keep file, add `LEGACY` header, remove from active module graph):
- `src/entities/prelude.rs` → reduce to empty or shim, references missing modules.
- `src/entities/types.rs` → split into canonical types under `entities::types::*` only.
- `src/entities/components.rs` → split or demote.
- `src/entities/structure/*` → triage per file.
- `src/entities/vehicles/road_vehicles.rs` → split into config (Serializable) + runtime (ECSRuntime).
- `src/io/templates.rs` → demote.

Symbol replacement map (carry forward):

| Old | New |
|---|---|
| `BiomeType` | `terrain::biome::TerrainClass` |
| `GameBiomeType` | `terrain::biome::BiomeBucket` |
| `TerrainType` (legacy) | `terrain::bevy_terrain::LegacyTerrainType` (kept) + canonical `TerrainClass` |
| `AluminumProductionSettings` | `AluminumProductionConfig` |
| `BauxiteMine` | `BauxiteMineRuntime` |
| `AluminaRefinery` | `AluminaRefineryRuntime` |
| `AluminumSmelter` | `AluminumSmelterRuntime` |
| `AluminumFabricationPlant` | `AluminumFabricationPlantRuntime` |
| `ConcreteProductionSettings` | `ConcreteProductionConfig` |
| `CementKiln` | `CementKilnRuntime` |
| `AggregateMine` | `AggregateMineRuntime` |
| `ConcreteMixer` | `ConcreteMixerRuntime` |
| `ConcreteProductionPlugin` | `ConcreteRuntimePlugin` |
| `AluminumProductionPlugin` | `AluminumRuntimePlugin` |
| `PowerSysPlugin` | `PowerRuntimePlugin` |

---

### Step 6 — Automated checks

Crate root `build.rs` runs a **compile-time** scan of `src/**/*.rs` (skips known legacy paths). Update `LEGACY_EXACT_FILES` there when adding new migration-only sources.

Banned-import patterns (must produce errors in CI/grep gate):

```
crate::production::                # wrong root
crate::entities::types_of          # missing namespace
crate::entities::e_componets       # missing
crate::entities::damages           # missing
crate::entities::e_states          # missing
crate::entities::e_flag_types      # missing
crate::entities::types_aliases     # missing
crate::entities::strukturave       # missing
crate::entities::production::power_comps   # legacy
crate::entities::production::prod_comps    # legacy (transitional)
crate::systems::production::power_systems  # legacy
crate::systems::production::production_consumption  # legacy
```

Recommended Powershell grep gate (manual until CI):

```powershell
$banned = @(
  'crate::production::',
  'crate::entities::types_of',
  'crate::entities::e_componets',
  'crate::entities::damages',
  'crate::entities::e_states',
  'crate::entities::e_flag_types',
  'crate::entities::types_aliases',
  'crate::entities::strukturave',
  'crate::entities::production::power_comps',
  'crate::entities::production::prod_comps',
  'crate::systems::production::power_systems',
  'crate::systems::production::production_consumption'
)
foreach ($p in $banned) {
  Get-ChildItem -Recurse -Include *.rs src |
    Select-String -Pattern ([regex]::Escape($p)) |
    Where-Object { $_.Line -notmatch '// LEGACY' }
}
```

Layer enforcement helpers:
- Every `Serializable` file SHOULD NOT contain `Component` / `Resource` / `Plugin` derives or impls.
- Every `ToolsUI` file SHOULD scope `egui` use to view/draw layer; mutations go through events.
- Every domain file MUST end with at least one of:
  - `pub fn <name>_system(`
  - `pub struct <Name>RuntimePlugin;`
  - `pub struct <Name>ToolsUiPlugin;`
  - `Serialize/Deserialize` derive
  …or be a `mod.rs` glue file.

---

### Cross-cutting follow-ups (beyond steps 1–6)

- Bevy upgrade: ⏳ **`Cargo.toml` targets `bevy = "0.18"`** — finish Phase D–H in `prompts/matrix/engine_bevy/bevy_0_18_migration_plan.md` (`cargo check` clean = done).
- ~~`EntityId` / `id_generator` overlap~~ → ✅ **`src/idgen.rs` canonical** (`src/core/id_generator.rs` removed). See designer legacy changelog.
- World gen entrypoints: ⏳ **still dual** (`world_generator_enhanced` vs `bevysubengines`); unify in a later pass (do not block boundary work).
- **`prompts/` index** — ✅ see `prompts/README.md` + `prompts/INDEX.md`. Matrix docs live under `prompts/matrix/`:
  1) `prompts/system_refactor.opai_.rtf` (+ `prompts/system_refactor_NOTE.md`)
  2) `prompts/matrix/terrain_biome/terrain_biome_migration_matrix_v1.md`
  3) `prompts/matrix/production/production_migration_matrix_v1.md`
  4) `prompts/matrix/repo/repo_boundary_matrix_v1.md`

---

### Prompt fragment for downstream agents

Use exactly:

1. Read `prompts/matrix/repo/repo_boundary_matrix_v1.md`.
2. Confirm any change preserves `Serializable`, `ECSRuntime`, `ToolsUI`, `LegacyMigration` boundaries.
3. Never re-import a banned path (Step 6).
4. Never write to ECS state from `Serializable` or `ToolsUI` directly.
5. When adding a new production domain, register it in `ProductionManifest::domains` and provide the plugin trio.
6. Update this matrix file when boundaries shift.
