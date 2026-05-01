# G3D — Runtime in-game UI remediation `v1`

> **STATUS:** Placeholder sub-pack. Do not author Rust steps until data source, authority, and owning spec are recorded.

**Pair:** G3 index [`g3_gui_todos_steps_v1.md`](g3_gui_todos_steps_v1.md) · GUI orchestrator [`../../../guides/gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) · gap orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md).

---

## Scope

Runtime in-game UI rewrite marker in [`../../../../src/gui/in_game_ui.rs`](../../../../src/gui/in_game_ui.rs). In-game menus are higher priority than options / secondary menus unless those are trivial to complete.

---

## Promotion blockers

| Question | Required answer |
|:---|:---|
| Data source | Which ECS resources / states drive runtime menus, HUD, alerts, and player controls? |
| Authority | Which interactions send events, write ECS components, or write files? Read-only unless a mutation path is explicitly justified. |
| UI boundary | Bevy UI is the target for runtime in-game menus. egui placeholders must be labelled `TEMP-EGUI` and scheduled for replacement. |
| UX priority | Which menu / panel ships first, and which options menus can remain deferred? |

---

## Placeholder sequencing

1. Data source and authority map.
2. Core in-game menu shell.
3. Runtime HUD / alert bridge.
4. Event-driven interaction model.
5. Remove or replace `TEMP-EGUI` runtime placeholders.

No `G3D-SNN` atomic Rust steps exist until blockers are answered.
