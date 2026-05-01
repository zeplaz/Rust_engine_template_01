# G3A — Faction editor UI remediation `v1`

> **STATUS:** Placeholder sub-pack. Do not author Rust steps until data source, authority, and owning spec are recorded.

**Pair:** G3 index [`g3_gui_todos_steps_v1.md`](g3_gui_todos_steps_v1.md) · GUI orchestrator [`../../../guides/gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) · gap orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md).

---

## Scope

Faction tooling panels in [`../../../../src/gui/faction_tools_ui.rs`](../../../../src/gui/faction_tools_ui.rs), including roster, blueprint binding, diplomacy matrix, and import/export TODOs.

---

## Promotion blockers

| Question | Required answer |
|:---|:---|
| Data source | Which resources / assets / scenario files supply faction roster, blueprints, and diplomacy state? |
| Authority | Which actions are read-only, which mutate scenario files, and which send ECS events? |
| UI boundary | Is the first pass `TEMP-EGUI` or Bevy UI? Desktop asset tools may edit files but must not render in-game UI. |
| Owning spec | Which faction / diplomacy matrix or designer question owns the behavior? If none exists, add `ASK:` before Rust. |

---

## Placeholder sequencing

1. Data source map.
2. Read-only roster view.
3. Blueprint binding UI.
4. Diplomacy matrix view / edit authority.
5. Import/export file boundary.

No `G3A-SNN` atomic Rust steps exist until blockers are answered.
