# G3 execution cycle — GUI ↔ backlog alternation `v1`

> **STATUS:** **Active.** One planned pass through **G3A–G3D** with **≥12** atomic steps. Each **GUI** step uses [`gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md); each **Backlog** step uses [`backlog_serialization_preview_streaming_runbook_v1.md`](../../../guides/backlog_serialization_preview_streaming_runbook_v1.md) in **S → P → C** order for that micro-pass. **Do not** scatter informal open-work notes in cycle artifacts: record outcomes only in [`g3_cycle_results_TEMPLATE.md`](g3_cycle_results_TEMPLATE.md) instances and in brief [`rulebook_backlog_designer_brief_v1.md`](../../../guides/rulebook_backlog_designer_brief_v1.md) **§4 BQ-###** rows.

Version: `v1.0.0`

---

## 1. Anchors

| Doc | Role |
|:---|:---|
| [`g3_gui_todos_steps_v1.md`](g3_gui_todos_steps_v1.md) | G3 index and sub-pack links |
| [`gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) | GUI orchestrator |
| [`backlog_serialization_preview_streaming_runbook_v1.md`](../../../guides/backlog_serialization_preview_streaming_runbook_v1.md) | S / P / C backlog orchestrator |
| [`rulebook_backlog_designer_brief_v1.md`](../../../guides/rulebook_backlog_designer_brief_v1.md) §4 | **BQ-###** owner table |
| [`g3_cycle_results_TEMPLATE.md`](g3_cycle_results_TEMPLATE.md) | Copy → one **results file per completed sub-pack** (and optional cycle-close rollup) |

---

## 2. Micro-loop (repeat every step)

1. Run **one** row from §3 for the active **Orchestrator** column (**GUI** or **Backlog**).
2. If **GUI**: follow [`gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) §5 (one atomic step, `cargo check` where Rust changed).
3. If **Backlog**: read the wave named in §3 (**S**, **P**, or **C**) in [`backlog_serialization_preview_streaming_runbook_v1.md`](../../../guides/backlog_serialization_preview_streaming_runbook_v1.md); perform **doc-only** audit / row note / `ASK:` placement — **no save-schema or streaming threshold invention** without brief/matrix alignment.
4. If a sub-pack reaches **closure criteria** (blockers cleared **or** recorded as `ASK:` with **BQ-###** and owner): write **`results/g3<letter>_cycle_results_v1.md`** from the template (one file per sub-pack completion).
5. Advance to the next §3 row.

**Engineering order:** [`g3_gui_todos_steps_v1.md`](g3_gui_todos_steps_v1.md) recommends **G3B → G3C → G3A → G3D** when unblocking the Bevy HUD early. You may run §3’s **GUI** rows in that sub-pack order instead of the table’s **G3A-first** labels — keep the **GUI ↔ Backlog alternation** and wave **S → P → C** cadence.

---

## 3. Step table (14 steps — alternating orchestrators)

| Step | Orchestrator | Action |
|:---:|:---:|:---|
| **01** | **GUI** | **G3A open.** Read [`g3a_faction_editor_steps_v1.md`](g3a_faction_editor_steps_v1.md); confirm or document **BQ-102** + **BQ-108** readiness for faction surface. |
| **02** | **Backlog** | **Wave S micro-pass.** [`backlog_serialization_preview_streaming_runbook_v1.md`](../../../guides/backlog_serialization_preview_streaming_runbook_v1.md) §4 — audit one terrain-related row in serialization matrix; note **BQ-107** if strategy drifts. |
| **03** | **GUI** | **G3A deliverable.** One atomic Rust or doc step for faction editor per sub-pack (halt per [`gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) §6 if authority missing). |
| **04** | **Backlog** | **Wave S (second touch).** Promote or stub **one** atomic serialization note aligned with matrix §8 / hybrid migration doc. |
| **05** | **GUI** | **G3B open.** Read [`g3b_production_hud_steps_v1.md`](g3b_production_hud_steps_v1.md); confirm **BQ-103** + **BQ-108** as needed for HUD data path. |
| **06** | **Backlog** | **Wave P micro-pass.** Backlog runbook §5 — audit preview / composite consumers against terrain matrix §§6, 9, 15 **read-only** rule. |
| **07** | **GUI** | **G3B deliverable.** One atomic step for production HUD per [`gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) §5. |
| **08** | **Backlog** | **Wave P (second touch).** Record inspector / registry surface choice implications for **BQ-108** only; defer product pick to brief §4 if unset. |
| **09** | **GUI** | **G3C open.** Read [`g3c_diagnostics_steps_v1.md`](g3c_diagnostics_steps_v1.md); align diagnostics with **TEMP-EGUI** invariant (**BQ-108**). |
| **10** | **Backlog** | **Wave C micro-pass.** Backlog runbook §6 — ghost bands / neighbor reads vs **BQ-101**; doc-only alignment with matrix §§13, 16. |
| **11** | **GUI** | **G3C deliverable.** One atomic diagnostics step. |
| **12** | **GUI** | **G3D open + deliverable.** Read [`g3d_runtime_ui_steps_v1.md`](g3d_runtime_ui_steps_v1.md); one atomic runtime UI step (or doc closure if blocked — record **BQ-105** / **BQ-108** in results file). |
| **13** | **Backlog** | **Wave C (closure touch).** Confirm **TaskPool / ChunkCache** wording still matches implementation; bubble **BQ-101** only via brief §4, not inline prose in engine code comments as permanent spec. |
| **14** | **GUI** (meta) | **Cycle close.** Run [`implementation_gap_hunt_runbook_v1.md`](../../../guides/implementation_gap_hunt_runbook_v1.md) §3 cadence on **G3-touching** paths only; resolve mechanical **placeholder / stub** items that need no new product input; append **§3 “Next cycle queue”** to the **last** results file (or add `g3_cycle_rollup_v1.md`) listing **BQ-###** still open and next §3 step index to resume. |

**Adjustment:** If a sub-pack finishes before step 12, still run the **Backlog** rows that follow until the wave **S → P → C** cadence has at least one pass each in the cycle; use step 14 for the global scan and rollup.

---

## 4. Sub-pack completion — single results file

When **G3A**, **G3B**, **G3C**, or **G3D** is **closed for the cycle** (all steps done **or** all blockers owning **BQ-###** in brief §4):

1. Copy [`g3_cycle_results_TEMPLATE.md`](g3_cycle_results_TEMPLATE.md) to `results/g3a_cycle_results_v1.md` (or `g3b` / `g3c` / `g3d`).
2. Fill **Designer summary**, **LLM handoff**, and **Backlog queue deltas** using **only BQ-###** references for open items — **no** informal “todo” / “unfinished” wording.
3. Link the new file from [`g3_gui_todos_steps_v1.md`](g3_gui_todos_steps_v1.md) status line or sub-pack header in the same PR / commit narrative.

---

## 5. Prompt fragment

> Execute [`g3_execution_cycle_v1.md`](g3_execution_cycle_v1.md) §§2-3 one row at a time. Alternate **GUI** and **Backlog** orchestrators as printed. After each sub-pack close, write **`results/g3<letter>_cycle_results_v1.md`** from [`g3_cycle_results_TEMPLATE.md`](g3_cycle_results_TEMPLATE.md). Route all open work through **BQ-###** in [`rulebook_backlog_designer_brief_v1.md`](../../../guides/rulebook_backlog_designer_brief_v1.md) §4 — not scratch TODO lines in the results file.
