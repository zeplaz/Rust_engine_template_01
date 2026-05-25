# Experience / Visual Aid v2 step packs

> **Orchestrator:** [`../../../guides/visual_aidv2_runbook_v1.md`](../../../guides/visual_aidv2_runbook_v1.md)  
> **Design:** [`../../../../src/dev/visual_aidv2.md`](../../../../src/dev/visual_aidv2.md)  
> **Live board:** [`../../../../src/dev/visual_aidv2_live_todos.rs`](../../../../src/dev/visual_aidv2_live_todos.rs)

## Sequencing

| Order | Pack | Phase | Status |
|:---:|:---|:---:|:---:|
| 0 | (orchestrator only) | VA0 bootstrap | Applied |
| 1 | [`v1_hud_steps_v1.md`](v1_hud_steps_v1.md) | HUD panel state machine | Pending |
| 2 | [`v2_footprint_steps_v1.md`](v2_footprint_steps_v1.md) | Footprint GPU tiles | Pending |
| 3 | [`v3_readability_steps_v1.md`](v3_readability_steps_v1.md) | Tile readability clamp | Pending |
| 4 | [`v4_lod_scale_steps_v1.md`](v4_lod_scale_steps_v1.md) | Band-driven building visual | Pending |
| 5 | [`v5_camera_steps_v1.md`](v5_camera_steps_v1.md) | Zoom bias + ortho scaffold | Pending |
| 6 | [`v6_icons_steps_v1.md`](v6_icons_steps_v1.md) | Strategic icons | Pending |

**Parallel after VA0:** VA1 and VA2 (disjoint file sets). **FULL_APP verify** after each merge.

## Invariants reminder

- Footprint: one producer → GPU `TileDebugInstanceMap` (WorldMain).
- HUD: `HudPanelState` only — no new `expanded: bool`.
- Readability: `LodInputs` bias only — no second resolver.
- Stage 5 exit unchanged — Visual Aid rows close on their own predicates.
