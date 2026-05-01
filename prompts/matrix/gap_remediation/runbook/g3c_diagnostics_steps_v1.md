# G3C — Diagnostics UI remediation `v1`

> **STATUS:** Placeholder sub-pack. Do not author Rust steps until data source, authority, and owning spec are recorded.

**Pair:** G3 index [`g3_gui_todos_steps_v1.md`](g3_gui_todos_steps_v1.md) · GUI orchestrator [`../../../guides/gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) · gap orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md).

---

## Scope

Diagnostics tabs in [`../../../../src/gui/diagnostics_ui.rs`](../../../../src/gui/diagnostics_ui.rs), including chunk streamer, production manifest, faction roster, FSM / dynamic-state views, and performance counters.

---

## Promotion blockers

| Question | Required answer |
|:---|:---|
| Data source | Which resources / events expose chunk streaming, production manifest, faction roster, and FSM diagnostics? |
| Authority | Diagnostics are read-only by default; any repair / reset button needs explicit command authority. |
| UI boundary | egui is acceptable for diagnostics and must be marked `TEMP-EGUI` only when it stands in for eventual in-game UI. |
| Thread safety | Which values are sampled / cached to avoid blocking gameplay threads? |

---

## Placeholder sequencing

1. Data source map.
2. Chunk streamer diagnostics tab.
3. Production manifest tab.
4. Faction roster diagnostics tab.
5. FSM / performance diagnostics tab.

No `G3C-SNN` atomic Rust steps exist until blockers are answered.
