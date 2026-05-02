# S1 — Cross-plugin schedule baseline `v1`

> **Orchestrator:** [`../../../guides/ecs_systems_schedule_runbook_v1.md`](../../../guides/ecs_systems_schedule_runbook_v1.md)  
> **Pre:** [`s0_inventory_steps_v1.md`](s0_inventory_steps_v1.md) complete.

Version: `v1.0.0`

---

### S1-S01 `SimControlSystemSet` + transport after tick

**Goal:** Expose explicit `SystemSet`s for simulation control so **transport** (and future gameplay) runs **after** pause handling and **after** `SimTick` advances for the frame.

**Anchor reads:** orchestrator §1, [`../../../../src/systems/sim_control.rs`](../../../../src/systems/sim_control.rs), [`../../../../src/systems/transport/mod.rs`](../../../../src/systems/transport/mod.rs).

**Touch:**
- `src/systems/sim_control.rs`
- `src/systems/transport/mod.rs`

**Verify:** `cargo check`

**Implementation notes:**
- `SimControlSystemSet::ApplyOperatorInput` — `keyboard_toggle_pause`
- `SimControlSystemSet::AdvanceSimTick` — `advance_sim_tick`; **after** ApplyOperatorInput
- `TransportSchedule::Topology` — **after** `AdvanceSimTick` (all three transport sets remain chained)

**Definition of done:**
- [ ] `cargo check` clean (no new errors).
- [ ] Same-frame: pause toggle applies before tick increment; transport field integrator sees updated `SimControlState` / tick ordering stable.

---

### S1-S02 Rename `Nav_Sets` → `NavSets` + doc pointer

**Goal:** Fix nonstandard type name; point draft nav motion sets at ECS schedule runbook.

**Anchor reads:** [`../../../../src/engine/sets.rs`](../../../../src/engine/sets.rs), orchestrator.

**Touch:** `src/engine/sets.rs`

**Verify:** `cargo check`

**Definition of done:**
- [ ] No `Nav_Sets` identifier left in `src/`.
- [ ] Top-of-file comment: “Wiring deferred — see `prompts/guides/ecs_systems_schedule_runbook_v1.md`.”
