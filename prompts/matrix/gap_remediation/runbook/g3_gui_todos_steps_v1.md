# G3 — GUI TODO remediation `v1`

> **STATUS:** **Active - next.** GUI gaps span factions, production HUD, diagnostics, and runtime in-game UI. This file is the index; execute concrete work through the sub-packs below and [`../../../guides/gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md).

**Pair:** orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md) · GUI orchestrator [`../../../guides/gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) · hunt guide [`../../../guides/implementation_gap_hunt_runbook_v1.md`](../../../guides/implementation_gap_hunt_runbook_v1.md) · UI boundary guide [`../../../guides/ui_boundary_guide_v1.md`](../../../guides/ui_boundary_guide_v1.md) where applicable.

---

## Scope

G3 owns GUI TODOs surfaced by gap hunt, especially:

- [`../../../../src/gui/faction_tools_ui.rs`](../../../../src/gui/faction_tools_ui.rs) — roster, blueprint binding, diplomacy matrix, import/export TODOs.
- [`../../../../src/gui/in_game_hud.rs`](../../../../src/gui/in_game_hud.rs) — resource counter placeholder and production/world data query.
- [`../../../../src/gui/diagnostics_ui.rs`](../../../../src/gui/diagnostics_ui.rs) — diagnostics tabs for chunk streamer, production manifest, faction roster.
- [`../../../../src/gui/in_game_ui.rs`](../../../../src/gui/in_game_ui.rs) — UI rewrite marker.

---

## Promotion blockers (recorded answers)

Before writing atomic Rust steps in any sub-pack, the sub-pack must record its own data source and read/write authority.

| Question | Required answer |
|:---|:---|
| Owner spec | One sub-pack per surface: faction editor, production HUD, diagnostics, runtime in-game UI. |
| UI boundary | Bevy in-game UI is the goal. egui may be used as a `TEMP-EGUI` placeholder for tweaking / diagnostics only. Desktop asset tools must not cross into in-game UI. |
| Data source | `TODO:` per surface; bubble to [`../../../guides/rulebook_backlog_designer_brief_v1.md`](../../../guides/rulebook_backlog_designer_brief_v1.md) §4 until answered. |
| Permission / authority | Read-only by default for thread safety and performance. Mutations require explicit IO / ECS authority in the surface sub-pack. |

---

## Sub-pack sequencing

Execute in this order unless a higher-priority bug forces a narrow detour:

1. [`g3a_faction_editor_steps_v1.md`](g3a_faction_editor_steps_v1.md) — faction tools panels.
2. [`g3b_production_hud_steps_v1.md`](g3b_production_hud_steps_v1.md) — in-game HUD resource display.
3. [`g3c_diagnostics_steps_v1.md`](g3c_diagnostics_steps_v1.md) — diagnostics tabs.
4. [`g3d_runtime_ui_steps_v1.md`](g3d_runtime_ui_steps_v1.md) — runtime in-game UI rewrite.
5. Matrix/spec close after the active sub-pack records verification.

Each sub-pack starts as a placeholder pack. Author `G3<letter>-SNN` Rust steps only after its surface-specific blockers are answered.
