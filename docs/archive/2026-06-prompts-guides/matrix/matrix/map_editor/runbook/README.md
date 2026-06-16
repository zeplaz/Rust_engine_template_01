# World map editor — step packs

> **STATUS:** Index of atomic-step packs for **Rust phases M1–M5**. Pair with the orchestrator at [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) and the matrix at [`../map_editor_matrix_v1.md`](../map_editor_matrix_v1.md).

Version: `v1.0.0`

## Step packs

| Phase | Pack | Scope |
|:---:|:---|:---|
| **M1** | [`m1_steps_v1.md`](m1_steps_v1.md) | Menus + `WorldGenFlowState` / `BaseState` routing: FullReady → Editor; load path stub → Editor |
| **M2** | [`m2_steps_v1.md`](m2_steps_v1.md) | `MapEditorPlugin`, `run_if(BaseState::Editor)`, `InGameEditorState` + tool resource, minimal egui palette (`TEMP-EGUI`) |
| **M3** | [`m3_steps_v1.md`](m3_steps_v1.md) | Pick tile + height brush + biome repaint on `TileMarker` entities |
| **M4** | [`m4_steps_v1.md`](m4_steps_v1.md) | Tile-aligned road (or structure) markers + placement mode |
| **M5** | [`m5_steps_v1.md`](m5_steps_v1.md) | Named snapshot serde, save/load from editor; menu **Open in editor** |

## Invariants reminder (full list in orchestrator §1)

- Single source of truth for biome/material types; no parallel classifiers without an explicit step.
- Determinism: same snapshot + configs ⇒ same hydration.
- Saves: **names not raw ids** (stable string keys).
- Hot-reload remains `Assets<T>`-only for designer files; editor mutations are ECS/snapshot paths.
- `ASK:` halts when a matrix or serialization row is missing.
- Runtime egui = `TEMP-EGUI` until GUI runbook replaces with Bevy UI.

## Sequencing

Finish phase **MN** (matrix status **Applied**) before starting **M(N+1)**.
