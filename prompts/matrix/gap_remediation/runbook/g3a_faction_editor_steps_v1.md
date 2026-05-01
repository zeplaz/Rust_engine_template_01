# G3A — Faction editor UI remediation `v1`

> **STATUS:** **Recorded direction** — promotion principles below are set. Maintain a living **data source / authority map** in this sub-pack as roster, blueprints, and diplomacy wiring evolve (assets + ECS). Author `G3A-SNN` steps only when a row in §3 is filled for the feature you touch.

**Pair:** G3 index [`g3_gui_todos_steps_v1.md`](g3_gui_todos_steps_v1.md) · GUI orchestrator [`../../../guides/gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) · gap orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md).

---

## Scope

Faction tooling panels in [`../../../../src/gui/faction_tools_ui.rs`](../../../../src/gui/faction_tools_ui.rs), including roster, blueprint binding, diplomacy matrix, and import/export.

---

## 1. Owning spec (principle)

- **G3A owns the faction-editor UI surface** — not “all UI.” Link behavior to **faction / diplomacy** deployment: designer specs under `prompts/designer_questions/factions/`, save-affecting DTO notes in [`../../serialization/serialization_hybrid_migration_matrix_v1.md`](../../serialization/serialization_hybrid_migration_matrix_v1.md) (paired **factions** row), and any faction matrix rows that apply.
- Other domains (buildings-only, terrain-only, etc.) get **their** specs; cross-link here only where the editor actually reads that domain.

---

## 2. Recorded answers (policy)

| Topic | Decision |
|:---|:---|
| **Data source** | **Assets + ECS resources** tied to faction roster, blueprints, diplomacy, and scenario/scenario files as implemented — **not** a one-time frozen list. Update §3 when new resources or loaders land. |
| **Authority** | **Per action** — read-only views vs scenario/file writes vs ECS events/commands. Each mutation path gets a **test** that proves the boundary ([`gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) §1). |
| **UI boundary** | First in-game pass can be **`TEMP-EGUI`** or **Bevy UI** per what ships first; **prefer Bevy** where widgets exist. Desktop asset tools **never** render in-game UI — file edit + reload only. |
| **Diagnostics overlap** | Deep streamer/production dumps belong under **G3C**; this pack stays **editor-** and **faction-domain**-shaped. |

---

## 3. Living map (fill / refresh per promotion)

| UI area | Data source (resources / assets / files) | Read / write authority | Notes |
|:---|:---|:---|:---|
| Roster | *to be filled as wired* | Read default; edits → *explicit path + test* | |
| Blueprint binding | *to be filled* | | |
| Diplomacy matrix | *to be filled* | | |
| Import/export | *to be filled* | File boundary vs ECS | |

---

## Placeholder sequencing

1. Complete §3 rows for the first shippable slice (or mark `ASK:` + **BQ-102**).
2. Read-only roster view.
3. Blueprint binding UI.
4. Diplomacy matrix view / edit authority (with tests).
5. Import/export file boundary.

First `G3A-SNN` step may start when at least one §3 row is concrete for that slice.
