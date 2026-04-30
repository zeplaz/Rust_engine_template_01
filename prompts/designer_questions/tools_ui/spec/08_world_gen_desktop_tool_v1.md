# World generation — desktop tool bridge `v1`

**Purpose:** Designers edit **`assets/config/world_gen_tuning.json`** in the **Python asset editor** (`World Gen` nav page) without running the game. The same file path is used at runtime (`WORLD_GEN_TUNING_JSON_PATH` in `world_generator_enhanced.rs`).

## Pointers

| Item | Location |
|:---|:---|
| Active tuning (optional on disk) | `assets/config/world_gen_tuning.json` |
| Committed reference | `assets/config/world_gen_tuning.example.json` |
| Rust overlay type | `src/terrain/generation/tuning_io.rs` (`WorldGenTuningOverlay`) |
| In-game panel | F8 → egui **World Generator** (`world_gen_ui.rs`) — reload/save same path |
| Headless / dev binary | `cargo run --bin world_generator` |
| Pipeline narrative | `prompts/designer_questions/terrain_world/spec/01_world_generation_pipeline.md` |
| Layered fields + pixel preview (designer topic) | `prompts/designer_questions/terrain_world/composite_style_worldgen_v1.md` |
| Preview / tuning integration matrix | `prompts/matrix/terrain_biome/composite_style_preview_integration_matrix_v1.md` |
| Material / tag / rule system (designer) | `prompts/designer_questions/terrain_world/material_tag_rule_system_v1.md` |
| Material unification matrix | `prompts/matrix/terrain_biome/material_unification_matrix_v1.md` |

## Editor UI

- **World Gen page:** `src/utils/asset_tools/src/pages/worldgen_page.py`
- **Terrain registry pages:** `src/utils/asset_tools/src/pages/terrain_registry_pages.py` — **Materials** (JSON), **Tags** (JSON), **Rules** (RON)
- **Paths:** `repo_paths.world_gen_tuning_*`; `repo_paths.material_registry_*`, `tag_registry_*`, `material_rules_*`

**World Gen** tabs: **Overview** (workflow + open docs), **Noise sampling**, **Biome tuning**, **Full JSON**.

**Materials / Tags / Rules:** single-tab editors each: Load active, Load example, Save active, Copy example → active; links to designer doc + [`material_unification_matrix_v1.md`](../../../matrix/terrain_biome/material_unification_matrix_v1.md).

Committed examples (copy to active filenames when bootstrapping):

- `assets/config/terrain/material_registry.example.json`
- `assets/config/terrain/tag_registry.example.json`
- `assets/config/terrain/material_rules.example.ron`

Workflow (world gen): **Load example** → edit in a tab → switch tabs (validated merge) → **Save active** (or **Copy example → active**). Use **Biome tuning** when you need Whittaker / threshold fields not all exposed in F8 egui.

Workflow (terrain): edit **Materials** / **Tags** / **Rules** separately; **Save active** writes `assets/config/terrain/<file>` (engine loaders **Pending** — see unification matrix U3+).

Buttons on **World Gen → Overview:** open pipeline **01**, designer topic `composite_style_worldgen_v1.md`, and `composite_style_preview_integration_matrix_v1.md`.

## Related

- [`06_asset_content_studio_workflow_v1.md`](06_asset_content_studio_workflow_v1.md) — `assets/` layout.
- [`07_asset_editor_dual_chain_audit_v1.md`](07_asset_editor_dual_chain_audit_v1.md) — canonical editor stack.
- [`composite_style_worldgen_v1.md`](../../terrain_world/composite_style_worldgen_v1.md) — layered world gen + preview narrative.
- [`../../../matrix/terrain_biome/composite_style_preview_integration_matrix_v1.md`](../../../matrix/terrain_biome/composite_style_preview_integration_matrix_v1.md) — preview / tuning integration matrix.
- [`material_tag_rule_system_v1.md`](../../terrain_world/material_tag_rule_system_v1.md) — registry, tags, rules pipeline.
- [`../../../matrix/terrain_biome/material_unification_matrix_v1.md`](../../../matrix/terrain_biome/material_unification_matrix_v1.md) — integration matrix + phases U0–U6.
