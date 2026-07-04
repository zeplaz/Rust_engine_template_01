# CITY program — remaining todos `v1`

| Field | Value |
|:---|:---|
| **Program** | PLAN-CITY-GRAMMAR-v1 |
| **Plan** | [`plan_city_grammar_upgrade_v1.md`](plan_city_grammar_upgrade_v1.md) |
| **Queue** | [`city_grammar_queue.json`](../tools/orchestrator/queues/city_grammar_queue.json) |
| **Bevy** | 0.19 · MIG-V1 green |
| **Status** | **CODER_CLOSED** 2026-07-03 |
| **Defer registry** | [`plan_deferral_registry_v1.md`](plan_deferral_registry_v1.md) DR-CITY-* |

---

## Closed (do not re-pick)

| Phase | IDs | Witness |
|:---|:---|:---|
| G0 | CITY-G0a/b/c (S11, S1C, WIT) | `city_g0_wit_001_live.json` |
| G1 | CITY-C1–C4, G1-C3 | `city_g1_c*_001_live.json` |
| G2 | CITY-C5, palette charter | `city_g2_c5_001_live.json` |
| G3 | C3 plugin/plaza/rollout, C7 staged, C6 footprint, C8 pipeline | `city_g3_rollout_live.json`, `city_c8_pipeline_001_live.json` |
| G3 | C6 BSN street furniture | `city_c6_bsn_001_live.json` |
| P | CITY-P1 static scene rollup | `city_p1_001_live.json` |
| P | CITY-P2 block LOD impostor | `city_p2_001_live.json` |
| DOC | CITY-DOC-002 plan + index refresh (0.19 closure) | `plan_city_grammar_upgrade_v1.md` |

**Maintenance fix (2026-07-03):** grammar RON `footprint_mode` uses bare enum identifiers (`rect`, `l_shape`, `yard_interior`) — required for G0 witness green.

---

## Active — coder

**None.**

---

## Blocked / deferred

| ID | Owner | Blocker |
|:---|:---|:---|
| DR-MIG-TILEMAP | steward | `bevy_ecs_tilemap` 0.19 not shipped — keep `bevy_tilemap_adapter` OFF |

---

## Next program (handoff)

**PLAN-BUILDING-QUALITY-v1** — Stream 2 in [`coder_non_migration_todos_v1.md`](coder_non_migration_todos_v1.md): BQ-A1 landed · BQ-C4 · APSR-S · BQ-H tail after A2.

---

## Verify bundle

```powershell
cargo test -p proc_A_dine01 --lib city_g0 city_g1 city_g3 city_p1 city_p2 city_c6
```
