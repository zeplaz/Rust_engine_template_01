# HAUL-AMPLIFY-1 — Formation doctrine mul on order-sim capacity

| Field | Value |
|:---|:---|
| **Id** | **HAUL-AMPLIFY-1** |
| **Parent** | [`plan_production_restock_spine_v1.md`](plan_production_restock_spine_v1.md) · [`plan_ref_books_upgrade_use_v1.md`](plan_ref_books_upgrade_use_v1.md) Order 8 |
| **Status** | **SHIPPED** |
| **Gate** | **PLAYCHECK-1 SHIPPED** |

## Goal

Scale haul trip capacity from Formation / doctrine inside **`LogisticsOrderSim` only**. No second `VehicleFleet` ECS.

## Exit

1. Formation (or doctrine) multiplier applied when computing `capacity_units` / trip payload in order-sim.
2. Lib test: mul scales delivered trip capacity; fleet entity count unchanged.
3. `validate_cargo_report` — 0 errors.
4. Spine row **Logistics formations (haul amplify)** → **SHIPPED**.

## Rejected

- Parallel vehicle/mission ECS
- UI→Sim writes
- Haul speed weather rewrite (already W4 mul)
