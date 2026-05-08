# Simulation expansion — step packs (umbrella) `v1`

> **STATUS:** **S0** + **S1** **verified**; **S2** + **S8** may proceed **in parallel** ([`s2_steps_v1.md`](s2_steps_v1.md), Python alignment runbook).

Version: `v1.0.5`

---

## Purpose

This folder holds **implementation step packs** for the Unified Simulation Expansion Program once owner **matrices** exist. Until then, follow:

- [`../../../guides/system_runbook_authoring_meta_v1.md`](../../../guides/system_runbook_authoring_meta_v1.md)  
- [`../../../guides/simulation_expansion_orchestrator_v1.md`](../../../guides/simulation_expansion_orchestrator_v1.md)

**Living matrices (this parent folder):**

- [`../asset_sim_ownership_matrix_v1.md`](../asset_sim_ownership_matrix_v1.md) — S0 asset → sim ownership.  
- [`../chunk_scheduler_gap_table_v1.md`](../chunk_scheduler_gap_table_v1.md) — S1 chunk coupling + persistence hints.  
- [`../python_asset_tools_a0_inventory_v1.md`](../python_asset_tools_a0_inventory_v1.md) — S8 / Python tools taxonomy inventory.

---

## Suggested future layout

| Phase / area | Notes |
|:---|:---|
| `S0` | Asset audit matrix + ownership rows **Applied** (may run **parallel** with S1) |
| `S1` | Chunk scheduler / dirty / persistence hooks (may run **parallel** with S0) |
| `S2` | Weather — **parallel** with **S8** ([`s2_steps_v1.md`](s2_steps_v1.md)) |
| `S3` | Infrastructure / environment integration |
| `S4` | Concrete industry loop |
| `S5` | Flora ecology fields |
| `S6` | Petroleum strategic flow + policy resources |
| `S7` | Petroleum UI wiring |
| `S8` | **Python asset tools** — **after S0** for broad rewrites; **parallel with S2** for incremental work; [`python_asset_tools_alignment_runbook_v1.md`](../../../guides/python_asset_tools_alignment_runbook_v1.md) |

**Phase letters are indicative;** rename to match [`system_runbook_authoring_meta_v1.md`](../../../guides/system_runbook_authoring_meta_v1.md) when a row is added to §3.

---

**Step packs (live):**

- [`s0_steps_v1.md`](s0_steps_v1.md) — asset → sim ownership (audit matrix).  
- [`s1_steps_v1.md`](s1_steps_v1.md) — chunk scheduler / dirty regions (parallel with S0).  
- [`s2_steps_v1.md`](s2_steps_v1.md) — weather (parallel with S8).

---

## When adding a real step pack

1. Add `sN_steps_v1.md` here (or split per-domain subfolders if matrices grow).  
2. Each step: **Goal · Anchor reads · Touch · Verify · Matrix update · DoD** (copy terrain pattern).  
3. Link the step pack from the relevant **domain runbook** § “Step packs”.

---

## Index of domain runbooks

- [`../../../guides/weather_simulation_runbook_v1.md`](../../../guides/weather_simulation_runbook_v1.md)  
- [`../../../guides/flora_ecology_runbook_v1.md`](../../../guides/flora_ecology_runbook_v1.md)  
- [`../../../guides/chunk_scheduler_runbook_v1.md`](../../../guides/chunk_scheduler_runbook_v1.md)  
- [`../../../guides/infrastructure_environment_integration_v1.md`](../../../guides/infrastructure_environment_integration_v1.md)  
- [`../../../guides/concrete_industry_sim_runbook_v1.md`](../../../guides/concrete_industry_sim_runbook_v1.md)  
- [`../../../guides/asset_system_audit_runbook_v1.md`](../../../guides/asset_system_audit_runbook_v1.md)  
- [`../../../guides/petroleum_industry_simulation_runbook_v1.md`](../../../guides/petroleum_industry_simulation_runbook_v1.md)  
- [`../../../guides/ui/petroleum_industry_ui_snippet_v1.md`](../../../guides/ui/petroleum_industry_ui_snippet_v1.md)
- [`../../../guides/python_asset_tools_alignment_runbook_v1.md`](../../../guides/python_asset_tools_alignment_runbook_v1.md)

---

## Proposal index

[`../../../guides/new_propsal_guide_may202608.md`](../../../guides/new_propsal_guide_may202608.md)

---

## Document history

- **2026-05-06:** `v1.0.5` — **S2 ∥ S8**; [`s2_steps_v1.md`](s2_steps_v1.md); `WeatherPlugin` scaffold in engine.  
- **2026-05-06:** `v1.0.4` — S0/S1 packs **verified**; [`chunk_scheduler_gap_table_v1.md`](../chunk_scheduler_gap_table_v1.md); matrix `v1.2.0`.
- **2026-05-06:** `v1.0.3` — **S0** / **S1** step packs added (`s0_steps_v1.md`, `s1_steps_v1.md`).
- **2026-05-06:** `v1.0.2` — S0 parallel with S1; S8 prefers after S0 for engine-faithful tool vocabularies.
