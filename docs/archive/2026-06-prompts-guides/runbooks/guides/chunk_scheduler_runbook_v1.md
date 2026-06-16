# Chunk scheduler, FixedUpdate, dirty regions & persistence `v1`

> **STATUS:** Draft **v1** — design scaffold. Partial patterns exist (e.g. material chunk dirty); full program **Pending**.

Version: `v1.0.0`  
Parent: [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

---

## 1. Purpose

Enable **deterministic**, **scalable** simulation for **100k+ field entities** and **10k+ active detailed entities** via **world partition**, **chunk activation**, **dirty propagation**, **FixedUpdate** scheduling, and **LOD sim tiers**.

---

## 2. Target execution model

```
World partition
  → Chunk activation
  → Dirty region tracking
  → FixedUpdate scheduling
  → LOD simulation tier
```

---

## 3. Simulation LOD tiers

| Tier | Scope | Typical work |
|:---:|:---|:---|
| **LOD0** | Active local | Full ECS detail, agents, physics-adjacent |
| **LOD1** | Nearby chunks | Reduced field updates, simplified agents |
| **LOD2** | Strategic / statistical | Regional aggregates, economy pressure |
| **LOD3** | Dormant | Persisted state only; minimal background refresh |

---

## 4. Dirty chunk rule

Recompute **fields**, **weather**, **routing**, **overlays**, **ecology** only when **inputs change** (registry hash, tuning, neighbor chunk, policy, incidents).

**Align with existing patterns:** material pipeline `ChunkDependency` / `ChunkDirty` is a precedent for **pass-level** invalidation — extend consistently for other subsystems. **Concrete hooks:** `src/terrain/material/dependency.rs` (`ChunkDependency`, `ChunkDirty`), `src/systems/terrain/material_plugin.rs` (`rebuild_dirty_chunks`). **Step pack:** [`../matrix/simulation_expansion/runbook/s1_steps_v1.md`](../matrix/simulation_expansion/runbook/s1_steps_v1.md) (S1-S01).

---

## 5. Persistence policy (default guidance)

| Persist | Examples |
|:---|:---|
| Infrastructure topology | Roads, power graph ownership |
| Economy / ownership | Stockpiles, contracts, faction holds |
| Major overlays | When design requires continuity across saves |
| Weather seeds / climate state | If needed for determinism |

| Recompute | Examples |
|:---|:---|
| Local derived fields | Slopes, transient moisture modifiers |
| Temporary routing caches | Unless saved as intentional player-built state |
| Cached previews | Editor / debug |

Refine per-game; document exceptions in domain runbooks.

---

## 6. Cross-links

- [`weather_simulation_runbook_v1.md`](weather_simulation_runbook_v1.md) — chunk weather tick.  
- [`flora_ecology_runbook_v1.md`](flora_ecology_runbook_v1.md) — field updates.  
- [`petroleum_industry_simulation_runbook_v1.md`](petroleum_industry_simulation_runbook_v1.md) — throughput fields vs agents.  
- ECS schedule: [`ecs_systems_schedule_runbook_v1.md`](ecs_systems_schedule_runbook_v1.md).  
- **Coupling inventory:** [`../matrix/simulation_expansion/chunk_scheduler_gap_table_v1.md`](../matrix/simulation_expansion/chunk_scheduler_gap_table_v1.md) (S1).

---

## 7. Step packs

[`../matrix/simulation_expansion/runbook/README.md`](../matrix/simulation_expansion/runbook/README.md) · **S1:** [`../matrix/simulation_expansion/runbook/s1_steps_v1.md`](../matrix/simulation_expansion/runbook/s1_steps_v1.md)
