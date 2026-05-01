# G3C — Diagnostics UI remediation `v1`

> **STATUS:** **Recorded direction** — same policy spine as [`gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) and **G3B** (ECS + assets, Bevy-first, `TEMP-EGUI` when needed, cadence by churn). Author `G3C-SNN` steps against §2–§3.

**Pair:** G3 index [`g3_gui_todos_steps_v1.md`](g3_gui_todos_steps_v1.md) · GUI orchestrator [`../../../guides/gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) · gap orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md).

---

## Scope

Diagnostics tabs in [`../../../../src/gui/diagnostics_ui.rs`](../../../../src/gui/diagnostics_ui.rs), including chunk streamer, production manifest, faction roster, FSM / dynamic-state views, and performance counters.

---

## 1. Owning spec

- **G3C owns the diagnostics UI surface** — link to whatever **subsystem** each tab exposes (streaming/chunks, production manifests, factions, FSMs). Specs and matrices for those domains are **separate**; this file only routes each tab to its backend.

---

## 2. Recorded answers (replaces open blocker table)

| Topic | Decision |
|:---|:---|
| **Data source** | **Chunk streaming**, **production manifest**, **faction roster**, and **FSM / dynamic-state** views read from the **ECS resources, components, and events** (plus **assets** where manifest/config is file-backed) that those systems already publish. Tab = thin view over that truth; extend queries when a system adds telemetry — no second hidden pipeline. |
| **Authority** | **Read-only by default.** Any **repair / reset / force** control is **opt-in**: named command or event + **test** per [`gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) §1. |
| **UI boundary** | **egui** is fine for diagnostics tooling. If the tab **stands in** for eventual in-game presentation, mark **`TEMP-EGUI`**. **Prefer Bevy** for any surface that is product-facing long term. |
| **Thread safety / sampling** | Same rule as G3B: **sample or cache** heavy reads; high-churn or safety-critical fields refresh more often; bulky summaries can use **configurable** slower ticks. |

---

## 3. Living map (per tab)

| Tab | Backend (resources / events / assets) |
|:---|:---|
| Chunk streamer | *match `diagnostics_ui` + chunk systems* |
| Production manifest | *production ECS + manifests* |
| Faction roster | *faction ECS / scenario* |
| FSM / performance | *FSM components + counters* |

---

## Placeholder sequencing

1. Align §3 with current `diagnostics_ui.rs`.
2. Chunk streamer diagnostics tab.
3. Production manifest tab.
4. Faction roster diagnostics tab.
5. FSM / performance diagnostics tab.
