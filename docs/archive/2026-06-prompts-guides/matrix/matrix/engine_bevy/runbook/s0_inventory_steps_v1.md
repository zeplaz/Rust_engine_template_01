# S0 — Systems & plugins inventory `v1`

> **Orchestrator:** [`../../../guides/ecs_systems_schedule_runbook_v1.md`](../../../guides/ecs_systems_schedule_runbook_v1.md)

Version: `v1.0.0`

---

### S0-S01 Inventory table + legacy sets note

**Goal:** Capture every `impl Plugin` registration site and existing `SystemSet` usage so refactors don’t miss a plugin.

**Anchor reads:** orchestrator §§1–2, [`../bevy_0_18_migration_plan.md`](../bevy_0_18_migration_plan.md) §1.1, [`../../../../src/engine/engine_with_worldgen.rs`](../../../../src/engine/engine_with_worldgen.rs).

**Touch:** this file (table below) OR a short `SYSTEMS_INVENTORY.md` in `src/engine/` `ASK:` — **prefer updating this pack** as the single source.

**Verify:** `rg "impl Plugin for" src` matches table row count (approximate).

**Definition of done:**
- [ ] Table reviewed against `engine_with_worldgen.rs` order.
- [ ] `NavSets` / `NavigationSchedulePlugin` / `DamageSystem` reflected post-**S2**.

#### Plugin registration order (`engine_with_worldgen.rs`)

| Order | Plugin | Domain |
|:---:|:---|:---|
| 1 | DefaultPlugins + Asset/Window | Core |
| 2 | EguiPlugin | UI framework |
| 3 | SplashPlugin, BaseMenuPlugin | Menus |
| 4 | MapEditorPlugin | Editor |
| 5 | SimControlPlugin | Pause / tick |
| 6 | TransportSimulationPlugin | Transport spine |
| 7 | NavigationSchedulePlugin | `NavSets` ordering (damage → motion) |
| 8 | DamageSystem | Road damage (`NavSets::DamageSpeedAdjustment`) |
| 9 | MaterialUnificationPlugin | Terrain materials |
| 10 | TilemapAdapterPlugin (feature) | Tiles |
| 11+ | Keybindings, Diagnostics, Faction, HUD, Logistics, WorldGenTools, … | Tools + sim |

*(Extended rows: grep `add_plugins` in `engine_with_worldgen.rs` for full chain.)*

#### `SystemSet` in repo (snapshot post-**S2**)

| Location | Sets | Wired? |
|:---|:---|:---:|
| `src/systems/transport/mod.rs` | `TransportSchedule` | Yes |
| `src/systems/sim_control.rs` | `SimControlSystemSet` | Yes (**S1**) |
| `src/engine/sets.rs` | `NavSets`, `GameSystemSet` | **NavSets** wired (**S2**); `GameSystemSet` still draft |
| `src/systems/navigation/schedule_plugin.rs` | `NavigationSchedulePlugin` (edges to `TransportSchedule`) | Yes (**S2**) |
| `src/gui/gui_sets.rs` | (stub / extend) | Partial |

---

### S0-S02 Grep contract for future steps

**Goal:** Standard commands for agents validating schedule refactors.

**Touch:** none required.

**Verify:**
```bash
rg "SystemSet|configure_sets|in_set\\(" src
rg "impl Plugin for" src
cargo check
```

**Definition of done:** Commands recorded; CI/manual can reuse.
