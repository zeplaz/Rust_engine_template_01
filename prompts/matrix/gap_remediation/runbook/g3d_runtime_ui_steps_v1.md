# G3D — Runtime in-game UI remediation `v1`

> **STATUS:** **Recorded direction** — runtime menus, HUD bridges, alerts, and controls are driven by **ECS**; authority is **per interaction**; ship menus as a **continuous flow** without an artificial “defer” list unless product re-prioritizes. Author `G3D-SNN` steps against §2–§3.

**Pair:** G3 index [`g3_gui_todos_steps_v1.md`](g3_gui_todos_steps_v1.md) · GUI orchestrator [`../../../guides/gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) · gap orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md).

---

## Scope

Runtime in-game UI in [`../../../../src/gui/in_game_ui.rs`](../../../../src/gui/in_game_ui.rs). In-game menus remain **higher priority** than secondary/options chrome unless the latter is trivial.

---

## 1. Owning spec

- **G3D owns the runtime in-game menu / shell surface** — cross-link to gameplay UX specs as they exist (pause, alerts, input); do not merge faction-editor or desktop-tool specs into this doc.

---

## 2. Recorded answers (policy)

| Topic | Decision |
|:---|:---|
| **Data source** | **ECS resources, components, and states** drive runtime menus, HUD integration, alerts, and player-facing controls — same evolution rule as G3A–G3C: **extend queries** as simulation features land. |
| **Authority** | **Per interaction** — each path that mutates ECS or files is explicit (+ **test**). Read-only is default for display-only surfaces ([`gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) §1). |
| **UI boundary** | **Bevy UI** is the **target** for runtime in-game menus. **egui** only as **`TEMP-EGUI`** with replacement intent. |
| **UX priority** | **Menus are worked continuously** until product says otherwise — **no fixed “ship A before B”** list here; use engineering order that keeps the game playable and tested. Options/secondary can follow core flows unless they are trivial wins. |

---

## 3. Living map

| Flow | Data source | Authority notes |
|:---|:---|:---|
| Core menu shell | *ECS + game state* | |
| HUD / alert bridge | *ties to G3B presenters* | Read-mostly; mutations → events |
| Player controls / binding UI | *input + settings resources* | Per-setting tests |

---

## Placeholder sequencing

1. Data source + authority map (§3) for first menu slice.
2. Core in-game menu shell (Bevy).
3. Runtime HUD / alert bridge.
4. Event-driven interaction model (+ tests per mutation).
5. Remove or replace `TEMP-EGUI` runtime placeholders.
