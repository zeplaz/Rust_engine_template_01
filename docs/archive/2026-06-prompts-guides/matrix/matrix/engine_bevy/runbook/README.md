# `engine_bevy` — step packs (ECS schedule)

> **Orchestrator:** [`../../../guides/ecs_systems_schedule_runbook_v1.md`](../../../guides/ecs_systems_schedule_runbook_v1.md)  
> **Matrix:** [`../bevy_0_18_migration_plan.md`](../bevy_0_18_migration_plan.md)

| Phase | Pack | Status |
|:---:|:---|:---|
| **S0** | [`s0_inventory_steps_v1.md`](s0_inventory_steps_v1.md) | Applied (inventory doc) |
| **S1** | [`s1_schedule_cross_plugin_steps_v1.md`](s1_schedule_cross_plugin_steps_v1.md) | Applied (`SimControlSystemSet`, transport after tick, `NavSets` rename) |
| **S2** | [`s2_schedule_navigation_steps_v1.md`](s2_schedule_navigation_steps_v1.md) | Applied (`NavigationSchedulePlugin`, `DamageSystem`, `NavSets` after `CostCache`) |

**Update frame:** `SimControl` → `Transport` (topology → field → **cost**) → **`NavSets` damage** → **`NavSets` motion** (placeholder) → …
