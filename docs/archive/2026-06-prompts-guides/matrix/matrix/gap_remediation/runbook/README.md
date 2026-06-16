# Gap remediation runbook — step packs

> **STATUS:** Index of atomic-step packs for **Rust phases G1-G5** of implementation-gap remediation. Pair with the orchestrator at [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md), the hunt guide at [`../../../guides/implementation_gap_hunt_runbook_v1.md`](../../../guides/implementation_gap_hunt_runbook_v1.md), and the active owning matrix for each phase.

Version: `v1.0.1`

---

## Active vs closed

| Band | Phases | Entry |
|:---|:---|:---|
| **Active** | **G3** (GUI + execution cycle); **G2 / G4 / G5** placeholder packs (parked — no Rust steps until brief §4 **BQ-109+** resolved) | This README · [`g3_gui_todos_steps_v1.md`](g3_gui_todos_steps_v1.md) · [`g3_execution_cycle_v1.md`](g3_execution_cycle_v1.md) |
| **Closed / legacy** | **G1** hydrology **Applied** (capsule); **U3–U7** terrain maintenance (separate track) | [`g1_hydrology_steps_v1.md`](g1_hydrology_steps_v1.md) (capsule) · full steps [`../../../legacy_runbooks/gap_remediation/g1_hydrology_steps_v1_FULL.md`](../../../legacy_runbooks/gap_remediation/g1_hydrology_steps_v1_FULL.md) · terrain [`../../../legacy_runbooks/terrain/terrain_u_applied_maintenance_v1.md`](../../../legacy_runbooks/terrain/terrain_u_applied_maintenance_v1.md) · index [`../../../legacy_runbooks/README.md`](../../../legacy_runbooks/README.md) |

---

## How to use

1. Read the orchestrator first: [`gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md) §§1-6 (invariants, anchor set, schema, loop protocol, halt rules).
2. Open the **single** step pack for the active phase. **G1 is Applied** — use it for audits only; active work starts from **[`g3_execution_cycle_v1.md`](g3_execution_cycle_v1.md)**, **G3 GUI**, and the backlog wave (see orchestrator §10).
3. Execute exactly **one atomic step** per loop iteration; verify with `cargo check -p proc_A_dine01` + the named test in the step (or *Verify: N/A* for doc-only steps).
4. Flip the matching matrix/routing row named in the step before moving on.

---

## Step packs

| Phase | Pack | Scope |
|:---:|:---|:---|
| **G1** | [`g1_hydrology_steps_v1.md`](g1_hydrology_steps_v1.md) | Hydrology priority pack: unify legacy ECS hydrology, legacy subengine stubs, and p4 chunk pipeline; replace greedy flow with D8 / priority-flood; wire ECS visuals to p4 output |
| **G2** | [`g2_power_placeholders_steps_v1.md`](g2_power_placeholders_steps_v1.md) | Power placeholders; **G2-S00** doc traceability first; Rust steps after promotion blockers |
| **G3** | [`g3_gui_todos_steps_v1.md`](g3_gui_todos_steps_v1.md) · [`g3_execution_cycle_v1.md`](g3_execution_cycle_v1.md) | **Active** GUI index + 14-step GUI ↔ backlog cycle; results template [`g3_cycle_results_TEMPLATE.md`](g3_cycle_results_TEMPLATE.md) |
| **G4** | [`g4_serialization_stubs_steps_v1.md`](g4_serialization_stubs_steps_v1.md) · [`g4_transport_r8_network_slice_steps_v1.md`](g4_transport_r8_network_slice_steps_v1.md) | Serialization stubs + **Transport / R8** slice (hydrate boundary; pair [`../../transport/runbook/r9_authoring_bake_order_steps_v1.md`](../../transport/runbook/r9_authoring_bake_order_steps_v1.md)) |
| **G5** | [`g5_nav_damage_steps_v1.md`](g5_nav_damage_steps_v1.md) | Navigation, damage, and manufacturing placeholders; placeholder pack until owners and matrices are chosen |

---

## Invariants reminder (full list in orchestrator §1)

- One canonical implementation per concept; stubs must route to real behavior or be explicitly deprecated.
- Atomic steps touch **1-3 files** only.
- Do not invent gameplay behavior, ids, save schema, or asset paths; write `ASK:` instead.
- Deterministic systems must remain deterministic under the same seed + committed configs.
- Save-visible values use names, not raw ids, unless an owning matrix says otherwise.
- Hot-reload remains `Assets<T>` / file watcher driven; tools do not create a second mutation path.
- G1 hydrology must not create a second river/lake system beside p4.

---

## G3 GUI sub-packs

| Sub-pack | Surface | Status |
|:---|:---|:---:|
| [`g3a_faction_editor_steps_v1.md`](g3a_faction_editor_steps_v1.md) | Faction editor | **Recorded policy** + §3 living map |
| [`g3b_production_hud_steps_v1.md`](g3b_production_hud_steps_v1.md) | Production HUD | **Recorded policy** + §3 living map |
| [`g3c_diagnostics_steps_v1.md`](g3c_diagnostics_steps_v1.md) | Diagnostics | **Recorded policy** + §3 living map |
| [`g3d_runtime_ui_steps_v1.md`](g3d_runtime_ui_steps_v1.md) | Runtime in-game UI | **Recorded policy** + §3 living map |

Pair with [`../../../guides/gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) and execute steps in [`g3_execution_cycle_v1.md`](g3_execution_cycle_v1.md). Completed sub-packs: write `results/g3<letter>_cycle_results_v1.md` from [`g3_cycle_results_TEMPLATE.md`](g3_cycle_results_TEMPLATE.md).

---

## Sequencing

**G1** is **Applied** (hydrology). Active sequence is **G3 GUI** → backlog wave **S > P > C** via [`../../../guides/backlog_serialization_preview_streaming_runbook_v1.md`](../../../guides/backlog_serialization_preview_streaming_runbook_v1.md). **G2/G4/G5** remain parked until [`../../../guides/rulebook_backlog_designer_brief_v1.md`](../../../guides/rulebook_backlog_designer_brief_v1.md) and [`../../../guides/implementation_gap_hunt_runbook_v1.md`](../../../guides/implementation_gap_hunt_runbook_v1.md) §5 answers identify owner, matrix row, and release priority.
