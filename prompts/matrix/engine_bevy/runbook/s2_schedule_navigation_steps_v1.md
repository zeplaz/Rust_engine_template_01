# S2 — Navigation & damage schedule `v1`

> **Orchestrator:** [`../../../guides/ecs_systems_schedule_runbook_v1.md`](../../../guides/ecs_systems_schedule_runbook_v1.md)  
> **Pre:** [`s1_schedule_cross_plugin_steps_v1.md`](s1_schedule_cross_plugin_steps_v1.md) applied.

Version: `v1.0.0`

---

### S2-S01 `NavSets` after `TransportSchedule::CostCache`

**Goal:** Lock **damage / speed** stage **before** **motion** stage, both **after** transport cost cache (rulebook: weights before agents consume paths).

**Anchor reads:** orchestrator §1, [`../../../../src/systems/transport/mod.rs`](../../../../src/systems/transport/mod.rs), [`../../../../src/engine/sets.rs`](../../../../src/engine/sets.rs).

**Touch:**

- `src/systems/navigation/schedule_plugin.rs` (new)
- `src/systems/navigation/mod.rs`
- `src/systems/damage/damage_system.rs`
- `src/engine/engine_with_worldgen.rs`
- `src/engine/sets.rs` (doc + variant order)

**Verify:**

```bash
cargo check
```

**Ordering (Update):**

1. `SimControlSystemSet::ApplyOperatorInput` → `AdvanceSimTick`
2. `TransportSchedule::Topology` → `FieldIntegrate` → `CostCache`
3. `NavSets::DamageSpeedAdjustment` (e.g. `apply_road_damage`)
4. `NavSets::MotionCalculation` (`nav_motion_stage_placeholder` until W3)

**Definition of done:**

- [ ] `NavigationSchedulePlugin` registered **before** `DamageSystem` (both after `TransportSimulationPlugin`).
- [ ] `DamageSystem` was previously **not** in `EnginePlugin`; it is now registered.
- [ ] `cargo check` passes.

---

### S2-S02 Doc sync

**Goal:** Phase index and inventory reflect S2.

**Touch:**

- [`README.md`](README.md) (this folder)
- [`../../../guides/ecs_systems_schedule_runbook_v1.md`](../../../guides/ecs_systems_schedule_runbook_v1.md) §4
- [`s0_inventory_steps_v1.md`](s0_inventory_steps_v1.md) — add plugins row

**Verify:** links resolve.

**Definition of done:** [ ] README status **Applied** for S2.
