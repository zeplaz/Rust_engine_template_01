# G3 — GUI gap remediation `v1`

> **STATUS:** **Active - next.** GUI gaps span factions, production HUD, diagnostics, and runtime in-game UI. This file is the index; execute concrete work through **[`g3_execution_cycle_v1.md`](g3_execution_cycle_v1.md)** (14-step GUI ↔ backlog alternation), the sub-packs below, and [`../../../guides/gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md).

**Pair:** orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md) · GUI orchestrator [`../../../guides/gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) · hunt guide [`../../../guides/implementation_gap_hunt_runbook_v1.md`](../../../guides/implementation_gap_hunt_runbook_v1.md) · UI boundary guide [`../../../guides/ui_boundary_guide_v1.md`](../../../guides/ui_boundary_guide_v1.md) where applicable.

---

## Scope

G3 owns GUI gaps surfaced by gap hunt, especially:

- [`../../../../src/gui/faction_tools_ui.rs`](../../../../src/gui/faction_tools_ui.rs) — roster, blueprint binding, diplomacy matrix, import/export TODOs.
- [`../../../../src/gui/in_game_hud.rs`](../../../../src/gui/in_game_hud.rs) — **Bevy HUD**: **site-bound** `ResourceStorage` strip (glyph + bar + stock + on-entity flows); [`HudLogisticsFocus`](../../../../src/gui/logistics_focus.rs); **F9** cycle; **F10** [`logistics_targets_panel`](../../../../src/gui/logistics_targets_panel.rs).
- [`../../../../src/gui/diagnostics_ui.rs`](../../../../src/gui/diagnostics_ui.rs) — diagnostics tabs for chunk streamer, production manifest, faction roster.
- [`../../../../src/gui/in_game_ui.rs`](../../../../src/gui/in_game_ui.rs) — UI rewrite marker.

---

## Execution order (logical review)

| Priority | Sub-pack | Rationale |
|:---:|:---|:---|
| **1** | **[G3B](g3b_production_hud_steps_v1.md) Production HUD** | Already **Bevy UI** (no egui replatform). One clear data path: **`ResourceStorage`** aggregates. Unblocks visible player feedback with minimal domain risk. |
| **2** | **[G3C](g3c_diagnostics_steps_v1.md) Diagnostics** | **egui** is appropriate here; panel works and extends with **tabs** without waiting on FactionBlueprint. |
| **3** | **[G3A](g3a_faction_editor_steps_v1.md) Faction tools** | **F4** shell exists but panels need **faction data model** wiring; better after HUD/diagnostics patterns exist. |
| **4** | **[G3D](g3d_runtime_ui_steps_v1.md) Runtime menus** | **`InGameUiPlugin` is empty** — full shell/State rewrite; highest structural cost → last. |

---

## Promotion policy (index)

Before writing atomic Rust steps in any sub-pack, align with **recorded policy** in that sub-pack and [`../../../guides/gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) §1.

| Topic | Policy |
|:---|:---|
| Owner spec | **One owning spec per surface** (G3A–G3D), linked to its domain (faction, production, streaming, runtime) — see each sub-pack §1. |
| UI boundary | **Bevy first** where widgets exist; **`TEMP-EGUI`** when not; diagnostics may use egui per [`gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) §1. |
| Data source | **ECS + assets**; **living §3 tables** in each sub-pack. **BQ-102–105** only for a slice still missing a concrete `ASK:` owner. |
| Permission / authority | **Per usage path + test**; read-only default ([`gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) §1). |

---

## Sub-pack sequencing

**Recommended** order matches **Execution order** above (faster HUD / devtools wins):

1. [`g3b_production_hud_steps_v1.md`](g3b_production_hud_steps_v1.md) — in-game HUD (Bevy).
2. [`g3c_diagnostics_steps_v1.md`](g3c_diagnostics_steps_v1.md) — diagnostics tabs (egui).
3. [`g3a_faction_editor_steps_v1.md`](g3a_faction_editor_steps_v1.md) — faction tools panels.
4. [`g3d_runtime_ui_steps_v1.md`](g3d_runtime_ui_steps_v1.md) — runtime in-game UI rewrite.
5. Matrix/spec close after the active sub-pack records verification.

Alternate **G3A-first** order is valid when faction work blocks everything else; document the switch in cycle **results**.

Policy for **owning spec**, **Bevy-first / `TEMP-EGUI`**, **ECS-driven data**, **per-action authority + tests**, and **cadence** is recorded in each sub-pack and the GUI orchestrator. Author **`G3<letter>-SNN`** when the sub-pack’s **§3 living map** covers the slice you implement (or after **`ASK:`** + **BQ-###** for that slice only).
