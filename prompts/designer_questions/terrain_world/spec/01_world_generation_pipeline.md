# World generation pipeline `01`

## Inputs

- **Seed**, region params, island/falloff knobs — align `world_generator_enhanced` / UI (`src/gui/editor/world_gen_ui.rs`).
- **Layered fields + preview** (scalar stack, chunk determinism, pixel preview vs sim): [`../composite_style_worldgen_v1.md`](../composite_style_worldgen_v1.md) · matrix: `prompts/matrix/terrain_biome/composite_style_preview_integration_matrix_v1.md`.
- **Material / tag / rule chain** (registry JSON, RON rules, `MaterialId` runtime): [`../material_tag_rule_system_v1.md`](../material_tag_rule_system_v1.md) · matrix: `prompts/matrix/terrain_biome/material_unification_matrix_v1.md`.
- **Political seed** 📎: optional layer for initial claims (scenario author) vs purely emergent from cities.

## Stages (conceptual)

1. **Height / macro shape** → 2. **Biome weights** → 3. **Hydrology** (`hydrology_v1.md`) → 4. **City sites** (§04) → 5. **Road/rail hints** for nav graph bake 📎.

## Determinism

- Same **integer seed + versioned schema** ⇒ same `ChunkId` set for fixed inputs — tests in `implementation_questions_v1.md` (items 33–34).

## Plugins (code-accurate)

- **`WorldGenToolsPlugin`** = **`WorldGenerationInGamePlugin`** + **`WorldGenerationToolsUiPlugin`** in `world_generation_plugin.rs`.
- **`WorldGenerationPlugin`** is a different bundle (`WorldGeneratorPlugin` + `WorldGenUiPlugin` + `WorldPreviewPlugin`) — confirm which entry binary uses.
- **F8** — `world_gen_key_input` toggles editor UI; keep in sync with `tools_ui` hotkey inventory.

## Serialization

- What lives in **scenario header** vs per-chunk blobs — pair `serialization` matrix.
