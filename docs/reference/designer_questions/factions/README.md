# Designer Q — factions & editor tooling

**Paired matrixes:** `prompts/matrix/repo/` (layers), `prompts/matrix/serialization/`, `prompts/matrix/assets/`

> **Prompt use:** Serialization + layer rules before new faction DTOs · cite `src/` persistence touchpoints · `ASK:` for editor UX when blocking sim work.

| File / folder | Role |
|:---|:---|
| `faction_editor_tooling_matrix_v1.md` | Build matrix FE-01…FE-06, v1 product decisions |
| **`faction_editor/README.md`** | **Start here** — index for `00`–`05` spec files |
| `faction_editor/00_scope.md` | In/out of scope, MP |
| `faction_editor/01_data_model.md` | Blueprints, tags, diplomacy DTOs |
| `faction_editor/02_ui_egui_panels.md` | **Tools UI:** panels, plugin pattern, hotkeys |
| `faction_editor/03_persistence.md` | Scenario, save, assets, overlays |
| `faction_editor/04_validation.md` | Validation without hard faction cap |
| `faction_editor/05_integration_tests.md` | CI / headless checks |
| **`implementation_questions_v1.md`** | **Engineering checklist** |
| `diplomacy_bargaining_reference_outline_v1.md` | **Non-authoritative** Spaniel / game-theory outline (Bayesian beliefs, signaling, equilibrium, bargaining-failure causes) — pre-authoring scratchpad until a diplomacy matrix exists |
| **Meta-runbook (authoring guide)** | [`../../guides/system_runbook_authoring_meta_v1.md`](../../guides/system_runbook_authoring_meta_v1.md) — spawn `factions_runbook_v1.md` when ready; **§12** paired norms |
| **Paired runbook pattern (terrain example)** | [`../../guides/terrain_paired_runbooks_queue_v1.md`](../../guides/terrain_paired_runbooks_queue_v1.md) — Q0–Q6 queue when two runbooks must stay in sync |

**Code touchpoints:** `src/systems/agents/permissions.rs`, `src/gui/agent_permissions_ui.rs`, `src/systems/production/tools_ui.rs` (ToolsUI plugin pattern).
