# G3B — Production HUD remediation `v1`

> **STATUS:** Placeholder sub-pack. Do not author Rust steps until data source, authority, and owning spec are recorded.

**Pair:** G3 index [`g3_gui_todos_steps_v1.md`](g3_gui_todos_steps_v1.md) · GUI orchestrator [`../../../guides/gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) · gap orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md).

---

## Scope

In-game HUD resource display in [`../../../../src/gui/in_game_hud.rs`](../../../../src/gui/in_game_hud.rs), including resource counters and production / world data queries.

---

## Promotion blockers

| Question | Required answer |
|:---|:---|
| Data source | Which ECS resources / queries provide resource flows, destruction counts, production state, and world-level aggregates? |
| Authority | HUD is read-only by default; any button / mutation must send explicit events or commands. |
| UI boundary | Bevy in-game UI is the target. egui may be `TEMP-EGUI` only for debugging / tuning while the Bevy version is not ready. |
| Performance | What refresh cadence / cache boundary avoids expensive world queries each frame? |

---

## Placeholder sequencing

1. Data source map.
2. Read-only resource counter.
3. Production summary panel.
4. World data summary panel.
5. Bevy UI replacement for any `TEMP-EGUI` placeholder.

No `G3B-SNN` atomic Rust steps exist until blockers are answered.
