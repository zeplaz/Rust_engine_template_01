# Prompts — index for humans & LLMs

**Token discipline:** read **`llm_agent_brief.md`** first (**Prompt contract**, **Workflow — design Q&A vs implementation**, **Minimal load order**), then **one** matrix row + **one** designer `README.md` → optional **`spec/`** → **`implementation_questions_v1.md`** when coding.  
**Scope:** prompt docs here are for **engine implementation** (types, schedules, phases, tests, tooling) — not marketing or release planning.

---

## Root files

| File | Role |
|:---|:---|
| [`llm_agent_brief.md`](llm_agent_brief.md) | Rules, symbols, hot paths |
| [`INDEX.md`](INDEX.md) | Ultra-short map |
| [`matrix/README.md`](matrix/README.md) | Matrix subsystems ↔ paired designer folders + **nav / factions** (no matrix folder) |

### `guides/` — cross-cutting

| File | Role |
|:---|:---|
| [`guides/ui_boundary_guide_v1.md`](guides/ui_boundary_guide_v1.md) | Bevy UI vs egui |
| [`guides/system_refactor_NOTE.md`](guides/system_refactor_NOTE.md) | How to use RTF vs matrices |
| [`guides/system_refactor.opai_.rtf`](guides/system_refactor.opai_.rtf) | Historical refactor essay |

---

## `matrix/` — nested by subsystem

| Path | Doc |
|:---|:---|
| [`matrix/repo/repo_boundary_matrix_v1.md`](matrix/repo/repo_boundary_matrix_v1.md) | Serializable / ECS / ToolsUI inventory |
| [`matrix/terrain_biome/terrain_biome_migration_matrix_v1.md`](matrix/terrain_biome/terrain_biome_migration_matrix_v1.md) | Biome / terrain type migration |
| [`matrix/production/production_migration_matrix_v1.md`](matrix/production/production_migration_matrix_v1.md) | Production domain plugins |
| [`matrix/engine_bevy/bevy_0_18_migration_plan.md`](matrix/engine_bevy/bevy_0_18_migration_plan.md) | Bevy upgrade checklist |
| [`matrix/serialization/serialization_hybrid_migration_matrix_v1.md`](matrix/serialization/serialization_hybrid_migration_matrix_v1.md) | Save header + binary body |
| [`matrix/strategic_platforms/strategic_platforms_matrix_v1.md`](matrix/strategic_platforms/strategic_platforms_matrix_v1.md) | Platforms, munitions, sensors, EW boundaries |
| [`matrix/assets/bevy_asset_config_migration_matrix_v1.md`](matrix/assets/bevy_asset_config_migration_matrix_v1.md) | RON / asset pipeline |

---

## `designer_questions/` — nested by subsystem

| Folder | Start here |
|:---|:---|
| [`designer_questions/terrain_world/README.md`](designer_questions/terrain_world/README.md) | World/theatre **`spec/`**, chunks, LOD, hydrology + **implementation** |
| [`designer_questions/navigation/README.md`](designer_questions/navigation/README.md) | **`spec/`**, pathfinding + **implementation** |
| [`designer_questions/production_economy/README.md`](designer_questions/production_economy/README.md) | **`spec/`**, power, damage, repair + **implementation** |
| [`designer_questions/strategic_platforms/README.md`](designer_questions/strategic_platforms/README.md) | **`spec/`**, phased delivery, EW/platforms + **implementation** |
| [`designer_questions/tools_ui/README.md`](designer_questions/tools_ui/README.md) | **`spec/`**, debug/perf egui vs Bevy |
| [`designer_questions/factions/README.md`](designer_questions/factions/README.md) | **`faction_editor/`** specs, diplomacy + **implementation** |
| [`designer_questions/_legacy/README.md`](designer_questions/_legacy/README.md) | Monolithic Q&A archive |

---

## Suggested read order (deep refactor)

1. `llm_agent_brief.md`
2. `matrix/repo/repo_boundary_matrix_v1.md`
3. Target **matrix** row + matching **designer_questions** (`README.md` → topic / `spec/` → `implementation_questions_v1.md`)
4. `guides/ui_boundary_guide_v1.md` if touching UI

---

## Conventions

- Preserve Serializable / ECSRuntime / ToolsUI boundaries (`matrix/repo/…`).
- No banned import paths (`build.rs` points at `matrix/repo/…`).
- New production domains → `ProductionManifest` in `src/systems/production/manifest.rs`.
