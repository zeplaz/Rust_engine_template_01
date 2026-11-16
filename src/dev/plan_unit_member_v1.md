# PLAN-UNIT-MEMBER-v1 — formation members → MOVE-05/06/07 → TAC-10

| Field | Value |
|:---|:---|
| **Id** | **PLAN-UNIT-MEMBER-v1** |
| **Status** | **REOPENED 2026-11-16** — member systems tick on zero entities (spawn is test-only); TAC-10 type is doc-only. **MOVE-05 → PARTIAL**, **TAC-10 → UNWIRED** · cleanup **CQ-54, CQ-57n** |
| **Date** | 2026-11-15 |
| **Owner** | `@planner` → `@coder` |
| **Parents** | [`plan_ref_books_defer_unlock_v1.md`](plan_ref_books_defer_unlock_v1.md) W1 · PLAN-FRONT-COMBAT FC-5/6 · v3 PROGRAM CLOSE |
| **Deferral** | **DR-BOOKS-UNIT-MEMBER** |
| **Book rows** | **MOVE-05** · **MOVE-06** · **MOVE-07** · **TAC-10** (aig3 3.7 / 6.4.1) · LEGACY coordinated movement |

**Trigger to start UM-05:** Front combat wants **visible unit positions** (not one Formation blob / `faction_slot`).

---

## Review 2026-11-16 — reopened

> Operator code review of the Cursor wave ([`plan_cleanup_cursor_wave_v1.md`](plan_cleanup_cursor_wave_v1.md)). **Operator choice pending:** are individual soldiers drawn on the map? If yes, wire spawn + instanced draw; if no, unschedule the member systems.

| Finding | Evidence | Fix |
|:---|:---|:---|
| No members in play | `spawn_formation_with_members` / `spawn_formation_members` called only from a test (`formation_members.rs:313`); 4 systems scheduled (`combat/mod.rs:230-233`) on empty queries | Spawn members in the formation spawn path (CQ-54) |
| Overwatch role thrashes | `OverwatchRole` flips every tick while `target` set; nothing reads it | Phase timer + a reader, or remove the role |
| Spacing | `potential_field_navigation_system` (separation/avoid/arbitrate) is never scheduled | Use the CQ-53 steering layer for member spacing |
| Not visible | formations render as front ribbons/tiles only (`render/world_map_front_mesh.rs`) | GPU-instanced member sprites |
| TAC-10 doc-only | `CombatAuthorityTier` has no reader | Delete (CQ-57n) |

**Close again only when:** members exist in a normal scenario run, move by the shared steering layer, and are visible (or the operator chose "no members" and the systems are unscheduled). Every ADOPTED claim cites a production caller `file:line` (a test alone does not count).

## Authority (frozen)

```text
⊚DeploymentOrderQueue.enqueue ═▶ ⊨order ingress (UI/AI enqueue-only)
⊚execute_deployment_orders_system ═▶ ⊨Formation.posture / target / fortify
⊚resolve.rs ═▶ ⊨strength / morale / break-rally posture
⊚advance_formations_system ═▶ ⊨Formation.tile + move_progress   (anchor only)
⊚assign_member_slots_system ═▶ ⊨Member.slot_id                 [UM-06]
⊚steer_members_to_slots_system ═▶ ⊨Member.slot_offset / member tile [UM-05]
  ⊰ ⊚Formation.tile (anchor)
  ⛔ Member systems NEVER write Formation.tile / posture / target
⊚sync_formation_zones_system ═▶ ⊨Zone on Formation — unchanged
⊚ChunkStrategicOverlay / relax ⊰ Zones — no second control grid
```

### Schedule insert

```text
… orders execute → supply → resolve →
  advance_formations_system (anchor) →
  assign_member_slots_system (UM-06) →
  steer_members_to_slots_system (UM-05) →
  bounding_overwatch_tick_system (UM-07) →
  sync_formation_zones_system → relax …
```

---

## Slices

### UM-00 — charter lock
| Field | Value |
|:---|:---|
| **Needs** | Trigger true · this plan signed |
| **Files** | this doc only |
| **Do** | Freeze authority matrix; list rejects |
| **Don't** | Implement members |
| **Accept** | Plan path exists; cites Formation sole posture/tile writers |
| **Unlocks** | enables UM-05 |

### UM-05 — members + two-level steer (**MOVE-05**)
| Field | Value |
|:---|:---|
| **Needs** | UM-00 |
| **Files** | `src/combat/formation.rs` · new `src/combat/formation_members.rs` · `src/combat/mod.rs` (schedule) |
| **Do** | `FormationMember` entities; slot offsets from kind TOE; two-level steer (anchor then members); spawn/despawn with formation |
| **Don't** | Member writes `Formation.tile` · second Zone writer · per-member control overlay |
| **Accept** | `cargo test -p proc_A_dine01 --lib move05` — members follow slots when anchor moves |
| **Dual-cite** | MOVE-05 · aig3 3.7.2/3.7.4 |
| **Risk** | Dual mover |
| **Unlocks** | **MOVE-05** → flip `adoption.tsv` |

### UM-06 — cost-matrix slot assign (**MOVE-06**)
| Field | Value |
|:---|:---|
| **Needs** | UM-05 green |
| **Files** | `src/combat/formation_members.rs` · `src/combat/orders.rs` (read `FormationKind` only) |
| **Do** | Assign by cost ≠ index; deterministic ties (entity index / slot id); hysteresis so no thrash every frame |
| **Don't** | Cost matrix as second movement authority |
| **Accept** | `cargo test -p proc_A_dine01 --lib move06` — same inputs → same assignment |
| **Dual-cite** | MOVE-06 · aig3 3.7.7–3.7.9 |
| **Unlocks** | **MOVE-06** |

### UM-07 — bounding overwatch (**MOVE-07**)
| Field | Value |
|:---|:---|
| **Needs** | UM-06 green |
| **Files** | `formation_members.rs` · `front.rs` (RO) · posture via **orders** path only |
| **Do** | Bounding pairs (A move / B cover) |
| **Don't** | Bypass order executor · break front causality · write overlay threat |
| **Accept** | `cargo test -p proc_A_dine01 --lib move07` · front tests still green |
| **Dual-cite** | MOVE-07 · aig3 3.7.10 / 6.4.4 |
| **Unlocks** | **MOVE-07** |

### UM-10 — multi-tier handoff (**TAC-10**)
| Field | Value |
|:---|:---|
| **Needs** | UM-05/06/07 ADOPTED |
| **Files** | `ai/faction/mod.rs` · `combat/formation.rs` · `combat/drone_missions.rs` (docs + handoff types; no cross-write) |
| **Do** | Explicit tiers: Faction decide → Formation orders enqueue → Member steer. MissionBook stays MissionIntentQueue path. |
| **Don't** | Soft hierarchy · AI `Query<&mut Formation>` for tile |
| **Accept** | `cargo test -p proc_A_dine01 --lib tac10` · tripwire: faction decide has zero `Query<&mut Formation>` |
| **Dual-cite** | TAC-10 · aig3 6.4.1 |
| **Unlocks** | **TAC-10** |

---

## Rejects

- Second control/`Zone` writer for members  
- Members as logistics vehicles / `TransportTopology` agents  
- Test-shaped `members_green` in production  
- UI writing member tiles bypassing orders  

**Deferrals:** [`plan_deferral_registry_v1.md`](plan_deferral_registry_v1.md) — **DR-BOOKS-UNIT-MEMBER**.

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| 1.0.0 | 2026-11-15 | SIGNED — UM-00…UM-10 slice cards; authority frozen. |
| 1.1.0 | 2026-11-15 | **UM-05 ADOPTED** — `formation_members.rs`; Accept `--lib move05`; MOVE-05 ledger. |
| 1.2.0 | 2026-11-15 | **UM-06 ADOPTED** — `assign_slots_by_cost`; Accept `--lib move06`; MOVE-06 ledger. |
| 1.3.0 | 2026-11-15 | **UM-07 ADOPTED** — `OverwatchRole` + `bounding_overwatch_tick_system`; Accept `--lib move07`; MOVE-07 ledger. |
| 1.4.0 | 2026-11-15 | **UM-10 ADOPTED** — `tier_handoff.rs` + `tac10` tripwire; Accept `--lib tac10`; TAC-10 ledger. **PROGRAM CLOSE**. |
| 1.5.0 | 2026-11-16 | **REOPENED** — review: members never spawned in play, overwatch thrash, TAC-10 doc-only; MOVE-05 → PARTIAL; cleanup CQ-54. |
