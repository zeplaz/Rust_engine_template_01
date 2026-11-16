# PLAN-COMBAT-SUSTAINMENT-v1 — shared front→TOE refill kernel (fix clone farms)

| Field | Value |
|:---|:---|
| **Id** | **PLAN-COMBAT-SUSTAINMENT-v1** |
| **Status** | **PROGRAM CLOSE** — CS-1…6 landed (`refill_toe_from_front_system` + typed sinks) |
| **Date** | 2026-11-15 |
| **Owner** | `@planner` → `@coder` |
| **Parents** | [`plan_production_restock_spine_v1.md`](plan_production_restock_spine_v1.md) (P3/P5/P6 SHIPPED) · FC-6 front supply · [`combat/mod.rs`](../combat/mod.rs) |
| **Trigger** | Schedule shows three near-identical `refill_*` + `publish_*` twins — abstraction smell, not a third sustainment authority |

---

## 1. Why it feels wrong (honest)

Looking at this chain:

```text
step_drone_missions_system
refill_drone_packages_from_front_system
refill_missile_packages_from_front_system
refill_personnel_credits_from_front_system
formation_supply_system
```

…it *looks* like three different sustainment authorities bolted beside combat. That is the smell.

**What actually happened:** the restock spine shipped P3 (drone) / P5 (personnel) / P6 (missile) as separate slices. Each copied the same loop:

```text
scan FrontSupplyPoints × LogisticsOrdersSnapshot
  → sort formations
  → walk CapabilityToe BOM
  → ConsumeStock
  → bump sink count / credit
```

So the **product sinks** were intentional. The **protocol** was copy-pasted. That is the flawed layer — not “drones vs missiles should be one blob.”

Latent brittleness: each refill rebuilds its own `available` map and ignores sibling `ConsumeStock` in the same frame. Safe today only because cargo tags are disjoint.

---

## 2. What must stay distinct

| Keep separate | Why |
|:---|:---|
| `MissionBook` / `step_drone_missions` | Sortie/weather/C2; **take** packages — not restock |
| `formation_supply` → `Formation.supply` | Continuous 0..1 feed from coverage — not discrete BOM |
| FC-6 `draw_front_supply` / continuous ammo·fuel·rations | kg/s tonne loop — different metering |
| Personnel → credits → `combat_resolve` | Refill must **never** write `Formation.strength` |
| `DronePackage` vs `MissilePackage` **types** | Different role/class/sortie consumers — merge *protocol*, not components |
| Deployment orders / combat clock | Different domains |

**Reject:** one god-system that refills drones + missiles + personnel **and** mutates strength/supply.

---

## 3. Target architecture

```text
⊚LogisticsOrdersSnapshotRes ═▶ ⊨ front site stock
⊚FrontSupplyPoints ⊰ theaters
⊚CapabilityToeRegistry ═▶ ⊨ BOM lines (data)

⊚FrontSustainmentKernel (NEW — pure + thin systems)
  publish_stage: ToeDeficit → aggregated PublishDemand
  refill_stage:  FrontStockPool (once/frame) → allocate → ConsumeStock → typed sinks

Sinks (typed adapters, not erased):
  DronePackage / MissilePackage     — discrete TOE magazines
  PersonnelRestockBook              — credit ledger → resolve applies strength
  (FC-6 stays outside package kernel)

⊚step_drone_missions ═▶ ⊨ sortie consume (take only)
⊚combat_resolve ═▶ ⊨ strength/morale
```

### Schedule shape

```text
CombatClock
  → MissionConsume (step_drone_missions)     // take
  → FrontStockPool::from_snapshot            // once
  → refill_toe_from_front_system             // CS-6: drone|missile|personnel sinks
  → formation_supply
  → combat_resolve
PublishDemand (may stay late):
  publish_front_continuous ∥ publish_toe_deficits (one aggregator)
```

---

## 4. Migration phases (no dual-write)

| Slice | Goal | Accept |
|:---|:---|:---|
| **CS-0** | This charter + writer map + tag-disjoint assert | `--lib combat::` filter named in slice; no behavior change |
| **CS-1** | Extract `FrontStockPool` + `consume_bom_unit`; drone refill calls them only | `--lib drone_supply` bit-identical consume order |
| **CS-2** | Missile refill → same helpers; delete duplicated scan | `--lib missile_supply` |
| **CS-3** | Personnel refill → same pool API | `--lib personnel_supply` |
| **CS-4** | Merge three publish systems → one `publish_toe_cargo_demands` (sectioned book or single book) | dump counters stable |
| **CS-5** | Schedule: pool once before chained refills; document take vs add writers | SystemSet / `.chain()` comment |
| **CS-6** | Optional: single `refill_toe_packages_system` for drone+missile; personnel sibling if resolve boundary needs it | `validate_cargo_report` error_count 0 |

**Rollback:** any phase that makes refill write `Formation.strength` or MissionBook write stock.

---

## 5. How to read the current schedule (until CS lands)

| Block | Role |
|:---|:---|
| `advance_combat_clock` | Tick fence |
| `execute_deployment_orders` | Posture/target — **not** sustainment |
| `step_drone_missions` | Consume magazines for sorties |
| `refill_toe_from_front_system` | **One protocol, three typed sinks** — CS-6 |
| `formation_supply` | Continuous supply scalar — keep |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| 1.0.0 | 2026-11-15 | SIGNED — NEW_ABSTRACTION verdict; CS-0…CS-6; keep-distinct table. |
| 1.1.0 | 2026-11-15 | CS-1/2/3 — `front_stock_pool.rs`; drone/missile/personnel refill use shared pool. |
| 1.2.0 | 2026-11-15 | CS-4 `publish_toe_cargo_demands_system` · CS-5 `FrontStockPoolFrame` once/frame before chained refill. |
| 1.3.0 | 2026-11-15 | **CS-6** — `toe_refill::refill_toe_from_front_system`; typed sinks; Accept `--lib cs6`. **PROGRAM CLOSE**. |
