# Global asset system audit runbook `v1`

> **STATUS:** Draft **v1** — process scaffold. **Start here** before expanding simulation domains.

Version: `v1.0.1`  
Parent: [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

---

## 1. Purpose

Many assets **classify** but do not declare **simulation ownership**. This runbook mandates an **audit matrix**: for each asset type, record **runtime owner**, **sim layer**, and **lifecycle** (static / dynamic / persistent / recomputed).

---

## 2. Audit matrix (template)

Extend rows for every asset class in the project.

| Asset type | Runtime owner(s) | Sim layer | Static / dynamic | Persistent? |
|:---|:---|:---|:---|:---|
| FuelType | logistics + power | economy / industry | both | partial |
| Surface / material family | terrain + mobility | ontology + derived | both | names yes |
| FireState | damage / ecology | overlay | dynamic | situational |
| VehicleType | transport + agents | agents / logistics | both | partial |
| CargoType | logistics | economy | dynamic | partial |
| ConcreteType | construction + industry | industry | both | partial |
| SegmentMembership | factions / AI | strategic | dynamic | yes |

**Rule:** no row ⇒ not safe to build dependent simulation yet.

---

## 3. Required questions (per asset)

1. What **system owns interpretation** (single primary owner)?  
2. Which layer: **ontology**, **derived metric**, **overlay**, **agent**, **infrastructure**?  
3. Is it **static**, **dynamic**, **persistent**, or **recomputed**?

---

## 4. Critical warning

**Taxonomy-first** design (enums without owners) blocks **simulation ownership architecture**. This audit is the bridge.

---

## 5. Deliverables

1. Maintained **matrix markdown:** [`asset_sim_ownership_matrix_v1.md`](../matrix/simulation_expansion/asset_sim_ownership_matrix_v1.md) (extend rows before building dependent simulation).  
2. Updates to domain runbooks when ownership changes.

---

## 6. Cross-links

- [`new_propsal_guide_may202608.md`](new_propsal_guide_may202608.md) — config vs code ownership.  
- [`implementation_gap_hunt_runbook_v1.md`](implementation_gap_hunt_runbook_v1.md) — find placeholder vs shipped behavior.  
- [`world_assets_tools_rulebook_v1.md`](world_assets_tools_rulebook_v1.md) — tooling touches files, not sim internals.

---

## 7. Step packs

[`../matrix/simulation_expansion/runbook/README.md`](../matrix/simulation_expansion/runbook/README.md) · **S0:** [`../matrix/simulation_expansion/runbook/s0_steps_v1.md`](../matrix/simulation_expansion/runbook/s0_steps_v1.md)

---

## Document history

- **2026-05-06:** `v1.0.1` — deliverable path points at live [`asset_sim_ownership_matrix_v1.md`](../matrix/simulation_expansion/asset_sim_ownership_matrix_v1.md).
