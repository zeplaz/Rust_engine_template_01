# PLAN-REF-BOOKS-NAV-13-v1 — time-varying route costs / forecast windows

| Field | Value |
|:---|:---|
| **Id** | **PLAN-REF-BOOKS-NAV-13-v1** |
| **Matrix row** | **NAV-13** · aig3:4.8 (Continuous Time Pathfinding) |
| **Parent** | [`plan_ref_books_upgrade_v2.md`](plan_ref_books_upgrade_v2.md) §8 · [`plan_logistics_orders_v1.md`](plan_logistics_orders_v1.md) |
| **Trigger** | **L1 (`NAV-09`) + L3 (`NAV-10`) ADOPTED** — open-goal source pick + `LogisticsRouteDirtyTiles` |
| **Book orient** | `python src/dev/ref_books_index/lookup.py --row NAV-13` |
| **Owner lane** | `@coder` (navigation + logistics orders) |

## Summary

Extend logistics pathfinding so **edge weight depends on departure time and traverse duration**, using **forecast weather windows** from `WeatherField` step rhythm and W1 stations — not a second pathfinder. Vehicles “wait for window” only when encoded in order-sim policy; v1 focuses on **cost integration** (book §4.8: time-dependent costs) on the existing Dijkstra/flood grid and site graph hooks.

## Authority map (sole writers)

| Surface | Sole writer | Consumers (read-only) |
|:---|:---|:---|
| `WeatherField` | `weather_field_step` | forecast builder input |
| `WeatherPollingStations` | `sync_weather_polling_stations` | haul + forecast samples |
| `LogisticsRouteDirtyTiles` | route invalidation systems (L3) | triggers replan when **space** changes |
| **`LogisticsCostForecastBook`** (new) | `sync_logistics_cost_forecast` | floodfill, route cache rebuild, `source_pick` (when site edges gain time costs) |
| `RouteCache` / `CachedRoute` | logistics route systems | store path + **departure_tick** metadata |
| `LogisticsOrderSim` | order-sim step | applies `weather_haul_mul`; does not write forecast book |

Relations:

```text
⊚WeatherField ═▶ ⊚LogisticsCostForecastBook ⊰ ⊚WeatherPollingStations
⊚logistics_path_dijkstra_concern ◂⊳[cost_fn(t)] ⊚LogisticsCostForecastBook
⊚LogisticsRouteDirtyTiles ⊰ fire/topology events (spatial invalidation)
⊚RouteCache ◂⊳ paths; invalidation = spatial dirty OR forecast_bucket revision
```

**Reject:** parallel A* implementation; UI mutating routes; per-order private weather caches.

## ECS schedule placement

```text
ChunkEnvironmentSet::Weather
  … → sync_weather_polling_stations

LogisticsSimulationSet (after transport topology, before/at route recompute)
  sync_logistics_cost_forecast          (NEW — reads WeatherField prev/next + stations)
  invalidate_routes_from_dirty_tiles    (existing L3)
  rebuild_affected_routes               (existing — consume time-aware cost_fn)
```

Order-sim `sync_haul_weather_mul_system` stays downstream of stations; **N13-P2** aligns grid edge costs with the same `weather_haul_speed_mul` semantics extended across **time buckets** aligned to `weather_step_secs`.

Grid searches: `systems/navigation/logistics_floodfill.rs` — inject `LogisticsEdgeCostAtTime` closure, defaulting to static `cost_mul` when forecast book absent (compat).

## Save impact

| Item | Verdict |
|:---|:---|
| `LogisticsCostForecastBook` | **DERIVED** — rebuilt each tick from field + tuning |
| `CachedRoute` | **PARTIAL** — may add `planned_departure_tick: u64` optional field; bump `route_cache_schema` if persisted |
| RON `logistics_path_concerns_v1.ron` | extend with time-weight knobs — non-breaking defaults |

Save v1: omit forecast book; cached routes without departure tick replay as **static** costs until replanned (**documented** in hydrate path). Full time-consistency on load **DEFER**.

## Implementation phases (DAG)

| Phase | Goal | Status |
|:---|:---|:---|
| **N13-P0** | Pure `edge_cost_at(depart_tick, edge, duration_ticks)` using piecewise-constant weather buckets | **SHIPPED** |
| **N13-P1** | `LogisticsCostForecastBook` + `sync_logistics_cost_forecast` (K buckets from `WeatherField` prev/next) | **SHIPPED** |
| **N13-P2** | Wire `logistics_path_dijkstra_concern` optional time-aware edge weight; tests on grid fixture | **SHIPPED** |
| **N13-P3** | Invalidate/rebuild routes when **forecast_revision** changes (not only dirty tiles) | **SHIPPED** |
| **N13-P4** | `source_pick` / site graph: edge ticks = f(forecast) for cross-site legs used by NAV-09 | **SHIPPED** |
| **N13-P5** | Order advisor explains “window” (read-only snapshot) — **DEFER** to designer slice | **DEFER** |

★ **First slice to cut:** **N13-P0** (pure time-cost function + tests).

## Risks

| Risk | Mitigation |
|:---|:---|
| Exploding replans every weather step | `forecast_revision` only bumps when bucket integrals change beyond epsilon |
| Mismatch haul scalar vs grid mul | Single `weather_haul_speed_mul` / mobility helper shared by book + `weather_haul.rs` |
| Continuous-time vs discrete ticks | Document integration rule: **trapezoid over buckets** or min-cost bucket (pick one in P0, test-locked) |
| `travel.rs` straight-line interim | P4 scopes site graph; cell floodfill is authoritative for road risk (NAV-14 already) |

## SHIPPED / PLANNED / DEFER

| Label | Items |
|:---|:---|
| **SHIPPED** | N13-P0–P4 forecast book + time-aware floodfill + revision invalidate + source_pick edges |
| **PLANNED** | — |
| **DEFER** | N13-P5 advisor; full **reservation / schedule-aware** routing; fire-front **predictive** cost; LO-5 transport-graph trip model replacement |

## Diagnostics

- `LogisticsCostForecastBook`: `revision`, `bucket_secs`, per-bucket min/max mul (for tests).
- Reuse logistics diagnostics channel when route replanned for **time** vs **space** reason (enum tag).

## Open questions

- ⌁? Integrate fire advance as time-varying block (matrix mentions fire fronts) — pair with `vehicle_fire` dirty tiles or separate forecast layer?
- ⌁? Default bucket count K (suggest 4 = prev/current/next/placeholder tied to `WeatherField` lerp).

---

## First slice card — **N13-P0** 🧠 Time-varying edge cost pure fn

| | |
|:---|:---|
| **Ref** | `python src/dev/ref_books_index/lookup.py --row NAV-13` |
| **Files** | `src/systems/navigation/cost_forecast.rs` (new), `src/systems/navigation/mod.rs`, `src/economy/logistics/orders/weather_haul.rs` (shared mul helper re-use) |
| **Recon** | `grep -n "weather_haul_speed_mul\|cost_mul\|blend_cost" src/systems/navigation src/economy/logistics/orders/weather_haul.rs` — confirm no existing `depart_tick` cost (`Done-if`: absent). |
| **Do** | Define `WeatherCostBucket { start_tick, mul }` and `integrated_edge_cost(base_mul, depart_tick, traverse_ticks, buckets)` with **fixed** bucket iteration (deterministic). Map samples via `weather_haul_speed_mul` inverted to cost (slower weather → higher cost). |
| **Don't** | ECS; change `LogisticsOrderSim` writer; new pathfinder. |
| **Accept** | `cargo test -p proc_A_dine01 --lib systems::navigation::cost_forecast` — (1) constant buckets → `base * traverse`; (2) single bad window mid-trip increases integral vs departing earlier; (3) two identical calls match bit-for-bit. |

**Joint review question (@coder):** Should NAV-13 P2 use **max** risk across time (conservative) or **integrated** cost for concern blend (NAV-14 interaction)?