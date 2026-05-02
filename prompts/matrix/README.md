# Matrix docs — migrations & boundaries

> **Prompt routing:** Pick **one row** below → open that matrix → matching `../designer_questions/<subsystem>/README.md` → optional `spec/` or topic `*_v1.md` → **`implementation_questions_v1.md`** when coding. If the change affects **saves**, also read **`serialization/`**. Confirm ✅ rows against `src/`. Paths relative to `prompts/`.

Each **subsystem folder** holds one migration/boundary doc. **Paired** design + implementation questions live under `prompts/designer_questions/<subsystem>/`.

| Folder | Matrix doc | Paired designer folder |
|:---|:---|:---|
| `terrain_biome/` | `terrain_biome_migration_matrix_v1.md` · addenda **`composite_style_preview_integration_matrix_v1.md`** (preview) · **`material_unification_matrix_v1.md`** (registry/tags/rules; §§13–18 invalidation / layers / U7) · **`runbook/`** (atomic step packs U3–U7; orchestrator at `../../guides/terrain_unification_runbook_v1.md`) | `../designer_questions/terrain_world/` → `terrain_world/spec/README.md` |
| **Meta-runbook** | [`../guides/system_runbook_authoring_meta_v1.md`](../guides/system_runbook_authoring_meta_v1.md) — authoring template for sibling system runbooks (power, weapons, buildings, construction, navigation, factions, diplomacy); **paired terrain queue** [`../guides/terrain_paired_runbooks_queue_v1.md`](../guides/terrain_paired_runbooks_queue_v1.md) | n/a — produces new runbooks under `prompts/guides/` + `prompts/matrix/<area>/runbook/` |
| `production/` | `production_migration_matrix_v1.md` | `../designer_questions/production_economy/` → `production_economy/spec/README.md` |
| `strategic_platforms/` | `strategic_platforms_matrix_v1.md` | `../designer_questions/strategic_platforms/` → `strategic_platforms/spec/README.md` |
| `repo/` | `repo_boundary_matrix_v1.md` | (cross-cutting — read with any subsystem) |
| `engine_bevy/` | `bevy_0_18_migration_plan.md` | `../designer_questions/tools_ui/` → `tools_ui/spec/README.md` |
| `serialization/` | `serialization_hybrid_migration_matrix_v1.md` | `terrain_world/` + `production_economy/` + **`factions/`** (any save-affecting DTO) |
| `assets/` | `bevy_asset_config_migration_matrix_v1.md` · **`runbook/`** (paired terrain asset policy A1–A3; orchestrator [`../../guides/bevy_asset_terrain_runbook_v1.md`](../guides/bevy_asset_terrain_runbook_v1.md)) | `terrain_world/` + `production_economy/` + `factions/` (RON / hand-edited configs) |
| **`map_editor/`** | **`map_editor_matrix_v1.md`** · **`runbook/`** (M1–M5; orchestrator [`../../guides/map_editor_runbook_v1.md`](../guides/map_editor_runbook_v1.md)) | `terrain_world/` (tile ECS, procedural handoff) + **`serialization/`** (M5 saves) |

### Subsystems **without** a dedicated `matrix/<name>/` folder

| Workstream | Use these matrices | Paired designer folder |
|:---|:---|:---|
| **Navigation / pathfinding** | `repo/` (layers) until a nav matrix is split | `../designer_questions/navigation/` → `navigation/spec/README.md` |
| **Factions / diplomacy tooling** | `repo/` + `serialization/` + `assets/` | `../designer_questions/factions/` → `faction_editor/README.md` |

**Entry:** `prompts/llm_agent_brief.md` · **Agent workflow:** design vs implementation — `llm_agent_brief.md` § **Workflow — design Q&A vs implementation** · **Guides:** `prompts/guides/`  
**Scope:** Matrices + designer Qs are for **technical engine delivery** (types, phases, tests) — not marketing or distribution.
